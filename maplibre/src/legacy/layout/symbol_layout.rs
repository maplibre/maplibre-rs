//! Translated from https://github.com/maplibre/maplibre-native/blob/4add9ea/src/mbgl/layout/symbol_layout.cpp

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
    legacy::{
        bidi::{apply_arabic_shaping, BiDi, Char16},
        buckets::symbol_bucket::{
            DynamicVertex, OpacityVertex, PlacedSymbol, Segment, SymbolBucket, SymbolBucketBuffer,
            SymbolVertex,
        },
        geometry::{
            anchor::{Anchor, Anchors},
            feature_index::{IndexedSubfeature, RefIndexedSubfeature},
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
        shaping::{get_anchor_justification, get_shaping, PositionedIcon},
        style_types::*,
        tagged_string::{SectionOptions, TaggedString},
        util::{constants::ONE_EM, i18n, lower_bound, math::deg2radf},
        CanonicalTileID, MapMode,
    },
};

// TODO
/// maplibre/maplibre-native#4add9ea original name: SymbolLayer
#[derive(Clone, Debug)]
pub struct SymbolLayer {
    pub layout: SymbolLayoutProperties_Unevaluated,
}
/// maplibre/maplibre-native#4add9ea original name: SymbolLayer_Impl
pub type SymbolLayer_Impl = SymbolLayer;
// TODO
/// maplibre/maplibre-native#4add9ea original name: LayerProperties
#[derive(Clone, Debug)]
pub struct LayerProperties {
    pub id: String,
    pub layer: SymbolLayer_Impl,
}
/// maplibre/maplibre-native#4add9ea original name: SymbolLayerProperties
pub type SymbolLayerProperties = LayerProperties;
impl LayerProperties {
    /// maplibre/maplibre-native#4add9ea original name: layerImpl
    pub fn layer_impl(&self) -> &SymbolLayer_Impl {
        // TODO
        &self.layer
    }

    /// maplibre/maplibre-native#4add9ea original name: baseImpl
    pub fn base_impl(&self) -> &Self {
        self
    }
}
/// maplibre/maplibre-native#4add9ea original name: Bucket
pub type Bucket = SymbolBucket;

/// maplibre/maplibre-native#4add9ea original name: LayerRenderData
#[derive(Debug)]
pub struct LayerRenderData {
    pub bucket: Bucket,
    pub layer_properties: LayerProperties,
}

/// maplibre/maplibre-native#4add9ea original name: SortKeyRange
#[derive(Clone, Copy, Debug)]
pub struct SortKeyRange {
    sort_key: f64,
    start: usize,
    end: usize,
}

impl SortKeyRange {
    /// maplibre/maplibre-native#4add9ea original name: isFirstRange
    pub fn is_first_range(&self) -> bool {
        self.start == 0
    }
}

// index
/// maplibre/maplibre-native#4add9ea original name: FeatureIndex
pub struct FeatureIndex;

/// maplibre/maplibre-native#4add9ea original name: sectionOptionsToValue
fn section_options_to_value(options: &SectionOptions) -> expression::Value {
    let mut result: HashMap<String, expression::Value> = Default::default();
    // TODO: Data driven properties that can be overridden on per section basis.
    // TextOpacity
    // TextHaloColor
    // TextHaloWidth
    // TextHaloBlur
    if let Some(text_color) = &(options.text_color) {
        result.insert(
            expression::K_FORMATTED_SECTION_TEXT_COLOR.to_string(),
            expression::Value::Color(text_color.clone()),
        );
    }
    expression::Value::Object(result)
}

/// maplibre/maplibre-native#4add9ea original name: toSymbolLayerProperties
fn to_symbol_layer_properties(layer: &LayerProperties) -> &SymbolLayerProperties {
    // TODO
    //return static_cast<const SymbolLayerProperties&>(*layer);
    layer
}

/// maplibre/maplibre-native#4add9ea original name: createLayout
fn create_layout(
    unevaluated: &SymbolLayoutProperties_Unevaluated,
    zoom: f64,
) -> SymbolLayoutProperties_PossiblyEvaluated {
    let mut layout = unevaluated.evaluate(PropertyEvaluationParameters(zoom));

    if layout.get::<IconRotationAlignment>() == AlignmentType::Auto {
        if layout.get::<SymbolPlacement>() != SymbolPlacementType::Point {
            layout.set::<IconRotationAlignment>(AlignmentType::Map);
        } else {
            layout.set::<IconRotationAlignment>(AlignmentType::Viewport);
        }
    }

    if layout.get::<TextRotationAlignment>() == AlignmentType::Auto {
        if layout.get::<SymbolPlacement>() != SymbolPlacementType::Point {
            layout.set::<TextRotationAlignment>(AlignmentType::Map);
        } else {
            layout.set::<TextRotationAlignment>(AlignmentType::Viewport);
        }
    }

    // If unspecified `*-pitch-alignment` inherits `*-rotation-alignment`
    if layout.get::<TextPitchAlignment>() == AlignmentType::Auto {
        layout.set::<TextPitchAlignment>(layout.get::<TextRotationAlignment>());
    }
    if layout.get::<IconPitchAlignment>() == AlignmentType::Auto {
        layout.set::<IconPitchAlignment>(layout.get::<IconRotationAlignment>());
    }

    layout
}

// The radial offset is to the edge of the text box
// In the horizontal direction, the edge of the text box is where glyphs start
// But in the vertical direction, the glyphs appear to "start" at the baseline
// We don't actually load baseline data, but we assume an offset of ONE_EM - 17
// (see "yOffset" in shaping.js)
const BASELINE_OFFSET: f64 = 7.0;

// We don't care which shaping we get because this is used for collision
// purposes and all the justifications have the same collision box.
/// maplibre/maplibre-native#4add9ea original name: getDefaultHorizontalShaping
fn get_default_horizontal_shaping(shaped_text_orientations: &ShapedTextOrientations) -> &Shaping {
    if shaped_text_orientations.right().is_any_line_not_empty() {
        return shaped_text_orientations.right();
    }
    if shaped_text_orientations.center().is_any_line_not_empty() {
        return shaped_text_orientations.center();
    }
    if shaped_text_orientations.left().is_any_line_not_empty() {
        return shaped_text_orientations.left();
    }
    return shaped_text_orientations.horizontal();
}

/// maplibre/maplibre-native#4add9ea original name: shapingForTextJustifyType
fn shaping_for_text_justify_type(
    shaped_text_orientations: &ShapedTextOrientations,
    type_: TextJustifyType,
) -> &Shaping {
    match type_ {
        TextJustifyType::Right => {
            return shaped_text_orientations.right();
        }

        TextJustifyType::Left => {
            return shaped_text_orientations.left();
        }

        TextJustifyType::Center => {
            return shaped_text_orientations.center();
        }
        _ => {
            assert!(false);
            return shaped_text_orientations.horizontal();
        }
    }
}

/// maplibre/maplibre-native#4add9ea original name: evaluateRadialOffset
fn evaluate_radial_offset(anchor: SymbolAnchorType, mut radial_offset: f64) -> [f64; 2] {
    let mut result = [0.0, 0.0];
    if radial_offset < 0.0 {
        radial_offset = 0.0; // Ignore negative offset.
    }
    // solve for r where r^2 + r^2 = radialOffset^2
    let sqrt2 = 1.41421356237;
    let hypotenuse = radial_offset / sqrt2;

    match anchor {
        SymbolAnchorType::TopRight | SymbolAnchorType::TopLeft => {
            result[1] = hypotenuse - BASELINE_OFFSET;
        }

        SymbolAnchorType::BottomRight | SymbolAnchorType::BottomLeft => {
            result[1] = -hypotenuse + BASELINE_OFFSET;
        }
        SymbolAnchorType::Bottom => {
            result[1] = -radial_offset + BASELINE_OFFSET;
        }
        SymbolAnchorType::Top => {
            result[1] = radial_offset - BASELINE_OFFSET;
        }

        _ => {}
    }

    match anchor {
        SymbolAnchorType::TopRight | SymbolAnchorType::BottomRight => {
            result[0] = -hypotenuse;
        }
        SymbolAnchorType::TopLeft | SymbolAnchorType::BottomLeft => {
            result[0] = hypotenuse;
        }
        SymbolAnchorType::Left => {
            result[0] = radial_offset;
        }
        SymbolAnchorType::Right => {
            result[0] = -radial_offset;
        }

        _ => {}
    }

    result
}

/// maplibre/maplibre-native#4add9ea original name: SymbolLayout
pub struct SymbolLayout {
    pub layer_paint_properties: BTreeMap<String, LayerProperties>,
    pub bucket_leader_id: String,
    pub symbol_instances: Vec<SymbolInstance>,
    pub sort_key_ranges: Vec<SortKeyRange>,

    // Stores the layer so that we can hold on to GeometryTileFeature instances
    // in SymbolFeature, which may reference data from this object.
    source_layer: Box<SymbolGeometryTileLayer>,
    overscaling: f64,
    zoom: f64,
    canonical_id: CanonicalTileID,
    mode: MapMode,
    pixel_ratio: f64,

    tile_size: u32,
    tile_pixel_ratio: f64,

    icons_need_linear: bool,
    sort_features_by_y: bool,
    sort_features_by_key: bool,
    allow_vertical_placement: bool,
    icons_in_text: bool,
    placement_modes: Vec<TextWritingModeType>,

    text_size: <TextSize as DataDrivenLayoutProperty>::UnevaluatedType,
    icon_size: <IconSize as DataDrivenLayoutProperty>::UnevaluatedType,
    text_radial_offset: <TextRadialOffset as DataDrivenLayoutProperty>::UnevaluatedType,
    layout: SymbolLayoutProperties_PossiblyEvaluated,
    features: Vec<SymbolGeometryTileFeature>,

    bidi: BiDi, // Consider moving this up to geometry tile worker to reduce
    // reinstantiation costs, use of BiDi/ubiditransform object must
    // be rained to one thread
    compare_text: BTreeMap<U16String, Vec<Anchor>>,
}

impl SymbolLayout {
    /// maplibre/maplibre-native#4add9ea original name: new
    pub fn new(
        parameters: &BucketParameters,
        layers: &Vec<LayerProperties>,
        source_layer: Box<SymbolGeometryTileLayer>,
        layout_parameters: &mut LayoutParameters, // TODO is this output?
    ) -> Option<Self> {
        let overscaling = parameters.tile_id.overscale_factor() as f64;
        let zoom = parameters.tile_id.overscaled_z as f64;
        let tile_size = (TILE_SIZE * overscaling) as u32;

        let leader: &SymbolLayer_Impl =
            to_symbol_layer_properties(layers.first().unwrap()).layer_impl();

        let mut self_ = Self {
            bucket_leader_id: layers.first().unwrap().base_impl().id.clone(),

            source_layer,
            overscaling,
            zoom,
            canonical_id: parameters.tile_id.canonical,
            mode: parameters.mode,
            pixel_ratio: parameters.pixel_ratio,
            tile_size: tile_size,
            tile_pixel_ratio: EXTENT / tile_size as f64,
            layout: create_layout(
                &to_symbol_layer_properties(layers.first().unwrap())
                    .layer_impl()
                    .layout,
                zoom,
            ),
            text_size: leader.layout.get_dynamic::<TextSize>(),
            icon_size: leader.layout.get_dynamic::<IconSize>(),
            text_radial_offset: leader.layout.get_dynamic::<TextRadialOffset>(),

            // default values
            layer_paint_properties: Default::default(),
            symbol_instances: vec![],
            sort_key_ranges: vec![],
            icons_need_linear: false,
            sort_features_by_y: false,
            sort_features_by_key: false,
            allow_vertical_placement: false,
            icons_in_text: false,
            placement_modes: vec![],
            features: vec![],
            bidi: BiDi,
            compare_text: Default::default(),
        };

        let has_text = self_.layout.has::<TextField>() && self_.layout.has::<TextFont>();
        let has_icon = self_.layout.has::<IconImage>();

        if !has_text && !has_icon {
            return None;
        }

        let has_symbol_sort_key = !leader.layout.get_dynamic::<SymbolSortKey>().is_undefined();
        let symbol_zorder = self_.layout.get::<SymbolZOrder>();
        self_.sort_features_by_key =
            symbol_zorder != SymbolZOrderType::ViewportY && has_symbol_sort_key;
        let z_order_by_viewport_y = symbol_zorder == SymbolZOrderType::ViewportY
            || (symbol_zorder == SymbolZOrderType::Auto && !self_.sort_features_by_key);
        self_.sort_features_by_y = z_order_by_viewport_y
            && (self_.layout.get::<TextAllowOverlap>()
                || self_.layout.get::<IconAllowOverlap>()
                || self_.layout.get::<TextIgnorePlacement>()
                || self_.layout.get::<IconIgnorePlacement>());
        if self_.layout.get::<SymbolPlacement>() == SymbolPlacementType::Point {
            let mut modes = self_.layout.get::<TextWritingMode>();

            // Remove duplicates and preserve order.
            // TODO Verify if this is correct. Maybe make this better.
            let mut seen: BTreeSet<TextWritingModeType> = BTreeSet::new();
            modes = modes
                .iter()
                .filter(|placement_mode| {
                    self_.allow_vertical_placement = self_.allow_vertical_placement
                        || **placement_mode == TextWritingModeType::Vertical;
                    seen.insert(**placement_mode)
                })
                .cloned()
                .collect();

            self_.placement_modes = modes;
        }

        for layer in layers {
            self_
                .layer_paint_properties
                .insert(layer.base_impl().id.clone(), layer.clone());
        }

        // Determine glyph dependencies
        let feature_count = self_.source_layer.feature_count();
        for i in 0..feature_count {
            let feature = self_.source_layer.get_feature(i);

            // TODO
            //if (!leader.filter(expression::EvaluationContext::new(self_.zoom, feature.get()).withCanonicalTileID(&parameters.tileID.canonical), )) {
            //    continue;
            //}

            let mut ft: SymbolGeometryTileFeature = *feature.clone();

            ft.index = i;

            if has_text {
                let formatted = self_.layout.evaluate4::<TextField>(
                    self_.zoom,
                    &ft,
                    layout_parameters.available_images,
                    self_.canonical_id,
                );
                let text_transform =
                    self_
                        .layout
                        .evaluate::<TextTransform>(self_.zoom, &ft, self_.canonical_id);
                let base_font_stack =
                    self_
                        .layout
                        .evaluate::<TextFont>(self_.zoom, &ft, self_.canonical_id);

                ft.formatted_text = Some(TaggedString::default());
                let ft_formatted_text = ft.formatted_text.as_mut().unwrap();
                for section in &formatted.sections {
                    if let Some(image) = &section.image {
                        layout_parameters
                            .image_dependencies
                            .insert(image.image_id.clone(), ImageType::Icon);
                        ft_formatted_text.add_image_section(image.image_id.clone());
                    } else {
                        let mut u8string = section.text.clone();
                        if text_transform == TextTransformType::Uppercase {
                            u8string = u8string.to_uppercase();
                        } else if text_transform == TextTransformType::Lowercase {
                            u8string = u8string.to_lowercase();
                        }

                        // TODO seems like invalid UTF-8 can not be in a tile? if let Err(e) =
                        ft_formatted_text.add_text_section(
                            &apply_arabic_shaping(&U16String::from(u8string.as_str())),
                            if let Some(font_scale) = section.font_scale {
                                font_scale
                            } else {
                                1.0
                            },
                            if let Some(font_stack) = &section.font_stack {
                                font_stack.clone()
                            } else {
                                base_font_stack.clone()
                            },
                            section.text_color.clone(),
                        )
                        //{
                        //    log::error!("Encountered section with invalid UTF-8 in tile, source: {} z: {} x: {} y: {}", self_.sourceLayer.getName(), self_.canonicalID.z, self_.canonicalID.x, self_.canonicalID.y);
                        //    continue; // skip section
                        //}
                    }
                }

                let can_verticalize_text = self_.layout.get::<TextRotationAlignment>()
                    == AlignmentType::Map
                    && self_.layout.get::<SymbolPlacement>() != SymbolPlacementType::Point
                    && ft_formatted_text.allows_vertical_writing_mode();

                // Loop through all characters of this text and collect unique codepoints.
                for j in 0..ft_formatted_text.length() {
                    let section =
                        &formatted.sections[ft_formatted_text.get_section_index(j) as usize];
                    if section.image.is_some() {
                        continue;
                    }

                    let section_font_stack = &section.font_stack;
                    let dependencies: &mut GlyphIDs = layout_parameters
                        .glyph_dependencies
                        .entry(if let Some(section_font_stack) = section_font_stack {
                            section_font_stack.clone()
                        } else {
                            base_font_stack.clone()
                        })
                        .or_default(); // TODO this is different in C++, as C++ always creates the default apparently
                    let code_point: Char16 = ft_formatted_text.get_char_code_at(j);
                    dependencies.insert(code_point);
                    if can_verticalize_text
                        || (self_.allow_vertical_placement
                            && ft_formatted_text.allows_vertical_writing_mode())
                    {
                        let vertical_chr: Char16 = i18n::verticalize_punctuation(code_point);
                        if vertical_chr != 0 {
                            dependencies.insert(vertical_chr);
                        }
                    }
                }
            }

            if has_icon {
                ft.icon = Some(self_.layout.evaluate4::<IconImage>(
                    self_.zoom,
                    &ft,
                    layout_parameters.available_images,
                    self_.canonical_id,
                )); // TODO it might be that this is None?
                layout_parameters
                    .image_dependencies
                    .insert(ft.icon.as_ref().unwrap().image_id.clone(), ImageType::Icon);
            }

            if ft.formatted_text.is_some() || ft.icon.is_some() {
                if self_.sort_features_by_key {
                    ft.sort_key =
                        self_
                            .layout
                            .evaluate::<SymbolSortKey>(self_.zoom, &ft, self_.canonical_id);

                    let lower_bound = lower_bound(&self_.features, &ft);
                    self_.features.insert(lower_bound, ft);
                } else {
                    self_.features.push(ft);
                }
            }
        }

        if self_.layout.get::<SymbolPlacement>() == SymbolPlacementType::Line {
            todo!()
            // TODO mergeLines(self_.features);
        }

        Some(self_)
    }
    /// maplibre/maplibre-native#4add9ea original name: prepareSymbols
    pub fn prepare_symbols(
        &mut self,
        glyph_map: &GlyphMap,
        glyph_positions: &GlyphPositions,
        image_map: &ImageMap,
        image_positions: &ImagePositions,
    ) {
        let is_point_placement: bool =
            self.layout.get::<SymbolPlacement>() == SymbolPlacementType::Point;
        let text_along_line: bool =
            self.layout.get::<TextRotationAlignment>() == AlignmentType::Map && !is_point_placement;

        let mut to_process_features = Vec::new();

        for (feature_index, feature) in self.features.iter_mut().enumerate() {
            // TODO expensive clone
            if feature.geometry.is_empty() {
                continue;
            }

            let mut shaped_text_orientations: ShapedTextOrientations =
                ShapedTextOrientations::default();
            let mut shaped_icon: Option<PositionedIcon> = None;
            let mut text_offset = [0.0, 0.0];
            let layout_text_size: f64 =
                self.layout
                    .evaluate::<TextSize>(self.zoom + 1., feature, self.canonical_id);
            let layout_text_size_at_bucket_zoom_level: f64 =
                self.layout
                    .evaluate::<TextSize>(self.zoom, feature, self.canonical_id);
            let layout_icon_size: f64 =
                self.layout
                    .evaluate::<IconSize>(self.zoom + 1., feature, self.canonical_id);

            // if feature has text, shape the text
            if let Some(mut feature_formatted_text) = feature.formatted_text.clone() {
                if layout_text_size > 0.0 {
                    let line_height: f64 = self.layout.get::<TextLineHeight>() * ONE_EM;
                    let spacing: f64 =
                        if i18n::allows_letter_spacing(feature_formatted_text.raw_text()) {
                            self.layout.evaluate::<TextLetterSpacing>(
                                self.zoom,
                                feature,
                                self.canonical_id,
                            ) * ONE_EM
                        } else {
                            0.0
                        };

                    let apply_shaping = |formatted_text: &TaggedString,
                                         writing_mode: WritingModeType,
                                         text_anchor: SymbolAnchorType,
                                         text_justify: TextJustifyType,
                                         text_offset: &[f64; 2]|
                     -> Shaping {
                        get_shaping(
                            /* string */ formatted_text,
                            /* maxWidth: ems */
                            if is_point_placement {
                                self.layout.evaluate::<TextMaxWidth>(
                                    self.zoom,
                                    feature,
                                    self.canonical_id,
                                ) * ONE_EM
                            } else {
                                0.0
                            },
                            /* ems */ line_height,
                            text_anchor,
                            text_justify,
                            /* ems */ spacing,
                            /* translate */ text_offset,
                            /* writingMode */ writing_mode,
                            /* bidirectional algorithm object */ &self.bidi,
                            glyph_map,
                            /* glyphs */ glyph_positions,
                            /* images */ image_positions,
                            layout_text_size,
                            layout_text_size_at_bucket_zoom_level,
                            self.allow_vertical_placement,
                        )
                    };

                    let variable_text_anchor: Vec<TextVariableAnchorType> =
                        self.layout.evaluate_static::<TextVariableAnchor>(
                            self.zoom,
                            feature,
                            self.canonical_id,
                        );
                    let text_anchor: SymbolAnchorType =
                        self.layout
                            .evaluate::<TextAnchor>(self.zoom, feature, self.canonical_id);
                    if variable_text_anchor.is_empty() {
                        // Layers with variable anchors use the `text-radial-offset`
                        // property and the [x, y] offset vector is calculated at
                        // placement time instead of layout time
                        let radial_offset: f64 = self.layout.evaluate::<TextRadialOffset>(
                            self.zoom,
                            feature,
                            self.canonical_id,
                        );
                        if radial_offset > 0.0 {
                            // The style spec says don't use `text-offset` and
                            // `text-radial-offset` together but doesn't actually
                            // specify what happens if you use both. We go with the
                            // radial offset.
                            text_offset =
                                evaluate_radial_offset(text_anchor, radial_offset * ONE_EM);
                        } else {
                            text_offset = [
                                self.layout.evaluate::<TextOffset>(
                                    self.zoom,
                                    feature,
                                    self.canonical_id,
                                )[0] * ONE_EM,
                                self.layout.evaluate::<TextOffset>(
                                    self.zoom,
                                    feature,
                                    self.canonical_id,
                                )[1] * ONE_EM,
                            ];
                        }
                    }
                    let mut text_justify = if text_along_line {
                        TextJustifyType::Center
                    } else {
                        self.layout
                            .evaluate::<TextJustify>(self.zoom, feature, self.canonical_id)
                    };

                    let add_vertical_shaping_for_point_label_if_needed =
                        |shaped_text_orientations: &mut ShapedTextOrientations,
                         feature_formatted_text: &mut TaggedString| {
                            if self.allow_vertical_placement
                                && feature_formatted_text.allows_vertical_writing_mode()
                            {
                                feature_formatted_text.verticalize_punctuation();
                                // Vertical POI label placement is meant to be used for
                                // scripts that support vertical writing mode, thus, default
                                // TextJustifyType::Left justification is used. If
                                // Latin scripts would need to be supported, this should
                                // take into account other justifications.
                                shaped_text_orientations.set_vertical(apply_shaping(
                                    feature_formatted_text,
                                    WritingModeType::Vertical,
                                    text_anchor,
                                    TextJustifyType::Left,
                                    &text_offset,
                                ))
                            }
                        };

                    // If this layer uses text-variable-anchor, generate shapings for
                    // all justification possibilities.
                    if !text_along_line && !variable_text_anchor.is_empty() {
                        let mut justifications: Vec<TextJustifyType> = Vec::new();
                        if text_justify != TextJustifyType::Auto {
                            justifications.push(text_justify);
                        } else {
                            for anchor in &variable_text_anchor {
                                justifications.push(get_anchor_justification(anchor));
                            }
                        }
                        for justification in justifications {
                            let mut shaping_for_justification = shaping_for_text_justify_type(
                                &shaped_text_orientations,
                                justification,
                            );
                            if shaping_for_justification.is_any_line_not_empty() {
                                continue;
                            }
                            // If using text-variable-anchor for the layer, we use a
                            // center anchor for all shapings and apply the offsets for
                            // the anchor in the placement step.
                            let shaping = apply_shaping(
                                &feature_formatted_text,
                                WritingModeType::Horizontal,
                                SymbolAnchorType::Center,
                                justification,
                                &text_offset,
                            );
                            if shaping.is_any_line_not_empty() {
                                shaping_for_justification = &shaping;
                                if shaping_for_justification.positioned_lines.len() == 1 {
                                    shaped_text_orientations.single_line = true;
                                    break;
                                }
                            }
                        }

                        // Vertical point label shaping if allowVerticalPlacement is enabled.
                        add_vertical_shaping_for_point_label_if_needed(
                            &mut shaped_text_orientations,
                            &mut feature_formatted_text,
                        );
                    } else {
                        if text_justify == TextJustifyType::Auto {
                            text_justify = get_anchor_justification(&text_anchor);
                        }

                        // Horizontal point or line label.
                        let shaping = apply_shaping(
                            &feature_formatted_text,
                            WritingModeType::Horizontal,
                            text_anchor,
                            text_justify,
                            &text_offset,
                        );
                        if shaping.is_any_line_not_empty() {
                            shaped_text_orientations.set_horizontal(shaping)
                        }

                        // Vertical point label shaping if allowVerticalPlacement is enabled.
                        add_vertical_shaping_for_point_label_if_needed(
                            &mut shaped_text_orientations,
                            &mut feature_formatted_text,
                        );

                        // Verticalized line label.
                        if text_along_line && feature_formatted_text.allows_vertical_writing_mode()
                        {
                            feature_formatted_text.verticalize_punctuation();
                            shaped_text_orientations.set_vertical(apply_shaping(
                                &feature_formatted_text,
                                WritingModeType::Vertical,
                                text_anchor,
                                text_justify,
                                &text_offset,
                            ));
                        }
                    }
                }

                feature.formatted_text = Some(feature_formatted_text);
            }

            // if feature has icon, get sprite atlas position
            let mut icon_type: SymbolContent = SymbolContent::None;
            if let Some(icon) = &feature.icon {
                let image = image_map.get(&icon.image_id);
                if let Some(image) = image {
                    icon_type = SymbolContent::IconRGBA;
                    shaped_icon = Some(PositionedIcon::shape_icon(
                        image_positions.get(&icon.image_id).unwrap().clone(),
                        &self
                            .layout
                            .evaluate::<IconOffset>(self.zoom, feature, self.canonical_id),
                        self.layout
                            .evaluate::<IconAnchor>(self.zoom, feature, self.canonical_id),
                    ));
                    if image.sdf {
                        icon_type = SymbolContent::IconSDF;
                    }
                    if image.pixel_ratio != self.pixel_ratio {
                        self.icons_need_linear = true;
                    } else if self.layout.get_dynamic::<IconRotate>().constant_or(1.0) != 0.0 {
                        self.icons_need_linear = true;
                    }
                }
            }

            // if either shapedText or icon position is present, add the feature
            let default_shaping = get_default_horizontal_shaping(&shaped_text_orientations);
            self.icons_in_text = if default_shaping.is_any_line_not_empty() {
                default_shaping.icons_in_text
            } else {
                false
            };
            if default_shaping.is_any_line_not_empty() || shaped_icon.is_some() {
                // TODO borrow conflict with self.features
                to_process_features.push((
                    feature_index,
                    shaped_text_orientations,
                    shaped_icon,
                    image_map,
                    text_offset,
                    layout_text_size,
                    layout_icon_size,
                    icon_type,
                ));
            }
        }

        for (
            feature_index,
            shaped_text_orientations,
            shaped_icon,
            imageMap,
            text_offset,
            layout_text_size,
            layout_icon_size,
            icon_type,
        ) in to_process_features
        {
            self.add_feature(
                feature_index,
                &self.features[feature_index].clone(), // TODO likely wrong clone
                &shaped_text_orientations,
                shaped_icon,
                imageMap,
                text_offset,
                layout_text_size,
                layout_icon_size,
                icon_type,
            );

            self.features[feature_index].geometry.clear();
        }

        self.compare_text.clear();
    }

    /// maplibre/maplibre-native#4add9ea original name: createBucket
    pub fn create_bucket(
        &self,
        _image_positions: ImagePositions,
        _feature_index: Box<FeatureIndex>,
        render_data: &mut HashMap<String, LayerRenderData>,
        first_load: bool,
        show_collision_boxes: bool,
        canonical: &CanonicalTileID,
    ) {
        let mut symbol_instances = self.symbol_instances.clone(); // TODO should we clone or modify the symbol instances?
        let mut bucket: SymbolBucket = SymbolBucket::new(
            self.layout.clone(),
            &self.layer_paint_properties,
            &self.text_size,
            &self.icon_size,
            self.zoom,
            self.icons_need_linear,
            self.sort_features_by_y,
            self.bucket_leader_id.clone(),
            self.symbol_instances.clone(), // TODO should we clone or modify the symbol instances?
            self.sort_key_ranges.clone(),
            self.tile_pixel_ratio,
            self.allow_vertical_placement,
            self.placement_modes.clone(),
            self.icons_in_text,
        );

        for symbol_instance in &mut symbol_instances {
            let has_text = symbol_instance.has_text();
            let has_icon = symbol_instance.has_icon();
            let single_line = symbol_instance.single_line;

            let feature = self
                .features
                .get(symbol_instance.layout_feature_index)
                .unwrap();

            // Insert final placement into collision tree and add glyphs/icons to buffers

            // Process icon first, so that text symbols would have reference to
            // iconIndex which is used when dynamic vertices for icon-text-fit image
            // have to be updated.
            if has_icon {
                let size_data: Range<f64> = bucket.icon_size_binder.get_vertex_size_data(feature); // TODO verify usage of range
                let icon_buffer = if symbol_instance.has_sdf_icon() {
                    &mut bucket.sdf_icon
                } else {
                    &mut bucket.icon
                };
                let mut place_icon =
                    |icon_quads: &SymbolQuads, mut index: usize, writing_mode: WritingModeType| {
                        let mut icon_symbol = PlacedSymbol {
                            anchor_point: symbol_instance.anchor.point,
                            segment: symbol_instance.anchor.segment.unwrap_or(0),
                            lower_size: size_data.start,
                            upper_size: size_data.end,
                            line_offset: symbol_instance.icon_offset,
                            writing_modes: writing_mode,
                            line: symbol_instance.line().clone(),
                            tile_distances: Vec::new(),
                            glyph_offsets: vec![],
                            hidden: false,
                            vertex_start_index: 0,
                            cross_tile_id: 0,
                            placed_orientation: None,
                            angle: if self.allow_vertical_placement
                                && writing_mode == WritingModeType::Vertical
                            {
                                PI / 2.
                            } else {
                                0.0
                            },
                            placed_icon_index: None,
                        };

                        icon_symbol.vertex_start_index = self.add_symbols(
                            icon_buffer,
                            size_data.clone(),
                            icon_quads,
                            &symbol_instance.anchor,
                            &mut icon_symbol,
                            feature.sort_key,
                        );

                        icon_buffer.placed_symbols.push(icon_symbol);
                        index = icon_buffer.placed_symbols.len() - 1; // TODO we receive an index but always overwrite it
                    };

                place_icon(
                    symbol_instance.icon_quads().as_ref().unwrap(),
                    symbol_instance.placed_icon_index.unwrap(),
                    WritingModeType::None,
                );
                if let Some(vertical_icon_quads) = symbol_instance.vertical_icon_quads() {
                    place_icon(
                        vertical_icon_quads,
                        symbol_instance.placed_vertical_icon_index.unwrap(),
                        WritingModeType::Vertical,
                    );
                }

                // TODO
                assert!(bucket.paint_properties.is_empty())
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

            if has_text && feature.formatted_text.is_some() {
                let mut last_added_section: Option<usize> = None;
                if single_line {
                    let mut placed_text_index: Option<usize> = None;
                    let (new_last_added_section, new_placed_index) = self.add_symbol_glyph_quads(
                        &mut bucket,
                        symbol_instance,
                        feature,
                        symbol_instance.writing_modes,
                        placed_text_index,
                        symbol_instance.right_justified_glyph_quads(),
                        canonical,
                        last_added_section,
                    );
                    last_added_section = Some(new_last_added_section);
                    placed_text_index = new_placed_index;
                    symbol_instance.placed_right_text_index = placed_text_index;
                    symbol_instance.placed_center_text_index = placed_text_index;
                    symbol_instance.placed_left_text_index = placed_text_index;
                } else {
                    if symbol_instance.right_justified_glyph_quads_size != 0 {
                        let (new_last_added_section, new_placed_index) = self
                            .add_symbol_glyph_quads(
                                &mut bucket,
                                symbol_instance,
                                feature,
                                symbol_instance.writing_modes,
                                symbol_instance.placed_right_text_index,
                                symbol_instance.right_justified_glyph_quads(),
                                canonical,
                                last_added_section,
                            );
                        last_added_section = Some(new_last_added_section);
                        symbol_instance.placed_right_text_index = new_placed_index
                    }
                    if symbol_instance.center_justified_glyph_quads_size != 0 {
                        let (new_last_added_section, new_placed_index) = self
                            .add_symbol_glyph_quads(
                                &mut bucket,
                                symbol_instance,
                                feature,
                                symbol_instance.writing_modes,
                                symbol_instance.placed_center_text_index,
                                symbol_instance.center_justified_glyph_quads(),
                                canonical,
                                last_added_section,
                            );
                        last_added_section = Some(new_last_added_section);
                        symbol_instance.placed_center_text_index = new_placed_index
                    }
                    if symbol_instance.left_justified_glyph_quads_size != 0 {
                        let (new_last_added_section, new_placed_index) = self
                            .add_symbol_glyph_quads(
                                &mut bucket,
                                symbol_instance,
                                feature,
                                symbol_instance.writing_modes,
                                symbol_instance.placed_left_text_index,
                                symbol_instance.left_justified_glyph_quads(),
                                canonical,
                                last_added_section,
                            );
                        last_added_section = Some(new_last_added_section);
                        symbol_instance.placed_left_text_index = new_placed_index
                    }
                }
                if symbol_instance.writing_modes.contains(WritingModeType::Vertical) // TODO is bitset op correct?
                    && symbol_instance.vertical_glyph_quads_size != 0
                {
                    let (new_last_added_section, new_placed_index) = self.add_symbol_glyph_quads(
                        &mut bucket,
                        symbol_instance,
                        feature,
                        WritingModeType::Vertical,
                        symbol_instance.placed_vertical_text_index,
                        symbol_instance.vertical_glyph_quads(),
                        canonical,
                        last_added_section,
                    );
                    last_added_section = Some(new_last_added_section);
                    symbol_instance.placed_vertical_text_index = new_placed_index
                }
                assert!(last_added_section.is_some()); // True, as hasText == true;
                self.update_paint_properties_for_section(
                    &mut bucket,
                    feature,
                    last_added_section.unwrap(),
                    canonical,
                );
            }

            symbol_instance.release_shared_data();
        }

        if show_collision_boxes {
            self.add_to_debug_buffers(&mut bucket);
        }
        if bucket.has_data() {
            for pair in &self.layer_paint_properties {
                if !first_load {
                    bucket.just_reloaded = true;
                }
                render_data.insert(
                    pair.0.clone(),
                    LayerRenderData {
                        bucket: bucket.clone(), // TODO is cloning intended here?
                        layer_properties: pair.1.clone(),
                    },
                );
            }
        }
    }

    /// maplibre/maplibre-native#4add9ea original name: hasSymbolInstances
    fn has_symbol_instances(&self) -> bool {
        !self.symbol_instances.is_empty()
    }
    /// maplibre/maplibre-native#4add9ea original name: hasDependencies
    fn has_dependencies(&self) -> bool {
        !self.features.is_empty()
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
    /// maplibre/maplibre-native#4add9ea original name: evaluateVariableOffset
    pub fn evaluate_variable_offset(anchor: SymbolAnchorType, mut offset: [f64; 2]) -> [f64; 2] {
        if offset[1] == Self::INVALID_OFFSET_VALUE {
            return evaluate_radial_offset(anchor, offset[0]);
        }
        let mut result = [0.0, 0.0];
        offset[0] = (offset[0]).abs();
        offset[1] = (offset[1]).abs();

        match anchor {
            SymbolAnchorType::TopRight | SymbolAnchorType::TopLeft | SymbolAnchorType::Top => {
                result[1] = offset[1] - BASELINE_OFFSET;
            }

            SymbolAnchorType::BottomRight
            | SymbolAnchorType::BottomLeft
            | SymbolAnchorType::Bottom => {
                result[1] = -offset[1] + BASELINE_OFFSET;
            }

            SymbolAnchorType::Center | SymbolAnchorType::Left | SymbolAnchorType::Right => {}
        }

        match anchor {
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

        result
    }

    // Analog of `addToLineVertexArray` in JS. This version doesn't need to build up
    // a line array like the JS version does, but it uses the same logic to
    // calculate tile distances.
    /// maplibre/maplibre-native#4add9ea original name: calculateTileDistances
    pub fn calculate_tile_distances(line: &GeometryCoordinates, anchor: &Anchor) -> Vec<f64> {
        let mut tile_distances: Vec<f64> = vec![0.0; line.len()];
        if let Some(segment) = anchor.segment {
            assert!(segment < line.len());
            let mut sum_forward_length = if segment + 1 < line.len() {
                anchor.point.distance_to(line[segment + 1].cast::<f64>())
            } else {
                0.0
            };
            let mut sum_backward_length = anchor.point.distance_to(line[segment].cast::<f64>());
            for i in segment + 1..line.len() {
                tile_distances[i] = sum_forward_length;
                if i < line.len() - 1 {
                    sum_forward_length +=
                        line[i + 1].cast::<f64>().distance_to(line[i].cast::<f64>());
                }
            }

            let mut i = segment;
            loop {
                tile_distances[i] = sum_backward_length;
                if i != 0 {
                    sum_backward_length +=
                        line[i - 1].cast::<f64>().distance_to(line[i].cast::<f64>());
                } else {
                    break; // Add break to avoid unsigned integer overflow when i==0
                }
                i -= 1;
            }
        }
        tile_distances
    }
}

impl SymbolLayout {
    /// maplibre/maplibre-native#4add9ea original name: addFeature
    fn add_feature(
        &mut self,
        layout_feature_index: usize,
        feature: &SymbolGeometryTileFeature,
        shaped_text_orientations: &ShapedTextOrientations,
        mut shaped_icon: Option<PositionedIcon>, // TODO should this be an output?
        image_map: &ImageMap,
        text_offset: [f64; 2],
        layout_text_size: f64,
        layout_icon_size: f64,
        icon_type: SymbolContent,
    ) {
        let min_scale = 0.5;
        let glyph_size = 24.0;

        let icon_offset: [f64; 2] =
            self.layout
                .evaluate::<IconOffset>(self.zoom, feature, self.canonical_id);

        // To reduce the number of labels that jump around when zooming we need
        // to use a text-size value that is the same for all zoom levels.
        // This calculates text-size at a high zoom level so that all tiles can
        // use the same value when calculating anchor positions.
        let text_max_size = self
            .layout
            .evaluate::<TextSize>(18., feature, self.canonical_id);

        let font_scale = layout_text_size / glyph_size;
        let text_box_scale = self.tile_pixel_ratio * font_scale;
        let text_max_box_scale = self.tile_pixel_ratio * text_max_size / glyph_size;
        let icon_box_scale = self.tile_pixel_ratio * layout_icon_size;
        let symbol_spacing = self.tile_pixel_ratio * self.layout.get::<SymbolSpacing>();
        let text_padding = self.layout.get::<TextPadding>() * self.tile_pixel_ratio;
        let icon_padding = self.layout.get::<IconPadding>() * self.tile_pixel_ratio;
        let text_max_angle = deg2radf(self.layout.get::<TextMaxAngle>());
        let icon_rotation =
            self.layout
                .evaluate::<IconRotate>(self.zoom, feature, self.canonical_id);
        let text_rotation =
            self.layout
                .evaluate::<TextRotate>(self.zoom, feature, self.canonical_id);
        let variable_text_offset: [f64; 2];
        if !self.text_radial_offset.is_undefined() {
            variable_text_offset = [
                self.layout
                    .evaluate::<TextRadialOffset>(self.zoom, feature, self.canonical_id)
                    * ONE_EM,
                Self::INVALID_OFFSET_VALUE,
            ];
        } else {
            variable_text_offset = [
                self.layout
                    .evaluate::<TextOffset>(self.zoom, feature, self.canonical_id)[0]
                    * ONE_EM,
                self.layout
                    .evaluate::<TextOffset>(self.zoom, feature, self.canonical_id)[1]
                    * ONE_EM,
            ];
        }

        let text_placement: SymbolPlacementType =
            if self.layout.get::<TextRotationAlignment>() != AlignmentType::Map {
                SymbolPlacementType::Point
            } else {
                self.layout.get::<SymbolPlacement>()
            };

        let text_repeat_distance: f64 = symbol_spacing / 2.;
        let evaluated_layout_properties: SymbolLayoutProperties_Evaluated =
            self.layout.evaluate_feature(self.zoom, feature);
        let indexed_feature = IndexedSubfeature {
            ref_: RefIndexedSubfeature {
                index: feature.index,
                sort_index: self.symbol_instances.len(),
                source_layer_name: self.source_layer.get_name().to_string(),
                bucket_leader_id: self.bucket_leader_id.clone(),
                bucket_instance_id: 0,
                collision_group_id: 0,
            },
            source_layer_name_copy: self.source_layer.get_name().to_string(),
            bucket_leader_idcopy: self.bucket_leader_id.clone(),
        };

        let icon_text_fit = evaluated_layout_properties.get::<IconTextFit>();
        let has_icon_text_fit = icon_text_fit != IconTextFitType::None;
        // Adjust shaped icon size when icon-text-fit is used.
        let mut vertically_shaped_icon: Option<PositionedIcon> = None;

        if let Some(shaped_icon) = &mut shaped_icon {
            if has_icon_text_fit {
                // Create vertically shaped icon for vertical writing mode if needed.
                if self.allow_vertical_placement
                    && shaped_text_orientations.vertical().is_any_line_not_empty()
                {
                    vertically_shaped_icon = Some(shaped_icon.clone());
                    vertically_shaped_icon.as_mut().unwrap().fit_icon_to_text(
                        shaped_text_orientations.vertical(),
                        icon_text_fit,
                        &self.layout.get::<IconTextFitPadding>(),
                        &icon_offset,
                        font_scale,
                    );
                }
                let shaped_text = get_default_horizontal_shaping(shaped_text_orientations);
                if shaped_text.is_any_line_not_empty() {
                    shaped_icon.fit_icon_to_text(
                        shaped_text,
                        icon_text_fit,
                        &self.layout.get::<IconTextFitPadding>(),
                        &icon_offset,
                        font_scale,
                    );
                }
            }
        }

        let mut add_symbol_instance =
            |anchor: &Anchor, shared_data: Rc<SymbolInstanceSharedData>| {
                // assert!(sharedData); TODO
                let anchor_inside_tile = anchor.point.x >= 0.
                    && anchor.point.x < EXTENT
                    && anchor.point.y >= 0.
                    && anchor.point.y < EXTENT;

                if self.mode == MapMode::Tile || anchor_inside_tile {
                    // For static/continuous rendering, only add symbols anchored within this tile:
                    //  neighboring symbols will be added as part of the neighboring tiles.
                    // In tiled rendering mode, add all symbols in the buffers so that we can:
                    //  (1) render symbols that overlap into this tile
                    //  (2) approximate collision detection effects from neighboring symbols
                    self.symbol_instances.push(SymbolInstance::new(
                        *anchor,
                        shared_data,
                        shaped_text_orientations,
                        &shaped_icon,
                        &vertically_shaped_icon,
                        text_box_scale,
                        text_padding,
                        text_placement,
                        text_offset,
                        icon_box_scale,
                        icon_padding,
                        icon_offset,
                        indexed_feature.clone(),
                        layout_feature_index,
                        feature.index,
                        if let Some(formatted_text) = &feature.formatted_text {
                            formatted_text.raw_text().clone()
                        } else {
                            U16String::new()
                        },
                        self.overscaling,
                        icon_rotation,
                        text_rotation,
                        variable_text_offset,
                        self.allow_vertical_placement,
                        icon_type,
                    ));

                    if self.sort_features_by_key {
                        if !self.sort_key_ranges.is_empty()
                            && self.sort_key_ranges.last().unwrap().sort_key == feature.sort_key
                        {
                            self.sort_key_ranges.last_mut().unwrap().end =
                                self.symbol_instances.len();
                        } else {
                            self.sort_key_ranges.push(SortKeyRange {
                                sort_key: feature.sort_key,
                                start: self.symbol_instances.len() - 1,
                                end: self.symbol_instances.len(),
                            });
                        }
                    }
                }
            };

        let create_symbol_instance_shared_data = |line: GeometryCoordinates| {
            Rc::new(SymbolInstanceSharedData::new(
                line,
                shaped_text_orientations,
                shaped_icon.clone(),
                vertically_shaped_icon.clone(),
                &evaluated_layout_properties,
                text_placement,
                text_offset,
                image_map,
                icon_rotation,
                icon_type,
                has_icon_text_fit,
                self.allow_vertical_placement,
            ))
        };

        let type_ = feature.get_type();

        if self.layout.get::<SymbolPlacement>() == SymbolPlacementType::Line {
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
        } else if self.layout.get::<SymbolPlacement>() == SymbolPlacementType::LineCenter {
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
        } else if type_ == FeatureType::Polygon {
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
        } else if type_ == FeatureType::LineString {
            for line in &feature.geometry {
                // Skip invalid LineStrings.
                if line.0.is_empty() {
                    continue;
                }

                let anchor = Anchor {
                    point: Point2D::new((line[0].x) as f64, (line[0].y) as f64),
                    angle: 0.0,
                    segment: Some((min_scale) as usize),
                };
                add_symbol_instance(&anchor, create_symbol_instance_shared_data(line.clone()));
            }
        } else if type_ == FeatureType::Point {
            for points in &feature.geometry {
                for point in &points.0 {
                    let anchor = Anchor {
                        point: Point2D::new((point.x) as f64, (point.y) as f64),
                        angle: 0.0,
                        segment: Some((min_scale) as usize),
                    };
                    add_symbol_instance(
                        &anchor,
                        create_symbol_instance_shared_data(GeometryCoordinates(vec![*point])),
                    );
                }
            }
        }
    }

    /// maplibre/maplibre-native#4add9ea original name: anchorIsTooClose
    fn anchor_is_too_close(
        &mut self,
        text: &U16String,
        repeat_distance: f64,
        anchor: &Anchor,
    ) -> bool {
        if let Some(other_anchors) = self.compare_text.get(text) {
            for other_anchor in other_anchors {
                if anchor.point.distance_to(other_anchor.point) < repeat_distance {
                    return true;
                }
            }
        } else {
            self.compare_text.insert(text.clone(), Anchors::new());
        }

        let anchors = self.compare_text.get_mut(text).unwrap();
        anchors.push(*anchor);
        false
    }

    /// maplibre/maplibre-native#4add9ea original name: addToDebugBuffers
    fn add_to_debug_buffers(&self, bucket: &mut SymbolBucket) {
        todo!()
    }

    // Adds placed items to the buffer.
    /// maplibre/maplibre-native#4add9ea original name: addSymbol
    fn add_symbol(
        &self,
        buffer: &mut SymbolBucketBuffer,
        size_data: Range<f64>,
        symbol: &SymbolQuad,
        label_anchor: &Anchor,
        sort_key: f64,
    ) -> usize {
        let vertex_length: u16 = 4;

        let tl = symbol.tl;
        let tr = symbol.tr;
        let bl = symbol.bl;
        let br = symbol.br;
        let tex = symbol.tex;
        let pixel_offset_tl = symbol.pixel_offset_tl;
        let pixel_offset_br = symbol.pixel_offset_br;
        let min_font_scale = symbol.min_font_scale;

        if buffer.segments.is_empty()
            || buffer.segments.last().unwrap().vertex_length + vertex_length as usize
                > u16::MAX as usize
            || (buffer.segments.last().unwrap().sort_key - sort_key).abs() > f64::EPSILON
        {
            buffer.segments.push(Segment {
                vertex_offset: buffer.shared_vertices.len(),
                index_offset: buffer.triangles.len(),
                vertex_length: 0,
                index_length: 0,
                sort_key,
                _phandom_data: Default::default(),
            });
        }

        // We're generating triangle fans, so we always start with the first
        // coordinate in this polygon.
        let segment = buffer.segments.last_mut().unwrap();
        assert!(segment.vertex_length <= u16::MAX as usize);
        let index = (segment.vertex_length) as u16;

        // coordinates (2 triangles)
        let vertices = &mut buffer.shared_vertices;
        vertices.push(SymbolVertex::new(
            label_anchor.point,
            tl,
            symbol.glyph_offset.y,
            tex.origin.x,
            tex.origin.y,
            size_data.clone(),
            symbol.is_sdf,
            pixel_offset_tl,
            min_font_scale,
        ));
        vertices.push(SymbolVertex::new(
            label_anchor.point,
            tr,
            symbol.glyph_offset.y,
            tex.origin.x + tex.width(),
            tex.origin.y,
            size_data.clone(),
            symbol.is_sdf,
            Point2D::new(pixel_offset_br.x, pixel_offset_tl.y),
            min_font_scale,
        ));
        vertices.push(SymbolVertex::new(
            label_anchor.point,
            bl,
            symbol.glyph_offset.y,
            tex.origin.x,
            tex.origin.y + tex.height(),
            size_data.clone(),
            symbol.is_sdf,
            Point2D::new(pixel_offset_tl.x, pixel_offset_br.y),
            min_font_scale,
        ));
        vertices.push(SymbolVertex::new(
            label_anchor.point,
            br,
            symbol.glyph_offset.y,
            tex.origin.x + tex.width(),
            tex.origin.y + tex.height(),
            size_data.clone(),
            symbol.is_sdf,
            pixel_offset_br,
            min_font_scale,
        ));

        // Dynamic/Opacity vertices are initialized so that the vertex count always
        // agrees with the layout vertex buffer, but they will always be updated
        // before rendering happens
        let dynamic_vertex = DynamicVertex::new(label_anchor.point, 0.);
        buffer.shared_dynamic_vertices.push(dynamic_vertex);
        buffer.shared_dynamic_vertices.push(dynamic_vertex);
        buffer.shared_dynamic_vertices.push(dynamic_vertex);
        buffer.shared_dynamic_vertices.push(dynamic_vertex);

        let opacity_vertex = OpacityVertex::new(true, 1.0);
        buffer.shared_opacity_vertices.push(opacity_vertex);
        buffer.shared_opacity_vertices.push(opacity_vertex);
        buffer.shared_opacity_vertices.push(opacity_vertex);
        buffer.shared_opacity_vertices.push(opacity_vertex);

        // add the two triangles, referencing the four coordinates we just inserted.
        buffer.triangles.push(index, index + 1, index + 2);
        buffer.triangles.push(index + 1, index + 2, index + 3);

        segment.vertex_length += vertex_length as usize;
        segment.index_length += 6;

        index as usize
    }
    /// maplibre/maplibre-native#4add9ea original name: addSymbols
    fn add_symbols(
        &self,
        buffer: &mut SymbolBucketBuffer,
        size_data: Range<f64>,
        symbols: &SymbolQuads,
        label_anchor: &Anchor,
        placed_symbol: &mut PlacedSymbol,
        sort_key: f64,
    ) -> usize {
        let mut first_symbol = true;
        let mut first_index = 0;
        for symbol in symbols {
            let index = self.add_symbol(buffer, size_data.clone(), symbol, label_anchor, sort_key);
            placed_symbol.glyph_offsets.push(symbol.glyph_offset.x);
            if first_symbol {
                first_index = index;
                first_symbol = false;
            }
        }
        first_index
    }

    // Adds symbol quads to bucket and returns formatted section index of last
    // added quad.
    /// maplibre/maplibre-native#4add9ea original name: addSymbolGlyphQuads
    fn add_symbol_glyph_quads(
        &self,
        bucket: &mut SymbolBucket,
        symbol_instance: &SymbolInstance,
        feature: &SymbolGeometryTileFeature,
        writing_mode: WritingModeType,
        placed_index: Option<usize>, // TODO should this be an output?
        glyph_quads: &SymbolQuads,
        canonical: &CanonicalTileID,
        mut last_added_section: Option<usize>, // TODO should this be an output?
    ) -> (usize, Option<usize>) {
        let mut output_placed_index = placed_index;
        let size_data: Range<f64> = bucket.text_size_binder.get_vertex_size_data(feature); // TODO verify if usage of range is oke, empty ranges, reverse
        let has_format_section_overrides: bool = bucket.has_format_section_overrides();
        let placed_icon_index = if writing_mode == WritingModeType::Vertical {
            symbol_instance.placed_vertical_icon_index
        } else {
            symbol_instance.placed_icon_index
        };

        // TODO is this PlacedSymbol correct?
        let mut newly_placed_symbol = PlacedSymbol {
            anchor_point: symbol_instance.anchor.point,
            segment: symbol_instance.anchor.segment.unwrap_or(0),
            lower_size: size_data.start,
            upper_size: size_data.end,
            line_offset: symbol_instance.text_offset,
            writing_modes: writing_mode,
            line: symbol_instance.line().clone(),
            tile_distances: Self::calculate_tile_distances(
                symbol_instance.line(),
                &symbol_instance.anchor,
            ),

            glyph_offsets: vec![],
            hidden: false,
            vertex_start_index: 0,
            cross_tile_id: 0,
            placed_orientation: None,
            angle: if self.allow_vertical_placement && writing_mode == WritingModeType::Vertical {
                PI / 2.
            } else {
                0.0
            },
            placed_icon_index,
        };

        let mut first_symbol = true;
        for symbol_quad in glyph_quads {
            if has_format_section_overrides {
                if let Some(last_added_section) = last_added_section {
                    if last_added_section != symbol_quad.section_index {
                        self.update_paint_properties_for_section(
                            bucket,
                            feature,
                            last_added_section,
                            canonical,
                        );
                    }
                }

                last_added_section = Some(symbol_quad.section_index);
            }
            let index = self.add_symbol(
                &mut bucket.text,
                size_data.clone(),
                symbol_quad,
                &symbol_instance.anchor,
                feature.sort_key,
            );

            newly_placed_symbol
                .glyph_offsets
                .push(symbol_quad.glyph_offset.x);

            if first_symbol {
                newly_placed_symbol.vertex_start_index = index;
                first_symbol = false;
            }
        }

        bucket.text.placed_symbols.push(newly_placed_symbol);
        output_placed_index = Some(bucket.text.placed_symbols.len() - 1);

        if let Some(last_added_section) = last_added_section {
            (last_added_section, output_placed_index)
        } else {
            (0, output_placed_index)
        }
    }

    /// maplibre/maplibre-native#4add9ea original name: updatePaintPropertiesForSection
    fn update_paint_properties_for_section(
        &self,
        bucket: &SymbolBucket,
        feature: &SymbolGeometryTileFeature,
        section_index: usize,
        canonical: &CanonicalTileID,
    ) -> usize {
        let formatted_section = section_options_to_value(
            feature
                .formatted_text
                .as_ref()
                .unwrap()
                .section_at(section_index),
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
        0
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::{
        euclid::{Point2D, Rect, Size2D},
        legacy::{
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
    /// maplibre/maplibre-native#4add9ea original name: test
    fn test() {
        let font_stack = vec![
            "Open Sans Regular".to_string(),
            "Arial Unicode MS Regular".to_string(),
        ];

        let section_options = SectionOptions::new(1.0, font_stack.clone(), None);

        let mut glyph_dependencies = GlyphDependencies::new();

        let tile_id = OverscaledTileID {
            canonical: CanonicalTileID { x: 0, y: 0, z: 0 },
            overscaled_z: 0,
        };
        let mut parameters = BucketParameters {
            tile_id: tile_id,
            mode: MapMode::Continuous,
            pixel_ratio: 1.0,
            layer_type: LayerTypeInfo,
        };
        let layer_data = SymbolGeometryTileLayer {
            name: "layer".to_string(),
            features: vec![SymbolGeometryTileFeature::new(Box::new(
                VectorGeometryTileFeature {
                    geometry: vec![GeometryCoordinates(vec![Point2D::new(1024, 1024)])],
                },
            ))],
        };
        let layer_properties = vec![LayerProperties {
            id: "layer".to_string(),
            layer: SymbolLayer {
                layout: SymbolLayoutProperties_Unevaluated,
            },
        }];

        let image_positions = ImagePositions::new();

        let mut glyph_position = GlyphPosition {
            rect: Rect::new(Point2D::new(0, 0), Size2D::new(10, 10)),
            metrics: GlyphMetrics {
                width: 18,
                height: 18,
                left: 2,
                top: -8,
                advance: 21,
            },
        };
        let glyph_positions: GlyphPositions = GlyphPositions::from([(
            FontStackHasher::new(&font_stack),
            GlyphPositionMap::from([('中' as Char16, glyph_position)]),
        )]);

        let mut glyph = Glyph::default();
        glyph.id = '中' as Char16;
        glyph.metrics = glyph_position.metrics;

        let glyphs: GlyphMap = GlyphMap::from([(
            FontStackHasher::new(&font_stack),
            Glyphs::from([('中' as Char16, Some(glyph))]),
        )]);

        let mut layout = SymbolLayout::new(
            &parameters,
            &layer_properties,
            Box::new(layer_data),
            &mut LayoutParameters {
                bucket_parameters: &mut parameters.clone(),
                glyph_dependencies: &mut glyph_dependencies,
                image_dependencies: &mut Default::default(),
                available_images: &mut Default::default(),
            },
        )
        .unwrap();

        assert_eq!(glyph_dependencies.len(), 1);

        let empty_image_map = ImageMap::new();
        layout.prepare_symbols(
            &glyphs,
            &glyph_positions,
            &empty_image_map,
            &image_positions,
        );

        let mut output = HashMap::new();
        layout.create_bucket(
            image_positions,
            Box::new(FeatureIndex),
            &mut output,
            false,
            false,
            &tile_id.canonical,
        );

        println!(
            "{:#?}",
            output.get("layer").unwrap().bucket.text.shared_vertices
        )
    }
}
