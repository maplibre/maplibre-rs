use lyon::tessellation::{VertexBuffers, VertexId};

use super::{
    granularity_for_zoom, subdivide_line_segment, subdivide_triangles, FillSubdivisionOptions,
};
use crate::render::ShaderVertex;

#[test]
fn granularity_matches_gl_js_globe_policy() {
    assert_eq!(granularity_for_zoom(128, 2, 0), 128);
    assert_eq!(granularity_for_zoom(128, 2, 6), 2);
    assert_eq!(granularity_for_zoom(128, 2, 31), 2);
    assert_eq!(granularity_for_zoom(512, 0, 0), 512);
    assert_eq!(granularity_for_zoom(512, 0, 9), 1);
}

#[test]
fn line_subdivision_matches_reference_examples() {
    let horizontal = subdivide_line_segment([0.0, 0.0], [4096.0, 0.0], 4)
        .expect("reference line should subdivide");
    assert_eq!(
        horizontal,
        vec![[1024.0, 0.0], [2048.0, 0.0], [3072.0, 0.0], [4096.0, 0.0]]
    );

    let diagonal = subdivide_line_segment([4096.0, 0.0], [0.0, 4096.0], 4)
        .expect("reference diagonal should subdivide");
    assert_eq!(
        diagonal,
        vec![
            [3072.0, 1024.0],
            [2048.0, 2048.0],
            [1024.0, 3072.0],
            [0.0, 4096.0]
        ]
    );
}

#[test]
fn fill_triangle_is_cut_on_both_grid_axes() {
    let mut buffer = VertexBuffers::<ShaderVertex, u32> {
        vertices: vec![
            ShaderVertex::new([0.0, 0.0], [0.0; 2]),
            ShaderVertex::new([4096.0, 0.0], [0.0; 2]),
            ShaderVertex::new([0.0, 4096.0], [0.0; 2]),
        ],
        indices: vec![0, 1, 2],
    };

    subdivide_triangles(&mut buffer, 0, options(2)).expect("reference triangle should subdivide");

    assert!(buffer.indices.len() > 3);
    for index in &buffer.indices {
        assert!((*index as usize) < buffer.vertices.len());
    }
    assert!(buffer
        .vertices
        .iter()
        .any(|vertex| vertex.position == [2048.0, 0.0]));
    assert!(buffer
        .vertices
        .iter()
        .any(|vertex| vertex.position == [0.0, 2048.0]));
    let _: VertexId = 0_u32.into();
}

#[test]
fn zoom_zero_clipping_drops_buffered_antimeridian_geometry() {
    let mut buffer = VertexBuffers::<ShaderVertex, u32> {
        vertices: vec![
            ShaderVertex::new([-10.0, 0.0], [0.0; 2]),
            ShaderVertex::new([10.0, 0.0], [0.0; 2]),
            ShaderVertex::new([10.0, 10.0], [0.0; 2]),
        ],
        indices: vec![0, 1, 2],
    };

    let mut subdivision = options(2);
    subdivision.clip_x_to_tile = true;
    subdivide_triangles(&mut buffer, 0, subdivision).expect("clipped triangle should be valid");

    assert!(!buffer.indices.is_empty());
    assert!(buffer.indices.iter().all(|index| {
        let x = buffer.vertices[*index as usize].position[0];
        (0.0..=4096.0).contains(&x)
    }));
}

#[test]
fn north_edge_generates_special_pole_vertices() {
    let mut buffer = VertexBuffers::<ShaderVertex, u32> {
        vertices: vec![
            ShaderVertex::new([0.0, 0.0], [0.0; 2]),
            ShaderVertex::new([4096.0, 0.0], [0.0; 2]),
            ShaderVertex::new([0.0, 4096.0], [0.0; 2]),
        ],
        indices: vec![0, 1, 2],
    };
    let mut subdivision = options(2);
    subdivision.extend_to_north_pole = true;

    subdivide_triangles(&mut buffer, 0, subdivision).expect("pole fill should subdivide");

    assert!(buffer
        .vertices
        .iter()
        .any(|vertex| vertex.position[1] == i16::MIN as f32));
}

fn options(granularity: u32) -> FillSubdivisionOptions {
    FillSubdivisionOptions {
        granularity,
        clip_x_to_tile: false,
        extend_to_north_pole: false,
        extend_to_south_pole: false,
    }
}
