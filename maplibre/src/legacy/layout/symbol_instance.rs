//! Translated from https://github.com/maplibre/maplibre-native/blob/4add9ea/src/mbgl/layout/symbol_instance.cpp

use std::rc::Rc;

use bitflags::bitflags;
use widestring::U16String;

use crate::legacy::{
    collision_feature::CollisionFeature,
    geometry::{anchor::Anchor, feature_index::IndexedSubfeature},
    geometry_tile_data::GeometryCoordinates,
    glyph::{Shaping, WritingModeType},
    image::ImageMap,
    quads::{get_glyph_quads, get_icon_quads, SymbolQuads},
    shaping::PositionedIcon,
    style_types::{SymbolLayoutProperties_Evaluated, SymbolPlacementType},
};

/// maplibre/maplibre-native#4add9ea original name: getAnyShaping
fn get_any_shaping(shaped_text_orientations: &ShapedTextOrientations) -> &Shaping {
    if shaped_text_orientations.right().is_any_line_not_empty() {
        return shaped_text_orientations.right();
    }
    if shaped_text_orientations.center.is_any_line_not_empty() {
        return &(shaped_text_orientations.center);
    }
    if shaped_text_orientations.left.is_any_line_not_empty() {
        return &(shaped_text_orientations.left);
    }
    if shaped_text_orientations.vertical.is_any_line_not_empty() {
        return &(shaped_text_orientations.vertical);
    }
    &shaped_text_orientations.horizontal
}

/// maplibre/maplibre-native#4add9ea original name: ShapedTextOrientations
#[derive(Default)]
pub struct ShapedTextOrientations {
    horizontal: Shaping,
    vertical: Shaping,
    // The following are used with variable text placement on, including right()
    center: Shaping,
    left: Shaping,
    pub single_line: bool,
}

impl ShapedTextOrientations {
    /// maplibre/maplibre-native#4add9ea original name: new
    pub fn new(
        horizontal: Shaping,
        vertical: Shaping,
        right: Option<Shaping>,
        center: Shaping,
        left: Shaping,
        single_line: bool,
    ) -> Self {
        Self {
            horizontal: (horizontal),
            vertical: (vertical),
            center: (center),
            left: (left),
            single_line: single_line,
        }
    }

    /// maplibre/maplibre-native#4add9ea original name: horizontal
    pub fn horizontal(&self) -> &Shaping {
        &self.horizontal
    }
    /// maplibre/maplibre-native#4add9ea original name: vertical
    pub fn vertical(&self) -> &Shaping {
        &self.vertical
    }
    /// maplibre/maplibre-native#4add9ea original name: right
    pub fn right(&self) -> &Shaping {
        &self.horizontal
    }
    /// maplibre/maplibre-native#4add9ea original name: center
    pub fn center(&self) -> &Shaping {
        &self.center
    }
    /// maplibre/maplibre-native#4add9ea original name: left
    pub fn left(&self) -> &Shaping {
        &self.left
    }

    /// maplibre/maplibre-native#4add9ea original name: set_horizontal
    pub fn set_horizontal(&mut self, horizontal: Shaping) {
        self.horizontal = horizontal;
    }
    /// maplibre/maplibre-native#4add9ea original name: set_vertical
    pub fn set_vertical(&mut self, vertical: Shaping) {
        self.vertical = vertical;
    }
}

bitflags! {
    /// maplibre/maplibre-native#4add9ea original name: SymbolContent:
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct SymbolContent: u8 {
         const None = 0;
         const Text = 1 << 0;
         const IconRGBA = 1 << 1;
         const IconSDF = 1 << 2;
    }
}

/// maplibre/maplibre-native#4add9ea original name: SymbolInstanceSharedData
#[derive(Default)]
pub struct SymbolInstanceSharedData {
    line: GeometryCoordinates,
    // Note: When singleLine == true, only `rightJustifiedGlyphQuads` is populated.
    right_justified_glyph_quads: SymbolQuads,
    center_justified_glyph_quads: SymbolQuads,
    left_justified_glyph_quads: SymbolQuads,
    vertical_glyph_quads: SymbolQuads,
    icon_quads: Option<SymbolQuads>,
    vertical_icon_quads: Option<SymbolQuads>,
}

impl SymbolInstanceSharedData {
    /// maplibre/maplibre-native#4add9ea original name: new
    pub fn new(
        line_: GeometryCoordinates,
        shaped_text_orientations: &ShapedTextOrientations,
        shaped_icon: Option<PositionedIcon>,
        vertically_shaped_icon: Option<PositionedIcon>,
        layout: &SymbolLayoutProperties_Evaluated,
        text_placement: SymbolPlacementType,
        text_offset: [f64; 2],
        image_map: &ImageMap,
        icon_rotation: f64,
        icon_type: SymbolContent,
        has_icon_text_fit: bool,
        allow_vertical_placement: bool,
    ) -> Self {
        let mut self_ = Self {
            line: line_,
            ..Self::default()
        };
        // Create the quads used for rendering the icon and glyphs.
        if let Some(shapedIcon) = &shaped_icon {
            self_.icon_quads = Some(get_icon_quads(
                shapedIcon,
                icon_rotation,
                icon_type,
                has_icon_text_fit,
            ));
            if let Some(verticallyShapedIcon) = &vertically_shaped_icon {
                self_.vertical_icon_quads = Some(get_icon_quads(
                    verticallyShapedIcon,
                    icon_rotation,
                    icon_type,
                    has_icon_text_fit,
                ));
            }
        }

        // todo is this translation correct?
        if !shaped_text_orientations.single_line {
            if shaped_text_orientations.right().is_any_line_not_empty() {
                self_.right_justified_glyph_quads = get_glyph_quads(
                    shaped_text_orientations.right(),
                    text_offset,
                    layout,
                    text_placement,
                    image_map,
                    allow_vertical_placement,
                );
            }

            if shaped_text_orientations.center.is_any_line_not_empty() {
                self_.center_justified_glyph_quads = get_glyph_quads(
                    &shaped_text_orientations.center,
                    text_offset,
                    layout,
                    text_placement,
                    image_map,
                    allow_vertical_placement,
                );
            }

            if shaped_text_orientations.left.is_any_line_not_empty() {
                self_.left_justified_glyph_quads = get_glyph_quads(
                    &shaped_text_orientations.left,
                    text_offset,
                    layout,
                    text_placement,
                    image_map,
                    allow_vertical_placement,
                );
            }
        } else {
            let shape = if shaped_text_orientations.right().is_any_line_not_empty() {
                Some(shaped_text_orientations.right())
            } else if shaped_text_orientations.center.is_any_line_not_empty() {
                Some(&shaped_text_orientations.center)
            } else if shaped_text_orientations.left.is_any_line_not_empty() {
                Some(&shaped_text_orientations.left)
            } else {
                None
            };

            if let Some(shape) = shape {
                self_.right_justified_glyph_quads = get_glyph_quads(
                    shape,
                    text_offset,
                    layout,
                    text_placement,
                    image_map,
                    allow_vertical_placement,
                );
            }
        }

        if shaped_text_orientations.vertical.is_any_line_not_empty() {
            self_.vertical_glyph_quads = get_glyph_quads(
                &shaped_text_orientations.vertical,
                text_offset,
                layout,
                text_placement,
                image_map,
                allow_vertical_placement,
            );
        }
        self_
    }
    /// maplibre/maplibre-native#4add9ea original name: empty
    fn empty(&self) -> bool {
        self.right_justified_glyph_quads.is_empty()
            && self.center_justified_glyph_quads.is_empty()
            && self.left_justified_glyph_quads.is_empty()
            && self.vertical_glyph_quads.is_empty()
    }
}

/// maplibre/maplibre-native#4add9ea original name: SymbolInstance
#[derive(Clone)]
pub struct SymbolInstance {
    shared_data: Rc<SymbolInstanceSharedData>,

    pub anchor: Anchor,
    pub symbol_content: SymbolContent,

    pub right_justified_glyph_quads_size: usize,
    pub center_justified_glyph_quads_size: usize,
    pub left_justified_glyph_quads_size: usize,
    pub vertical_glyph_quads_size: usize,
    pub icon_quads_size: usize,

    pub text_collision_feature: CollisionFeature,
    pub icon_collision_feature: CollisionFeature,
    pub vertical_text_collision_feature: Option<CollisionFeature>,
    pub vertical_icon_collision_feature: Option<CollisionFeature>,
    pub writing_modes: WritingModeType,
    pub layout_feature_index: usize, // Index into the set of features included at layout time
    pub data_feature_index: usize,   // Index into the underlying tile data feature set
    pub text_offset: [f64; 2],
    pub icon_offset: [f64; 2],
    pub key: U16String,
    pub placed_right_text_index: Option<usize>,
    pub placed_center_text_index: Option<usize>,
    pub placed_left_text_index: Option<usize>,
    pub placed_vertical_text_index: Option<usize>,
    pub placed_icon_index: Option<usize>,
    pub placed_vertical_icon_index: Option<usize>,
    pub text_box_scale: f64,
    pub variable_text_offset: [f64; 2],
    pub single_line: bool,
    pub cross_tile_id: u32,
}

impl SymbolInstance {
    /// maplibre/maplibre-native#4add9ea original name: new
    pub fn new(
        anchor_: Anchor,
        shared_data: Rc<SymbolInstanceSharedData>,
        shaped_text_orientations: &ShapedTextOrientations,
        shaped_icon: &Option<PositionedIcon>,
        vertically_shaped_icon: &Option<PositionedIcon>,
        text_box_scale: f64,
        text_padding: f64,
        text_placement: SymbolPlacementType,
        text_offset: [f64; 2],
        icon_box_scale: f64,
        icon_padding: f64,
        icon_offset: [f64; 2],
        indexed_feature: IndexedSubfeature,
        layout_feature_index: usize,
        data_feature_index: usize,
        key_: U16String,
        overscaling: f64,
        icon_rotation: f64,
        text_rotation: f64,
        variable_text_offset: [f64; 2],
        allow_vertical_placement: bool,
        icon_type: SymbolContent,
    ) -> Self {
        let mut self_ = Self {
            symbol_content: icon_type,
            // Create the collision features that will be used to check whether this
            // symbol instance can be placed As a collision approximation, we can use
            // either the vertical or any of the horizontal versions of the feature
            text_collision_feature: CollisionFeature::new_from_text(
                &shared_data.line,
                &anchor_,
                get_any_shaping(shaped_text_orientations).clone(),
                text_box_scale,
                text_padding,
                text_placement,
                indexed_feature.clone(),
                overscaling,
                text_rotation,
            ),
            icon_collision_feature: CollisionFeature::new_from_icon(
                &shared_data.line,
                &anchor_,
                shaped_icon,
                icon_box_scale,
                icon_padding,
                indexed_feature.clone(),
                icon_rotation,
            ),

            shared_data: shared_data,
            anchor: anchor_,
            writing_modes: WritingModeType::None,
            layout_feature_index: layout_feature_index,
            data_feature_index: data_feature_index,
            text_offset: text_offset,
            icon_offset: icon_offset,
            key: key_,

            text_box_scale: text_box_scale,
            variable_text_offset: variable_text_offset,
            single_line: shaped_text_orientations.single_line,

            right_justified_glyph_quads_size: 0,
            center_justified_glyph_quads_size: 0,
            left_justified_glyph_quads_size: 0,
            vertical_glyph_quads_size: 0,
            icon_quads_size: 0,

            vertical_text_collision_feature: None,
            placed_right_text_index: None,
            placed_center_text_index: None,
            placed_left_text_index: None,
            placed_vertical_text_index: None,
            placed_icon_index: None,
            placed_vertical_icon_index: None,

            vertical_icon_collision_feature: None,
            cross_tile_id: 0,
        };

        // 'hasText' depends on finding at least one glyph in the shaping that's also in the GlyphPositionMap
        if !self_.shared_data.empty() {
            self_.symbol_content |= SymbolContent::Text;
        }
        if allow_vertical_placement && shaped_text_orientations.vertical.is_any_line_not_empty() {
            let vertical_point_label_angle = 90.0;
            self_.vertical_text_collision_feature = Some(CollisionFeature::new_from_text(
                self_.line(),
                &self_.anchor,
                shaped_text_orientations.vertical.clone(),
                text_box_scale,
                text_padding,
                text_placement,
                indexed_feature.clone(),
                overscaling,
                text_rotation + vertical_point_label_angle,
            ));
            if vertically_shaped_icon.is_some() {
                self_.vertical_icon_collision_feature = Some(CollisionFeature::new_from_icon(
                    &self_.shared_data.line,
                    &self_.anchor,
                    vertically_shaped_icon,
                    icon_box_scale,
                    icon_padding,
                    indexed_feature,
                    icon_rotation + vertical_point_label_angle,
                ));
            }
        }

        self_.right_justified_glyph_quads_size =
            self_.shared_data.right_justified_glyph_quads.len();
        self_.center_justified_glyph_quads_size =
            self_.shared_data.center_justified_glyph_quads.len();
        self_.left_justified_glyph_quads_size = self_.shared_data.left_justified_glyph_quads.len();
        self_.vertical_glyph_quads_size = self_.shared_data.vertical_glyph_quads.len();

        self_.icon_quads_size = if let Some(iconQuads) = &self_.shared_data.icon_quads {
            iconQuads.len()
        } else {
            0
        };

        if self_.right_justified_glyph_quads_size != 0
            || self_.center_justified_glyph_quads_size != 0
            || self_.left_justified_glyph_quads_size != 0
        {
            self_.writing_modes |= WritingModeType::Horizontal;
        }

        if self_.vertical_glyph_quads_size != 0 {
            self_.writing_modes |= WritingModeType::Vertical;
        }

        self_
    }
    /// maplibre/maplibre-native#4add9ea original name: getDefaultHorizontalPlacedTextIndex
    pub fn get_default_horizontal_placed_text_index(&self) -> Option<usize> {
        if let Some(index) = self.placed_right_text_index {
            return Some(index);
        }
        if let Some(index) = self.placed_center_text_index {
            return Some(index);
        }
        if let Some(index) = self.placed_left_text_index {
            return Some(index);
        }
        None
    }
    /// maplibre/maplibre-native#4add9ea original name: line
    pub fn line(&self) -> &GeometryCoordinates {
        &self.shared_data.line
    }
    /// maplibre/maplibre-native#4add9ea original name: rightJustifiedGlyphQuads
    pub fn right_justified_glyph_quads(&self) -> &SymbolQuads {
        &self.shared_data.right_justified_glyph_quads
    }
    /// maplibre/maplibre-native#4add9ea original name: leftJustifiedGlyphQuads
    pub fn left_justified_glyph_quads(&self) -> &SymbolQuads {
        &self.shared_data.left_justified_glyph_quads
    }
    /// maplibre/maplibre-native#4add9ea original name: centerJustifiedGlyphQuads
    pub fn center_justified_glyph_quads(&self) -> &SymbolQuads {
        &self.shared_data.center_justified_glyph_quads
    }
    /// maplibre/maplibre-native#4add9ea original name: verticalGlyphQuads
    pub fn vertical_glyph_quads(&self) -> &SymbolQuads {
        &self.shared_data.vertical_glyph_quads
    }
    /// maplibre/maplibre-native#4add9ea original name: hasText
    pub fn has_text(&self) -> bool {
        self.symbol_content.contains(SymbolContent::Text) // TODO Is this correct?
    }
    /// maplibre/maplibre-native#4add9ea original name: hasIcon
    pub fn has_icon(&self) -> bool {
        self.symbol_content.contains(SymbolContent::IconRGBA) || self.has_sdf_icon()
    }
    /// maplibre/maplibre-native#4add9ea original name: hasSdfIcon
    pub fn has_sdf_icon(&self) -> bool {
        self.symbol_content.contains(SymbolContent::IconSDF)
    }
    /// maplibre/maplibre-native#4add9ea original name: iconQuads
    pub fn icon_quads(&self) -> &Option<SymbolQuads> {
        &self.shared_data.icon_quads
    }
    /// maplibre/maplibre-native#4add9ea original name: verticalIconQuads
    pub fn vertical_icon_quads(&self) -> &Option<SymbolQuads> {
        &self.shared_data.vertical_icon_quads
    }
    /// maplibre/maplibre-native#4add9ea original name: releaseSharedData
    pub fn release_shared_data(&self) {
        // todo!()
        // TODO not sure how to do this self.sharedData.reset();
    }

    /// maplibre/maplibre-native#4add9ea original name: invalidCrossTileID
    fn invalid_cross_tile_id() -> u32 {
        u32::MAX
    }
}

/// maplibre/maplibre-native#4add9ea original name: SymbolInstanceReferences
type SymbolInstanceReferences = Vec<SymbolInstance>;
