//! Translated from https://github.com/maplibre/maplibre-native/blob/4add9ea/src/mbgl/text/collision_index.cpp


use std::collections::HashMap;

use bitflags::bitflags;
use cgmath::{Matrix4, Vector4};

use crate::{
    coords::EXTENT,
    euclid::{Box2D, Point2D},
    render::{camera::ModelViewProjection, view_state::ViewState},
    legacy::{
        TileSpace,
        MapMode, ScreenSpace,
        buckets::symbol_bucket::PlacedSymbol,
        collision_feature::{CollisionBox, CollisionFeature, ProjectedCollisionBox},
        geometry::feature_index::IndexedSubfeature,
        grid_index::{Circle, GridIndex},
        layout::symbol_projection::{placeFirstAndLastGlyph, project, TileDistance},
        util::geo::ScreenLineString,

    },
};

type TransformState = ViewState;

type CollisionBoundaries = Box2D<f64, ScreenSpace>; // [f64; 4]; // [x1, y1, x2, y2]

bitflags! {
    /// Represents a set of flags.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    struct IntersectStatusFlags: u8 {
        const None = 0;
        const HorizontalBorders = 1 << 0;
        const VerticalBorders = 1 << 1;

    }
}

impl Default for IntersectStatusFlags {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Default)]
struct IntersectStatus {
    flags: IntersectStatusFlags,
    // Assuming tile border divides box in two sections
    minSectionLength: i32,
}

// When a symbol crosses the edge that causes it to be included in
// collision detection, it will cause changes in the symbols around
// it. This ant specifies how many pixels to pad the edge of
// the viewport for collision detection so that the bulk of the changes
// occur offscreen. Making this ant greater increases label
// stability, but it's expensive.
// TODO remove const viewportPaddingDefault: f64 = -10.;
const viewportPaddingDefault: f64 = 100.;
// Viewport padding must be much larger for static tiles to avoid clipped labels.
const viewportPaddingForStaticTiles: f64 = 1024.;

fn findViewportPadding(transformState: &TransformState, mapMode: MapMode) -> f64 {
    if (mapMode == MapMode::Tile) {
        return viewportPaddingForStaticTiles;
    }
    return if (transformState.camera().get_pitch().0 != 0.0) {
        viewportPaddingDefault * 2.
    } else {
        viewportPaddingDefault
    };
}

type CollisionGrid = GridIndex<IndexedSubfeature>;
pub struct CollisionIndex {
    transformState: TransformState,
    viewportPadding: f64,
    collisionGrid: CollisionGrid,
    ignoredGrid: CollisionGrid,
    screenRightBoundary: f64,
    screenBottomBoundary: f64,
    gridRightBoundary: f64,
    gridBottomBoundary: f64,
    pitchFactor: f64,
}

impl CollisionIndex {
    pub fn new(transformState: &TransformState, mapMode: MapMode) -> Self {
        let viewportPadding = findViewportPadding(transformState, mapMode);
        Self {
            transformState: transformState.clone(),
            viewportPadding: viewportPadding,
            collisionGrid: CollisionGrid::new(
                transformState.width() + 2. * viewportPadding,
                transformState.height() + 2. * viewportPadding,
                25,
            ),
            ignoredGrid: CollisionGrid::new(
                transformState.width() + 2. * viewportPadding,
                transformState.height() + 2. * viewportPadding,
                25,
            ),
            screenRightBoundary: transformState.width() + viewportPadding,
            screenBottomBoundary: transformState.height() + viewportPadding,
            gridRightBoundary: transformState.width() + 2. * viewportPadding,
            gridBottomBoundary: transformState.height() + 2. * viewportPadding,
            pitchFactor: transformState.camera().get_pitch().0.cos()
                * transformState.camera_to_center_distance(),
        }
    }

    pub fn intersectsTileEdges(
        &self,
        box_: &CollisionBox,
        shift: Point2D<f64, ScreenSpace>,
        posMatrix: &ModelViewProjection,
        textPixelRatio: f64,
        tileEdges: CollisionBoundaries,
    ) -> IntersectStatus {
        let boundaries =
            self.getProjectedCollisionBoundaries(posMatrix, shift, textPixelRatio, box_);
        let mut result: IntersectStatus = IntersectStatus::default();
        let x1 = boundaries.min.x;
        let y1 = boundaries.min.y;
        let x2 = boundaries.max.x;
        let y2 = boundaries.max.y;

        let tileX1 = tileEdges.min.x;
        let tileY1 = tileEdges.min.y;
        let tileX2 = tileEdges.max.x;
        let tileY2 = tileEdges.max.y;

        // Check left border
        let mut minSectionLength = ((tileX1 - x1).min(x2 - tileX1)) as i32;
        if (minSectionLength <= 0) {
            // Check right border
            minSectionLength = ((tileX2 - x1).min(x2 - tileX2)) as i32;
        }
        if (minSectionLength > 0) {
            result.flags |= IntersectStatusFlags::VerticalBorders;
            result.minSectionLength = minSectionLength;
        }
        // Check top border
        minSectionLength = ((tileY1 - y1).min(y2 - tileY1)) as i32;
        if (minSectionLength <= 0) {
            // Check bottom border
            minSectionLength = ((tileY2 - y1).min(y2 - tileY2)) as i32;
        }
        if (minSectionLength > 0) {
            result.flags |= IntersectStatusFlags::HorizontalBorders;
            result.minSectionLength = (result.minSectionLength).min(minSectionLength);
        }
        return result;
    }

    pub fn placeFeature<F>(
        &self,
        feature: &CollisionFeature,
        shift: Point2D<f64, ScreenSpace>,
        posMatrix: &ModelViewProjection,
        labelPlaneMatrix: &Matrix4<f64>,
        textPixelRatio: f64,
        symbol: &PlacedSymbol,
        scale: f64,
        fontSize: f64,
        allowOverlap: bool,
        pitchWithMap: bool,
        collisionDebug: bool,
        avoidEdges: Option<CollisionBoundaries>,
        collisionGroupPredicate: Option<F>,
        projectedBoxes: &mut Vec<ProjectedCollisionBox>, /*out*/
    ) -> (bool, bool)
    where
        F: Fn(&IndexedSubfeature) -> bool,
    {
        assert!(projectedBoxes.is_empty());
        if (!feature.alongLine) {
            let box_ = feature.boxes.first().unwrap();
            let collisionBoundaries =
                self.getProjectedCollisionBoundaries(posMatrix, shift, textPixelRatio, box_);
            projectedBoxes.push(ProjectedCollisionBox::Box(collisionBoundaries));

            if let Some(avoidEdges) = avoidEdges {
                if !self.isInsideTile(&collisionBoundaries, &avoidEdges) {
                    return (false, false);
                }
            }

            if !self.isInsideGrid(&collisionBoundaries)
                || (!allowOverlap
                    && self.collisionGrid.hit_test(
                        projectedBoxes.last().unwrap().box_(),
                        collisionGroupPredicate,
                    ))
            {
                return (false, false);
            }

            return (true, self.isOffscreen(&collisionBoundaries));
        } else {
            return self.placeLineFeature(
                feature,
                posMatrix,
                labelPlaneMatrix,
                textPixelRatio,
                symbol,
                scale,
                fontSize,
                allowOverlap,
                pitchWithMap,
                collisionDebug,
                avoidEdges,
                collisionGroupPredicate,
                projectedBoxes,
            );
        }
    }

    pub fn insertFeature(
        &mut self,
        feature: CollisionFeature,
        projectedBoxes: &Vec<ProjectedCollisionBox>,
        ignorePlacement: bool,
        bucketInstanceId: u32,
        collisionGroupId: u16,
    ) {
        if (feature.alongLine) {
            for circle in projectedBoxes {
                if (!circle.isCircle()) {
                    continue;
                }

                if (ignorePlacement) {
                    self.ignoredGrid.insert_circle(
                        IndexedSubfeature::new(
                            feature.indexedFeature.clone(),
                            bucketInstanceId,
                            collisionGroupId,
                        ), // FIXME clone() should not be needed?
                        *circle.circle(),
                    );
                } else {
                    self.collisionGrid.insert_circle(
                        IndexedSubfeature::new(
                            feature.indexedFeature.clone(),
                            bucketInstanceId,
                            collisionGroupId,
                        ), // FIXME clone() should not be needed?
                        *circle.circle(),
                    );
                }
            }
        } else if (!projectedBoxes.is_empty()) {
            assert!(projectedBoxes.len() == 1);
            let box_ = projectedBoxes[0];
            // TODO assert!(box_.isBox());
            if (ignorePlacement) {
                self.ignoredGrid.insert(
                    IndexedSubfeature::new(
                        feature.indexedFeature,
                        bucketInstanceId,
                        collisionGroupId,
                    ),
                    *box_.box_(),
                );
            } else {
                self.collisionGrid.insert(
                    IndexedSubfeature::new(
                        feature.indexedFeature,
                        bucketInstanceId,
                        collisionGroupId,
                    ),
                    *box_.box_(),
                );
            }
        }
    }

    pub fn queryRenderedSymbols(
        &self,
        line_string: &ScreenLineString,
    ) -> HashMap<u32, Vec<IndexedSubfeature>> {
        todo!()
    }

    pub fn projectTileBoundaries(&self, posMatrix: &ModelViewProjection) -> CollisionBoundaries {
        let topLeft = self.projectPoint(posMatrix, &Point2D::zero());
        let bottomRight = self.projectPoint(posMatrix, &Point2D::new(EXTENT, EXTENT)); // FIXME: maplibre-native uses here 8192 for extent

        return CollisionBoundaries::new(
            Point2D::new(topLeft.x, topLeft.y),
            Point2D::new(bottomRight.x, bottomRight.y),
        );
    }

    pub fn getTransformState(&self) -> &TransformState {
        return &self.transformState;
    }

    pub fn getViewportPadding(&self) -> f64 {
        return self.viewportPadding;
    }
}

impl CollisionIndex {
    fn isOffscreen(&self, boundaries: &CollisionBoundaries) -> bool {
        return boundaries.max.x < self.viewportPadding
            || boundaries.min.x >= self.screenRightBoundary
            || boundaries.max.y < self.viewportPadding
            || boundaries.min.y >= self.screenBottomBoundary;
    }
    fn isInsideGrid(&self, boundaries: &CollisionBoundaries) -> bool {
        return boundaries.max.x >= 0.
            && boundaries.min.x < self.gridRightBoundary
            && boundaries.max.y >= 0.
            && boundaries.min.y < self.gridBottomBoundary;
    }

    fn isInsideTile(
        &self,
        boundaries: &CollisionBoundaries,
        tileBoundaries: &CollisionBoundaries,
    ) -> bool {
        return boundaries.min.x >= tileBoundaries.min.x
            && boundaries.min.y >= tileBoundaries.min.y
            && boundaries.max.x < tileBoundaries.max.x
            && boundaries.max.y < tileBoundaries.max.y;
    }

    fn overlapsTile(
        &self,
        boundaries: &CollisionBoundaries,
        tileBoundaries: &CollisionBoundaries,
    ) -> bool {
        return boundaries.min.x < tileBoundaries.max.x
            && boundaries.max.x > tileBoundaries.min.x
            && boundaries.min.y < tileBoundaries.max.y
            && boundaries.max.y > tileBoundaries.min.y;
    }

    fn placeLineFeature<F>(
        &self,
        feature: &CollisionFeature,
        posMatrix: &ModelViewProjection,
        labelPlaneMatrix: &Matrix4<f64>,
        textPixelRatio: f64,
        symbol: &PlacedSymbol,
        scale: f64,
        fontSize: f64,
        allowOverlap: bool,
        pitchWithMap: bool,
        collisionDebug: bool,
        avoidEdges: Option<CollisionBoundaries>,
        collisionGroupPredicate: Option<F>,
        projectedBoxes: &mut Vec<ProjectedCollisionBox>, /*out*/
    ) -> (bool, bool)
    where
        F: Fn(&IndexedSubfeature) -> bool,
    {
        assert!(feature.alongLine);
        assert!(projectedBoxes.is_empty());
        let tileUnitAnchorPoint = symbol.anchorPoint;
        let projectedAnchor = self.projectAnchor(posMatrix, &tileUnitAnchorPoint);

        let fontScale = fontSize / 24.;
        let lineOffsetX = symbol.lineOffset[0] * fontSize;
        let lineOffsetY = symbol.lineOffset[1] * fontSize;

        let labelPlaneAnchorPoint = project(tileUnitAnchorPoint, labelPlaneMatrix).0;

        let firstAndLastGlyph = placeFirstAndLastGlyph(
            fontScale,
            lineOffsetX,
            lineOffsetY,
            /*flip*/ false,
            labelPlaneAnchorPoint,
            tileUnitAnchorPoint,
            symbol,
            labelPlaneMatrix,
            /*return tile distance*/ true,
        );

        let mut collisionDetected = false;
        let mut inGrid = false;
        let mut entirelyOffscreen = true;

        let tileToViewport = projectedAnchor.0 * textPixelRatio;
        // pixelsToTileUnits is used for translating line geometry to tile units
        // ... so we care about 'scale' but not 'perspectiveRatio'
        // equivalent to pixel_to_tile_units
        let pixelsToTileUnits = 1. / (textPixelRatio * scale);

        let mut firstTileDistance = 0.;
        let mut lastTileDistance = 0.;
        if let Some(firstAndLastGlyph) = &firstAndLastGlyph {
            firstTileDistance = self.approximateTileDistance(
                firstAndLastGlyph.0.tileDistance.as_ref().unwrap(),
                firstAndLastGlyph.0.angle,
                pixelsToTileUnits,
                projectedAnchor.1,
                pitchWithMap,
            );
            lastTileDistance = self.approximateTileDistance(
                firstAndLastGlyph.1.tileDistance.as_ref().unwrap(),
                firstAndLastGlyph.1.angle,
                pixelsToTileUnits,
                projectedAnchor.1,
                pitchWithMap,
            );
        }

        let mut previousCirclePlaced = false;
        projectedBoxes.resize(feature.boxes.len(), ProjectedCollisionBox::default());
        for i in 0..feature.boxes.len() {
            let circle = feature.boxes[i];
            let boxSignedDistanceFromAnchor = circle.signedDistanceFromAnchor;
            if (firstAndLastGlyph.is_none()
                || (boxSignedDistanceFromAnchor < -firstTileDistance)
                || (boxSignedDistanceFromAnchor > lastTileDistance))
            {
                // The label either doesn't fit on its line or we
                // don't need to use this circle because the label
                // doesn't extend this far. Either way, mark the circle unused.
                previousCirclePlaced = false;
                continue;
            }

            let projectedPoint = self.projectPoint(posMatrix, &circle.anchor);
            let tileUnitRadius = (circle.x2 - circle.x1) / 2.;
            let radius = tileUnitRadius * tileToViewport;

            if (previousCirclePlaced) {
                let previousCircle = &projectedBoxes[i - 1];
                assert!(previousCircle.isCircle());
                let previousCenter = previousCircle.circle().center;
                let dx = projectedPoint.x - previousCenter.x;
                let dy = projectedPoint.y - previousCenter.y;
                // The circle edges touch when the distance between their centers is
                // 2x the radius When the distance is 1x the radius, they're doubled
                // up, and we could remove every other circle while keeping them all
                // in touch. We actually start removing circles when the distance is
                // √2x the radius:
                //  thinning the number of circles as much as possible is a major
                //  performance win, and the small gaps introduced don't make a very
                //  noticeable difference.
                let placedTooDensely = radius * radius * 2. > dx * dx + dy * dy;
                if (placedTooDensely) {
                    let atLeastOneMoreCircle = (i + 1) < feature.boxes.len();
                    if (atLeastOneMoreCircle) {
                        let nextCircle = feature.boxes[i + 1];
                        let nextBoxDistanceFromAnchor = nextCircle.signedDistanceFromAnchor;
                        if ((nextBoxDistanceFromAnchor > -firstTileDistance)
                            && (nextBoxDistanceFromAnchor < lastTileDistance))
                        {
                            // Hide significantly overlapping circles, unless this
                            // is the last one we can use, in which case we want to
                            // keep it in place even if it's tightly packed with the
                            // one before it.
                            previousCirclePlaced = false;
                            continue;
                        }
                    }
                }
            }

            previousCirclePlaced = true;

            let collisionBoundaries = CollisionBoundaries::new(
                Point2D::new(projectedPoint.x - radius, projectedPoint.y - radius),
                Point2D::new(projectedPoint.x + radius, projectedPoint.y + radius),
            );

            projectedBoxes[i] = ProjectedCollisionBox::Circle(Circle::new(
                Point2D::new(projectedPoint.x, projectedPoint.y),
                radius,
            ));

            entirelyOffscreen &= self.isOffscreen(&collisionBoundaries);
            inGrid |= self.isInsideGrid(&collisionBoundaries);

            if let Some(avoidEdges) = avoidEdges {
                if (!self.isInsideTile(&collisionBoundaries, &avoidEdges)) {
                    if (!collisionDebug) {
                        return (false, false);
                    } else {
                        // Don't early exit if we're showing the debug circles because
                        // we still want to calculate which circles are in use
                        collisionDetected = true;
                    }
                }
            }

            if (!allowOverlap
                && self
                    .collisionGrid
                    .hit_test_circle(projectedBoxes[i].circle(), collisionGroupPredicate.as_ref()))
            {
                if (!collisionDebug) {
                    return (false, false);
                } else {
                    // Don't early exit if we're showing the debug circles because
                    // we still want to calculate which circles are in use
                    collisionDetected = true;
                }
            }
        }

        return (
            !collisionDetected && firstAndLastGlyph.is_some() && inGrid,
            entirelyOffscreen,
        );
    }

    fn approximateTileDistance(
        &self,
        tileDistance: &TileDistance,
        lastSegmentAngle: f64,
        pixelsToTileUnits: f64,
        cameraToAnchorDistance: f64,
        pitchWithMap: bool,
    ) -> f64 {
        // This is a quick and dirty solution for chosing which collision circles to
        // use (since collision circles are laid out in tile units). Ideally, I
        // think we should generate collision circles on the fly in viewport
        // coordinates at the time we do collision detection.

        // incidenceStretch is the ratio of how much y space a label takes up on a
        // tile while drawn perpendicular to the viewport vs
        //  how much space it would take up if it were drawn flat on the tile
        // Using law of sines, camera_to_anchor/sin(ground_angle) =
        // camera_to_center/sin(incidence_angle) Incidence angle 90 -> head on,
        // sin(incidence_angle) = 1, no stretch Incidence angle 1 -> very oblique,
        // sin(incidence_angle) =~ 0, lots of stretch ground_angle = u_pitch + PI/2
        // -> sin(ground_angle) = cos(u_pitch) incidenceStretch = 1 /
        // sin(incidenceAngle)

        let incidenceStretch = if pitchWithMap {
            1.
        } else {
            cameraToAnchorDistance / self.pitchFactor
        };
        let lastSegmentTile = tileDistance.lastSegmentViewportDistance * pixelsToTileUnits;
        return tileDistance.prevTileDistance
            + lastSegmentTile
            + (incidenceStretch - 1.) * lastSegmentTile * lastSegmentAngle.sin().abs();
    }

    fn projectAnchor(
        &self,
        posMatrix: &ModelViewProjection,
        point: &Point2D<f64, TileSpace>,
    ) -> (f64, f64) {
        let p = Vector4::new(point.x, point.y, 0., 1.);
        let p = posMatrix.project(p); // TODO verify multiplication
        return (
            0.5 + 0.5 * (self.transformState.camera_to_center_distance() / p[3]),
            p[3],
        );
    }
    fn projectAndGetPerspectiveRatio(
        &self,
        posMatrix: &ModelViewProjection,
        point: &Point2D<f64, TileSpace>,
    ) -> (Point2D<f64, ScreenSpace>, f64) {
        let p = Vector4::new(point.x, point.y, 0., 1.);
        let p = posMatrix.project(p); // TODO verify multiplication
        let width = self.transformState.width();
        let height = self.transformState.height();
        let ccd = self.transformState.camera_to_center_distance();
        return (
            Point2D::new(
                ((p[0] / p[3] + 1.) / 2.) * width + self.viewportPadding,
                ((-p[1] / p[3] + 1.) / 2.) * height + self.viewportPadding,
            ),
            // See perspective ratio comment in symbol_sdf.vertex
            // We're doing collision detection in viewport space so we need
            // to scale down boxes in the distance
            0.5 + 0.5 * ccd / p[3],
        );
    }
    fn projectPoint(
        &self,
        posMatrix: &ModelViewProjection,
        point: &Point2D<f64, TileSpace>,
    ) -> Point2D<f64, ScreenSpace> {
        let p = Vector4::new(point.x, point.y, 0., 1.);
        let p = posMatrix.project(p); // TODO verify multiplication
        let width = self.transformState.width();
        let height = self.transformState.height();
        return Point2D::new(
            (((p[0] / p[3] + 1.) / 2.) * width + self.viewportPadding),
            (((-p[1] / p[3] + 1.) / 2.) * height + self.viewportPadding),
        );
    }

    fn getProjectedCollisionBoundaries(
        &self,
        posMatrix: &ModelViewProjection,
        shift: Point2D<f64, ScreenSpace>,
        textPixelRatio: f64,
        box_: &CollisionBox,
    ) -> CollisionBoundaries {
        let (projectedPoint, tileToViewport) =
            self.projectAndGetPerspectiveRatio(posMatrix, &box_.anchor);
        let tileToViewport = textPixelRatio * tileToViewport;
        let tileToViewport = 1.; // TODO
        return CollisionBoundaries::new(
            Point2D::new(
                (box_.x1 + shift.x) * tileToViewport + projectedPoint.x,
                (box_.y1 + shift.y) * tileToViewport + projectedPoint.y,
            ),
            Point2D::new(
                (box_.x2 + shift.x) * tileToViewport + projectedPoint.x,
                (box_.y2 + shift.y) * tileToViewport + projectedPoint.y,
            ),
        );
    }
}
