use std::collections::HashMap;
use crate::sdf::grid_index::{GridIndex};
use cgmath::{Matrix4};
use geo_types::LineString;
use lyon::geom::euclid::{Point2D, UnknownUnit};
use crate::render::view_state::ViewState;
use crate::sdf::collision_feature::{CollisionBox, CollisionFeature, ProjectedCollisionBox};
use crate::sdf::feature_index::{IndexedSubfeature, RefIndexedSubfeature};
use crate::sdf::Point;



type TransformState = ViewState;
enum  MapMode {
    ///< continually updating map
    Continuous,
    ///< a once-off still image of an arbitrary viewport
    Static,
    ///< a once-off still image of a single tile
    Tile,
}

struct PlacedSymbol; // TODO


type ScreenLineString = LineString;
struct TileDistance {
    prevTileDistance: f64,
    lastSegmentViewportDistance: f64,
}

type CollisionBoundaries = [f64; 4]; // [x1, y1, x2, y2]

enum IntersectStatusFlags {
    None = 0,
    HorizontalBorders = 1 << 0,
    VerticalBorders = 1 << 1,
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

type CollisionGrid = GridIndex<IndexedSubfeature>;
struct CollisionIndex {
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
    fn new(state: &TransformState, mode: MapMode) -> Self {
        unimplemented!()
    }

    pub fn intersectsTileEdges(
        &self,
        collision_box: CollisionBox,
        shift: Point2D<f64, UnknownUnit>,
        posMatrix: Matrix4<f64>,
        textPixelRatio: f64,
        tileEdges: CollisionBoundaries,
    ) -> IntersectStatus {
        unimplemented!()
    }
    pub fn placeFeature(
        &self,
        feature: &CollisionFeature,
        shift: Point<f64>,
        posMatrix: Matrix4<f64>,
        labelPlaneMatrix: Matrix4<f64>,
        textPixelRatio: f64,
        symbol: &PlacedSymbol,
        scale: f64,
        fontSize: f64,
        allowOverlap: bool,
        pitchWithMap: bool,
        collisionDebug: bool,
        avoidEdges: Option<&CollisionBoundaries>,
        collisionGroupPredicate: Option<fn(&RefIndexedSubfeature) -> bool>,
        out: &mut Vec<ProjectedCollisionBox>, /*out*/
    ) -> (bool, bool) {
        unimplemented!()
    }

    pub fn insertFeature(
        &self,
        feature: CollisionFeature,
        vec: &ProjectedCollisionBox,
        ignorePlacement: bool,
        ketInstanceId: u32,
        collisionGroupId: u16,
    ) {
        unimplemented!()
    }

    pub fn queryRenderedSymbols(
        &self,
        line_string: &ScreenLineString,
    ) -> HashMap<u32, Vec<IndexedSubfeature>> {
        unimplemented!()
    }

    pub fn projectTileBoundaries(&self, posMatrix: &Matrix4<f64>) -> CollisionBoundaries {
        unimplemented!()
    }

    pub fn getTransformState(&self) -> &TransformState {
        return &self.transformState;
    }

    pub fn getViewportPadding(&self) -> f64 {
        return self.viewportPadding;
    }
}

impl CollisionIndex {
    fn isOffscreen(boundaries: &CollisionBoundaries) -> bool {
        unimplemented!()
    }
    fn isInsideGrid(boundaries: &CollisionBoundaries) -> bool {
        unimplemented!()
    }

    fn isInsideTile(
        boundaries: &CollisionBoundaries,
        tileBoundaries: &CollisionBoundaries,
    ) -> bool {
        unimplemented!()
    }

    fn overlapsTile(
        boundaries: &CollisionBoundaries,
        tileBoundaries: &CollisionBoundaries,
    ) -> bool {
        unimplemented!()
    }

    fn placeLineFeature(
        &self,
        feature: &CollisionFeature,
        posMatrix: &Matrix4<f64>,
        labelPlaneMatrix: &Matrix4<f64>,
        textPixelRatio: f64,
        symbol: &PlacedSymbol,
        scale: f64,
        fontSize: f64,
        allowOverlap: bool,
        pitchWithMap: bool,
        collisionDebug: bool,
        avoidEdges: &Option<CollisionBoundaries>,
        collisionGroupPredicate: Option<fn(feature: &RefIndexedSubfeature) -> bool>,
        out: &Vec<ProjectedCollisionBox>, /*out*/
    ) -> (f64, f64) {
        unimplemented!()
    }

    fn approximateTileDistance(
        tileDistance: &TileDistance,
        lastSegmentAngle: f64,
        pixelsToTileUnits: f64,
        cameraToAnchorDistance: f64,
        pitchWithMap: bool,
    ) -> f64 {
        unimplemented!()
    }

    fn projectAnchor(posMatrix: &Matrix4<f64>, point: &Point<f64>) -> (f64, f64) {
        unimplemented!()
    }
    fn projectAndGetPerspectiveRatio(posMatrix: &Matrix4<f64>, point: &Point<f64>) -> (f64, f64) {
        unimplemented!()
    }
    fn projectPoint(posMatrix: &Matrix4<f64>, point: &Point<f64>) -> Point<f64> {
        unimplemented!()
    }
    fn getProjectedCollisionBoundaries(
        posMatrix: &Matrix4<f64>,
        shift: Point<f64>,
        textPixelRatio: f64,
        box_: &CollisionBox,
    ) -> CollisionBoundaries {
        unimplemented!()
    }
}
