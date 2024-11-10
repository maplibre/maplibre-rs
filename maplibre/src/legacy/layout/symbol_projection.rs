//! Translated from https://github.com/maplibre/maplibre-native/blob/4add9ea/src/mbgl/layout/symbol_projection.cpp

use std::f64::consts::PI;

use cgmath::{Matrix4, Vector4};

use crate::{
    euclid::Point2D,
    legacy::{
        buckets::symbol_bucket::PlacedSymbol,
        geometry_tile_data::GeometryCoordinates,
        util::math::{convert_point_f64, perp},
        TileSpace,
    },
};

/// maplibre/maplibre-native#4add9ea original name: PointAndCameraDistance
type PointAndCameraDistance = (Point2D<f64, TileSpace>, f64); // TODO is the Unit correct?

/// maplibre/maplibre-native#4add9ea original name: TileDistance
pub struct TileDistance {
    pub prevTileDistance: f64,
    pub lastSegmentViewportDistance: f64,
}

/// maplibre/maplibre-native#4add9ea original name: project
pub fn project(point: Point2D<f64, TileSpace>, matrix: &Matrix4<f64>) -> PointAndCameraDistance {
    let pos = Vector4::new(point.x, point.y, 0., 1.);
    let pos = matrix * pos; // TODO verify this multiplications
    (Point2D::new(pos[0] / pos[3], pos[1] / pos[3]), pos[3])
}

/// maplibre/maplibre-native#4add9ea original name: PlacedGlyph
pub struct PlacedGlyph {
    pub point: Point2D<f64, TileSpace>,
    pub angle: f64,
    pub tileDistance: Option<TileDistance>,
}

/// maplibre/maplibre-native#4add9ea original name: placeFirstAndLastGlyph
pub fn place_first_and_last_glyph(
    font_scale: f64,
    line_offset_x: f64,
    line_offset_y: f64,
    flip: bool,
    anchor_point: Point2D<f64, TileSpace>,
    tile_anchor_point: Point2D<f64, TileSpace>,
    symbol: &PlacedSymbol,
    label_plane_matrix: &Matrix4<f64>,
    return_tile_distance: bool,
) -> Option<(PlacedGlyph, PlacedGlyph)> {
    if symbol.glyph_offsets.is_empty() {
        assert!(false);
        return None;
    }

    let first_glyph_offset = *symbol.glyph_offsets.first().unwrap();
    let last_glyph_offset = *symbol.glyph_offsets.last().unwrap();

    if let (Some(firstPlacedGlyph), Some(lastPlacedGlyph)) = (
        place_glyph_along_line(
            font_scale * first_glyph_offset,
            line_offset_x,
            line_offset_y,
            flip,
            &anchor_point,
            &tile_anchor_point,
            symbol.segment as i16,
            &symbol.line,
            &symbol.tile_distances,
            label_plane_matrix,
            return_tile_distance,
        ),
        place_glyph_along_line(
            font_scale * last_glyph_offset,
            line_offset_x,
            line_offset_y,
            flip,
            &anchor_point,
            &tile_anchor_point,
            symbol.segment as i16,
            &symbol.line,
            &symbol.tile_distances,
            label_plane_matrix,
            return_tile_distance,
        ),
    ) {
        return Some((firstPlacedGlyph, lastPlacedGlyph));
    }

    None
}

/// maplibre/maplibre-native#4add9ea original name: placeGlyphAlongLine
fn place_glyph_along_line(
    offset_x: f64,
    line_offset_x: f64,
    line_offset_y: f64,
    flip: bool,
    projected_anchor_point: &Point2D<f64, TileSpace>,
    tile_anchor_point: &Point2D<f64, TileSpace>,
    anchor_segment: i16,
    line: &GeometryCoordinates,
    tile_distances: &Vec<f64>,
    label_plane_matrix: &Matrix4<f64>,
    return_tile_distance: bool,
) -> Option<PlacedGlyph> {
    let combined_offset_x = if flip {
        offset_x - line_offset_x
    } else {
        offset_x + line_offset_x
    };

    let mut dir: i16 = if combined_offset_x > 0. { 1 } else { -1 };

    let mut angle = 0.0;
    if flip {
        // The label needs to be flipped to keep text upright.
        // Iterate in the reverse direction.
        dir *= -1;
        angle = PI;
    }

    if dir < 0 {
        angle += PI;
    }

    let mut current_index = if dir > 0 {
        anchor_segment
    } else {
        anchor_segment + 1
    };

    let initial_index = current_index;
    let mut current = *projected_anchor_point;
    let mut prev = *projected_anchor_point;
    let mut distance_to_prev = 0.0;
    let mut current_segment_distance = 0.0;
    let abs_offset_x = combined_offset_x.abs();

    while distance_to_prev + current_segment_distance <= abs_offset_x {
        current_index += dir;

        // offset does not fit on the projected line
        if current_index < 0 || current_index >= line.len() as i16 {
            return None;
        }

        prev = current;
        let projection = project(
            convert_point_f64(&line[current_index as usize]),
            label_plane_matrix,
        );
        if projection.1 > 0. {
            current = projection.0;
        } else {
            // The vertex is behind the plane of the camera, so we can't project it
            // Instead, we'll create a vertex along the line that's far enough to include the glyph
            let previous_tile_point = if distance_to_prev == 0. {
                *tile_anchor_point
            } else {
                convert_point_f64(&line[(current_index - dir) as usize])
            };

            let current_tile_point = convert_point_f64(&line[current_index as usize]);
            current = project_truncated_line_segment(
                &previous_tile_point,
                &current_tile_point,
                &prev,
                abs_offset_x - distance_to_prev + 1.,
                label_plane_matrix,
            );
        }

        distance_to_prev += current_segment_distance;
        current_segment_distance = prev.distance_to(current); // TODO verify distance calculation is correct
    }

    // The point is on the current segment. Interpolate to find it.
    let segment_interpolation_t = (abs_offset_x - distance_to_prev) / current_segment_distance;
    let prev_to_current = current - prev;
    let mut p = prev + (prev_to_current * segment_interpolation_t);

    // offset the point from the line to text-offset and icon-offset
    p += perp(&prev_to_current) * (line_offset_y * dir as f64 / prev_to_current.length()); // TODO verify if mag impl is correct mag == length?

    let segment_angle = angle + (current.y - prev.y).atan2(current.x - prev.x); // TODO is this atan2 right?

    Some(PlacedGlyph {
        point: p,
        angle: segment_angle,
        tileDistance: if return_tile_distance {
            Some(TileDistance {
                // TODO are these the right fields assigned?
                prevTileDistance: if (current_index - dir) == initial_index {
                    0.
                } else {
                    tile_distances[(current_index - dir) as usize]
                },
                lastSegmentViewportDistance: abs_offset_x - distance_to_prev,
            })
        } else {
            None
        },
    })
}

/// maplibre/maplibre-native#4add9ea original name: projectTruncatedLineSegment
fn project_truncated_line_segment(
    &previousTilePoint: &Point2D<f64, TileSpace>,
    current_tile_point: &Point2D<f64, TileSpace>,
    previous_projected_point: &Point2D<f64, TileSpace>,
    minimum_length: f64,
    projection_matrix: &Matrix4<f64>,
) -> Point2D<f64, TileSpace> {
    // We are assuming "previousTilePoint" won't project to a point within one
    // unit of the camera plane If it did, that would mean our label extended
    // all the way out from within the viewport to a (very distant) point near
    // the plane of the camera. We wouldn't be able to render the label anyway
    // once it crossed the plane of the camera.
    let vec = previousTilePoint - *current_tile_point;
    let projected_unit_vertex = project(
        previousTilePoint + vec.try_normalize().unwrap_or(vec),
        projection_matrix,
    )
    .0;
    let projected_unit_segment = *previous_projected_point - projected_unit_vertex;

    *previous_projected_point
        + (projected_unit_segment * (minimum_length / projected_unit_segment.length()))
    // TODO verify if mag impl is correct mag == length?
}
