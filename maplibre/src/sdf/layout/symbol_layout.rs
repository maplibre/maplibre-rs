use crate::coords::EXTENT;
use crate::sdf::bidi::{BiDi, Char16};
use crate::sdf::buckets::symbol_bucket::PlacedSymbol;
use crate::sdf::geometry::feature_index::{IndexedSubfeature, RefIndexedSubfeature};
use crate::sdf::geometry::{Anchor, Anchors};
use crate::sdf::geometry_tile_data::{FeatureType, GeometryCoordinates, GeometryTileLayer};
use crate::sdf::glyph::{GlyphIDs, GlyphMap, Shaping, WritingModeType};
use crate::sdf::glyph_atlas::GlyphPositions;
use crate::sdf::image::{ImageMap, ImageType};
use crate::sdf::image_atlas::ImagePositions;
use crate::sdf::layout::symbol_feature::SymbolFeature;
use crate::sdf::layout::symbol_instance::{
    ShapedTextOrientations, SymbolContent, SymbolInstance, SymbolInstanceSharedData,
};
use crate::sdf::quads::{SymbolQuad, SymbolQuads};
use crate::sdf::shaping::{getAnchorJustification, getShaping, PositionedIcon};
use crate::sdf::style_types::*;
use crate::sdf::tagged_string::{SectionOptions, TaggedString};
use crate::sdf::util::constants::ONE_EM;
use crate::sdf::util::i18n;
use crate::sdf::util::math::deg2radf;
use crate::sdf::MapMode;
use geo::HasDimensions;
use geo_types::Polygon;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::f64::consts::PI;
use std::ops::Range;
use std::rc::Rc;
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
//                                         [](let) { return true; });
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
    pub fn new(
        parameters: &BucketParameters,
        layers: &Vec<LayerProperties>,
        sourceLayer_: Box<dyn GeometryTileLayer>,
        layoutParameters: &LayoutParameters,
    ) -> Self {
        let mut self_ = Self {};
        /*
        bucketLeaderID(layers.front()->baseImpl->id),
          sourceLayer(std::move(sourceLayer_)),
          overscaling(static_cast<float>(parameters.tileID.overscaleFactor())),
          zoom(parameters.tileID.overscaledZ),
          canonicalID(parameters.tileID.canonical),
          mode(parameters.mode),
          pixelRatio(parameters.pixelRatio),
          tileSize(static_cast<uint32_t>(util::tileSize_D * overscaling)),
          tilePixelRatio(static_cast<float>(util::EXTENT) / tileSize),
          layout(createLayout(toSymbolLayerProperties(layers.at(0)).layerImpl().layout, zoom)
         */

        let leader: &SymbolLayer::Impl = toSymbolLayerProperties(layers.at(0)).layerImpl();

        self_.textSize = leader.layout.get::<TextSize>();
        self_.iconSize = leader.layout.get::<IconSize>();
        self_.textRadialOffset = leader.layout.get::<TextRadialOffset>();

        let hasText = has::<TextField>(*layout) && has::<TextFont>(*layout);
        let hasIcon = has::<IconImage>(*layout);

        if (!hasText && !hasIcon) {
            return None;
        }

        let hasSymbolSortKey = !leader.layout.get::<SymbolSortKey>().isUndefined();
        let symbolZOrder = self_.layout.get::<SymbolZOrder>();
        self_.sortFeaturesByKey = symbolZOrder != SymbolZOrderType::ViewportY && hasSymbolSortKey;
        let zOrderByViewportY = symbolZOrder == SymbolZOrderType::ViewportY
            || (symbolZOrder == SymbolZOrderType::Auto && !self_.sortFeaturesByKey);
        self_.sortFeaturesByY = zOrderByViewportY
            && (self_.layout.get::<TextAllowOverlap>()
                || self_.layout.get::<IconAllowOverlap>()
                || self_.layout.get::<TextIgnorePlacement>()
                || self_.layout.get::<IconIgnorePlacement>());
        if (self_.layout.get::<SymbolPlacement>() == SymbolPlacementType::Point) {
            let modes = self_.layout.get::<TextWritingMode>();
            // Remove duplicates and preserve order.
            let mut seen: BTreeSet<TextWritingModeType>;
            let end = std::remove_if(modes.begin(), modes.end(), |placementMode| {
                self_.allowVerticalPlacement =
                    self_.allowVerticalPlacement || placementMode == TextWritingModeType::Vertical;
                return !seen.insert(placementMode).second;
            });
            modes.erase(end, modes.end());
            self_.placementModes = modes;
        }

        for layer in layers {
            self_.layerPaintProperties.emplace(layer.baseImpl.id, layer);
        }

        // Determine glyph dependencies
        let featureCount = sourceLayer.featureCount();
        for i in 0..featureCount {
            let feature = sourceLayer.getFeature(i);
            if (!leader.filter(
                expression::EvaluationContext(self_.zoom, feature.get())
                    .withCanonicalTileID(&parameters.tileID.canonical),
            )) {
                continue;
            }

            let mut ft: SymbolFeature = feature;

            ft.index = i;

            if (hasText) {
                let formatted = self_.layout.evaluate::<TextField>(
                    self_.zoom,
                    &ft,
                    layoutParameters.availableImages,
                    self_.canonicalID,
                );
                let textTransform =
                    self_
                        .layout
                        .evaluate::<TextTransform>(self_.zoom, &ft, self_.canonicalID);
                let baseFontStack =
                    self_
                        .layout
                        .evaluate::<TextFont>(self_.zoom, &ft, self_.canonicalID);

                ft.formattedText = TaggedString::default();
                for section in &formatted.sections {
                    if (!section.image) {
                        let u8string = section.text;
                        if (textTransform == TextTransformType::Uppercase) {
                            u8string = platform::uppercase(u8string);
                        } else if (textTransform == TextTransformType::Lowercase) {
                            u8string = platform::lowercase(u8string);
                        }

                        if let Err(e) = ft.formattedText.addTextSection(
                            applyArabicShaping(util::convertUTF8ToUTF16(u8string)),
                            if section.fontScale {
                                *section.fontScale
                            } else {
                                1.0
                            },
                            if section.fontScale {
                                *section.fontStack
                            } else {
                                baseFontStack
                            },
                            section.textColor,
                        ) {
                            log::error!("Encountered section with invalid UTF-8 in tile, source: {} z: {} x: {} y: {}", sourceLayer.getName(), canonicalID.z, canonicalID.x, canonicalID.y);
                            continue; // skip section
                        }
                    } else {
                        layoutParameters
                            .imageDependencies
                            .emplace(section.image.id(), ImageType::Icon);
                        ft.formattedText.addImageSection(section.image.id());
                    }
                }

                let canVerticalizeText = self_.layout.get::<TextRotationAlignment>()
                    == AlignmentType::Map
                    && self_.layout.get::<SymbolPlacement>() != SymbolPlacementType::Point
                    && ft.formattedText.allowsVerticalWritingMode();

                // Loop through all characters of this text and collect unique codepoints.
                for j in 0..ft.formattedText.len() {
                    let section = formatted.sections[ft.formattedText.getSectionIndex(j)];
                    if (section.image) {
                        continue;
                    }

                    let sectionFontStack = section.fontStack;
                    let dependencies: &GlyphIDs = layoutParameters.glyphDependencies
                        [if sectionFontStack? {
                            *sectionFontStack
                        } else {
                            baseFontStack
                        }];
                    let codePoint: Char16 = ft.formattedText.getCharCodeAt(j);
                    dependencies.insert(codePoint);
                    if (canVerticalizeText
                        || (self_.allowVerticalPlacement
                            && ft.formattedText.allowsVerticalWritingMode()))
                    {
                        let verticalChr: Char16 = util::i18n::verticalizePunctuation(codePoint);
                        if (verticalChr) {
                            dependencies.insert(verticalChr);
                        }
                    }
                }
            }

            if (hasIcon) {
                ft.icon = self_.layout.evaluate::<IconImage>(
                    zoom,
                    ft,
                    layoutParameters.availableImages,
                    canonicalID,
                );
                layoutParameters
                    .imageDependencies
                    .emplace(ft.icon.id(), ImageType::Icon);
            }

            if (ft.formattedText || ft.icon) {
                if (self_.sortFeaturesByKey) {
                    ft.sortKey = self_
                        .layout
                        .evaluate::<SymbolSortKey>(zoom, ft, canonicalID);
                    let lowerBound = std::lower_bound(features.begin(), features.end(), ft);
                    self_.features.insert(lowerBound, ft);
                } else {
                    self_.features.push_back(ft);
                }
            }
        }

        if (self_.layout.get::<SymbolPlacement>() == SymbolPlacementType::Line) {
            util::mergeLines(self_.features);
        }
    }
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
            if let Some(mut feature_formattedText) = feature.formattedText.clone() {
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
                                            textJustify: TextJustifyType,
                                            textOffset: &[f64; 2]|
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
                            /* translate */ textOffset,
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

                    let mut addVerticalShapingForPointLabelIfNeeded =
                        |shapedTextOrientations: &mut ShapedTextOrientations,
                         feature_formattedText: &mut TaggedString| {
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
                                    &feature_formattedText,
                                    WritingModeType::Vertical,
                                    textAnchor,
                                    TextJustifyType::Left,
                                    &textOffset,
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
                                &feature_formattedText,
                                WritingModeType::Horizontal,
                                SymbolAnchorType::Center,
                                justification,
                                &textOffset,
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
                        addVerticalShapingForPointLabelIfNeeded(
                            &mut shapedTextOrientations,
                            &mut feature_formattedText,
                        );
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
                            &textOffset,
                        );
                        if (shaping.isAnyLineNotEmpty()) {
                            shapedTextOrientations.horizontal = shaping;
                        }

                        // Vertical point label shaping if allowVerticalPlacement is enabled.
                        addVerticalShapingForPointLabelIfNeeded(
                            &mut shapedTextOrientations,
                            &mut feature_formattedText,
                        );

                        // Verticalized line label.
                        if (textAlongLine && feature_formattedText.allowsVerticalWritingMode()) {
                            feature_formattedText.verticalizePunctuation();
                            shapedTextOrientations.vertical = applyShaping(
                                &feature_formattedText,
                                WritingModeType::Vertical,
                                textAnchor,
                                textJustify,
                                &textOffset,
                            );
                        }
                    }
                }

                feature.formattedText = Some(feature_formattedText);
            }

            // if feature has icon, get sprite atlas position
            let mut iconType: SymbolContent = SymbolContent::None;
            if let Some(icon) = &feature.icon {
                let image = imageMap.get(&icon.imageID);
                if let Some(image) = image {
                    iconType = SymbolContent::IconRGBA;
                    shapedIcon = Some(PositionedIcon::shapeIcon(
                        imagePositions.get(&icon.imageID).unwrap().clone(),
                        &self
                            .layout
                            .evaluate::<IconOffset>(self.zoom, feature, self.canonicalID),
                        self.layout
                            .evaluate::<IconAnchor>(self.zoom, feature, self.canonicalID),
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
                // TODO borrow conflict with self.features
                //self.addFeature(
                //    feature_index,
                //    feature,
                //    &shapedTextOrientations,
                //    shapedIcon,
                //    imageMap,
                //    textOffset,
                //    layoutTextSize,
                //    layoutIconSize,
                //    iconType,
                //);
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
        let bucket = std::make_shared::<SymbolBucket>(
            layout,
            layerPaintProperties,
            textSize,
            iconSize,
            zoom,
            iconsNeedLinear,
            sortFeaturesByY,
            bucketLeaderID,
            symbolInstances,
            sortKeyRanges,
            tilePixelRatio,
            allowVerticalPlacement,
            placementModes,
            iconsInText,
        );

        for symbolInstance in bucket.symbolInstances {
            let hasText = symbolInstance.hasText();
            let hasIcon = symbolInstance.hasIcon();
            let singleLine = symbolInstance.singleLine;

            let feature = self.features.at(symbolInstance.layoutFeatureIndex);

            // Insert final placement into collision tree and add glyphs/icons to buffers

            // Process icon first, so that text symbols would have reference to
            // iconIndex which is used when dynamic vertices for icon-text-fit image
            // have to be updated.
            if (hasIcon) {
                let sizeData: Range<f64> = bucket.iconSizeBinder.getVertexSizeData(feature);
                let iconBuffer = if symbolInstance.hasSdfIcon() {
                    bucket.sdfIcon
                } else {
                    bucket.icon
                };
                let placeIcon = |iconQuads: &SymbolQuads, index, writingMode: WritingModeType| {
                    iconBuffer.placedSymbols.emplace_back(
                        symbolInstance.anchor.point,
                        symbolInstance.anchor.segment.value_or(0),
                        sizeData.min,
                        sizeData.max,
                        symbolInstance.iconOffset,
                        writingMode,
                        symbolInstance.line(),
                        Vec::new(),
                    );
                    index = iconBuffer.placedSymbols.size() - 1;
                    let iconSymbol: PlacedSymbol = iconBuffer.placedSymbols.back();
                    iconSymbol.angle = if (self.allowVerticalPlacement
                        && writingMode == WritingModeType::Vertical)
                    {
                        PI / 2.
                    } else {
                        0.0
                    };
                    iconSymbol.vertexStartIndex = self.addSymbols(
                        iconBuffer,
                        sizeData,
                        iconQuads,
                        symbolInstance.anchor,
                        iconSymbol,
                        feature.sortKey,
                    );
                };

                placeIcon(
                    *symbolInstance.iconQuads(),
                    symbolInstance.placedIconIndex,
                    WritingModeType::None,
                );
                if (symbolInstance.verticalIconQuads()) {
                    placeIcon(
                        *symbolInstance.verticalIconQuads(),
                        symbolInstance.placedVerticalIconIndex,
                        WritingModeType::Vertical,
                    );
                }

                for pair in bucket.paintProperties {
                    pair.second.iconBinders.populateVertexVectors(
                        feature,
                        iconBuffer.vertices().elements(),
                        symbolInstance.dataFeatureIndex,
                        {},
                        {},
                        canonical,
                    );
                }
            }

            if (hasText && feature.formattedText) {
                let lastAddedSection: Option<usize>;
                if (singleLine) {
                    let placedTextIndex: Option<usize>;
                    lastAddedSection = self.addSymbolGlyphQuads(
                        *bucket,
                        symbolInstance,
                        feature,
                        symbolInstance.writingModes,
                        placedTextIndex,
                        symbolInstance.rightJustifiedGlyphQuads(),
                        canonical,
                        lastAddedSection,
                    );
                    symbolInstance.placedRightTextIndex = placedTextIndex;
                    symbolInstance.placedCenterTextIndex = placedTextIndex;
                    symbolInstance.placedLeftTextIndex = placedTextIndex;
                } else {
                    if (symbolInstance.rightJustifiedGlyphQuadsSize) {
                        lastAddedSection = self.addSymbolGlyphQuads(
                            *bucket,
                            symbolInstance,
                            feature,
                            symbolInstance.writingModes,
                            symbolInstance.placedRightTextIndex,
                            symbolInstance.rightJustifiedGlyphQuads(),
                            canonical,
                            lastAddedSection,
                        );
                    }
                    if (symbolInstance.centerJustifiedGlyphQuadsSize) {
                        lastAddedSection = self.addSymbolGlyphQuads(
                            *bucket,
                            symbolInstance,
                            feature,
                            symbolInstance.writingModes,
                            symbolInstance.placedCenterTextIndex,
                            symbolInstance.centerJustifiedGlyphQuads(),
                            canonical,
                            lastAddedSection,
                        );
                    }
                    if (symbolInstance.leftJustifiedGlyphQuadsSize) {
                        lastAddedSection = self.addSymbolGlyphQuads(
                            *bucket,
                            symbolInstance,
                            feature,
                            symbolInstance.writingModes,
                            symbolInstance.placedLeftTextIndex,
                            symbolInstance.leftJustifiedGlyphQuads(),
                            canonical,
                            lastAddedSection,
                        );
                    }
                }
                if (symbolInstance.writingModes & WritingModeType::Vertical
                    && symbolInstance.verticalGlyphQuadsSize)
                {
                    lastAddedSection = self.addSymbolGlyphQuads(
                        *bucket,
                        symbolInstance,
                        feature,
                        WritingModeType::Vertical,
                        symbolInstance.placedVerticalTextIndex,
                        symbolInstance.verticalGlyphQuads(),
                        canonical,
                        lastAddedSection,
                    );
                }
                assert!(lastAddedSection); // True, as hasText == true;
                self.updatePaintPropertiesForSection(
                    *bucket,
                    feature,
                    *lastAddedSection,
                    canonical,
                );
            }

            symbolInstance.releaseSharedData();
        }

        if (showCollisionBoxes) {
            self.addToDebugBuffers(*bucket);
        }
        if (bucket.hasData()) {
            for pair in self.layerPaintProperties {
                if (!firstLoad) {
                    bucket.justReloaded = true;
                }
                renderData.emplace(pair.first, LayerRenderData::new(bucket, pair.second));
            }
        }
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
        let minScale = 0.5;
        let glyphSize = 24.0;

        let iconOffset: [f64; 2] =
            self.layout
                .evaluate::<IconOffset>(self.zoom, feature, self.canonicalID);

        // To reduce the number of labels that jump around when zooming we need
        // to use a text-size value that is the same for all zoom levels.
        // This calculates text-size at a high zoom level so that all tiles can
        // use the same value when calculating anchor positions.
        let textMaxSize = self
            .layout
            .evaluate::<TextSize>(18., feature, self.canonicalID);

        let fontScale = layoutTextSize / glyphSize;
        let textBoxScale = self.tilePixelRatio * fontScale;
        let textMaxBoxScale = self.tilePixelRatio * textMaxSize / glyphSize;
        let iconBoxScale = self.tilePixelRatio * layoutIconSize;
        let symbolSpacing = self.tilePixelRatio * self.layout.get::<SymbolSpacing>();
        let textPadding = self.layout.get::<TextPadding>() * self.tilePixelRatio;
        let iconPadding = self.layout.get::<IconPadding>() * self.tilePixelRatio;
        let textMaxAngle = deg2radf(self.layout.get::<TextMaxAngle>());
        let iconRotation = self
            .layout
            .evaluate::<IconRotate>(self.zoom, feature, self.canonicalID);
        let textRotation = self
            .layout
            .evaluate::<TextRotate>(self.zoom, feature, self.canonicalID);
        let variableTextOffset: [f64; 2];
        if (!self.textRadialOffset.isUndefined()) {
            variableTextOffset = [
                self.layout
                    .evaluate::<TextRadialOffset>(self.zoom, feature, self.canonicalID)
                    * ONE_EM,
                INVALID_OFFSET_VALUE,
            ];
        } else {
            variableTextOffset = [
                self.layout
                    .evaluate::<TextOffset>(self.zoom, feature, self.canonicalID)[0]
                    * ONE_EM,
                self.layout
                    .evaluate::<TextOffset>(self.zoom, feature, self.canonicalID)[1]
                    * ONE_EM,
            ];
        }

        let textPlacement: SymbolPlacementType =
            if self.layout.get::<TextRotationAlignment>() != AlignmentType::Map {
                SymbolPlacementType::Point
            } else {
                self.layout.get::<SymbolPlacement>()
            };

        let textRepeatDistance: f64 = symbolSpacing / 2;
        let evaluatedLayoutProperties = self.layout.evaluate(self.zoom, feature);
        let indexedFeature = RefIndexedSubfeature::new(
            feature.index,
            self.sourceLayer.getName(),
            &self.bucketLeaderID,
            self.symbolInstances.len(),
        );

        let iconTextFit = evaluatedLayoutProperties.get::<IconTextFit>();
        let hasIconTextFit = iconTextFit != IconTextFitType::None;
        // Adjust shaped icon size when icon-text-fit is used.
        let verticallyShapedIcon: Option<PositionedIcon>;
        if (shapedIcon && hasIconTextFit) {
            // Create vertically shaped icon for vertical writing mode if needed.
            if (self.allowVerticalPlacement && shapedTextOrientations.vertical) {
                verticallyShapedIcon = shapedIcon;
                verticallyShapedIcon.fitIconToText(
                    shapedTextOrientations.vertical,
                    iconTextFit,
                    self.layout.get::<IconTextFitPadding>(),
                    iconOffset,
                    fontScale,
                );
            }
            let shapedText = getDefaultHorizontalShaping(shapedTextOrientations);
            if (shapedText) {
                shapedIcon.fitIconToText(
                    shapedText,
                    iconTextFit,
                    self.layout.get::<IconTextFitPadding>(),
                    iconOffset,
                    fontScale,
                );
            }
        }

        let addSymbolInstance = |anchor: &Anchor, sharedData: Rc<SymbolInstanceSharedData>| {
            assert!(sharedData);
            let anchorInsideTile = anchor.point.x >= 0
                && anchor.point.x < EXTENT
                && anchor.point.y >= 0
                && anchor.point.y < EXTENT;

            if (self.mode == MapMode::Tile || anchorInsideTile) {
                // For static/continuous rendering, only add symbols anchored within this tile:
                //  neighboring symbols will be added as part of the neighboring tiles.
                // In tiled rendering mode, add all symbols in the buffers so that we can:
                //  (1) render symbols that overlap into this tile
                //  (2) approximate collision detection effects from neighboring symbols
                self.symbolInstances.emplace_back(
                    anchor,
                    sharedData,
                    shapedTextOrientations,
                    shapedIcon,
                    verticallyShapedIcon,
                    textBoxScale,
                    textPadding,
                    textPlacement,
                    textOffset,
                    iconBoxScale,
                    iconPadding,
                    iconOffset,
                    indexedFeature,
                    layoutFeatureIndex,
                    feature.index,
                    if let Some(formattedText) = &feature.formattedText {
                        formattedText.rawText()
                    } else {
                        &U16String::new()
                    },
                    self.overscaling,
                    iconRotation,
                    textRotation,
                    variableTextOffset,
                    self.allowVerticalPlacement,
                    iconType,
                );

                if (self.sortFeaturesByKey) {
                    if (!self.sortKeyRanges.empty()
                        && self.sortKeyRanges.back().sortKey == feature.sortKey)
                    {
                        self.sortKeyRanges.back().end = self.symbolInstances.size();
                    } else {
                        self.sortKeyRanges.push_back(SortKeyRange::new(
                            feature.sortKey,
                            self.symbolInstances.size() - 1,
                            self.symbolInstances.size(),
                        ));
                    }
                }
            }
        };

        let createSymbolInstanceSharedData = |line: GeometryCoordinates| {
            return std::make_shared::<SymbolInstanceSharedData>(
                line,
                shapedTextOrientations,
                shapedIcon,
                verticallyShapedIcon,
                evaluatedLayoutProperties,
                textPlacement,
                textOffset,
                imageMap,
                iconRotation,
                iconType,
                hasIconTextFit,
                self.allowVerticalPlacement,
            );
        };

        let type_ = feature.getType();

        if (self.layout.get::<SymbolPlacement>() == SymbolPlacementType::Line) {
            let clippedLines = util::clipLines(feature.geometry, 0, 0, EXTENT, EXTENT);
            for line in clippedLines {
                let anchors: Anchors = getAnchors(
                    line,
                    symbolSpacing,
                    textMaxAngle,
                    (if shapedTextOrientations.vertical {
                        shapedTextOrientations.vertical
                    } else {
                        getDefaultHorizontalShaping(shapedTextOrientations)
                    })
                    .left,
                    (if shapedTextOrientations.vertical {
                        shapedTextOrientations.vertical
                    } else {
                        getDefaultHorizontalShaping(shapedTextOrientations)
                    })
                    .right,
                    (if shapedIcon { shapedIcon.left() } else { 0 }),
                    (if shapedIcon { shapedIcon.right() } else { 0 }),
                    glyphSize,
                    textMaxBoxScale,
                    self.overscaling,
                );
                let sharedData = createSymbolInstanceSharedData(line);
                for anchor in anchors {
                    if (!feature.formattedText
                        || !self.anchorIsTooClose(
                            feature.formattedText.rawText(),
                            textRepeatDistance,
                            anchor,
                        ))
                    {
                        addSymbolInstance(anchor, sharedData);
                    }
                }
            }
        } else if (self.layout.get::<SymbolPlacement>() == SymbolPlacementType::LineCenter) {
            // No clipping, multiple lines per feature are allowed
            // "lines" with only one point are ignored as in clipLines
            for line in feature.geometry {
                if (line.size() > 1) {
                    let anchor: Option<Anchor> = getCenterAnchor(
                        line,
                        textMaxAngle,
                        (if shapedTextOrientations.vertical {
                            shapedTextOrientations.vertical
                        } else {
                            getDefaultHorizontalShaping(shapedTextOrientations)
                        })
                        .left,
                        (if shapedTextOrientations.vertical {
                            shapedTextOrientations.vertical
                        } else {
                            getDefaultHorizontalShaping(shapedTextOrientations)
                        })
                        .right,
                        (if shapedIcon { shapedIcon.left() } else { 0 }),
                        (if shapedIcon { shapedIcon.right() } else { 0 }),
                        glyphSize,
                        textMaxBoxScale,
                    );
                    if (anchor) {
                        addSymbolInstance(*anchor, createSymbolInstanceSharedData(line));
                    }
                }
            }
        } else if (type_ == FeatureType::Polygon) {
            for polygon in classifyRings(feature.geometry) {
                let poly: Polygon<f64>;
                for ring in polygon {
                    let r: LinearRing<f64>;
                    for p in ring {
                        r.push_back(convertPoint::<double>(p));
                    }
                    poly.push_back(r);
                }

                // 1 pixel worth of precision, in tile coordinates
                let poi = mapbox::polylabel(poly, EXTENT / util::tileSize_D);
                let anchor = Anchor::new((poi.x) as f64, (poi.y) as f64, 0.0f, (minScale) as usize);
                addSymbolInstance(anchor, createSymbolInstanceSharedData(polygon[0]));
            }
        } else if (type_ == FeatureType::LineString) {
            for line in feature.geometry {
                // Skip invalid LineStrings.
                if (line.is_empty()) {
                    continue;
                }

                let anchor = Anchor::new(
                    (line[0].x) as f64,
                    (line[0].y) as f64,
                    0.0,
                    (minScale) as usize,
                );
                addSymbolInstance(anchor, createSymbolInstanceSharedData(line));
            }
        } else if (type_ == FeatureType::Point) {
            for points in feature.geometry {
                for point in points {
                    let anchor =
                        Anchor::new((point.x) as f64, (point.y) as f64, 0.0, (minScale) as usize);
                    addSymbolInstance(anchor, createSymbolInstanceSharedData({ point }));
                }
            }
        }
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
        let vertexLength: u16 = 4;

        let tl = symbol.tl;
        let tr = symbol.tr;
        let bl = symbol.bl;
        let br = symbol.br;
        let tex = symbol.tex;
        let pixelOffsetTL = symbol.pixelOffsetTL;
        let pixelOffsetBR = symbol.pixelOffsetBR;
        let minFontScale = symbol.minFontScale;

        if (buffer.segments.empty()
            || buffer.segments.back().vertexLength + vertexLength
                > std::numeric_limits::<uint16_t>::max()
            || std::fabs(buffer.segments.back().sortKey - sortKey)
                > std::numeric_limits::<float>::epsilon())
        {
            buffer.segments.emplace_back(
                buffer.vertices().elements(),
                buffer.triangles.elements(),
                0,
                0,
                sortKey,
            );
        }

        // We're generating triangle fans, so we always start with the first
        // coordinate in this polygon.
        let segment = buffer.segments.back();
        assert!(segment.vertexLength <= u16::MAX);
        let index = (segment.vertexLength) as u16;

        // coordinates (2 triangles)
        let vertices = buffer.vertices();
        vertices.emplace_back(SymbolSDFIconProgram::layoutVertex(
            labelAnchor.point,
            tl,
            symbol.glyphOffset.y,
            tex.x,
            tex.y,
            sizeData,
            symbol.isSDF,
            pixelOffsetTL,
            minFontScale,
        ));
        vertices.emplace_back(SymbolSDFIconProgram::layoutVertex(
            labelAnchor.point,
            tr,
            symbol.glyphOffset.y,
            tex.x + tex.w,
            tex.y,
            sizeData,
            symbol.isSDF,
            [pixelOffsetBR.x, pixelOffsetTL.y],
            minFontScale,
        ));
        vertices.emplace_back(SymbolSDFIconProgram::layoutVertex(
            labelAnchor.point,
            bl,
            symbol.glyphOffset.y,
            tex.x,
            tex.y + tex.h,
            sizeData,
            symbol.isSDF,
            [pixelOffsetTL.x, pixelOffsetBR.y],
            minFontScale,
        ));
        vertices.emplace_back(SymbolSDFIconProgram::layoutVertex(
            labelAnchor.point,
            br,
            symbol.glyphOffset.y,
            tex.x + tex.w,
            tex.y + tex.h,
            sizeData,
            symbol.isSDF,
            pixelOffsetBR,
            minFontScale,
        ));

        // Dynamic/Opacity vertices are initialized so that the vertex count always
        // agrees with the layout vertex buffer, but they will always be updated
        // before rendering happens
        let dynamicVertex = SymbolSDFIconProgram::dynamicLayoutVertex(labelAnchor.point, 0);
        buffer.dynamicVertices().emplace_back(dynamicVertex);
        buffer.dynamicVertices().emplace_back(dynamicVertex);
        buffer.dynamicVertices().emplace_back(dynamicVertex);
        buffer.dynamicVertices().emplace_back(dynamicVertex);

        let opacityVertex = SymbolSDFIconProgram::opacityVertex(1.0, 1.0);
        buffer.opacityVertices().emplace_back(opacityVertex);
        buffer.opacityVertices().emplace_back(opacityVertex);
        buffer.opacityVertices().emplace_back(opacityVertex);
        buffer.opacityVertices().emplace_back(opacityVertex);

        // add the two triangles, referencing the four coordinates we just inserted.
        buffer
            .triangles
            .emplace_back(index + 0, index + 1, index + 2);
        buffer
            .triangles
            .emplace_back(index + 1, index + 2, index + 3);

        segment.vertexLength += vertexLength;
        segment.indexLength += 6;

        placedSymbol.glyphOffsets.push_back(symbol.glyphOffset.x);

        return index;
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
        let mut firstSymbol = true;
        let mut firstIndex = 0;
        for symbol in symbols {
            let index = self.addSymbol(
                buffer,
                sizeData.clone(),
                symbol,
                labelAnchor,
                placedSymbol,
                sortKey,
            );
            if (firstSymbol) {
                firstIndex = index;
                firstSymbol = false;
            }
        }
        return firstIndex;
    }

    // Adds symbol quads to bucket and returns formatted section index of last
    // added quad.
    fn addSymbolGlyphQuads(
        &self,
        bucket: &SymbolBucket,
        symbolInstance: &SymbolInstance,
        feature: &SymbolFeature,
        writingMode: WritingModeType,
        placedIndex: Option<usize>,
        glyphQuads: &SymbolQuads,
        canonical: &CanonicalTileID,
        lastAddedSection: Option<usize>,
    ) -> usize {
        let sizeData: Range<f64> = bucket.textSizeBinder.getVertexSizeData(feature);
        let hasFormatSectionOverrides: bool = bucket.hasFormatSectionOverrides();
        let placedIconIndex = if writingMode == WritingModeType::Vertical {
            symbolInstance.placedVerticalIconIndex
        } else {
            symbolInstance.placedIconIndex
        };
        bucket.text.placedSymbols.emplace_back(
            symbolInstance.anchor.point,
            symbolInstance.anchor.segment.value_or(0),
            sizeData.min,
            sizeData.max,
            symbolInstance.textOffset,
            writingMode,
            symbolInstance.line(),
            Self::calculateTileDistances(symbolInstance.line(), symbolInstance.anchor),
            placedIconIndex,
        );
        placedIndex = bucket.text.placedSymbols.size() - 1;
        let placedSymbol: &PlacedSymbol = bucket.text.placedSymbols.back();
        placedSymbol.angle =
            if (self.allowVerticalPlacement && writingMode == WritingModeType::Vertical) {
                PI / 2.
            } else {
                0.0
            };

        let mut firstSymbol = true;
        for symbolQuad in glyphQuads {
            if (hasFormatSectionOverrides) {
                if (lastAddedSection && *lastAddedSection != symbolQuad.sectionIndex) {
                    self.updatePaintPropertiesForSection(
                        bucket,
                        feature,
                        *lastAddedSection,
                        canonical,
                    );
                }
                lastAddedSection = symbolQuad.sectionIndex;
            }
            let index = self.addSymbol(
                bucket.text,
                sizeData.clone(),
                symbolQuad,
                &symbolInstance.anchor,
                placedSymbol,
                feature.sortKey,
            );
            if (firstSymbol) {
                placedSymbol.vertexStartIndex = index;
                firstSymbol = false;
            }
        }

        return if lastAddedSection {
            *lastAddedSection
        } else {
            0
        };
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
