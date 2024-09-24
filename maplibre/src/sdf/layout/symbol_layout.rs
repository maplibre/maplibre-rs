use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
    f64::consts::PI,
    ops::Range,
    rc::Rc,
};

use lyon::geom::euclid::Point2D;
use widestring::U16String;

use crate::{
    coords::{EXTENT, TILE_SIZE},
    sdf::{
        bidi::{applyArabicShaping, BiDi, Char16},
        buckets::symbol_bucket::{
            DynamicVertex, OpacityVertex, PlacedSymbol, Segment, SymbolBucket, SymbolBucketBuffer,
            SymbolVertex,
        },
        geometry::{
            feature_index::{IndexedSubfeature, RefIndexedSubfeature},
            Anchor, Anchors,
        },
        geometry_tile_data::{FeatureType, GeometryCoordinates, SymbolGeometryTileLayer},
        glyph::{GlyphIDs, GlyphMap, Shaping, WritingModeType},
        glyph_atlas::GlyphPositions,
        image::{ImageMap, ImageType},
        image_atlas::ImagePositions,
        layout::{
            layout::{BucketParameters, LayoutParameters},
            symbol_feature::SymbolGeometryTileFeature,
            symbol_instance::{
                ShapedTextOrientations, SymbolContent, SymbolInstance, SymbolInstanceSharedData,
            },
        },
        quads::{SymbolQuad, SymbolQuads},
        shaping::{getAnchorJustification, getShaping, PositionedIcon},
        style_types::*,
        tagged_string::{SectionOptions, TaggedString},
        util::{constants::ONE_EM, i18n, lower_bound, math::deg2radf},
        CanonicalTileID, MapMode,
    },
};

// TODO
#[derive(Clone, Debug)]
pub struct SymbolLayer {
    layout: SymbolLayoutProperties_Unevaluated,
}
pub type SymbolLayer_Impl = SymbolLayer;
// TODO
#[derive(Clone, Debug)]
pub struct LayerProperties {
    pub id: String,
    pub layer: SymbolLayer_Impl,
}
pub type SymbolLayerProperties = LayerProperties;
impl LayerProperties {
    pub fn layerImpl(&self) -> &SymbolLayer_Impl {
        // TODO
        &self.layer
    }

    pub fn baseImpl(&self) -> &Self {
        self
    }
}
pub type Bucket = SymbolBucket;

#[derive(Debug)]
pub struct LayerRenderData {
    bucket: Rc<Bucket>,
    layerProperties: LayerProperties,
}

#[derive(Clone, Copy, Debug)]
pub struct SortKeyRange {
    sortKey: f64,
    start: usize,
    end: usize,
}

impl SortKeyRange {
    pub fn isFirstRange(&self) -> bool {
        return self.start == 0;
    }
}

// index
struct FeatureIndex;

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
    // TODO
    //return static_cast<const SymbolLayerProperties&>(*layer);
    return layer;
}

fn createLayout(
    unevaluated: &SymbolLayoutProperties_Unevaluated,
    zoom: f64,
) -> SymbolLayoutProperties_PossiblyEvaluated {
    let mut layout = unevaluated.evaluate(PropertyEvaluationParameters(zoom));

    if (layout.get::<IconRotationAlignment>() == AlignmentType::Auto) {
        if (layout.get::<SymbolPlacement>() != SymbolPlacementType::Point) {
            layout.set::<IconRotationAlignment>(AlignmentType::Map);
        } else {
            layout.set::<IconRotationAlignment>(AlignmentType::Viewport);
        }
    }

    if (layout.get::<TextRotationAlignment>() == AlignmentType::Auto) {
        if (layout.get::<SymbolPlacement>() != SymbolPlacementType::Point) {
            layout.set::<TextRotationAlignment>(AlignmentType::Map);
        } else {
            layout.set::<TextRotationAlignment>(AlignmentType::Viewport);
        }
    }

    // If unspecified `*-pitch-alignment` inherits `*-rotation-alignment`
    if (layout.get::<TextPitchAlignment>() == AlignmentType::Auto) {
        layout.set::<TextPitchAlignment>(layout.get::<TextRotationAlignment>());
    }
    if (layout.get::<IconPitchAlignment>() == AlignmentType::Auto) {
        layout.set::<IconPitchAlignment>(layout.get::<IconRotationAlignment>());
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
    if shapedTextOrientations.right().isAnyLineNotEmpty() {
        return &shapedTextOrientations.right();
    }
    if shapedTextOrientations.center().isAnyLineNotEmpty() {
        return &shapedTextOrientations.center();
    }
    if shapedTextOrientations.left().isAnyLineNotEmpty() {
        return &shapedTextOrientations.left();
    }
    return &shapedTextOrientations.horizontal();
}

fn shapingForTextJustifyType(
    shapedTextOrientations: &ShapedTextOrientations,
    type_: TextJustifyType,
) -> &Shaping {
    match (type_) {
        TextJustifyType::Right => {
            return &shapedTextOrientations.right();
        }

        TextJustifyType::Left => {
            return &shapedTextOrientations.left();
        }

        TextJustifyType::Center => {
            return &shapedTextOrientations.center();
        }
        _ => {
            assert!(false);
            return &shapedTextOrientations.horizontal();
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
    sourceLayer: Box<SymbolGeometryTileLayer>,
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
    features: Vec<SymbolGeometryTileFeature>,

    bidi: BiDi, // Consider moving this up to geometry tile worker to reduce
    // reinstantiation costs, use of BiDi/ubiditransform object must
    // be rained to one thread
    compareText: BTreeMap<U16String, Vec<Anchor>>,
}

impl SymbolLayout {
    pub fn new(
        parameters: &BucketParameters,
        layers: &Vec<LayerProperties>,
        sourceLayer_: Box<SymbolGeometryTileLayer>,
        layoutParameters: &mut LayoutParameters, // TODO is this output?
    ) -> Option<Self> {
        let overscaling = parameters.tileID.overscaleFactor() as f64;
        let zoom = parameters.tileID.overscaledZ as f64;
        let tileSize = (TILE_SIZE * overscaling) as u32;

        let leader: &SymbolLayer_Impl = toSymbolLayerProperties(layers.get(0).unwrap()).layerImpl();

        let mut self_ = Self {
            bucketLeaderID: layers.first().unwrap().baseImpl().id.clone(),

            sourceLayer: sourceLayer_,
            overscaling,
            zoom,
            canonicalID: parameters.tileID.canonical,
            mode: parameters.mode,
            pixelRatio: parameters.pixelRatio,
            tileSize,
            tilePixelRatio: EXTENT / tileSize as f64,
            layout: createLayout(
                &toSymbolLayerProperties(layers.get(0).unwrap())
                    .layerImpl()
                    .layout,
                zoom,
            ),
            textSize: leader.layout.get_dynamic::<TextSize>(),
            iconSize: leader.layout.get_dynamic::<IconSize>(),
            textRadialOffset: leader.layout.get_dynamic::<TextRadialOffset>(),

            // default values
            layerPaintProperties: Default::default(),
            symbolInstances: vec![],
            sortKeyRanges: vec![],
            iconsNeedLinear: false,
            sortFeaturesByY: false,
            sortFeaturesByKey: false,
            allowVerticalPlacement: false,
            iconsInText: false,
            placementModes: vec![],
            features: vec![],
            bidi: BiDi,
            compareText: Default::default(),
        };

        let hasText = self_.layout.has::<TextField>() && self_.layout.has::<TextFont>();
        let hasIcon = self_.layout.has::<IconImage>();

        if (!hasText && !hasIcon) {
            return None;
        }

        let hasSymbolSortKey = !leader.layout.get_dynamic::<SymbolSortKey>().isUndefined();
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
            let mut modes = self_.layout.get::<TextWritingMode>();

            // Remove duplicates and preserve order.
            // TODO Verify if this is correct. Maybe make this better.
            let mut seen: BTreeSet<TextWritingModeType> = BTreeSet::new();
            modes = modes
                .iter()
                .filter(|placementMode| {
                    self_.allowVerticalPlacement = self_.allowVerticalPlacement
                        || **placementMode == TextWritingModeType::Vertical;
                    return seen.insert(**placementMode);
                })
                .cloned()
                .collect();

            self_.placementModes = modes;
        }

        for layer in layers {
            self_
                .layerPaintProperties
                .insert(layer.baseImpl().id.clone(), layer.clone());
        }

        // Determine glyph dependencies
        let featureCount = self_.sourceLayer.featureCount();
        for i in 0..featureCount {
            let feature = self_.sourceLayer.getFeature(i);

            // TODO
            //if (!leader.filter(expression::EvaluationContext::new(self_.zoom, feature.get()).withCanonicalTileID(&parameters.tileID.canonical), )) {
            //    continue;
            //}

            let mut ft: SymbolGeometryTileFeature = *feature.clone();

            ft.index = i;

            if (hasText) {
                let formatted = self_.layout.evaluate4::<TextField>(
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

                ft.formattedText = Some(TaggedString::default());
                let ft_formattedText = ft.formattedText.as_mut().unwrap();
                for section in &formatted.sections {
                    if let Some(image) = (&section.image) {
                        layoutParameters
                            .imageDependencies
                            .insert(image.imageID.clone(), ImageType::Icon);
                        ft_formattedText.addImageSection(image.imageID.clone());
                    } else {
                        let mut u8string = section.text.clone();
                        if (textTransform == TextTransformType::Uppercase) {
                            u8string = u8string.to_uppercase();
                        } else if (textTransform == TextTransformType::Lowercase) {
                            u8string = u8string.to_lowercase();
                        }

                        // TODO seems like invalid UTF-8 can not be in a tile? if let Err(e) =
                        ft_formattedText.addTextSection(
                            &applyArabicShaping(&U16String::from(u8string.as_str())),
                            if let Some(fontScale) = section.fontScale {
                                fontScale
                            } else {
                                1.0
                            },
                            if let Some(fontStack) = &section.fontStack {
                                fontStack.clone()
                            } else {
                                baseFontStack.clone()
                            },
                            section.textColor.clone(),
                        )
                        //{
                        //    log::error!("Encountered section with invalid UTF-8 in tile, source: {} z: {} x: {} y: {}", self_.sourceLayer.getName(), self_.canonicalID.z, self_.canonicalID.x, self_.canonicalID.y);
                        //    continue; // skip section
                        //}
                    }
                }

                let canVerticalizeText = self_.layout.get::<TextRotationAlignment>()
                    == AlignmentType::Map
                    && self_.layout.get::<SymbolPlacement>() != SymbolPlacementType::Point
                    && ft_formattedText.allowsVerticalWritingMode();

                // Loop through all characters of this text and collect unique codepoints.
                for j in 0..ft_formattedText.length() {
                    let section = &formatted.sections[ft_formattedText.getSectionIndex(j) as usize];
                    if (section.image.is_some()) {
                        continue;
                    }

                    let sectionFontStack = &section.fontStack;
                    let dependencies: &mut GlyphIDs = &mut layoutParameters
                        .glyphDependencies
                        .entry(if let Some(sectionFontStack) = sectionFontStack {
                            sectionFontStack.clone()
                        } else {
                            baseFontStack.clone()
                        })
                        .or_default(); // TODO this is different in C++, as C++ always creates the default apparently
                    let codePoint: Char16 = ft_formattedText.getCharCodeAt(j);
                    dependencies.insert(codePoint);
                    if (canVerticalizeText
                        || (self_.allowVerticalPlacement
                            && ft_formattedText.allowsVerticalWritingMode()))
                    {
                        let verticalChr: Char16 = i18n::verticalizePunctuation(codePoint);
                        if (verticalChr != 0) {
                            dependencies.insert(verticalChr);
                        }
                    }
                }
            }

            if (hasIcon) {
                ft.icon = Some(self_.layout.evaluate4::<IconImage>(
                    self_.zoom,
                    &ft,
                    layoutParameters.availableImages,
                    self_.canonicalID,
                )); // TODO it might be that this is None?
                layoutParameters
                    .imageDependencies
                    .insert(ft.icon.as_ref().unwrap().imageID.clone(), ImageType::Icon);
            }

            if (ft.formattedText.is_some() || ft.icon.is_some()) {
                if (self_.sortFeaturesByKey) {
                    ft.sortKey =
                        self_
                            .layout
                            .evaluate::<SymbolSortKey>(self_.zoom, &ft, self_.canonicalID);

                    let lowerBound = lower_bound(&self_.features, &ft);
                    self_.features.insert(lowerBound, ft);
                } else {
                    self_.features.push(ft);
                }
            }
        }

        if (self_.layout.get::<SymbolPlacement>() == SymbolPlacementType::Line) {
            todo!()
            // TODO mergeLines(self_.features);
        }

        Some(self_)
    }
    pub fn prepareSymbols(
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

        let mut toProcessFeatures = Vec::new();

        for (feature_index, feature) in self.features.iter_mut().enumerate() {
            // TODO expensive clone
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
                                shapedTextOrientations.set_vertical(applyShaping(
                                    &feature_formattedText,
                                    WritingModeType::Vertical,
                                    textAnchor,
                                    TextJustifyType::Left,
                                    &textOffset,
                                ))
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
                            shapedTextOrientations.set_horizontal(shaping)
                        }

                        // Vertical point label shaping if allowVerticalPlacement is enabled.
                        addVerticalShapingForPointLabelIfNeeded(
                            &mut shapedTextOrientations,
                            &mut feature_formattedText,
                        );

                        // Verticalized line label.
                        if (textAlongLine && feature_formattedText.allowsVerticalWritingMode()) {
                            feature_formattedText.verticalizePunctuation();
                            shapedTextOrientations.set_vertical(applyShaping(
                                &feature_formattedText,
                                WritingModeType::Vertical,
                                textAnchor,
                                textJustify,
                                &textOffset,
                            ));
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
                toProcessFeatures.push((
                    feature_index,
                    shapedTextOrientations,
                    shapedIcon,
                    imageMap.clone(),
                    textOffset,
                    layoutTextSize,
                    layoutIconSize,
                    iconType,
                ));
            }
        }

        for (
            feature_index,
            shapedTextOrientations,
            shapedIcon,
            imageMap,
            textOffset,
            layoutTextSize,
            layoutIconSize,
            iconType,
        ) in toProcessFeatures
        {
            self.addFeature(
                feature_index,
                &self.features[feature_index].clone(), // TODO likely wrong clone
                &shapedTextOrientations,
                shapedIcon,
                imageMap,
                textOffset,
                layoutTextSize,
                layoutIconSize,
                iconType,
            );

            self.features[feature_index].geometry.clear();
        }

        self.compareText.clear();
    }

    fn createBucket(
        &self,
        imagePositions: ImagePositions,
        feature_index: Box<FeatureIndex>,
        renderData: &mut HashMap<String, LayerRenderData>,
        firstLoad: bool,
        showCollisionBoxes: bool,
        canonical: &CanonicalTileID,
    ) {
        let mut symbolInstances = self.symbolInstances.clone(); // TODO should we clone or modify the symbol instances?
        let mut bucket: SymbolBucket = SymbolBucket::new(
            self.layout.clone(),
            &self.layerPaintProperties,
            &self.textSize,
            &self.iconSize,
            self.zoom,
            self.iconsNeedLinear,
            self.sortFeaturesByY,
            self.bucketLeaderID.clone(),
            self.symbolInstances.clone(), // TODO should we clone or modify the symbol instances?
            self.sortKeyRanges.clone(),
            self.tilePixelRatio,
            self.allowVerticalPlacement,
            self.placementModes.clone(),
            self.iconsInText,
        );

        for symbolInstance in &mut symbolInstances {
            let hasText = symbolInstance.hasText();
            let hasIcon = symbolInstance.hasIcon();
            let singleLine = symbolInstance.singleLine;

            let feature = self
                .features
                .get(symbolInstance.layoutFeatureIndex)
                .unwrap();

            // Insert final placement into collision tree and add glyphs/icons to buffers

            // Process icon first, so that text symbols would have reference to
            // iconIndex which is used when dynamic vertices for icon-text-fit image
            // have to be updated.
            if (hasIcon) {
                let sizeData: Range<f64> = bucket.iconSizeBinder.getVertexSizeData(feature); // TODO verify usage of range
                let iconBuffer = if symbolInstance.hasSdfIcon() {
                    &mut bucket.sdfIcon
                } else {
                    &mut bucket.icon
                };
                let mut placeIcon =
                    |iconQuads: &SymbolQuads, mut index: usize, writingMode: WritingModeType| {
                        let mut iconSymbol = PlacedSymbol {
                            anchorPoint: symbolInstance.anchor.point,
                            segment: symbolInstance.anchor.segment.unwrap_or(0),
                            lowerSize: sizeData.start,
                            upperSize: sizeData.end,
                            lineOffset: symbolInstance.iconOffset,
                            writingModes: writingMode,
                            line: symbolInstance.line().clone(),
                            tileDistances: Vec::new(),
                            glyphOffsets: vec![],
                            hidden: false,
                            vertexStartIndex: 0,
                            crossTileID: 0,
                            placedOrientation: None,
                            angle: if (self.allowVerticalPlacement
                                && writingMode == WritingModeType::Vertical)
                            {
                                PI / 2.
                            } else {
                                0.0
                            },
                            placedIconIndex: None,
                        };

                        iconSymbol.vertexStartIndex = self.addSymbols(
                            iconBuffer,
                            sizeData.clone(),
                            iconQuads,
                            &symbolInstance.anchor,
                            &mut iconSymbol,
                            feature.sortKey,
                        );

                        iconBuffer.placedSymbols.push(iconSymbol);
                        index = iconBuffer.placedSymbols.len() - 1; // TODO we receive an index but always overwrite it
                    };

                placeIcon(
                    symbolInstance.iconQuads().as_ref().unwrap(),
                    symbolInstance.placedIconIndex.unwrap(),
                    WritingModeType::None,
                );
                if let Some(verticalIconQuads) = (symbolInstance.verticalIconQuads()) {
                    placeIcon(
                        verticalIconQuads,
                        symbolInstance.placedVerticalIconIndex.unwrap(),
                        WritingModeType::Vertical,
                    );
                }

                // TODO
                assert!(bucket.paintProperties.is_empty())
                //for pair in bucket.paintProperties {
                //    pair.1.iconBinders.populateVertexVectors(
                //        feature,
                //        iconBuffer.sharedVertices.elements(),
                //        symbolInstance.dataFeatureIndex,
                //        {},
                //        {},
                //        canonical,
                //    );
                //}
            }

            if (hasText && feature.formattedText.is_some()) {
                let mut lastAddedSection: Option<usize> = None;
                if (singleLine) {
                    let mut placedTextIndex: Option<usize> = None;
                    let (newLastAddedSection, newPlacedIndex) = self.addSymbolGlyphQuads(
                        &mut bucket,
                        symbolInstance,
                        feature,
                        symbolInstance.writingModes,
                        placedTextIndex,
                        symbolInstance.rightJustifiedGlyphQuads(),
                        canonical,
                        lastAddedSection,
                    );
                    lastAddedSection = Some(newLastAddedSection);
                    placedTextIndex = newPlacedIndex;
                    symbolInstance.placedRightTextIndex = placedTextIndex;
                    symbolInstance.placedCenterTextIndex = placedTextIndex;
                    symbolInstance.placedLeftTextIndex = placedTextIndex;
                } else {
                    if (symbolInstance.rightJustifiedGlyphQuadsSize != 0) {
                        let (newLastAddedSection, newPlacedIndex) = self.addSymbolGlyphQuads(
                            &mut bucket,
                            symbolInstance,
                            feature,
                            symbolInstance.writingModes,
                            symbolInstance.placedRightTextIndex,
                            symbolInstance.rightJustifiedGlyphQuads(),
                            canonical,
                            lastAddedSection,
                        );
                        lastAddedSection = Some(newLastAddedSection);
                        symbolInstance.placedRightTextIndex = newPlacedIndex
                    }
                    if (symbolInstance.centerJustifiedGlyphQuadsSize != 0) {
                        let (newLastAddedSection, newPlacedIndex) = self.addSymbolGlyphQuads(
                            &mut bucket,
                            symbolInstance,
                            feature,
                            symbolInstance.writingModes,
                            symbolInstance.placedCenterTextIndex,
                            symbolInstance.centerJustifiedGlyphQuads(),
                            canonical,
                            lastAddedSection,
                        );
                        lastAddedSection = Some(newLastAddedSection);
                        symbolInstance.placedCenterTextIndex = newPlacedIndex
                    }
                    if (symbolInstance.leftJustifiedGlyphQuadsSize != 0) {
                        let (newLastAddedSection, newPlacedIndex) = self.addSymbolGlyphQuads(
                            &mut bucket,
                            symbolInstance,
                            feature,
                            symbolInstance.writingModes,
                            symbolInstance.placedLeftTextIndex,
                            symbolInstance.leftJustifiedGlyphQuads(),
                            canonical,
                            lastAddedSection,
                        );
                        lastAddedSection = Some(newLastAddedSection);
                        symbolInstance.placedLeftTextIndex = newPlacedIndex
                    }
                }
                if (symbolInstance.writingModes.contains(WritingModeType::Vertical) // TODO is bitset op correct?
                    && symbolInstance.verticalGlyphQuadsSize != 0)
                {
                    let (newLastAddedSection, newPlacedIndex) = self.addSymbolGlyphQuads(
                        &mut bucket,
                        symbolInstance,
                        feature,
                        WritingModeType::Vertical,
                        symbolInstance.placedVerticalTextIndex,
                        symbolInstance.verticalGlyphQuads(),
                        canonical,
                        lastAddedSection,
                    );
                    lastAddedSection = Some(newLastAddedSection);
                    symbolInstance.placedVerticalTextIndex = newPlacedIndex
                }
                assert!(lastAddedSection.is_some()); // True, as hasText == true;
                self.updatePaintPropertiesForSection(
                    &mut bucket,
                    feature,
                    lastAddedSection.unwrap(),
                    canonical,
                );
            }

            symbolInstance.releaseSharedData();
        }

        if (showCollisionBoxes) {
            self.addToDebugBuffers(&mut bucket);
        }
        if (bucket.hasData()) {
            for pair in &self.layerPaintProperties {
                if (!firstLoad) {
                    bucket.justReloaded = true;
                }
                renderData.insert(
                    pair.0.clone(),
                    LayerRenderData {
                        bucket: Rc::new(bucket.clone()), // TODO is cloning intended here?
                        layerProperties: pair.1.clone(),
                    },
                );
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
        &mut self,
        layoutFeatureIndex: usize,
        feature: &SymbolGeometryTileFeature,
        shapedTextOrientations: &ShapedTextOrientations,
        mut shapedIcon: Option<PositionedIcon>, // TODO should this be an output?
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
                Self::INVALID_OFFSET_VALUE,
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

        let textRepeatDistance: f64 = symbolSpacing / 2.;
        let evaluatedLayoutProperties: SymbolLayoutProperties_Evaluated =
            self.layout.evaluate_feature(self.zoom, feature);
        let indexedFeature = IndexedSubfeature {
            ref_: RefIndexedSubfeature {
                index: feature.index,
                sortIndex: self.symbolInstances.len(),
                sourceLayerName: self.sourceLayer.getName().to_string(),
                bucketLeaderID: self.bucketLeaderID.clone(),
                bucketInstanceId: 0,
                collisionGroupId: 0,
            },
            sourceLayerNameCopy: self.sourceLayer.getName().to_string(),
            bucketLeaderIDCopy: self.bucketLeaderID.clone(),
        };

        let iconTextFit = evaluatedLayoutProperties.get::<IconTextFit>();
        let hasIconTextFit = iconTextFit != IconTextFitType::None;
        // Adjust shaped icon size when icon-text-fit is used.
        let mut verticallyShapedIcon: Option<PositionedIcon> = None;

        if let Some(shapedIcon) = &mut shapedIcon {
            if (hasIconTextFit) {
                // Create vertically shaped icon for vertical writing mode if needed.
                if (self.allowVerticalPlacement
                    && shapedTextOrientations.vertical().isAnyLineNotEmpty())
                {
                    verticallyShapedIcon = Some(shapedIcon.clone());
                    verticallyShapedIcon.as_mut().unwrap().fitIconToText(
                        &shapedTextOrientations.vertical(),
                        iconTextFit,
                        &self.layout.get::<IconTextFitPadding>(),
                        &iconOffset,
                        fontScale,
                    );
                }
                let shapedText = getDefaultHorizontalShaping(shapedTextOrientations);
                if (shapedText.isAnyLineNotEmpty()) {
                    shapedIcon.fitIconToText(
                        shapedText,
                        iconTextFit,
                        &self.layout.get::<IconTextFitPadding>(),
                        &iconOffset,
                        fontScale,
                    );
                }
            }
        }

        let mut addSymbolInstance = |anchor: &Anchor, sharedData: Rc<SymbolInstanceSharedData>| {
            // assert!(sharedData); TODO
            let anchorInsideTile = anchor.point.x >= 0.
                && anchor.point.x < EXTENT
                && anchor.point.y >= 0.
                && anchor.point.y < EXTENT;

            if (self.mode == MapMode::Tile || anchorInsideTile) {
                // For static/continuous rendering, only add symbols anchored within this tile:
                //  neighboring symbols will be added as part of the neighboring tiles.
                // In tiled rendering mode, add all symbols in the buffers so that we can:
                //  (1) render symbols that overlap into this tile
                //  (2) approximate collision detection effects from neighboring symbols
                self.symbolInstances.push(SymbolInstance::new(
                    anchor.clone(),
                    sharedData,
                    shapedTextOrientations,
                    &shapedIcon,
                    &verticallyShapedIcon,
                    textBoxScale,
                    textPadding,
                    textPlacement,
                    textOffset,
                    iconBoxScale,
                    iconPadding,
                    iconOffset,
                    indexedFeature.clone(),
                    layoutFeatureIndex,
                    feature.index,
                    if let Some(formattedText) = &feature.formattedText {
                        formattedText.rawText().clone()
                    } else {
                        U16String::new()
                    },
                    self.overscaling,
                    iconRotation,
                    textRotation,
                    variableTextOffset,
                    self.allowVerticalPlacement,
                    iconType,
                ));

                if (self.sortFeaturesByKey) {
                    if (!self.sortKeyRanges.is_empty()
                        && self.sortKeyRanges.last().unwrap().sortKey == feature.sortKey)
                    {
                        self.sortKeyRanges.last_mut().unwrap().end = self.symbolInstances.len();
                    } else {
                        self.sortKeyRanges.push(SortKeyRange {
                            sortKey: feature.sortKey,
                            start: self.symbolInstances.len() - 1,
                            end: self.symbolInstances.len(),
                        });
                    }
                }
            }
        };

        let createSymbolInstanceSharedData = |line: GeometryCoordinates| {
            return Rc::new(SymbolInstanceSharedData::new(
                line,
                shapedTextOrientations,
                shapedIcon.clone(),
                verticallyShapedIcon.clone(),
                &evaluatedLayoutProperties,
                textPlacement,
                textOffset,
                imageMap,
                iconRotation,
                iconType,
                hasIconTextFit,
                self.allowVerticalPlacement,
            ));
        };

        let type_ = feature.getType();

        if (self.layout.get::<SymbolPlacement>() == SymbolPlacementType::Line) {
            todo!()
            /*let clippedLines = clipLines(feature.geometry, 0, 0, EXTENT, EXTENT);
            for line in clippedLines {
                let anchors: Anchors = getAnchors(
                    line,
                    symbolSpacing,
                    textMaxAngle,
                    (if shapedTextOrientations.vertical {
                        &shapedTextOrientations.vertical
                    } else {
                        getDefaultHorizontalShaping(shapedTextOrientations)
                    })
                    .left,
                    (if shapedTextOrientations.vertical {
                        &shapedTextOrientations.vertical
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
            }*/
        } else if (self.layout.get::<SymbolPlacement>() == SymbolPlacementType::LineCenter) {
            todo!()
            /*
            // No clipping, multiple lines per feature are allowed
            // "lines" with only one point are ignored as in clipLines
            for line in feature.geometry {
                if (line.len() > 1) {
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
            }*/
        } else if (type_ == FeatureType::Polygon) {
            todo!()
            /*for polygon in classifyRings(feature.geometry) {
                let poly: Polygon<f64>;
                for ring in polygon {
                    let r: LinearRing<f64>;
                    for p in ring {
                        r.push(convertPoint::<double>(p));
                    }
                    poly.push(r);
                }

                // 1 pixel worth of precision, in tile coordinates
                let poi = mapbox::polylabel(poly, EXTENT / TILE_SIZE);
                let anchor = Anchor::new((poi.x) as f64, (poi.y) as f64, 0.0, (minScale) as usize);
                addSymbolInstance(anchor, createSymbolInstanceSharedData(polygon[0]));
            }*/
        } else if (type_ == FeatureType::LineString) {
            for line in &feature.geometry {
                // Skip invalid LineStrings.
                if (line.0.is_empty()) {
                    continue;
                }

                let anchor = Anchor {
                    point: Point2D::new((line[0].x) as f64, (line[0].y) as f64),
                    angle: 0.0,
                    segment: Some((minScale) as usize),
                };
                addSymbolInstance(&anchor, createSymbolInstanceSharedData(line.clone()));
            }
        } else if (type_ == FeatureType::Point) {
            for points in &feature.geometry {
                for point in &points.0 {
                    let anchor = Anchor {
                        point: Point2D::new((point.x) as f64, (point.y) as f64),
                        angle: 0.0,
                        segment: Some((minScale) as usize),
                    };
                    addSymbolInstance(
                        &anchor,
                        createSymbolInstanceSharedData(GeometryCoordinates(vec![*point])),
                    );
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

    fn addToDebugBuffers(&self, bucket: &mut SymbolBucket) {
        todo!()
    }

    // Adds placed items to the buffer.
    fn addSymbol(
        &self,
        buffer: &mut SymbolBucketBuffer,
        sizeData: Range<f64>,
        symbol: &SymbolQuad,
        labelAnchor: &Anchor,
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

        if (buffer.segments.is_empty()
            || buffer.segments.last().unwrap().vertexLength + vertexLength as usize
                > u16::MAX as usize
            || (buffer.segments.last().unwrap().sortKey - sortKey).abs() > f64::EPSILON)
        {
            buffer.segments.push(Segment {
                vertexOffset: buffer.sharedVertices.len(),
                indexOffset: buffer.triangles.len(),
                vertexLength: 0,
                indexLength: 0,
                sortKey,
                _phandom_data: Default::default(),
            });
        }

        // We're generating triangle fans, so we always start with the first
        // coordinate in this polygon.
        let segment = buffer.segments.last_mut().unwrap();
        assert!(segment.vertexLength <= u16::MAX as usize);
        let index = (segment.vertexLength) as u16;

        // coordinates (2 triangles)
        let vertices = &mut buffer.sharedVertices;
        vertices.push(SymbolVertex::new(
            labelAnchor.point,
            tl,
            symbol.glyphOffset.y,
            tex.origin.x,
            tex.origin.y,
            sizeData.clone(),
            symbol.isSDF,
            pixelOffsetTL,
            minFontScale,
        ));
        vertices.push(SymbolVertex::new(
            labelAnchor.point,
            tr,
            symbol.glyphOffset.y,
            tex.origin.x + tex.width(),
            tex.origin.y,
            sizeData.clone(),
            symbol.isSDF,
            Point2D::new(pixelOffsetBR.x, pixelOffsetTL.y),
            minFontScale,
        ));
        vertices.push(SymbolVertex::new(
            labelAnchor.point,
            bl,
            symbol.glyphOffset.y,
            tex.origin.x,
            tex.origin.y + tex.height(),
            sizeData.clone(),
            symbol.isSDF,
            Point2D::new(pixelOffsetTL.x, pixelOffsetBR.y),
            minFontScale,
        ));
        vertices.push(SymbolVertex::new(
            labelAnchor.point,
            br,
            symbol.glyphOffset.y,
            tex.origin.x + tex.width(),
            tex.origin.y + tex.height(),
            sizeData.clone(),
            symbol.isSDF,
            pixelOffsetBR,
            minFontScale,
        ));

        // Dynamic/Opacity vertices are initialized so that the vertex count always
        // agrees with the layout vertex buffer, but they will always be updated
        // before rendering happens
        let dynamicVertex = DynamicVertex::new(labelAnchor.point, 0.);
        buffer.sharedDynamicVertices.push(dynamicVertex);
        buffer.sharedDynamicVertices.push(dynamicVertex);
        buffer.sharedDynamicVertices.push(dynamicVertex);
        buffer.sharedDynamicVertices.push(dynamicVertex);

        let opacityVertex = OpacityVertex::new(true, 1.0);
        buffer.sharedOpacityVertices.push(opacityVertex);
        buffer.sharedOpacityVertices.push(opacityVertex);
        buffer.sharedOpacityVertices.push(opacityVertex);
        buffer.sharedOpacityVertices.push(opacityVertex);

        // add the two triangles, referencing the four coordinates we just inserted.
        buffer.triangles.push(index + 0, index + 1, index + 2);
        buffer.triangles.push(index + 1, index + 2, index + 3);

        segment.vertexLength += vertexLength as usize;
        segment.indexLength += 6;

        return index as usize;
    }
    fn addSymbols(
        &self,
        buffer: &mut SymbolBucketBuffer,
        sizeData: Range<f64>,
        symbols: &SymbolQuads,
        labelAnchor: &Anchor,
        placedSymbol: &mut PlacedSymbol,
        sortKey: f64,
    ) -> usize {
        let mut firstSymbol = true;
        let mut firstIndex = 0;
        for symbol in symbols {
            let index = self.addSymbol(buffer, sizeData.clone(), symbol, labelAnchor, sortKey);
            placedSymbol.glyphOffsets.push(symbol.glyphOffset.x);
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
        bucket: &mut SymbolBucket,
        symbolInstance: &SymbolInstance,
        feature: &SymbolGeometryTileFeature,
        writingMode: WritingModeType,
        placedIndex: Option<usize>, // TODO should this be an output?
        glyphQuads: &SymbolQuads,
        canonical: &CanonicalTileID,
        mut lastAddedSection: Option<usize>, // TODO should this be an output?
    ) -> (usize, Option<usize>) {
        let mut outputPlacedIndex = placedIndex;
        let sizeData: Range<f64> = bucket.textSizeBinder.getVertexSizeData(feature); // TODO verify if usage of range is oke, empty ranges, reverse
        let hasFormatSectionOverrides: bool = bucket.hasFormatSectionOverrides();
        let placedIconIndex = if writingMode == WritingModeType::Vertical {
            symbolInstance.placedVerticalIconIndex
        } else {
            symbolInstance.placedIconIndex
        };

        // TODO is this PlacedSymbol correct?
        let mut newlyPlacedSymbol = PlacedSymbol {
            anchorPoint: symbolInstance.anchor.point,
            segment: symbolInstance.anchor.segment.unwrap_or(0),
            lowerSize: sizeData.start,
            upperSize: sizeData.end,
            lineOffset: symbolInstance.textOffset,
            writingModes: writingMode,
            line: symbolInstance.line().clone(),
            tileDistances: Self::calculateTileDistances(
                symbolInstance.line(),
                &symbolInstance.anchor,
            ),

            glyphOffsets: vec![],
            hidden: false,
            vertexStartIndex: 0,
            crossTileID: 0,
            placedOrientation: None,
            angle: if (self.allowVerticalPlacement && writingMode == WritingModeType::Vertical) {
                PI / 2.
            } else {
                0.0
            },
            placedIconIndex,
        };

        let mut firstSymbol = true;
        for symbolQuad in glyphQuads {
            if (hasFormatSectionOverrides) {
                if let Some(lastAddedSection) = lastAddedSection {
                    if (lastAddedSection != symbolQuad.sectionIndex) {
                        self.updatePaintPropertiesForSection(
                            bucket,
                            feature,
                            lastAddedSection,
                            canonical,
                        );
                    }
                }

                lastAddedSection = Some(symbolQuad.sectionIndex);
            }
            let index = self.addSymbol(
                &mut bucket.text,
                sizeData.clone(),
                symbolQuad,
                &symbolInstance.anchor,
                feature.sortKey,
            );

            newlyPlacedSymbol
                .glyphOffsets
                .push(symbolQuad.glyphOffset.x);

            if (firstSymbol) {
                newlyPlacedSymbol.vertexStartIndex = index;
                firstSymbol = false;
            }
        }

        bucket.text.placedSymbols.push(newlyPlacedSymbol);
        outputPlacedIndex = Some(bucket.text.placedSymbols.len() - 1);

        return if let Some(lastAddedSection) = lastAddedSection {
            (lastAddedSection, outputPlacedIndex)
        } else {
            (0, outputPlacedIndex)
        };
    }

    fn updatePaintPropertiesForSection(
        &self,
        bucket: &SymbolBucket,
        feature: &SymbolGeometryTileFeature,
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
        // todo!()
        // for pair in bucket.paintProperties {
        //     pair.1.textBinders.populateVertexVectors(
        //         feature,
        //         bucket.text.vertices().elements(),
        //         feature.index,
        //         {},
        //         {},
        //         canonical,
        //         formattedSection,
        //     );
        // }
        return 0;
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::{
        euclid::{Point2D, Rect, Size2D},
        sdf::{
            bidi::Char16,
            font_stack::FontStackHasher,
            geometry_tile_data::{GeometryCoordinates, SymbolGeometryTileLayer},
            glyph::{Glyph, GlyphDependencies, GlyphMap, GlyphMetrics, Glyphs},
            glyph_atlas::{GlyphPosition, GlyphPositionMap, GlyphPositions},
            image::ImageMap,
            image_atlas::ImagePositions,
            layout::{
                layout::{BucketParameters, LayerTypeInfo, LayoutParameters},
                symbol_feature::{SymbolGeometryTileFeature, VectorGeometryTileFeature},
                symbol_layout::{FeatureIndex, LayerProperties, SymbolLayer, SymbolLayout},
            },
            style_types::SymbolLayoutProperties_Unevaluated,
            tagged_string::SectionOptions,
            CanonicalTileID, MapMode, OverscaledTileID,
        },
    };

    #[test]
    fn test() {
        let fontStack = vec![
            "Open Sans Regular".to_string(),
            "Arial Unicode MS Regular".to_string(),
        ];

        let sectionOptions = SectionOptions::new(1.0, fontStack.clone(), None);

        let mut glyphDependencies = GlyphDependencies::new();

        let tile_id = OverscaledTileID {
            canonical: CanonicalTileID { x: 0, y: 0, z: 0 },
            overscaledZ: 0,
        };
        let mut parameters = BucketParameters {
            tileID: tile_id,
            mode: MapMode::Continuous,
            pixelRatio: 1.0,
            layerType: LayerTypeInfo,
        };
        let mut layout = SymbolLayout::new(
            &parameters,
            &vec![LayerProperties {
                id: "layer".to_string(),
                layer: SymbolLayer {
                    layout: SymbolLayoutProperties_Unevaluated,
                },
            }],
            Box::new(SymbolGeometryTileLayer {
                name: "layer".to_string(),
                features: vec![SymbolGeometryTileFeature::new(Box::new(
                    VectorGeometryTileFeature {
                        geometry: vec![GeometryCoordinates(vec![Point2D::new(1024, 1024)])],
                    },
                ))],
            }),
            &mut LayoutParameters {
                bucketParameters: &mut parameters.clone(),
                glyphDependencies: &mut glyphDependencies,
                imageDependencies: &mut Default::default(),
                availableImages: &mut Default::default(),
            },
        )
        .unwrap();

        assert_eq!(glyphDependencies.len(), 1);

        // Now we prepare the data, when we have the glyphs available

        let image_positions = ImagePositions::new();

        let mut glyphPosition = GlyphPosition {
            rect: Rect::new(Point2D::new(0, 0), Size2D::new(10, 10)),
            metrics: GlyphMetrics {
                width: 18,
                height: 18,
                left: 2,
                top: -8,
                advance: 21,
            },
        };
        let glyphPositions: GlyphPositions = GlyphPositions::from([(
            FontStackHasher::new(&fontStack),
            GlyphPositionMap::from([('中' as Char16, glyphPosition)]),
        )]);

        let mut glyph = Glyph::default();
        glyph.id = '中' as Char16;
        glyph.metrics = glyphPosition.metrics;

        let glyphs: GlyphMap = GlyphMap::from([(
            FontStackHasher::new(&fontStack),
            Glyphs::from([('中' as Char16, Some(glyph))]),
        )]);

        let empty_image_map = ImageMap::new();
        layout.prepareSymbols(&glyphs, &glyphPositions, &empty_image_map, &image_positions);

        let mut output = HashMap::new();
        layout.createBucket(
            image_positions,
            Box::new(FeatureIndex),
            &mut output,
            false,
            false,
            &tile_id.canonical,
        );

        println!("{:#?}", output)
    }
}
