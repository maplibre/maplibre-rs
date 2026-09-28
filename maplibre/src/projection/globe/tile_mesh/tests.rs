#![allow(clippy::expect_used, clippy::panic)]

use super::{
    create_tile_mesh, TileIndexType, TileMeshError, TileMeshIndices, TileMeshOptions,
    TileMeshVertex, NORTH_POLE_Y, SOUTH_POLE_Y,
};

fn default_options() -> TileMeshOptions {
    TileMeshOptions::default()
}

#[test]
fn default_mesh_matches_gl_js_layout() {
    let mesh = create_tile_mesh(default_options(), TileIndexType::Auto)
        .expect("default mesh dimensions are valid");

    assert_eq!(
        mesh.vertices,
        vec![
            TileMeshVertex { x: 0, y: 0 },
            TileMeshVertex { x: 4096, y: 0 },
            TileMeshVertex { x: 0, y: 4096 },
            TileMeshVertex { x: 4096, y: 4096 },
        ]
    );
    assert_eq!(mesh.indices, TileMeshIndices::U16(vec![0, 2, 1, 1, 2, 3]));
}

#[test]
fn zero_granularity_is_normalized_to_one() {
    let mesh = create_tile_mesh(
        TileMeshOptions {
            granularity: 0,
            ..TileMeshOptions::default()
        },
        TileIndexType::Auto,
    )
    .expect("normalized mesh dimensions are valid");
    assert_eq!(mesh.vertices.len(), 4);
    assert_eq!(mesh.indices.len(), 6);
}

#[test]
fn vertex_layout_is_two_signed_16_bit_coordinates() {
    assert_eq!(std::mem::size_of::<TileMeshVertex>(), 4);
}

#[test]
fn forced_index_width_matches_request() {
    let u16_mesh =
        create_tile_mesh(default_options(), TileIndexType::U16).expect("quad fits in u16 indices");
    let u32_mesh =
        create_tile_mesh(default_options(), TileIndexType::U32).expect("quad fits in u32 indices");

    assert!(!u16_mesh.indices.uses_u32());
    assert!(u32_mesh.indices.uses_u32());
    assert_eq!(u16_mesh.indices.len(), u32_mesh.indices.len());
}

#[test]
fn granularity_subdivides_each_axis() {
    let mesh = create_tile_mesh(
        TileMeshOptions {
            granularity: 2,
            ..TileMeshOptions::default()
        },
        TileIndexType::Auto,
    )
    .expect("two by two grid is valid");

    assert_eq!(mesh.vertices.len(), 9);
    assert_eq!(mesh.indices.len(), 24);
    assert_eq!(mesh.vertices[4], TileMeshVertex { x: 2048, y: 2048 });
}

#[test]
fn border_adds_stencil_ring() {
    let mesh = create_tile_mesh(
        TileMeshOptions {
            granularity: 1,
            generate_borders: true,
            ..TileMeshOptions::default()
        },
        TileIndexType::Auto,
    )
    .expect("bordered grid is valid");

    assert_eq!(mesh.vertices.len(), 16);
    assert_eq!(mesh.indices.len(), 54);
    assert_eq!(mesh.vertices[0], TileMeshVertex { x: -32, y: -32 });
    assert_eq!(mesh.vertices[15], TileMeshVertex { x: 4128, y: 4128 });
}

#[test]
fn pole_extensions_replace_vertical_borders() {
    let mesh = create_tile_mesh(
        TileMeshOptions {
            granularity: 1,
            generate_borders: true,
            extend_to_north_pole: true,
            extend_to_south_pole: true,
        },
        TileIndexType::Auto,
    )
    .expect("pole grid is valid");

    assert!(mesh.vertices[..4]
        .iter()
        .all(|vertex| vertex.y == NORTH_POLE_Y));
    assert!(mesh.vertices[12..]
        .iter()
        .all(|vertex| vertex.y == SOUTH_POLE_Y));
}

#[test]
fn standalone_pole_extension_adds_only_requested_row() {
    let mesh = create_tile_mesh(
        TileMeshOptions {
            granularity: 2,
            extend_to_north_pole: true,
            ..TileMeshOptions::default()
        },
        TileIndexType::Auto,
    )
    .expect("north-pole grid is valid");

    assert_eq!(mesh.vertices.len(), 12);
    assert_eq!(mesh.indices.len(), 36);
    assert!(mesh.vertices[..3]
        .iter()
        .all(|vertex| vertex.y == NORTH_POLE_Y));
    assert_eq!(mesh.vertices[9].y, 4096);
}

#[test]
fn index_boundary_uses_u16_for_exactly_65536_vertices() {
    let mesh = create_tile_mesh(
        TileMeshOptions {
            granularity: 255,
            ..TileMeshOptions::default()
        },
        TileIndexType::Auto,
    )
    .expect("256 by 256 vertices fit u16 indices");

    assert_eq!(mesh.vertices.len(), 65_536);
    assert!(!mesh.indices.uses_u32());
}

#[test]
fn index_overflow_selects_u32_or_rejects_forced_u16() {
    let options = TileMeshOptions {
        granularity: 256,
        ..TileMeshOptions::default()
    };
    let mesh = create_tile_mesh(options, TileIndexType::Auto)
        .expect("257 by 257 vertices fit u32 indices");
    assert!(mesh.indices.uses_u32());

    let error = create_tile_mesh(options, TileIndexType::U16)
        .expect_err("257 by 257 vertices cannot use u16 indices");
    assert!(matches!(
        error,
        TileMeshError::RequiresU32Indices {
            vertex_count: 66_049
        }
    ));
}

#[test]
fn unsupported_vertex_count_fails_before_allocation() {
    let error = create_tile_mesh(
        TileMeshOptions {
            granularity: 65_536,
            ..TileMeshOptions::default()
        },
        TileIndexType::Auto,
    )
    .expect_err("more than 2^32 vertices exceed supported index widths");

    assert!(matches!(error, TileMeshError::TooManyVertices { .. }));
}

#[test]
fn every_triangle_index_addresses_a_vertex() {
    let mesh = create_tile_mesh(
        TileMeshOptions {
            granularity: 8,
            generate_borders: true,
            extend_to_north_pole: true,
            ..TileMeshOptions::default()
        },
        TileIndexType::Auto,
    )
    .expect("representative globe mesh is valid");

    let TileMeshIndices::U16(indices) = &mesh.indices else {
        panic!("representative mesh should use u16 indices");
    };
    assert!(indices
        .iter()
        .all(|index| usize::from(*index) < mesh.vertices.len()));
}
