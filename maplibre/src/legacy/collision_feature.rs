//! Translated from https://github.com/maplibre/maplibre-native/blob/4add9ea/src/mbgl/text/collision_feature.cpp

use crate::{
    euclid::{Box2D, Point2D, Vector2D},
    legacy::{
        geometry::{anchor::Anchor, feature_index::IndexedSubfeature},
        geometry_tile_data::{GeometryCoordinate, GeometryCoordinates},
        glyph::Shaping,
        grid_index::Circle,
        shaping::{Padding, PositionedIcon},
        style_types::SymbolPlacementType,
        util::math::{convert_point_f64, convert_point_i16, deg2radf, rotate, MinMax},
        ScreenSpace, TileSpace,
    },
};

/// maplibre/maplibre-native#4add9ea original name: CollisionFeature
#[derive(Clone)]
pub struct CollisionFeature {
    pub boxes: Vec<CollisionBox>,
    pub indexed_feature: IndexedSubfeature,
    pub along_line: bool,
}

impl CollisionFeature {
    /// maplibre/maplibre-native#4add9ea original name: new
    pub fn new(
        line: &GeometryCoordinates,
        anchor: &Anchor,
        top: f64,
        bottom: f64,
        left: f64,
        right: f64,
        collision_padding: Option<Padding>,
        box_scale: f64,
        padding: f64,
        placement: SymbolPlacementType,
        indexed_feature: IndexedSubfeature,
        overscaling: f64,
        rotate_: f64,
    ) -> Self {
        let mut self_ = Self {
            boxes: vec![],
            indexed_feature,
            along_line: placement != SymbolPlacementType::Point,
        };

        if top == 0. && bottom == 0. && left == 0. && right == 0. {
            return self_;
        }

        let mut y1 = top * box_scale - padding;
        let mut y2 = bottom * box_scale + padding;
        let mut x1 = left * box_scale - padding;
        let mut x2 = right * box_scale + padding;

        if let Some(collision_padding) = collision_padding {
            x1 -= collision_padding.left * box_scale;
            y1 -= collision_padding.top * box_scale;
            x2 += collision_padding.right * box_scale;
            y2 += collision_padding.bottom * box_scale;
        }

        if self_.along_line {
            let mut height = y2 - y1;
            let length = x2 - x1;

            if height <= 0.0 {
                return self_;
            }

            height = 10.0 * box_scale.max(height);

            let anchor_point = convert_point_i16(&anchor.point);
            self_.bboxify_label(
                line,
                &anchor_point,
                anchor.segment.unwrap_or(0),
                length,
                height,
                overscaling,
            );
        } else if rotate_ != 0. {
            // Account for *-rotate in point collision boxes
            // Doesn't account for icon-text-fit
            let rotate_radians = deg2radf(rotate_);

            let tl = rotate(&Vector2D::<_, TileSpace>::new(x1, y1), rotate_radians);
            let tr = rotate(&Vector2D::<_, TileSpace>::new(x2, y1), rotate_radians);
            let bl = rotate(&Vector2D::<_, TileSpace>::new(x1, y2), rotate_radians);
            let br = rotate(&Vector2D::<_, TileSpace>::new(x2, y2), rotate_radians);

            // Collision features require an "on-axis" geometry,
            // so take the envelope of the rotated geometry
            // (may be quite large for wide labels rotated 45 degrees)
            let x_min = [tl.x, tr.x, bl.x, br.x].min_value();
            let x_max = [tl.x, tr.x, bl.x, br.x].max_value();
            let y_min = [tl.y, tr.y, bl.y, br.y].min_value();
            let y_max = [tl.y, tr.y, bl.y, br.y].max_value();

            self_.boxes.push(CollisionBox {
                anchor: anchor.point,
                x1: x_min,
                y1: y_min,
                x2: x_max,
                y2: y_max,
                signed_distance_from_anchor: 0.0,
            });
        } else {
            self_.boxes.push(CollisionBox {
                anchor: anchor.point,
                x1,
                y1,
                x2,
                y2,
                signed_distance_from_anchor: 0.0,
            });
        }
        self_
    }

    // for text
    /// maplibre/maplibre-native#4add9ea original name: new_from_text
    pub fn new_from_text(
        line: &GeometryCoordinates,
        anchor: &Anchor,
        shaped_text: Shaping,
        box_scale: f64,
        padding: f64,
        placement: SymbolPlacementType,
        indexed_feature: IndexedSubfeature,
        overscaling: f64,
        rotate: f64,
    ) -> Self {
        Self::new(
            line,
            anchor,
            shaped_text.top,
            shaped_text.bottom,
            shaped_text.left,
            shaped_text.right,
            None,
            box_scale,
            padding,
            placement,
            indexed_feature,
            overscaling,
            rotate,
        )
    }

    // for icons
    // Icons collision features are always SymbolPlacementType::Point, which
    // means the collision feature will be viewport-rotation-aligned even if the
    // icon is map-rotation-aligned (e.g. `icon-rotation-alignment: map` _or_
    // `symbol-placement: line`). We're relying on most icons being "close
    // enough" to square that having incorrect rotation alignment doesn't throw
    // off collision detection too much. See:
    // https://github.com/mapbox/mapbox-gl-js/issues/4861
    /// maplibre/maplibre-native#4add9ea original name: new_from_icon
    pub fn new_from_icon(
        line: &GeometryCoordinates,
        anchor: &Anchor,
        shaped_icon: &Option<PositionedIcon>,
        box_scale: f64,
        padding: f64,
        indexed_feature: IndexedSubfeature,
        rotate: f64,
    ) -> Self {
        Self::new(
            line,
            anchor,
            if let Some(shaped_icon) = &shaped_icon {
                shaped_icon.top
            } else {
                0.
            },
            if let Some(shaped_icon) = &shaped_icon {
                shaped_icon.bottom
            } else {
                0.
            },
            if let Some(shaped_icon) = &shaped_icon {
                shaped_icon.left
            } else {
                0.
            },
            if let Some(shaped_icon) = &shaped_icon {
                shaped_icon.right
            } else {
                0.
            },
            shaped_icon
                .as_ref()
                .map(|shaped_icon| shaped_icon.collision_padding),
            box_scale,
            padding,
            SymbolPlacementType::Point,
            indexed_feature,
            1.,
            rotate,
        )
    }

    /// maplibre/maplibre-native#4add9ea original name: bboxifyLabel
    fn bboxify_label(
        &mut self,
        line: &GeometryCoordinates,
        anchor_point: &GeometryCoordinate,
        segment: usize,
        label_length: f64,
        box_size: f64,
        overscaling: f64,
    ) {
        let step = box_size / 2.;
        let n_boxes = ((label_length / step).floor() as i32).max(1);

        // We calculate line collision circles out to 300% of what would normally be
        // our max size, to allow collision detection to work on labels that expand
        // as they move into the distance Vertically oriented labels in the distant
        // field can extend past this padding This is a noticeable problem in
        // overscaled tiles where the pitch 0-based symbol spacing will put labels
        // very close together in a pitched map. To reduce the cost of adding extra
        // collision circles, we slowly increase them for overscaled tiles.
        let overscaling_padding_factor = 1. + 0.4 * overscaling.log2();
        let n_pitch_padding_boxes =
            ((n_boxes as f64 * overscaling_padding_factor / 2.).floor()) as i32;

        // offset the center of the first box by half a box so that the edge of the
        // box is at the edge of the label.
        let first_box_offset = -box_size / 2.;

        let mut p = anchor_point;
        let mut index = segment + 1;
        let mut anchor_distance = first_box_offset;
        let label_start_distance = -label_length / 2.;
        let padding_start_distance = label_start_distance - label_length / 8.;

        // move backwards along the line to the first segment the label appears on
        loop {
            if index == 0 {
                if anchor_distance > label_start_distance {
                    // there isn't enough room for the label after the beginning of
                    // the line checkMaxAngle should have already caught this
                    return;
                } else {
                    // The line doesn't extend far enough back for all of our padding,
                    // but we got far enough to show the label under most conditions.
                    index = 0;
                    break;
                }
            }

            index -= 1;
            anchor_distance -= convert_point_f64(&line[index]).distance_to(convert_point_f64(p));
            p = &line[index];

            if !(anchor_distance > padding_start_distance) {
                break;
            }
        }

        let mut segment_length =
            convert_point_f64(&line[index]).distance_to(convert_point_f64(&line[index + 1]));

        for i in -n_pitch_padding_boxes..n_boxes + n_pitch_padding_boxes {
            // the distance the box will be from the anchor
            let box_offset = i as f64 * step;
            let mut box_distance_to_anchor = label_start_distance + box_offset;

            // make the distance between pitch padding boxes bigger
            if box_offset < 0. {
                box_distance_to_anchor += box_offset;
            }
            if box_offset > label_length {
                box_distance_to_anchor += box_offset - label_length;
            }

            if box_distance_to_anchor < anchor_distance {
                // The line doesn't extend far enough back for this box, skip it
                // (This could allow for line collisions on distant tiles)
                continue;
            }

            // the box is not on the current segment. Move to the next segment.
            while anchor_distance + segment_length < box_distance_to_anchor {
                anchor_distance += segment_length;
                index += 1;

                // There isn't enough room before the end of the line.
                if index + 1 >= line.len() {
                    return;
                }

                segment_length = convert_point_f64(&line[index])
                    .distance_to(convert_point_f64(&line[index + 1]));
            }

            // the distance the box will be from the beginning of the segment
            let segment_box_distance = box_distance_to_anchor - anchor_distance;

            let p0 = line[index];
            let p1 = line[index + 1];

            let box_anchor = Point2D::new(
                p0.x as f64 + segment_box_distance / segment_length * (p1.x - p0.x) as f64,
                p0.y as f64 + segment_box_distance / segment_length * (p1.y - p0.y) as f64,
            );

            // If the box is within boxSize of the anchor, force the box to be used
            // (so even 0-width labels use at least one box)
            // Otherwise, the .8 multiplication gives us a little bit of conservative
            // padding in choosing which boxes to use (see CollisionIndex#placedCollisionCircles)
            let padded_anchor_distance = if (box_distance_to_anchor - first_box_offset).abs() < step
            {
                0.0
            } else {
                (box_distance_to_anchor - first_box_offset) * 0.8
            };

            self.boxes.push(CollisionBox {
                anchor: box_anchor,
                x1: -box_size / 2.,
                y1: -box_size / 2.,
                x2: box_size / 2.,
                y2: box_size / 2.,
                signed_distance_from_anchor: padded_anchor_distance,
            });
        }
    }
}

/// maplibre/maplibre-native#4add9ea original name: CollisionBox
#[derive(Default, Clone, Copy, Debug)]
pub struct CollisionBox {
    // the box is centered around the anchor point
    pub anchor: Point2D<f64, TileSpace>,

    // the offset of the box from the label's anchor point.
    // TODO: might be needed for #13526
    // Point<f64> offset;

    // distances to the edges from the anchor
    pub x1: f64,
    pub y1: f64,
    pub x2: f64,
    pub y2: f64,

    pub signed_distance_from_anchor: f64,
}

/// maplibre/maplibre-native#4add9ea original name: ProjectedCollisionBox
#[derive(Clone, Copy, Debug)]
pub enum ProjectedCollisionBox {
    Circle(Circle<f64>),
    Box(Box2D<f64, ScreenSpace>),
}

impl Default for ProjectedCollisionBox {
    /// maplibre/maplibre-native#4add9ea original name: default
    fn default() -> Self {
        Self::Box(Box2D::zero())
    }
}

impl ProjectedCollisionBox {
    /// maplibre/maplibre-native#4add9ea original name: box_
    pub fn box_(&self) -> &Box2D<f64, ScreenSpace> {
        match self {
            ProjectedCollisionBox::Circle(_) => panic!("not a box"),
            ProjectedCollisionBox::Box(box_) => box_,
        }
    }

    /// maplibre/maplibre-native#4add9ea original name: circle
    pub fn circle(&self) -> &Circle<f64> {
        match self {
            ProjectedCollisionBox::Circle(circle) => circle,
            ProjectedCollisionBox::Box(_) => panic!("not a circle"),
        }
    }

    /// maplibre/maplibre-native#4add9ea original name: isBox
    pub fn is_box(&self) -> bool {
        match self {
            ProjectedCollisionBox::Circle(_) => false,
            ProjectedCollisionBox::Box(_) => true,
        }
    }

    /// maplibre/maplibre-native#4add9ea original name: isCircle
    pub fn is_circle(&self) -> bool {
        match self {
            ProjectedCollisionBox::Circle(_) => true,
            ProjectedCollisionBox::Box(_) => false,
        }
    }
}
