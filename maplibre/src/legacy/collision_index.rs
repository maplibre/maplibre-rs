//! Translated from https://github.com/maplibre/maplibre-native/blob/4add9ea/src/mbgl/text/collision_index.cpp

use std::collections::HashMap;

use bitflags::bitflags;
use cgmath::{Matrix4, Vector4};

use crate::{
    coords::EXTENT,
    euclid::{Box2D, Point2D},
    legacy::{
        buckets::symbol_bucket::PlacedSymbol,
        collision_feature::{CollisionBox, CollisionFeature, ProjectedCollisionBox},
        geometry::feature_index::IndexedSubfeature,
        grid_index::{Circle, GridIndex},
        layout::symbol_projection::{place_first_and_last_glyph, project, TileDistance},
        util::geo::ScreenLineString,
        MapMode, ScreenSpace, TileSpace,
    },
    render::{camera::ModelViewProjection, view_state::ViewState},
};

/// maplibre/maplibre-native#4add9ea original name: TransformState
type TransformState = ViewState;

/// maplibre/maplibre-native#4add9ea original name: CollisionBoundaries
type CollisionBoundaries = Box2D<f64, ScreenSpace>; // [f64; 4]; // [x1, y1, x2, y2]

bitflags! {
    /// Represents a set of flags.
    /// maplibre/maplibre-native#4add9ea original name: IntersectStatusFlags:
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    struct IntersectStatusFlags: u8 {
        const None = 0;
        const HorizontalBorders = 1 << 0;
        const VerticalBorders = 1 << 1;

    }
}

impl Default for IntersectStatusFlags {
    /// maplibre/maplibre-native#4add9ea original name: default
    fn default() -> Self {
        Self::None
    }
}

/// maplibre/maplibre-native#4add9ea original name: IntersectStatus
#[derive(Default)]
struct IntersectStatus {
    flags: IntersectStatusFlags,
    // Assuming tile border divides box in two sections
    min_section_length: i32,
}

// When a symbol crosses the edge that causes it to be included in
// collision detection, it will cause changes in the symbols around
// it. This ant specifies how many pixels to pad the edge of
// the viewport for collision detection so that the bulk of the changes
// occur offscreen. Making this ant greater increases label
// stability, but it's expensive.
// TODO remove const viewportPaddingDefault: f64 = -10.;
const VIEWPORT_PADDING_DEFAULT: f64 = 100.;
// Viewport padding must be much larger for static tiles to avoid clipped labels.
const VIEWPORT_PADDING_FOR_STATIC_TILES: f64 = 1024.;

/// maplibre/maplibre-native#4add9ea original name: findViewportPadding
fn find_viewport_padding(transform_state: &TransformState, map_mode: MapMode) -> f64 {
    if map_mode == MapMode::Tile {
        return VIEWPORT_PADDING_FOR_STATIC_TILES;
    }
    if transform_state.camera().get_pitch().0 != 0.0 {
        VIEWPORT_PADDING_DEFAULT * 2.
    } else {
        VIEWPORT_PADDING_DEFAULT
    }
}

/// maplibre/maplibre-native#4add9ea original name: CollisionGrid
type CollisionGrid = GridIndex<IndexedSubfeature>;
/// maplibre/maplibre-native#4add9ea original name: CollisionIndex
pub struct CollisionIndex {
    transform_state: TransformState,
    viewport_padding: f64,
    collision_grid: CollisionGrid,
    ignored_grid: CollisionGrid,
    screen_right_boundary: f64,
    screen_bottom_boundary: f64,
    grid_right_boundary: f64,
    grid_bottom_boundary: f64,
    pitch_factor: f64,
}

impl CollisionIndex {
    /// maplibre/maplibre-native#4add9ea original name: new
    pub fn new(transform_state: &TransformState, map_mode: MapMode) -> Self {
        let viewport_padding = find_viewport_padding(transform_state, map_mode);
        Self {
            transform_state: transform_state.clone(),
            viewport_padding,
            collision_grid: CollisionGrid::new(
                transform_state.width() + 2. * viewport_padding,
                transform_state.height() + 2. * viewport_padding,
                25,
            ),
            ignored_grid: CollisionGrid::new(
                transform_state.width() + 2. * viewport_padding,
                transform_state.height() + 2. * viewport_padding,
                25,
            ),
            screen_right_boundary: transform_state.width() + viewport_padding,
            screen_bottom_boundary: transform_state.height() + viewport_padding,
            grid_right_boundary: transform_state.width() + 2. * viewport_padding,
            grid_bottom_boundary: transform_state.height() + 2. * viewport_padding,
            pitch_factor: transform_state.camera().get_pitch().0.cos()
                * transform_state.camera_to_center_distance(),
        }
    }

    /// maplibre/maplibre-native#4add9ea original name: intersectsTileEdges
    pub fn intersects_tile_edges(
        &self,
        box_: &CollisionBox,
        shift: Point2D<f64, ScreenSpace>,
        pos_matrix: &ModelViewProjection,
        text_pixel_ratio: f64,
        tile_edges: CollisionBoundaries,
    ) -> IntersectStatus {
        let boundaries =
            self.get_projected_collision_boundaries(pos_matrix, shift, text_pixel_ratio, box_);
        let mut result: IntersectStatus = IntersectStatus::default();
        let x1 = boundaries.min.x;
        let y1 = boundaries.min.y;
        let x2 = boundaries.max.x;
        let y2 = boundaries.max.y;

        let tile_x1 = tile_edges.min.x;
        let tile_y1 = tile_edges.min.y;
        let tile_x2 = tile_edges.max.x;
        let tile_y2 = tile_edges.max.y;

        // Check left border
        let mut min_section_length = ((tile_x1 - x1).min(x2 - tile_x1)) as i32;
        if min_section_length <= 0 {
            // Check right border
            min_section_length = ((tile_x2 - x1).min(x2 - tile_x2)) as i32;
        }
        if min_section_length > 0 {
            result.flags |= IntersectStatusFlags::VerticalBorders;
            result.min_section_length = min_section_length;
        }
        // Check top border
        min_section_length = ((tile_y1 - y1).min(y2 - tile_y1)) as i32;
        if min_section_length <= 0 {
            // Check bottom border
            min_section_length = ((tile_y2 - y1).min(y2 - tile_y2)) as i32;
        }
        if min_section_length > 0 {
            result.flags |= IntersectStatusFlags::HorizontalBorders;
            result.min_section_length = (result.min_section_length).min(min_section_length);
        }
        result
    }

    /// maplibre/maplibre-native#4add9ea original name: placeFeature
    pub fn place_feature<F>(
        &self,
        feature: &CollisionFeature,
        shift: Point2D<f64, ScreenSpace>,
        pos_matrix: &ModelViewProjection,
        label_plane_matrix: &Matrix4<f64>,
        text_pixel_ratio: f64,
        symbol: &PlacedSymbol,
        scale: f64,
        font_size: f64,
        allow_overlap: bool,
        pitch_with_map: bool,
        collision_debug: bool,
        avoid_edges: Option<CollisionBoundaries>,
        collision_group_predicate: Option<F>,
        projected_boxes: &mut Vec<ProjectedCollisionBox>, /*out*/
    ) -> (bool, bool)
    where
        F: Fn(&IndexedSubfeature) -> bool,
    {
        assert!(projected_boxes.is_empty());
        if !feature.along_line {
            let box_ = feature.boxes.first().unwrap();
            let collision_boundaries =
                self.get_projected_collision_boundaries(pos_matrix, shift, text_pixel_ratio, box_);
            projected_boxes.push(ProjectedCollisionBox::Box(collision_boundaries));

            if let Some(avoid_edges) = avoid_edges {
                if !self.is_inside_tile(&collision_boundaries, &avoid_edges) {
                    return (false, false);
                }
            }

            if !self.is_inside_grid(&collision_boundaries)
                || (!allow_overlap
                    && self.collision_grid.hit_test(
                        projected_boxes.last().unwrap().box_(),
                        collision_group_predicate,
                    ))
            {
                return (false, false);
            }

            (true, self.is_offscreen(&collision_boundaries))
        } else {
            self.place_line_feature(
                feature,
                pos_matrix,
                label_plane_matrix,
                text_pixel_ratio,
                symbol,
                scale,
                font_size,
                allow_overlap,
                pitch_with_map,
                collision_debug,
                avoid_edges,
                collision_group_predicate,
                projected_boxes,
            )
        }
    }

    /// maplibre/maplibre-native#4add9ea original name: insertFeature
    pub fn insert_feature(
        &mut self,
        feature: CollisionFeature,
        projected_boxes: &Vec<ProjectedCollisionBox>,
        ignore_placement: bool,
        bucket_instance_id: u32,
        collision_group_id: u16,
    ) {
        if feature.along_line {
            for circle in projected_boxes {
                if !circle.is_circle() {
                    continue;
                }

                if ignore_placement {
                    self.ignored_grid.insert_circle(
                        IndexedSubfeature::new(
                            feature.indexed_feature.clone(),
                            bucket_instance_id,
                            collision_group_id,
                        ), // FIXME clone() should not be needed?
                        *circle.circle(),
                    );
                } else {
                    self.collision_grid.insert_circle(
                        IndexedSubfeature::new(
                            feature.indexed_feature.clone(),
                            bucket_instance_id,
                            collision_group_id,
                        ), // FIXME clone() should not be needed?
                        *circle.circle(),
                    );
                }
            }
        } else if !projected_boxes.is_empty() {
            assert!(projected_boxes.len() == 1);
            let box_ = projected_boxes[0];
            // TODO assert!(box_.isBox());
            if ignore_placement {
                self.ignored_grid.insert(
                    IndexedSubfeature::new(
                        feature.indexed_feature,
                        bucket_instance_id,
                        collision_group_id,
                    ),
                    *box_.box_(),
                );
            } else {
                self.collision_grid.insert(
                    IndexedSubfeature::new(
                        feature.indexed_feature,
                        bucket_instance_id,
                        collision_group_id,
                    ),
                    *box_.box_(),
                );
            }
        }
    }

    /// maplibre/maplibre-native#4add9ea original name: queryRenderedSymbols
    pub fn query_rendered_symbols(
        &self,
        line_string: &ScreenLineString,
    ) -> HashMap<u32, Vec<IndexedSubfeature>> {
        todo!()
    }

    /// maplibre/maplibre-native#4add9ea original name: projectTileBoundaries
    pub fn project_tile_boundaries(&self, pos_matrix: &ModelViewProjection) -> CollisionBoundaries {
        let top_left = self.project_point(pos_matrix, &Point2D::zero());
        let bottom_right = self.project_point(pos_matrix, &Point2D::new(EXTENT, EXTENT)); // FIXME: maplibre-native uses here 8192 for extent

        CollisionBoundaries::new(
            Point2D::new(top_left.x, top_left.y),
            Point2D::new(bottom_right.x, bottom_right.y),
        )
    }

    /// maplibre/maplibre-native#4add9ea original name: getTransformState
    pub fn get_transform_state(&self) -> &TransformState {
        &self.transform_state
    }

    /// maplibre/maplibre-native#4add9ea original name: getViewportPadding
    pub fn get_viewport_padding(&self) -> f64 {
        self.viewport_padding
    }
}

impl CollisionIndex {
    /// maplibre/maplibre-native#4add9ea original name: isOffscreen
    fn is_offscreen(&self, boundaries: &CollisionBoundaries) -> bool {
        boundaries.max.x < self.viewport_padding
            || boundaries.min.x >= self.screen_right_boundary
            || boundaries.max.y < self.viewport_padding
            || boundaries.min.y >= self.screen_bottom_boundary
    }
    /// maplibre/maplibre-native#4add9ea original name: isInsideGrid
    fn is_inside_grid(&self, boundaries: &CollisionBoundaries) -> bool {
        boundaries.max.x >= 0.
            && boundaries.min.x < self.grid_right_boundary
            && boundaries.max.y >= 0.
            && boundaries.min.y < self.grid_bottom_boundary
    }

    /// maplibre/maplibre-native#4add9ea original name: isInsideTile
    fn is_inside_tile(
        &self,
        boundaries: &CollisionBoundaries,
        tile_boundaries: &CollisionBoundaries,
    ) -> bool {
        boundaries.min.x >= tile_boundaries.min.x
            && boundaries.min.y >= tile_boundaries.min.y
            && boundaries.max.x < tile_boundaries.max.x
            && boundaries.max.y < tile_boundaries.max.y
    }

    /// maplibre/maplibre-native#4add9ea original name: overlapsTile
    fn overlaps_tile(
        &self,
        boundaries: &CollisionBoundaries,
        tile_boundaries: &CollisionBoundaries,
    ) -> bool {
        boundaries.min.x < tile_boundaries.max.x
            && boundaries.max.x > tile_boundaries.min.x
            && boundaries.min.y < tile_boundaries.max.y
            && boundaries.max.y > tile_boundaries.min.y
    }

    /// maplibre/maplibre-native#4add9ea original name: placeLineFeature
    fn place_line_feature<F>(
        &self,
        feature: &CollisionFeature,
        pos_matrix: &ModelViewProjection,
        label_plane_matrix: &Matrix4<f64>,
        text_pixel_ratio: f64,
        symbol: &PlacedSymbol,
        scale: f64,
        font_size: f64,
        allow_overlap: bool,
        pitch_with_map: bool,
        collision_debug: bool,
        avoid_edges: Option<CollisionBoundaries>,
        collision_group_predicate: Option<F>,
        projected_boxes: &mut Vec<ProjectedCollisionBox>, /*out*/
    ) -> (bool, bool)
    where
        F: Fn(&IndexedSubfeature) -> bool,
    {
        assert!(feature.along_line);
        assert!(projected_boxes.is_empty());
        let tile_unit_anchor_point = symbol.anchor_point;
        let projected_anchor = self.project_anchor(pos_matrix, &tile_unit_anchor_point);

        let font_scale = font_size / 24.;
        let line_offset_x = symbol.line_offset[0] * font_size;
        let line_offset_y = symbol.line_offset[1] * font_size;

        let label_plane_anchor_point = project(tile_unit_anchor_point, label_plane_matrix).0;

        let first_and_last_glyph = place_first_and_last_glyph(
            font_scale,
            line_offset_x,
            line_offset_y,
            /*flip*/ false,
            label_plane_anchor_point,
            tile_unit_anchor_point,
            symbol,
            label_plane_matrix,
            /*return tile distance*/ true,
        );

        let mut collision_detected = false;
        let mut in_grid = false;
        let mut entirely_offscreen = true;

        let tile_to_viewport = projected_anchor.0 * text_pixel_ratio;
        // pixelsToTileUnits is used for translating line geometry to tile units
        // ... so we care about 'scale' but not 'perspectiveRatio'
        // equivalent to pixel_to_tile_units
        let pixels_to_tile_units = 1. / (text_pixel_ratio * scale);

        let mut first_tile_distance = 0.;
        let mut last_tile_distance = 0.;
        if let Some(first_and_last_glyph) = &first_and_last_glyph {
            first_tile_distance = self.approximate_tile_distance(
                first_and_last_glyph.0.tile_distance.as_ref().unwrap(),
                first_and_last_glyph.0.angle,
                pixels_to_tile_units,
                projected_anchor.1,
                pitch_with_map,
            );
            last_tile_distance = self.approximate_tile_distance(
                first_and_last_glyph.1.tile_distance.as_ref().unwrap(),
                first_and_last_glyph.1.angle,
                pixels_to_tile_units,
                projected_anchor.1,
                pitch_with_map,
            );
        }

        let mut previous_circle_placed = false;
        projected_boxes.resize(feature.boxes.len(), ProjectedCollisionBox::default());
        for i in 0..feature.boxes.len() {
            let circle = feature.boxes[i];
            let box_signed_distance_from_anchor = circle.signed_distance_from_anchor;
            if first_and_last_glyph.is_none()
                || (box_signed_distance_from_anchor < -first_tile_distance)
                || (box_signed_distance_from_anchor > last_tile_distance)
            {
                // The label either doesn't fit on its line or we
                // don't need to use this circle because the label
                // doesn't extend this far. Either way, mark the circle unused.
                previous_circle_placed = false;
                continue;
            }

            let projected_point = self.project_point(pos_matrix, &circle.anchor);
            let tile_unit_radius = (circle.x2 - circle.x1) / 2.;
            let radius = tile_unit_radius * tile_to_viewport;

            if previous_circle_placed {
                let previous_circle = &projected_boxes[i - 1];
                assert!(previous_circle.is_circle());
                let previous_center = previous_circle.circle().center;
                let dx = projected_point.x - previous_center.x;
                let dy = projected_point.y - previous_center.y;
                // The circle edges touch when the distance between their centers is
                // 2x the radius When the distance is 1x the radius, they're doubled
                // up, and we could remove every other circle while keeping them all
                // in touch. We actually start removing circles when the distance is
                // √2x the radius:
                //  thinning the number of circles as much as possible is a major
                //  performance win, and the small gaps introduced don't make a very
                //  noticeable difference.
                let placed_too_densely = radius * radius * 2. > dx * dx + dy * dy;
                if placed_too_densely {
                    let at_least_one_more_circle = (i + 1) < feature.boxes.len();
                    if at_least_one_more_circle {
                        let next_circle = feature.boxes[i + 1];
                        let next_box_distance_from_anchor = next_circle.signed_distance_from_anchor;
                        if (next_box_distance_from_anchor > -first_tile_distance)
                            && (next_box_distance_from_anchor < last_tile_distance)
                        {
                            // Hide significantly overlapping circles, unless this
                            // is the last one we can use, in which case we want to
                            // keep it in place even if it's tightly packed with the
                            // one before it.
                            previous_circle_placed = false;
                            continue;
                        }
                    }
                }
            }

            previous_circle_placed = true;

            let collision_boundaries = CollisionBoundaries::new(
                Point2D::new(projected_point.x - radius, projected_point.y - radius),
                Point2D::new(projected_point.x + radius, projected_point.y + radius),
            );

            projected_boxes[i] = ProjectedCollisionBox::Circle(Circle::new(
                Point2D::new(projected_point.x, projected_point.y),
                radius,
            ));

            entirely_offscreen &= self.is_offscreen(&collision_boundaries);
            in_grid |= self.is_inside_grid(&collision_boundaries);

            if let Some(avoid_edges) = avoid_edges {
                if !self.is_inside_tile(&collision_boundaries, &avoid_edges) {
                    if !collision_debug {
                        return (false, false);
                    } else {
                        // Don't early exit if we're showing the debug circles because
                        // we still want to calculate which circles are in use
                        collision_detected = true;
                    }
                }
            }

            if !allow_overlap
                && self.collision_grid.hit_test_circle(
                    projected_boxes[i].circle(),
                    collision_group_predicate.as_ref(),
                )
            {
                if !collision_debug {
                    return (false, false);
                } else {
                    // Don't early exit if we're showing the debug circles because
                    // we still want to calculate which circles are in use
                    collision_detected = true;
                }
            }
        }

        (
            !collision_detected && first_and_last_glyph.is_some() && in_grid,
            entirely_offscreen,
        )
    }

    /// maplibre/maplibre-native#4add9ea original name: approximateTileDistance
    fn approximate_tile_distance(
        &self,
        tile_distance: &TileDistance,
        last_segment_angle: f64,
        pixels_to_tile_units: f64,
        camera_to_anchor_distance: f64,
        pitch_with_map: bool,
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

        let incidence_stretch = if pitch_with_map {
            1.
        } else {
            camera_to_anchor_distance / self.pitch_factor
        };
        let last_segment_tile = tile_distance.last_segment_viewport_distance * pixels_to_tile_units;
        tile_distance.prev_tile_distance
            + last_segment_tile
            + (incidence_stretch - 1.) * last_segment_tile * last_segment_angle.sin().abs()
    }

    /// maplibre/maplibre-native#4add9ea original name: projectAnchor
    fn project_anchor(
        &self,
        pos_matrix: &ModelViewProjection,
        point: &Point2D<f64, TileSpace>,
    ) -> (f64, f64) {
        let p = Vector4::new(point.x, point.y, 0., 1.);
        let p = pos_matrix.project(p); // TODO verify multiplication
        (
            0.5 + 0.5 * (self.transform_state.camera_to_center_distance() / p[3]),
            p[3],
        )
    }
    /// maplibre/maplibre-native#4add9ea original name: projectAndGetPerspectiveRatio
    fn project_and_get_perspective_ratio(
        &self,
        pos_matrix: &ModelViewProjection,
        point: &Point2D<f64, TileSpace>,
    ) -> (Point2D<f64, ScreenSpace>, f64) {
        let p = Vector4::new(point.x, point.y, 0., 1.);
        let p = pos_matrix.project(p); // TODO verify multiplication
        let width = self.transform_state.width();
        let height = self.transform_state.height();
        let ccd = self.transform_state.camera_to_center_distance();
        (
            Point2D::new(
                ((p[0] / p[3] + 1.) / 2.) * width + self.viewport_padding,
                ((-p[1] / p[3] + 1.) / 2.) * height + self.viewport_padding,
            ),
            // See perspective ratio comment in symbol_sdf.vertex
            // We're doing collision detection in viewport space so we need
            // to scale down boxes in the distance
            0.5 + 0.5 * ccd / p[3],
        )
    }
    /// maplibre/maplibre-native#4add9ea original name: projectPoint
    fn project_point(
        &self,
        pos_matrix: &ModelViewProjection,
        point: &Point2D<f64, TileSpace>,
    ) -> Point2D<f64, ScreenSpace> {
        let p = Vector4::new(point.x, point.y, 0., 1.);
        let p = pos_matrix.project(p); // TODO verify multiplication
        let width = self.transform_state.width();
        let height = self.transform_state.height();
        Point2D::new(
            ((p[0] / p[3] + 1.) / 2.) * width + self.viewport_padding,
            ((-p[1] / p[3] + 1.) / 2.) * height + self.viewport_padding,
        )
    }

    /// maplibre/maplibre-native#4add9ea original name: getProjectedCollisionBoundaries
    fn get_projected_collision_boundaries(
        &self,
        pos_matrix: &ModelViewProjection,
        shift: Point2D<f64, ScreenSpace>,
        text_pixel_ratio: f64,
        box_: &CollisionBox,
    ) -> CollisionBoundaries {
        let (projected_point, tile_to_viewport) =
            self.project_and_get_perspective_ratio(pos_matrix, &box_.anchor);
        let tile_to_viewport = text_pixel_ratio * tile_to_viewport;
        let tile_to_viewport = 1.; // TODO
        CollisionBoundaries::new(
            Point2D::new(
                (box_.x1 + shift.x) * tile_to_viewport + projected_point.x,
                (box_.y1 + shift.y) * tile_to_viewport + projected_point.y,
            ),
            Point2D::new(
                (box_.x2 + shift.x) * tile_to_viewport + projected_point.x,
                (box_.y2 + shift.y) * tile_to_viewport + projected_point.y,
            ),
        )
    }
}
