use geozero::GeozeroDatasource;

use super::ProjectingTessellator;
use crate::{
    coords::WorldTileCoords,
    vector::tessellation::{IndexDataType, ZeroTessellator},
};

#[test]
fn globe_geojson_world_polygon_generates_subdivided_triangles() {
    let geometry = r#"{
        "type": "Polygon",
        "coordinates": [[
            [-180, -90], [-180, 90], [180, 90], [180, -90], [-180, -90]
        ]]
    }"#;
    let tessellator =
        ZeroTessellator::<IndexDataType>::default().with_globe_subdivision(32, true, true, true);
    let mut projecting = ProjectingTessellator::new(WorldTileCoords::default(), tessellator);
    let mut source = geozero::geojson::GeoJson(geometry);

    source
        .process(&mut projecting)
        .expect("world polygon should tessellate");
    let tessellator = projecting.into_inner();

    assert!(tessellator.buffer.vertices.len() > 4);
    assert!(tessellator.buffer.indices.len() > 6);
}
