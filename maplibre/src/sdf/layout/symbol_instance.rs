use crate::sdf::collision_feature::CollisionFeature;
use crate::sdf::geometry::feature_index::IndexedSubfeature;
use crate::sdf::geometry::Anchor;
use crate::sdf::geometry_tile_data::GeometryCoordinates;
use crate::sdf::glyph::{Shaping, WritingModeType};
use crate::sdf::image::ImageMap;
use crate::sdf::quads::{getGlyphQuads, getIconQuads, SymbolQuads};
use crate::sdf::shaping::PositionedIcon;
use crate::sdf::style_types::{SymbolLayoutProperties_Evaluated, SymbolPlacementType};
use bitflags::bitflags;
use std::rc::Rc;
use widestring::U16String;

fn getAnyShaping(shapedTextOrientations: &ShapedTextOrientations) -> &Shaping {
    if shapedTextOrientations.right.isAnyLineNotEmpty() {
        return &shapedTextOrientations.right;
    }
    if shapedTextOrientations.center.isAnyLineNotEmpty() {
        return &(shapedTextOrientations.center);
    }
    if shapedTextOrientations.left.isAnyLineNotEmpty() {
        return &(shapedTextOrientations.left);
    }
    if shapedTextOrientations.vertical.isAnyLineNotEmpty() {
        return &(shapedTextOrientations.vertical);
    }
    return &shapedTextOrientations.horizontal;
}

#[derive(Default)]
pub struct ShapedTextOrientations {
    pub horizontal: Shaping,
    pub vertical: Shaping,
    // The following are used with variable text placement on.
    pub right: Shaping,
    pub center: Shaping,
    pub left: Shaping,
    pub singleLine: bool,
}

impl ShapedTextOrientations {
    pub fn new(
        horizontal: Shaping,
        vertical: Shaping,
        right: Option<Shaping>,
        center: Shaping,
        left: Shaping,
        singleLine: bool,
    ) -> Self {
        Self {
            right: (right.unwrap_or(horizontal.clone())),
            horizontal: (horizontal),
            vertical: (vertical),
            center: (center),
            left: (left),
            singleLine,
        }
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct SymbolContent: u8 {
         const None = 0;
         const Text = 1 << 0;
         const IconRGBA = 1 << 1;
         const IconSDF = 1 << 2;
    }
}

#[derive(Default)]
pub struct SymbolInstanceSharedData {
    line: GeometryCoordinates,
    // Note: When singleLine == true, only `rightJustifiedGlyphQuads` is populated.
    rightJustifiedGlyphQuads: SymbolQuads,
    centerJustifiedGlyphQuads: SymbolQuads,
    leftJustifiedGlyphQuads: SymbolQuads,
    verticalGlyphQuads: SymbolQuads,
    iconQuads: Option<SymbolQuads>,
    verticalIconQuads: Option<SymbolQuads>,
}

impl SymbolInstanceSharedData {
    pub fn new(
        line_: GeometryCoordinates,
        shapedTextOrientations: &ShapedTextOrientations,
        shapedIcon: Option<PositionedIcon>,
        verticallyShapedIcon: Option<PositionedIcon>,
        layout: &SymbolLayoutProperties_Evaluated,
        textPlacement: SymbolPlacementType,
        textOffset: [f64; 2],
        imageMap: &ImageMap,
        iconRotation: f64,
        iconType: SymbolContent,
        hasIconTextFit: bool,
        allowVerticalPlacement: bool,
    ) -> Self {
        let mut self_ = Self {
            line: line_,
            ..Self::default()
        };
        // Create the quads used for rendering the icon and glyphs.
        if let Some(shapedIcon) = (&shapedIcon) {
            self_.iconQuads = Some(getIconQuads(
                shapedIcon,
                iconRotation,
                iconType,
                hasIconTextFit,
            ));
            if let Some(verticallyShapedIcon) = (&verticallyShapedIcon) {
                self_.verticalIconQuads = Some(getIconQuads(
                    verticallyShapedIcon,
                    iconRotation,
                    iconType,
                    hasIconTextFit,
                ));
            }
        }

        // todo is this translation correct?
        if (!shapedTextOrientations.singleLine) {
            if shapedTextOrientations.right.isAnyLineNotEmpty() {
                self_.rightJustifiedGlyphQuads = getGlyphQuads(
                    &shapedTextOrientations.right,
                    textOffset,
                    layout,
                    textPlacement,
                    imageMap,
                    allowVerticalPlacement,
                );
            }

            if shapedTextOrientations.center.isAnyLineNotEmpty() {
                self_.centerJustifiedGlyphQuads = getGlyphQuads(
                    &shapedTextOrientations.center,
                    textOffset,
                    layout,
                    textPlacement,
                    imageMap,
                    allowVerticalPlacement,
                );
            }

            if shapedTextOrientations.left.isAnyLineNotEmpty() {
                self_.leftJustifiedGlyphQuads = getGlyphQuads(
                    &shapedTextOrientations.left,
                    textOffset,
                    layout,
                    textPlacement,
                    imageMap,
                    allowVerticalPlacement,
                );
            }
        } else {
            let shape = if shapedTextOrientations.right.isAnyLineNotEmpty() {
                Some(&shapedTextOrientations.right)
            } else {
                if shapedTextOrientations.center.isAnyLineNotEmpty() {
                    Some(&shapedTextOrientations.center)
                } else {
                    if shapedTextOrientations.left.isAnyLineNotEmpty() {
                        Some(&shapedTextOrientations.left)
                    } else {
                        None
                    }
                }
            };

            if let Some(shape) = shape {
                self_.rightJustifiedGlyphQuads = getGlyphQuads(
                    shape,
                    textOffset,
                    layout,
                    textPlacement,
                    imageMap,
                    allowVerticalPlacement,
                );
            }
        }

        if shapedTextOrientations.vertical.isAnyLineNotEmpty() {
            self_.verticalGlyphQuads = getGlyphQuads(
                &shapedTextOrientations.vertical,
                textOffset,
                layout,
                textPlacement,
                imageMap,
                allowVerticalPlacement,
            );
        }
        self_
    }
    fn empty(&self) -> bool {
        return self.rightJustifiedGlyphQuads.is_empty()
            && self.centerJustifiedGlyphQuads.is_empty()
            && self.leftJustifiedGlyphQuads.is_empty()
            && self.verticalGlyphQuads.is_empty();
    }
}

pub struct SymbolInstance {
    sharedData: Rc<SymbolInstanceSharedData>,

    pub anchor: Anchor,
    pub symbolContent: SymbolContent,

    pub rightJustifiedGlyphQuadsSize: usize,
    pub centerJustifiedGlyphQuadsSize: usize,
    pub leftJustifiedGlyphQuadsSize: usize,
    pub verticalGlyphQuadsSize: usize,
    pub iconQuadsSize: usize,

    pub textCollisionFeature: CollisionFeature,
    pub iconCollisionFeature: CollisionFeature,
    pub verticalTextCollisionFeature: Option<CollisionFeature>,
    pub verticalIconCollisionFeature: Option<CollisionFeature>,
    pub writingModes: WritingModeType,
    pub layoutFeatureIndex: usize, // Index into the set of features included at layout time
    pub dataFeatureIndex: usize,   // Index into the underlying tile data feature set
    pub textOffset: [f64; 2],
    pub iconOffset: [f64; 2],
    pub key: U16String,
    pub placedRightTextIndex: Option<usize>,
    pub placedCenterTextIndex: Option<usize>,
    pub placedLeftTextIndex: Option<usize>,
    pub placedVerticalTextIndex: Option<usize>,
    pub placedIconIndex: Option<usize>,
    pub placedVerticalIconIndex: Option<usize>,
    pub textBoxScale: f64,
    pub variableTextOffset: [f64; 2],
    pub singleLine: bool,
    pub crossTileID: u32,
}

impl SymbolInstance {
    pub fn new(
        anchor_: Anchor,
        sharedData_: Rc<SymbolInstanceSharedData>,
        shapedTextOrientations: &ShapedTextOrientations,
        shapedIcon: &Option<PositionedIcon>,
        verticallyShapedIcon: &Option<PositionedIcon>,
        textBoxScale_: f64,
        textPadding: f64,
        textPlacement: SymbolPlacementType,
        textOffset_: [f64; 2],
        iconBoxScale: f64,
        iconPadding: f64,
        iconOffset_: [f64; 2],
        indexedFeature: IndexedSubfeature,
        layoutFeatureIndex_: usize,
        dataFeatureIndex_: usize,
        key_: U16String,
        overscaling: f64,
        iconRotation: f64,
        textRotation: f64,
        variableTextOffset_: [f64; 2],
        allowVerticalPlacement: bool,
        iconType: SymbolContent,
    ) -> Self {
        let mut self_ = Self {
            symbolContent: iconType,
            // Create the collision features that will be used to check whether this
            // symbol instance can be placed As a collision approximation, we can use
            // either the vertical or any of the horizontal versions of the feature
            textCollisionFeature: CollisionFeature::new_from_text(
                &sharedData_.line,
                &anchor_,
                getAnyShaping(shapedTextOrientations).clone(),
                textBoxScale_,
                textPadding,
                textPlacement,
                indexedFeature.clone(),
                overscaling,
                textRotation,
            )
            .unwrap(),
            iconCollisionFeature: CollisionFeature::new_from_icon(
                &sharedData_.line,
                &anchor_,
                shapedIcon,
                iconBoxScale,
                iconPadding,
                indexedFeature.clone(),
                iconRotation,
            )
            .unwrap(),

            sharedData: sharedData_,
            anchor: anchor_,
            writingModes: WritingModeType::None,
            layoutFeatureIndex: layoutFeatureIndex_,
            dataFeatureIndex: dataFeatureIndex_,
            textOffset: textOffset_,
            iconOffset: iconOffset_,
            key: key_,

            textBoxScale: textBoxScale_,
            variableTextOffset: variableTextOffset_,
            singleLine: shapedTextOrientations.singleLine,

            rightJustifiedGlyphQuadsSize: 0,
            centerJustifiedGlyphQuadsSize: 0,
            leftJustifiedGlyphQuadsSize: 0,
            verticalGlyphQuadsSize: 0,
            iconQuadsSize: 0,

            verticalTextCollisionFeature: None,
            placedRightTextIndex: None,
            placedCenterTextIndex: None,
            placedLeftTextIndex: None,
            placedVerticalTextIndex: None,
            placedIconIndex: None,
            placedVerticalIconIndex: None,

            verticalIconCollisionFeature: None,
            crossTileID: 0,
        };

        // 'hasText' depends on finding at least one glyph in the shaping that's also in the GlyphPositionMap
        if (!self_.sharedData.empty()) {
            self_.symbolContent |= SymbolContent::Text;
        }
        if (allowVerticalPlacement) {
            if shapedTextOrientations.vertical.isAnyLineNotEmpty() {
                let verticalPointLabelAngle = 90.0;
                self_.verticalTextCollisionFeature = CollisionFeature::new_from_text(
                    self_.line(),
                    &self_.anchor,
                    shapedTextOrientations.vertical.clone(),
                    textBoxScale_,
                    textPadding,
                    textPlacement,
                    indexedFeature.clone(),
                    overscaling,
                    textRotation + verticalPointLabelAngle,
                );
                if (verticallyShapedIcon.is_some()) {
                    self_.verticalIconCollisionFeature = CollisionFeature::new_from_icon(
                        &self_.sharedData.line,
                        &self_.anchor,
                        verticallyShapedIcon,
                        iconBoxScale,
                        iconPadding,
                        indexedFeature,
                        iconRotation + verticalPointLabelAngle,
                    );
                }
            }
        }

        self_.rightJustifiedGlyphQuadsSize = self_.sharedData.rightJustifiedGlyphQuads.len();
        self_.centerJustifiedGlyphQuadsSize = self_.sharedData.centerJustifiedGlyphQuads.len();
        self_.leftJustifiedGlyphQuadsSize = self_.sharedData.leftJustifiedGlyphQuads.len();
        self_.verticalGlyphQuadsSize = self_.sharedData.verticalGlyphQuads.len();

        self_.iconQuadsSize = if let Some(iconQuads) = &self_.sharedData.iconQuads {
            iconQuads.len()
        } else {
            0
        };

        if (self_.rightJustifiedGlyphQuadsSize != 0
            || self_.centerJustifiedGlyphQuadsSize != 0
            || self_.leftJustifiedGlyphQuadsSize != 0)
        {
            self_.writingModes |= WritingModeType::Horizontal;
        }

        if (self_.verticalGlyphQuadsSize != 0) {
            self_.writingModes |= WritingModeType::Vertical;
        }

        self_
    }
    pub fn getDefaultHorizontalPlacedTextIndex(&self) -> Option<usize> {
        if let Some(index) = (self.placedRightTextIndex) {
            return Some(index);
        }
        if let Some(index) = (self.placedCenterTextIndex) {
            return Some(index);
        }
        if let Some(index) = (self.placedLeftTextIndex) {
            return Some(index);
        }
        return None;
    }
    pub fn line(&self) -> &GeometryCoordinates {
        return &self.sharedData.line;
    }
    pub fn rightJustifiedGlyphQuads(&self) -> &SymbolQuads {
        return &self.sharedData.rightJustifiedGlyphQuads;
    }
    pub fn leftJustifiedGlyphQuads(&self) -> &SymbolQuads {
        return &self.sharedData.leftJustifiedGlyphQuads;
    }
    pub fn centerJustifiedGlyphQuads(&self) -> &SymbolQuads {
        return &self.sharedData.centerJustifiedGlyphQuads;
    }
    pub fn verticalGlyphQuads(&self) -> &SymbolQuads {
        return &self.sharedData.verticalGlyphQuads;
    }
    pub fn hasText(&self) -> bool {
        return self.symbolContent.contains(SymbolContent::Text); // TODO Is this correct?
    }
    pub fn hasIcon(&self) -> bool {
        return self.symbolContent.contains(SymbolContent::IconRGBA) || self.hasSdfIcon();
    }
    pub fn hasSdfIcon(&self) -> bool {
        return self.symbolContent.contains(SymbolContent::IconSDF);
    }
    pub fn iconQuads(&self) -> &Option<SymbolQuads> {
        return &self.sharedData.iconQuads;
    }
    pub fn verticalIconQuads(&self) -> &Option<SymbolQuads> {
        return &self.sharedData.verticalIconQuads;
    }
    pub fn releaseSharedData(&self) {
        todo!()
        // TODO not sure how to do this self.sharedData.reset();
    }

    fn invalidCrossTileID() -> u32 {
        return u32::MAX;
    }
}

type SymbolInstanceReferences = Vec<SymbolInstance>;
