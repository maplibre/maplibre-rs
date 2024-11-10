//! Translated from https://github.com/maplibre/maplibre-native/blob/4add9ea/src/mbgl/layout/symbol_projection.cpp


use std::f64::consts::PI;

use cgmath::{Matrix4, Vector4};

use crate::{
    euclid::Point2D,
    sdf::{
        buckets::symbol_bucket::PlacedSymbol,
        geometry_tile_data::GeometryCoordinates, util::math::perp, TileSpace,
    },
};
use crate::sdf::util::math::convert_point_f64;

type PointAndCameraDistance = (Point2D<f64, TileSpace>, f64); // TODO is the Unit correct?

pub struct TileDistance {
    pub prevTileDistance: f64,
    pub lastSegmentViewportDistance: f64,
}

pub fn project(point: Point2D<f64, TileSpace>, matrix: &Matrix4<f64>) -> PointAndCameraDistance {
    let pos = Vector4::new(point.x, point.y, 0., 1.);
    let pos = matrix * pos; // TODO verify this multiplications
    return (Point2D::new(pos[0] / pos[3], pos[1] / pos[3]), pos[3]);
}

pub struct PlacedGlyph {
    pub point: Point2D<f64, TileSpace>,
    pub angle: f64,
    pub tileDistance: Option<TileDistance>,
}

pub fn placeFirstAndLastGlyph(
    fontScale: f64,
    lineOffsetX: f64,
    lineOffsetY: f64,
    flip: bool,
    anchorPoint: Point2D<f64, TileSpace>,
    tileAnchorPoint: Point2D<f64, TileSpace>,
    symbol: &PlacedSymbol,
    labelPlaneMatrix: &Matrix4<f64>,
    returnTileDistance: bool,
) -> Option<(PlacedGlyph, PlacedGlyph)> {
    if (symbol.glyphOffsets.is_empty()) {
        assert!(false);
        return None;
    }

    let firstGlyphOffset = *symbol.glyphOffsets.first().unwrap();
    let lastGlyphOffset = *symbol.glyphOffsets.last().unwrap();

    if let (Some(firstPlacedGlyph), Some(lastPlacedGlyph)) = (
        placeGlyphAlongLine(
            fontScale * firstGlyphOffset,
            lineOffsetX,
            lineOffsetY,
            flip,
            &anchorPoint,
            &tileAnchorPoint,
            symbol.segment as i16,
            &symbol.line,
            &symbol.tileDistances,
            labelPlaneMatrix,
            returnTileDistance,
        ),
        placeGlyphAlongLine(
            fontScale * lastGlyphOffset,
            lineOffsetX,
            lineOffsetY,
            flip,
            &anchorPoint,
            &tileAnchorPoint,
            symbol.segment as i16,
            &symbol.line,
            &symbol.tileDistances,
            labelPlaneMatrix,
            returnTileDistance,
        ),
    ) {
        return Some((firstPlacedGlyph, lastPlacedGlyph));
    }

    return None;
}

fn placeGlyphAlongLine(
    offsetX: f64,
    lineOffsetX: f64,
    lineOffsetY: f64,
    flip: bool,
    projectedAnchorPoint: &Point2D<f64, TileSpace>,
    tileAnchorPoint: &Point2D<f64, TileSpace>,
    anchorSegment: i16,
    line: &GeometryCoordinates,
    tileDistances: &Vec<f64>,
    labelPlaneMatrix: &Matrix4<f64>,
    returnTileDistance: bool,
) -> Option<PlacedGlyph> {
    let combinedOffsetX = if flip {
        offsetX - lineOffsetX
    } else {
        offsetX + lineOffsetX
    };

    let mut dir: i16 = if combinedOffsetX > 0. { 1 } else { -1 };

    let mut angle = 0.0;
    if (flip) {
        // The label needs to be flipped to keep text upright.
        // Iterate in the reverse direction.
        dir *= -1;
        angle = PI;
    }

    if (dir < 0) {
        angle += PI;
    }

    let mut currentIndex = if dir > 0 {
        anchorSegment
    } else {
        anchorSegment + 1
    };

    let initialIndex = currentIndex;
    let mut current = *projectedAnchorPoint;
    let mut prev = *projectedAnchorPoint;
    let mut distanceToPrev = 0.0;
    let mut currentSegmentDistance = 0.0;
    let absOffsetX = combinedOffsetX.abs();

    while (distanceToPrev + currentSegmentDistance <= absOffsetX) {
        currentIndex += dir;

        // offset does not fit on the projected line
        if (currentIndex < 0 || currentIndex >= line.len() as i16) {
            return None;
        }

        prev = current;
        let projection = project(
            convert_point_f64(&line[currentIndex as usize]),
            labelPlaneMatrix,
        );
        if (projection.1 > 0.) {
            current = projection.0;
        } else {
            // The vertex is behind the plane of the camera, so we can't project it
            // Instead, we'll create a vertex along the line that's far enough to include the glyph
            let previousTilePoint = if distanceToPrev == 0. {
                *tileAnchorPoint
            } else {
                convert_point_f64(&line[(currentIndex - dir) as usize])
            };

            let currentTilePoint = convert_point_f64(&line[currentIndex as usize]);
            current = projectTruncatedLineSegment(
                &previousTilePoint,
                &currentTilePoint,
                &prev,
                absOffsetX - distanceToPrev + 1.,
                labelPlaneMatrix,
            );
        }

        distanceToPrev += currentSegmentDistance;
        currentSegmentDistance = prev.distance_to(current); // TODO verify distance calculation is correct
    }

    // The point is on the current segment. Interpolate to find it.
    let segmentInterpolationT = (absOffsetX - distanceToPrev) / currentSegmentDistance;
    let prevToCurrent = current - prev;
    let mut p = prev + (prevToCurrent * segmentInterpolationT);

    // offset the point from the line to text-offset and icon-offset
    p += perp(&prevToCurrent) * (lineOffsetY * dir as f64 / prevToCurrent.length()) as f64; // TODO verify if mag impl is correct mag == length?

    let segmentAngle = angle + (current.y - prev.y).atan2(current.x - prev.x); // TODO is this atan2 right?

    return Some(PlacedGlyph {
        point: p,
        angle: segmentAngle,
        tileDistance: if returnTileDistance {
            Some(TileDistance {
                // TODO are these the right fields assigned?
                prevTileDistance: if (currentIndex - dir) == initialIndex {
                    0.
                } else {
                    tileDistances[(currentIndex - dir) as usize]
                },
                lastSegmentViewportDistance: absOffsetX - distanceToPrev,
            })
        } else {
            None
        },
    });
}

fn projectTruncatedLineSegment(
    &previousTilePoint: &Point2D<f64, TileSpace>,
    currentTilePoint: &Point2D<f64, TileSpace>,
    previousProjectedPoint: &Point2D<f64, TileSpace>,
    minimumLength: f64,
    projectionMatrix: &Matrix4<f64>,
) -> Point2D<f64, TileSpace> {
    // We are assuming "previousTilePoint" won't project to a point within one
    // unit of the camera plane If it did, that would mean our label extended
    // all the way out from within the viewport to a (very distant) point near
    // the plane of the camera. We wouldn't be able to render the label anyway
    // once it crossed the plane of the camera.
    let vec = (previousTilePoint.clone() - currentTilePoint.clone());
    let projectedUnitVertex = project(
        previousTilePoint + vec.try_normalize().unwrap_or(vec),
        projectionMatrix,
    )
    .0;
    let projectedUnitSegment = previousProjectedPoint.clone() - projectedUnitVertex.clone();

    return previousProjectedPoint.clone()
        + (projectedUnitSegment * (minimumLength / projectedUnitSegment.length()));
    // TODO verify if mag impl is correct mag == length?
}
