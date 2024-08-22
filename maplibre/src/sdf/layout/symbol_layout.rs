use crate::sdf::bidi::BiDi;
use crate::sdf::buckets::symbol_bucket::PlacedSymbol;
use crate::sdf::geometry::{Anchor, Anchors};
use crate::sdf::geometry_tile_data::{GeometryCoordinates, GeometryTileLayer};
use crate::sdf::glyph::{GlyphMap, Shaping, WritingModeType};
use crate::sdf::glyph_atlas::GlyphPositions;
use crate::sdf::image::ImageMap;
use crate::sdf::image_atlas::ImagePositions;
use crate::sdf::layout::symbol_feature::SymbolFeature;
use crate::sdf::layout::symbol_instance::{ShapedTextOrientations, SymbolContent, SymbolInstance};
use crate::sdf::quads::{SymbolQuad, SymbolQuads};
use crate::sdf::shaping::{getAnchorJustification, getShaping, PositionedIcon};
use crate::sdf::style_types::*;
use crate::sdf::tagged_string::{SectionOptions, TaggedString};
use crate::sdf::util::constants::ONE_EM;
use crate::sdf::util::i18n;
use crate::sdf::MapMode;
use std::collections::{BTreeMap, HashMap};
use std::ops::Range;
use widestring::U16String;

// bucket
struct SymbolBucket;
struct SymbolBucket_Buffer;

// index
struct FeatureIndex;

// render
struct SortKeyRange;
struct LayerRenderData; // = Bucket + LayerProperties

// tile
#[derive(Copy, Clone)]
pub struct CanonicalTileID;

// TODO
// template <class Property>
// static bool has(const SymbolLayoutProperties::PossiblyEvaluated& layout) {
//     return layout.get<Property>().match([](const typename Property::Type& t) { return !t.empty(); },
//                                         [](const auto&) { return true; });
// }

fn sectionOptionsToValue(options: &SectionOptions) -> expression::Value {
    let mut result: HashMap<String, expression::Value> = Default::default();
    // TODO: Data driven properties that can be overridden on per section basis.
    // TextOpacity
    // TextHaloColor
    // TextHaloWidth
    // TextHaloBlur
    if let Some(textColor) = &(options.textColor) {
        result.insert(
            expression::kFormattedSectionTextColor.to_string(),
            expression::Value::Color(textColor.clone()),
        );
    }
    return expression::Value::Object(result);
}

fn toSymbolLayerProperties(layer: &LayerProperties) -> &SymbolLayerProperties {
    //return static_cast<const SymbolLayerProperties&>(*layer);
    todo!()
}

fn createLayout(
    unevaluated: &SymbolLayoutProperties_Unevaluated,
    zoom: f64,
) -> SymbolLayoutProperties_PossiblyEvaluated {
    let mut layout = unevaluated.evaluate(PropertyEvaluationParameters(zoom));

    if (layout.get::<IconRotationAlignment>() == AlignmentType::Auto) {
        if (layout.get::<SymbolPlacement>() != SymbolPlacementType::Point) {
            *layout.get_mut::<IconRotationAlignment>() = AlignmentType::Map;
        } else {
            *layout.get_mut::<IconRotationAlignment>() = AlignmentType::Viewport;
        }
    }

    if (layout.get::<TextRotationAlignment>() == AlignmentType::Auto) {
        if (layout.get::<SymbolPlacement>() != SymbolPlacementType::Point) {
            *layout.get_mut::<TextRotationAlignment>() = AlignmentType::Map;
        } else {
            *layout.get_mut::<TextRotationAlignment>() = AlignmentType::Viewport;
        }
    }

    // If unspecified `*-pitch-alignment` inherits `*-rotation-alignment`
    if (layout.get::<TextPitchAlignment>() == AlignmentType::Auto) {
        *layout.get_mut::<TextPitchAlignment>() = layout.get::<TextRotationAlignment>();
    }
    if (layout.get::<IconPitchAlignment>() == AlignmentType::Auto) {
        *layout.get_mut::<IconPitchAlignment>() = layout.get::<IconRotationAlignment>();
    }

    return layout;
}

// The radial offset is to the edge of the text box
// In the horizontal direction, the edge of the text box is where glyphs start
// But in the vertical direction, the glyphs appear to "start" at the baseline
// We don't actually load baseline data, but we assume an offset of ONE_EM - 17
// (see "yOffset" in shaping.js)
const baselineOffset: f64 = 7.0;

// We don't care which shaping we get because this is used for collision
// purposes and all the justifications have the same collision box.
fn getDefaultHorizontalShaping(shapedTextOrientations: &ShapedTextOrientations) -> &Shaping {
    if shapedTextOrientations.right.isAnyLineNotEmpty() {
        return &shapedTextOrientations.right;
    }
    if shapedTextOrientations.center.isAnyLineNotEmpty() {
        return &shapedTextOrientations.center;
    }
    if shapedTextOrientations.left.isAnyLineNotEmpty() {
        return &shapedTextOrientations.left;
    }
    return &shapedTextOrientations.horizontal;
}

fn shapingForTextJustifyType(
    shapedTextOrientations: &ShapedTextOrientations,
    type_: TextJustifyType,
) -> &Shaping {
    match (type_) {
        TextJustifyType::Right => {
            return &shapedTextOrientations.right;
        }

        TextJustifyType::Left => {
            return &shapedTextOrientations.left;
        }

        TextJustifyType::Center => {
            return &shapedTextOrientations.center;
        }
        _ => {
            assert!(false);
            return &shapedTextOrientations.horizontal;
        }
    }
}

fn evaluateRadialOffset(anchor: SymbolAnchorType, mut radialOffset: f64) -> [f64; 2] {
    let mut result = [0.0, 0.0];
    if (radialOffset < 0.0) {
        radialOffset = 0.0; // Ignore negative offset.
    }
    // solve for r where r^2 + r^2 = radialOffset^2
    let sqrt2 = 1.41421356237;
    let hypotenuse = radialOffset / sqrt2;

    match (anchor) {
        SymbolAnchorType::TopRight | SymbolAnchorType::TopLeft => {
            result[1] = hypotenuse - baselineOffset;
        }

        SymbolAnchorType::BottomRight | SymbolAnchorType::BottomLeft => {
            result[1] = -hypotenuse + baselineOffset;
        }
        SymbolAnchorType::Bottom => {
            result[1] = -radialOffset + baselineOffset;
        }
        SymbolAnchorType::Top => {
            result[1] = radialOffset - baselineOffset;
        }

        _ => {}
    }

    match (anchor) {
        SymbolAnchorType::TopRight | SymbolAnchorType::BottomRight => {
            result[0] = -hypotenuse;
        }
        SymbolAnchorType::TopLeft | SymbolAnchorType::BottomLeft => {
            result[0] = hypotenuse;
        }
        SymbolAnchorType::Left => {
            result[0] = radialOffset;
        }
        SymbolAnchorType::Right => {
            result[0] = -radialOffset;
        }

        _ => {}
    }

    return result;
}

struct SymbolLayout {
    pub layerPaintProperties: BTreeMap<String, LayerProperties>,
    pub bucketLeaderID: String,
    pub symbolInstances: Vec<SymbolInstance>,
    pub sortKeyRanges: Vec<SortKeyRange>,

    // Stores the layer so that we can hold on to GeometryTileFeature instances
    // in SymbolFeature, which may reference data from this object.
    sourceLayer: Box<dyn GeometryTileLayer>,
    overscaling: f64,
    zoom: f64,
    canonicalID: CanonicalTileID,
    mode: MapMode,
    pixelRatio: f64,

    tileSize: u32,
    tilePixelRatio: f64,

    iconsNeedLinear: bool,
    sortFeaturesByY: bool,
    sortFeaturesByKey: bool,
    allowVerticalPlacement: bool,
    iconsInText: bool,
    placementModes: Vec<TextWritingModeType>,

    textSize: <TextSize as DataDrivenLayoutProperty>::UnevaluatedType,
    iconSize: <IconSize as DataDrivenLayoutProperty>::UnevaluatedType,
    textRadialOffset: <TextRadialOffset as DataDrivenLayoutProperty>::UnevaluatedType,
    layout: SymbolLayoutProperties_PossiblyEvaluated,
    features: Vec<SymbolFeature>,

    bidi: BiDi, // Consider moving this up to geometry tile worker to reduce
    // reinstantiation costs, use of BiDi/ubiditransform object must
    // be rained to one thread
    compareText: BTreeMap<U16String, Vec<Anchor>>,
}

impl SymbolLayout {
    fn prepareSymbols(
        &mut self,
        glyphMap: &GlyphMap,
        glyphPositions: &GlyphPositions,
        imageMap: &ImageMap,
        imagePositions: &ImagePositions,
    ) {
        let isPointPlacement: bool =
            self.layout.get::<SymbolPlacement>() == SymbolPlacementType::Point;
        let textAlongLine: bool =
            self.layout.get::<TextRotationAlignment>() == AlignmentType::Map && !isPointPlacement;

        for (feature_index, feature) in self.features.iter_mut().enumerate() {
            if (feature.geometry.is_empty()) {
                continue;
            }

            let mut shapedTextOrientations: ShapedTextOrientations =
                ShapedTextOrientations::default();
            let mut shapedIcon: Option<PositionedIcon> = None;
            let mut textOffset = [0.0, 0.0];
            let layoutTextSize: f64 =
                self.layout
                    .evaluate::<TextSize>(self.zoom + 1., feature, self.canonicalID);
            let layoutTextSizeAtBucketZoomLevel: f64 =
                self.layout
                    .evaluate::<TextSize>(self.zoom, feature, self.canonicalID);
            let layoutIconSize: f64 =
                self.layout
                    .evaluate::<IconSize>(self.zoom + 1., feature, self.canonicalID);

            // if feature has text, shape the text
            if let Some(feature_formattedText) = &mut feature.formattedText {
                if (layoutTextSize > 0.0) {
                    let lineHeight: f64 = self.layout.get::<TextLineHeight>() * ONE_EM;
                    let spacing: f64 = if i18n::allowsLetterSpacing(feature_formattedText.rawText())
                    {
                        self.layout.evaluate::<TextLetterSpacing>(
                            self.zoom,
                            feature,
                            self.canonicalID,
                        ) * ONE_EM
                    } else {
                        0.0
                    };

                    let mut applyShaping = |formattedText: &TaggedString,
                                        writingMode: WritingModeType,
                                        textAnchor: SymbolAnchorType,
                                        textJustify: TextJustifyType|
                     -> Shaping {
                        let result = getShaping(
                            /* string */ formattedText,
                            /* maxWidth: ems */
                            if isPointPlacement {
                                self.layout.evaluate::<TextMaxWidth>(
                                    self.zoom,
                                    feature,
                                    self.canonicalID,
                                ) * ONE_EM
                            } else {
                                0.0
                            },
                            /* ems */ lineHeight,
                            textAnchor,
                            textJustify,
                            /* ems */ spacing,
                            /* translate */ &textOffset,
                            /* writingMode */ writingMode,
                            /* bidirectional algorithm object */ &self.bidi,
                            glyphMap,
                            /* glyphs */ glyphPositions,
                            /* images */ imagePositions,
                            layoutTextSize,
                            layoutTextSizeAtBucketZoomLevel,
                            self.allowVerticalPlacement,
                        );

                        return result;
                    };

                    let variableTextAnchor: Vec<TextVariableAnchorType> =
                        self.layout.evaluate_static::<TextVariableAnchor>(
                            self.zoom,
                            feature,
                            self.canonicalID,
                        );
                    let textAnchor: SymbolAnchorType =
                        self.layout
                            .evaluate::<TextAnchor>(self.zoom, feature, self.canonicalID);
                    if (variableTextAnchor.is_empty()) {
                        // Layers with variable anchors use the `text-radial-offset`
                        // property and the [x, y] offset vector is calculated at
                        // placement time instead of layout time
                        let radialOffset: f64 = self.layout.evaluate::<TextRadialOffset>(
                            self.zoom,
                            feature,
                            self.canonicalID,
                        );
                        if (radialOffset > 0.0) {
                            // The style spec says don't use `text-offset` and
                            // `text-radial-offset` together but doesn't actually
                            // specify what happens if you use both. We go with the
                            // radial offset.
                            textOffset = evaluateRadialOffset(textAnchor, radialOffset * ONE_EM);
                        } else {
                            textOffset = [
                                self.layout.evaluate::<TextOffset>(
                                    self.zoom,
                                    feature,
                                    self.canonicalID,
                                )[0] * ONE_EM,
                                self.layout.evaluate::<TextOffset>(
                                    self.zoom,
                                    feature,
                                    self.canonicalID,
                                )[1] * ONE_EM,
                            ];
                        }
                    }
                    let mut textJustify = if textAlongLine {
                        TextJustifyType::Center
                    } else {
                        self.layout
                            .evaluate::<TextJustify>(self.zoom, feature, self.canonicalID)
                    };

                    let mut addVerticalShapingForPointLabelIfNeeded = || {
                        if (self.allowVerticalPlacement
                            && feature_formattedText.allowsVerticalWritingMode())
                        {
                            feature_formattedText.verticalizePunctuation();
                            // Vertical POI label placement is meant to be used for
                            // scripts that support vertical writing mode, thus, default
                            // TextJustifyType::Left justification is used. If
                            // Latin scripts would need to be supported, this should
                            // take into account other justifications.
                            shapedTextOrientations.vertical = applyShaping(
                                feature_formattedText,
                                WritingModeType::Vertical,
                                textAnchor,
                                TextJustifyType::Left,
                            );
                        }
                    };

                    // If this layer uses text-variable-anchor, generate shapings for
                    // all justification possibilities.
                    if (!textAlongLine && !variableTextAnchor.is_empty()) {
                        let mut justifications: Vec<TextJustifyType> = Vec::new();
                        if (textJustify != TextJustifyType::Auto) {
                            justifications.push(textJustify);
                        } else {
                            for anchor in &variableTextAnchor {
                                justifications.push(getAnchorJustification(anchor));
                            }
                        }
                        for justification in justifications {
                            let mut shapingForJustification =
                                shapingForTextJustifyType(&shapedTextOrientations, justification);
                            if (shapingForJustification.isAnyLineNotEmpty()) {
                                continue;
                            }
                            // If using text-variable-anchor for the layer, we use a
                            // center anchor for all shapings and apply the offsets for
                            // the anchor in the placement step.
                            let shaping = applyShaping(
                                feature_formattedText,
                                WritingModeType::Horizontal,
                                SymbolAnchorType::Center,
                                justification,
                            );
                            if (shaping.isAnyLineNotEmpty()) {
                                shapingForJustification = &shaping;
                                if (shapingForJustification.positionedLines.len() == 1) {
                                    shapedTextOrientations.singleLine = true;
                                    break;
                                }
                            }
                        }

                        // Vertical point label shaping if allowVerticalPlacement is enabled.
                        addVerticalShapingForPointLabelIfNeeded();
                    } else {
                        if (textJustify == TextJustifyType::Auto) {
                            textJustify = getAnchorJustification(&textAnchor);
                        }

                        // Horizontal point or line label.
                        let shaping = applyShaping(
                            &feature_formattedText,
                            WritingModeType::Horizontal,
                            textAnchor,
                            textJustify,
                        );
                        if (shaping.isAnyLineNotEmpty()) {
                            shapedTextOrientations.horizontal = shaping;
                        }

                        // Vertical point label shaping if allowVerticalPlacement is enabled.
                        addVerticalShapingForPointLabelIfNeeded();

                        // Verticalized line label.
                        if (textAlongLine && feature_formattedText.allowsVerticalWritingMode()) {
                            feature_formattedText.verticalizePunctuation();
                            shapedTextOrientations.vertical = applyShaping(
                                feature_formattedText,
                                WritingModeType::Vertical,
                                textAnchor,
                                textJustify,
                            );
                        }
                    }
                }
            }

            // if feature has icon, get sprite atlas position
            let mut iconType: SymbolContent = SymbolContent::None;
            if let Some(icon) = &feature.icon {
                let image = imageMap.get(&icon.imageID);
                if let Some(image) = image {
                    iconType = SymbolContent::IconRGBA;
                    shapedIcon = Some(PositionedIcon::shapeIcon(
                        imagePositions.get(&icon.imageID).unwrap().clone(),
                        &self.layout.evaluate::<IconOffset>(self.zoom, feature, self.canonicalID),
                        self.layout.evaluate::<IconAnchor>(self.zoom, feature, self.canonicalID),
                    ));
                    if (image.sdf) {
                        iconType = SymbolContent::IconSDF;
                    }
                    if (image.pixelRatio != self.pixelRatio) {
                        self.iconsNeedLinear = true;
                    } else if (self.layout.get_dynamic::<IconRotate>().constantOr(1.0) != 0.0) {
                        self.iconsNeedLinear = true;
                    }
                }
            }

            // if either shapedText or icon position is present, add the feature
            let defaultShaping = getDefaultHorizontalShaping(&shapedTextOrientations);
            self.iconsInText = if defaultShaping.isAnyLineNotEmpty() {
                defaultShaping.iconsInText
            } else {
                false
            };
            if (defaultShaping.isAnyLineNotEmpty() || shapedIcon.is_some()) {
                self.addFeature(
                    feature_index,
                    feature,
                    &shapedTextOrientations,
                    shapedIcon,
                    imageMap,
                    textOffset,
                    layoutTextSize,
                    layoutIconSize,
                    iconType,
                );
            }

            feature.geometry.0.clear();
        }

        self.compareText.clear();
    }

    fn createBucket(
        &self,
        imagePositions: ImagePositions,
        feature_index: Box<FeatureIndex>,
        renderData: &HashMap<String, LayerRenderData>,
        firstLoad: bool,
        showCollisionBoxes: bool,
        canonical: &CanonicalTileID,
    ) {
        todo!()
    }

    fn hasSymbolInstances(&self) -> bool {
        return !self.symbolInstances.is_empty();
    }
    fn hasDependencies(&self) -> bool {
        return !self.features.is_empty();
    }

    pub const INVALID_OFFSET_VALUE: f64 = f64::MAX;
    /**
     * @brief Calculates variable text offset.
     *
     * @param anchor text anchor
     * @param textOffset Either `text-offset` or [ `text-radial-offset`,
     * INVALID_OFFSET_VALUE ]
     * @return std::array<f64, 2> offset along x- and y- axis correspondingly.
     */
    pub fn evaluateVariableOffset(anchor: SymbolAnchorType, mut offset: [f64; 2]) -> [f64; 2] {
        if (offset[1] == Self::INVALID_OFFSET_VALUE) {
            return evaluateRadialOffset(anchor, offset[0]);
        }
        let mut result = [0.0, 0.0];
        offset[0] = (offset[0]).abs();
        offset[1] = (offset[1]).abs();

        match (anchor) {
            SymbolAnchorType::TopRight | SymbolAnchorType::TopLeft | SymbolAnchorType::Top => {
                result[1] = offset[1] - baselineOffset;
            }

            SymbolAnchorType::BottomRight
            | SymbolAnchorType::BottomLeft
            | SymbolAnchorType::Bottom => {
                result[1] = -offset[1] + baselineOffset;
            }

            SymbolAnchorType::Center | SymbolAnchorType::Left | SymbolAnchorType::Right => {}
        }

        match (anchor) {
            SymbolAnchorType::TopRight
            | SymbolAnchorType::BottomRight
            | SymbolAnchorType::Right => {
                result[0] = -offset[0];
            }
            SymbolAnchorType::TopLeft | SymbolAnchorType::BottomLeft | SymbolAnchorType::Left => {
                result[0] = offset[0];
            }
            SymbolAnchorType::Center | SymbolAnchorType::Top | SymbolAnchorType::Bottom => {}
        }

        return result;
    }

    // Analog of `addToLineVertexArray` in JS. This version doesn't need to build up
    // a line array like the JS version does, but it uses the same logic to
    // calculate tile distances.
    pub fn calculateTileDistances(line: &GeometryCoordinates, anchor: &Anchor) -> Vec<f64> {
        let mut tileDistances: Vec<f64> = vec![0.0; line.len()];
        if let Some(segment) = (anchor.segment) {
            assert!(segment < line.len());
            let mut sumForwardLength = if (segment + 1 < line.len()) {
                anchor.point.distance_to(line[segment + 1].cast::<f64>())
            } else {
                0.0
            };
            let mut sumBackwardLength = anchor.point.distance_to(line[segment].cast::<f64>());
            for i in segment + 1..line.len() {
                tileDistances[i] = sumForwardLength;
                if (i < line.len() - 1) {
                    sumForwardLength +=
                        line[i + 1].cast::<f64>().distance_to(line[i].cast::<f64>());
                }
            }

            let mut i = segment;
            loop {
                tileDistances[i] = sumBackwardLength;
                if (i != 0) {
                    sumBackwardLength +=
                        line[i - 1].cast::<f64>().distance_to(line[i].cast::<f64>());
                } else {
                    break; // Add break to avoid unsigned integer overflow when i==0
                }
                i -= 1;
            }
        }
        return tileDistances;
    }
}

impl SymbolLayout {
    fn addFeature(
        &self,
        layoutFeatureIndex: usize,
        feature: &SymbolFeature,
        shapedTextOrientations: &ShapedTextOrientations,
        shapedIcon: Option<PositionedIcon>,
        imageMap: &ImageMap,
        textOffset: [f64; 2],
        layoutTextSize: f64,
        layoutIconSize: f64,
        iconType: SymbolContent,
    ) {
        todo!()
    }

    fn anchorIsTooClose(&mut self, text: &U16String, repeatDistance: f64, anchor: &Anchor) -> bool {
        if let Some(otherAnchors) = self.compareText.get(text) {
            for otherAnchor in otherAnchors {
                if (anchor.point.distance_to(otherAnchor.point) < repeatDistance) {
                    return true;
                }
            }
        } else {
            self.compareText.insert(text.clone(), Anchors::new());
        }

        let anchors = self.compareText.get_mut(text).unwrap();
        anchors.push(anchor.clone());
        return false;
    }

    fn addToDebugBuffers(&self, bucket: &SymbolBucket) {
        todo!()
    }

    // Adds placed items to the buffer.
    fn addSymbol(
        &self,
        buffer: &SymbolBucket_Buffer,
        sizeData: Range<f64>,
        symbol: &SymbolQuad,
        labelAnchor: &Anchor,
        placedSymbol: &PlacedSymbol,
        sortKey: f64,
    ) -> usize {
        todo!()
    }
    fn addSymbols(
        &self,
        buffer: &SymbolBucket_Buffer,
        sizeData: Range<f64>,
        symbols: &SymbolQuads,
        labelAnchor: &Anchor,
        placedSymbol: &PlacedSymbol,
        sortKey: f64,
    ) -> usize {
        todo!()
    }

    // Adds symbol quads to bucket and returns formatted section index of last
    // added quad.
    fn addSymbolGlyphQuads(
        &self,
        buffer: &SymbolBucket,
        symbolInstance: &SymbolInstance,
        feature: &SymbolFeature,
        writingMode: WritingModeType,
        placedIndex: Option<usize>,
        glyphQuads: &SymbolQuads,
        canonical: &CanonicalTileID,
        lastAddedSection: Option<usize>,
    ) -> usize {
        todo!()
    }

    fn updatePaintPropertiesForSection(
        &self,
        bucket: &SymbolBucket,
        feature: &SymbolFeature,
        sectionIndex: usize,
        canonical: &CanonicalTileID,
    ) -> usize {
        let formattedSection = sectionOptionsToValue(
            feature
                .formattedText
                .as_ref()
                .unwrap()
                .sectionAt(sectionIndex),
        );
        todo!()
        //for pair in bucket.paintProperties {
        //    pair.1.textBinders.populateVertexVectors(
        //        feature,
        //        bucket.text.vertices().elements(),
        //        feature.index,
        //        {},
        //        {},
        //        canonical,
        //        formattedSection,
        //    );
        //}
    }
}
