// This file was fully translated

use crate::euclid::{Box2D, Point2D, Vector2D};
use crate::sdf::geometry::feature_index::IndexedSubfeature;
use crate::sdf::geometry::{convert_point_f64, convert_point_i16, Anchor};
use crate::sdf::geometry_tile_data::{GeometryCoordinate, GeometryCoordinates};
use crate::sdf::glyph::Shaping;
use crate::sdf::grid_index::Circle;
use crate::sdf::shaping::{Padding, PositionedIcon};
use crate::sdf::style_types::SymbolPlacementType;
use crate::sdf::util::math::{deg2radf, rotate, MinMax};
use crate::sdf::{ScreenSpace, TileSpace};

pub struct CollisionFeature {
    pub boxes: Vec<CollisionBox>,
    pub indexedFeature: IndexedSubfeature,
    pub alongLine: bool,
}

impl CollisionFeature {
    pub fn new(
        line: &GeometryCoordinates,
        anchor: &Anchor,
        top: f64,
        bottom: f64,
        left: f64,
        right: f64,
        collisionPadding: Option<Padding>,
        boxScale: f64,
        padding: f64,
        placement: SymbolPlacementType,
        indexedFeature_: IndexedSubfeature,
        overscaling: f64,
        rotate_: f64,
    ) -> Option<Self> {
        let mut self_ = Self {
            boxes: vec![],
            indexedFeature: indexedFeature_,
            alongLine: placement != SymbolPlacementType::Point,
        };

        if (top == 0. && bottom == 0. && left == 0. && right == 0.) {
            return None;
        }

        let mut y1 = top * boxScale - padding;
        let mut y2 = bottom * boxScale + padding;
        let mut x1 = left * boxScale - padding;
        let mut x2 = right * boxScale + padding;

        if let Some(collisionPadding) = collisionPadding {
            x1 -= collisionPadding.left * boxScale;
            y1 -= collisionPadding.top * boxScale;
            x2 += collisionPadding.right * boxScale;
            y2 += collisionPadding.bottom * boxScale;
        }

        if (self_.alongLine) {
            let mut height = y2 - y1;
            let length = x2 - x1;

            if (height <= 0.0) {
                return None;
            }

            height = 10.0 * boxScale.max(height);

            let anchorPoint = convert_point_i16(&anchor.point);
            self_.bboxifyLabel(
                line,
                &anchorPoint,
                anchor.segment.unwrap_or(0),
                length,
                height,
                overscaling,
            );
        } else {
            if (rotate_ != 0.) {
                // Account for *-rotate in point collision boxes
                // Doesn't account for icon-text-fit
                let rotateRadians = deg2radf(rotate_);

                let tl = rotate(&Vector2D::<_, TileSpace>::new(x1, y1), rotateRadians);
                let tr = rotate(&Vector2D::<_, TileSpace>::new(x2, y1), rotateRadians);
                let bl = rotate(&Vector2D::<_, TileSpace>::new(x1, y2), rotateRadians);
                let br = rotate(&Vector2D::<_, TileSpace>::new(x2, y2), rotateRadians);

                // Collision features require an "on-axis" geometry,
                // so take the envelope of the rotated geometry
                // (may be quite large for wide labels rotated 45 degrees)
                let xMin = [tl.x, tr.x, bl.x, br.x].min_value();
                let xMax = [tl.x, tr.x, bl.x, br.x].max_value();
                let yMin = [tl.y, tr.y, bl.y, br.y].min_value();
                let yMax = [tl.y, tr.y, bl.y, br.y].max_value();

                self_.boxes.push(CollisionBox {
                    anchor: anchor.point,
                    x1: xMin,
                    y1: yMin,
                    x2: xMax,
                    y2: yMax,
                    signedDistanceFromAnchor: 0.0,
                });
            } else {
                self_.boxes.push(CollisionBox {
                    anchor: anchor.point,
                    x1,
                    y1,
                    x2,
                    y2,
                    signedDistanceFromAnchor: 0.0,
                });
            }
        }
        Some(self_)
    }

    // for text
    pub fn new_from_text(
        line: &GeometryCoordinates,
        anchor: &Anchor,
        shapedText: Shaping,
        boxScale: f64,
        padding: f64,
        placement: SymbolPlacementType,
        indexedFeature_: IndexedSubfeature,
        overscaling: f64,
        rotate: f64,
    ) -> Option<Self> {
        Self::new(
            line,
            anchor,
            shapedText.top,
            shapedText.bottom,
            shapedText.left,
            shapedText.right,
            None,
            boxScale,
            padding,
            placement,
            indexedFeature_,
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
    pub fn new_from_icon(
        line: &GeometryCoordinates,
        anchor: &Anchor,
        shapedIcon: &Option<PositionedIcon>,
        boxScale: f64,
        padding: f64,
        indexedFeature_: IndexedSubfeature,
        rotate: f64,
    ) -> Option<Self> {
        Self::new(
            line,
            anchor,
            if let Some(shapedIcon) = &shapedIcon {
                shapedIcon.top
            } else {
                0.
            },
            if let Some(shapedIcon) = &shapedIcon {
                shapedIcon.bottom
            } else {
                0.
            },
            if let Some(shapedIcon) = &shapedIcon {
                shapedIcon.left
            } else {
                0.
            },
            if let Some(shapedIcon) = &shapedIcon {
                shapedIcon.right
            } else {
                0.
            },
            if let Some(shapedIcon) = &shapedIcon {
                Some(shapedIcon.collisionPadding)
            } else {
                None
            },
            boxScale,
            padding,
            SymbolPlacementType::Point,
            indexedFeature_,
            1.,
            rotate,
        )
    }

    fn bboxifyLabel(
        &mut self,
        line: &GeometryCoordinates,
        anchorPoint: &GeometryCoordinate,
        segment: usize,
        labelLength: f64,
        boxSize: f64,
        overscaling: f64,
    ) {
        let step = boxSize / 2.;
        let nBoxes = ((labelLength / step).floor() as i32).max(1);

        // We calculate line collision circles out to 300% of what would normally be
        // our max size, to allow collision detection to work on labels that expand
        // as they move into the distance Vertically oriented labels in the distant
        // field can extend past this padding This is a noticeable problem in
        // overscaled tiles where the pitch 0-based symbol spacing will put labels
        // very close together in a pitched map. To reduce the cost of adding extra
        // collision circles, we slowly increase them for overscaled tiles.
        let overscalingPaddingFactor = 1. + 0.4 * (overscaling as f64).log2();
        let nPitchPaddingBoxes = ((nBoxes as f64 * overscalingPaddingFactor / 2.).floor()) as i32;

        // offset the center of the first box by half a box so that the edge of the
        // box is at the edge of the label.
        let firstBoxOffset = -boxSize / 2.;

        let mut p = anchorPoint;
        let mut index = segment + 1;
        let mut anchorDistance = firstBoxOffset;
        let labelStartDistance = -labelLength / 2.;
        let paddingStartDistance = labelStartDistance - labelLength / 8.;

        // move backwards along the line to the first segment the label appears on
        loop {
            if (index == 0) {
                if (anchorDistance > labelStartDistance) {
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
            anchorDistance -= convert_point_f64(&line[index]).distance_to(convert_point_f64(p));
            p = &line[index];

            if !(anchorDistance > paddingStartDistance) {
                break;
            }
        }

        let mut segmentLength =
            convert_point_f64(&line[index]).distance_to(convert_point_f64(&line[index + 1]));

        for i in -nPitchPaddingBoxes..nBoxes + nPitchPaddingBoxes {
            // the distance the box will be from the anchor
            let boxOffset = i as f64 * step;
            let mut boxDistanceToAnchor = labelStartDistance + boxOffset as f64;

            // make the distance between pitch padding boxes bigger
            if (boxOffset < 0.) {
                boxDistanceToAnchor += boxOffset;
            }
            if (boxOffset > labelLength) {
                boxDistanceToAnchor += boxOffset - labelLength;
            }

            if (boxDistanceToAnchor < anchorDistance) {
                // The line doesn't extend far enough back for this box, skip it
                // (This could allow for line collisions on distant tiles)
                continue;
            }

            // the box is not on the current segment. Move to the next segment.
            while (anchorDistance + segmentLength < boxDistanceToAnchor) {
                anchorDistance += segmentLength;
                index += 1;

                // There isn't enough room before the end of the line.
                if (index + 1 >= line.len()) {
                    return;
                }

                segmentLength = convert_point_f64(&line[index])
                    .distance_to(convert_point_f64(&line[index + 1]));
            }

            // the distance the box will be from the beginning of the segment
            let segmentBoxDistance = boxDistanceToAnchor - anchorDistance;

            let p0 = line[index];
            let p1 = line[index + 1];

            let boxAnchor = Point2D::new(
                p0.x as f64 + segmentBoxDistance / segmentLength * (p1.x - p0.x) as f64,
                p0.y as f64 + segmentBoxDistance / segmentLength * (p1.y - p0.y) as f64,
            );

            // If the box is within boxSize of the anchor, force the box to be used
            // (so even 0-width labels use at least one box)
            // Otherwise, the .8 multiplication gives us a little bit of conservative
            // padding in choosing which boxes to use (see CollisionIndex#placedCollisionCircles)
            let paddedAnchorDistance = if (boxDistanceToAnchor - firstBoxOffset).abs() < step {
                0.0
            } else {
                (boxDistanceToAnchor - firstBoxOffset) * 0.8
            };

            self.boxes.push(CollisionBox {
                anchor: boxAnchor,
                x1: -boxSize / 2.,
                y1: -boxSize / 2.,
                x2: boxSize / 2.,
                y2: boxSize / 2.,
                signedDistanceFromAnchor: paddedAnchorDistance,
            });
        }
    }
}

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

    pub signedDistanceFromAnchor: f64,
}

#[derive(Clone, Copy, Debug)]
pub enum ProjectedCollisionBox {
    Circle(Circle<f64>),
    Box(Box2D<f64, ScreenSpace>),
}

impl Default for ProjectedCollisionBox {
    fn default() -> Self {
        return Self::Box(Box2D::zero());
    }
}

impl ProjectedCollisionBox {
    pub fn box_(&self) -> &Box2D<f64, ScreenSpace> {
        match self {
            ProjectedCollisionBox::Circle(_) => panic!("not a box"),
            ProjectedCollisionBox::Box(box_) => box_,
        }
    }

    pub fn circle(&self) -> &Circle<f64> {
        match self {
            ProjectedCollisionBox::Circle(circle) => circle,
            ProjectedCollisionBox::Box(_) => panic!("not a circle"),
        }
    }

    pub fn isBox(&self) -> bool {
        match self {
            ProjectedCollisionBox::Circle(_) => false,
            ProjectedCollisionBox::Box(_) => true,
        }
    }

    pub fn isCircle(&self) -> bool {
        match self {
            ProjectedCollisionBox::Circle(_) => true,
            ProjectedCollisionBox::Box(_) => false,
        }
    }
}
