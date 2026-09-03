//! Geometry subdivision needed to approximate curved globe features.

use std::collections::HashMap;

use lyon::tessellation::{VertexBuffers, VertexId};
use thiserror::Error;

use crate::{coords::EXTENT_UINT, render::ShaderVertex};

const MAX_GENERATED_VERTICES_PER_SEGMENT: usize = 16_384;
const MAX_CELLS_PER_TRIANGLE: usize = 65_536;
const NORTH_POLE_Y: f32 = i16::MIN as f32;
const SOUTH_POLE_Y: f32 = i16::MAX as f32;

/// Controls fill-mesh subdivision and globe-only edge geometry.
#[derive(Clone, Copy, Debug)]
pub struct FillSubdivisionOptions {
    /// Number of grid cells along each tile axis.
    pub granularity: u32,
    /// Rejects buffered geometry beyond the z0 antimeridian edges.
    pub clip_x_to_tile: bool,
    /// Extends fill edges at y=0 to the north pole.
    pub extend_to_north_pole: bool,
    /// Extends fill edges at y=extent to the south pole.
    pub extend_to_south_pole: bool,
}

/// Failure while subdividing untrusted tile geometry.
#[derive(Clone, Copy, Debug, Error, PartialEq)]
pub enum SubdivisionError {
    /// A line would generate an unreasonable number of vertices.
    #[error("globe line subdivision exceeds {limit} generated vertices")]
    LineVertexLimit {
        /// Maximum allowed generated vertices.
        limit: usize,
    },
    /// A triangle overlaps an unreasonable number of subdivision cells.
    #[error("globe fill subdivision exceeds {limit} cells for one triangle")]
    TriangleCellLimit {
        /// Maximum allowed intersected cells.
        limit: usize,
    },
    /// A tessellation index does not address an existing vertex.
    #[error("globe subdivision index {index} is outside {vertex_count} vertices")]
    InvalidIndex {
        /// Invalid vertex index.
        index: usize,
        /// Number of available vertices.
        vertex_count: usize,
    },
}

/// Returns the GL JS subdivision policy for a canonical zoom level.
pub fn granularity_for_zoom(base: u32, minimum: u32, zoom: u8) -> u32 {
    let scaled = base.checked_shr(u32::from(zoom)).unwrap_or(0);
    scaled.max(minimum).max(1)
}

/// Inserts every grid-axis intersection along one line segment.
pub fn subdivide_line_segment(
    start: [f32; 2],
    end: [f32; 2],
    granularity: u32,
) -> Result<Vec<[f32; 2]>, SubdivisionError> {
    if granularity < 2 || start == end {
        return Ok(vec![end]);
    }
    let cell_size = (EXTENT_UINT / granularity).max(1) as f32;
    let delta = [end[0] - start[0], end[1] - start[1]];
    let mut crossings = Vec::new();
    collect_axis_crossings(start[0], end[0], cell_size, &mut crossings);
    collect_axis_crossings(start[1], end[1], cell_size, &mut crossings);
    crossings.sort_by(f32::total_cmp);
    crossings.dedup_by(|left, right| (*left - *right).abs() <= f32::EPSILON);
    if crossings.len() > MAX_GENERATED_VERTICES_PER_SEGMENT {
        return Err(SubdivisionError::LineVertexLimit {
            limit: MAX_GENERATED_VERTICES_PER_SEGMENT,
        });
    }
    let mut points = crossings
        .into_iter()
        .filter(|fraction| *fraction > 0.0 && *fraction < 1.0)
        .map(|fraction| {
            [
                (start[0] + delta[0] * fraction).round(),
                (start[1] + delta[1] * fraction).round(),
            ]
        })
        .collect::<Vec<_>>();
    points.dedup();
    if points.last().copied() != Some(end) {
        points.push(end);
    }
    Ok(points)
}

/// Clips newly tessellated triangles to the regular subdivision grid.
pub fn subdivide_triangles<I>(
    buffer: &mut VertexBuffers<ShaderVertex, I>,
    index_start: usize,
    options: FillSubdivisionOptions,
) -> Result<(), SubdivisionError>
where
    I: Copy + Into<u32> + From<VertexId>,
{
    if options.granularity < 2 {
        return Ok(());
    }
    sanitize_pole_sentinels(&mut buffer.vertices);
    let input = buffer.indices[index_start..].to_vec();
    buffer.indices.truncate(index_start);
    let mut vertices = HashMap::new();
    for triangle in input.chunks_exact(3) {
        let points = triangle_points(&buffer.vertices, triangle)?;
        subdivide_triangle(
            buffer,
            points,
            options.granularity,
            options.clip_x_to_tile,
            &mut vertices,
        )?;
    }
    append_pole_quads(buffer, index_start, options, &mut vertices);
    Ok(())
}

fn sanitize_pole_sentinels(vertices: &mut [ShaderVertex]) {
    for vertex in vertices {
        if vertex.position[1] == NORTH_POLE_Y {
            vertex.position[1] = NORTH_POLE_Y + 1.0;
        } else if vertex.position[1] == SOUTH_POLE_Y {
            vertex.position[1] = SOUTH_POLE_Y - 1.0;
        }
    }
}

fn collect_axis_crossings(start: f32, end: f32, cell_size: f32, output: &mut Vec<f32>) {
    if start == end {
        return;
    }
    let low = start.min(end);
    let high = start.max(end);
    let first = (low / cell_size).floor() as i64 + 1;
    let last = (high / cell_size).ceil() as i64;
    for axis in first..last {
        let coordinate = axis as f32 * cell_size;
        output.push((coordinate - start) / (end - start));
    }
}

fn triangle_points<I: Copy + Into<u32>>(
    vertices: &[ShaderVertex],
    indices: &[I],
) -> Result<[[f32; 2]; 3], SubdivisionError> {
    let mut points = [[0.0; 2]; 3];
    for (slot, index) in points.iter_mut().zip(indices) {
        let index = (*index).into() as usize;
        let vertex = vertices.get(index).ok_or(SubdivisionError::InvalidIndex {
            index,
            vertex_count: vertices.len(),
        })?;
        *slot = vertex.position;
    }
    Ok(points)
}

fn subdivide_triangle<I>(
    buffer: &mut VertexBuffers<ShaderVertex, I>,
    triangle: [[f32; 2]; 3],
    granularity: u32,
    clip_x_to_tile: bool,
    cache: &mut HashMap<(i32, i32), usize>,
) -> Result<(), SubdivisionError>
where
    I: From<VertexId>,
{
    let cell_size = (EXTENT_UINT / granularity).max(1) as f32;
    let bounds = triangle_bounds(triangle, cell_size);
    let cell_count = (bounds.1 - bounds.0) as usize * (bounds.3 - bounds.2) as usize;
    if cell_count > MAX_CELLS_PER_TRIANGLE {
        return Err(SubdivisionError::TriangleCellLimit {
            limit: MAX_CELLS_PER_TRIANGLE,
        });
    }
    for y in bounds.2..bounds.3 {
        for x in bounds.0..bounds.1 {
            let mut polygon = triangle.to_vec();
            polygon = clip_axis(polygon, 0, x as f32 * cell_size, true);
            polygon = clip_axis(polygon, 0, (x + 1) as f32 * cell_size, false);
            polygon = clip_axis(polygon, 1, y as f32 * cell_size, true);
            polygon = clip_axis(polygon, 1, (y + 1) as f32 * cell_size, false);
            append_polygon(buffer, polygon, clip_x_to_tile, cache);
        }
    }
    Ok(())
}

fn triangle_bounds(triangle: [[f32; 2]; 3], cell_size: f32) -> (i64, i64, i64, i64) {
    let min_x = triangle
        .iter()
        .map(|point| point[0])
        .fold(f32::INFINITY, f32::min);
    let max_x = triangle
        .iter()
        .map(|point| point[0])
        .fold(f32::NEG_INFINITY, f32::max);
    let min_y = triangle
        .iter()
        .map(|point| point[1])
        .fold(f32::INFINITY, f32::min);
    let max_y = triangle
        .iter()
        .map(|point| point[1])
        .fold(f32::NEG_INFINITY, f32::max);
    (
        (min_x / cell_size).floor() as i64,
        (max_x / cell_size).ceil() as i64,
        (min_y / cell_size).floor() as i64,
        (max_y / cell_size).ceil() as i64,
    )
}

fn clip_axis(
    input: Vec<[f32; 2]>,
    axis: usize,
    boundary: f32,
    keep_greater: bool,
) -> Vec<[f32; 2]> {
    let Some(mut previous) = input.last().copied() else {
        return input;
    };
    let mut output = Vec::with_capacity(input.len() + 1);
    for current in input {
        let previous_inside = (previous[axis] >= boundary) == keep_greater;
        let current_inside = (current[axis] >= boundary) == keep_greater;
        if previous_inside != current_inside {
            output.push(axis_intersection(previous, current, axis, boundary));
        }
        if current_inside {
            output.push(current);
        }
        previous = current;
    }
    output
}

fn axis_intersection(start: [f32; 2], end: [f32; 2], axis: usize, boundary: f32) -> [f32; 2] {
    let fraction = (boundary - start[axis]) / (end[axis] - start[axis]);
    let mut point = [
        (start[0] + (end[0] - start[0]) * fraction).round(),
        (start[1] + (end[1] - start[1]) * fraction).round(),
    ];
    point[axis] = boundary;
    point
}

fn append_polygon<I: From<VertexId>>(
    buffer: &mut VertexBuffers<ShaderVertex, I>,
    polygon: Vec<[f32; 2]>,
    clip_x_to_tile: bool,
    cache: &mut HashMap<(i32, i32), usize>,
) {
    if polygon.len() < 3
        || (clip_x_to_tile
            && polygon
                .iter()
                .any(|point| point[0] < 0.0 || point[0] > EXTENT_UINT as f32))
    {
        return;
    }
    let first = vertex_index(buffer, polygon[0], cache);
    for edge in 1..polygon.len() - 1 {
        let second = vertex_index(buffer, polygon[edge], cache);
        let third = vertex_index(buffer, polygon[edge + 1], cache);
        if first != second && second != third && third != first {
            buffer
                .indices
                .extend([first, second, third].map(|index| I::from(VertexId::from_usize(index))));
        }
    }
}

fn append_pole_quads<I>(
    buffer: &mut VertexBuffers<ShaderVertex, I>,
    index_start: usize,
    options: FillSubdivisionOptions,
    cache: &mut HashMap<(i32, i32), usize>,
) where
    I: Copy + Into<u32> + From<VertexId>,
{
    let input_end = buffer.indices.len();
    for offset in (index_start..input_end).step_by(3) {
        let triangle = [
            buffer.indices[offset].into() as usize,
            buffer.indices[offset + 1].into() as usize,
            buffer.indices[offset + 2].into() as usize,
        ];
        for edge in 0..3 {
            let first = triangle[edge];
            let second = triangle[(edge + 1) % 3];
            let first_position = buffer.vertices[first].position;
            let second_position = buffer.vertices[second].position;
            if options.extend_to_north_pole && first_position[1] == 0.0 && second_position[1] == 0.0
            {
                append_pole_quad(
                    buffer,
                    first,
                    second,
                    first_position,
                    second_position,
                    NORTH_POLE_Y,
                    cache,
                );
            }
            if options.extend_to_south_pole
                && first_position[1] == EXTENT_UINT as f32
                && second_position[1] == EXTENT_UINT as f32
            {
                append_pole_quad(
                    buffer,
                    first,
                    second,
                    first_position,
                    second_position,
                    SOUTH_POLE_Y,
                    cache,
                );
            }
        }
    }
}

fn append_pole_quad<I>(
    buffer: &mut VertexBuffers<ShaderVertex, I>,
    first: usize,
    second: usize,
    first_position: [f32; 2],
    second_position: [f32; 2],
    pole_y: f32,
    cache: &mut HashMap<(i32, i32), usize>,
) where
    I: From<VertexId>,
{
    let first_pole = vertex_index(buffer, [first_position[0], pole_y], cache);
    let second_pole = vertex_index(buffer, [second_position[0], pole_y], cache);
    let flip = (first_position[0] > second_position[0]) != (pole_y == NORTH_POLE_Y);
    let triangles = if flip {
        [
            [first, second, first_pole],
            [second, second_pole, first_pole],
        ]
    } else {
        [
            [second, first, first_pole],
            [second_pole, second, first_pole],
        ]
    };
    for triangle in triangles {
        buffer
            .indices
            .extend(triangle.map(|index| I::from(VertexId::from_usize(index))));
    }
}

fn vertex_index<I>(
    buffer: &mut VertexBuffers<ShaderVertex, I>,
    point: [f32; 2],
    cache: &mut HashMap<(i32, i32), usize>,
) -> usize {
    let point = [point[0].round(), point[1].round()];
    let key = (point[0] as i32, point[1] as i32);
    if let Some(index) = cache.get(&key) {
        return *index;
    }
    let index = buffer.vertices.len();
    buffer.vertices.push(ShaderVertex::new(point, [0.0, 0.0]));
    cache.insert(key, index);
    index
}

#[cfg(test)]
mod tests;
