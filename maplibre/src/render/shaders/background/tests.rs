use cgmath::Point2;

use super::AtmosphereLayerMetadata;
use crate::{
    coords::LatLon,
    projection::globe::camera::{GlobeCameraOptions, GlobeCameraState},
    style::light::LightSpecification,
};

fn camera() -> GlobeCameraState {
    GlobeCameraState::new(GlobeCameraOptions {
        width: 512.0,
        height: 512.0,
        field_of_view_degrees: 45.0,
        center: LatLon::new(0.0, 160.0),
        world_size: 1024.0,
        bearing_degrees: 20.0,
        pitch_degrees: 30.0,
        roll_degrees: 0.0,
        center_offset: Point2::new(0.0, 0.0),
    })
    .expect("camera should be valid")
}

#[test]
fn atmosphere_metadata_matches_gpu_vertex_layout() {
    assert_eq!(std::mem::size_of::<AtmosphereLayerMetadata>(), 112);

    let light: LightSpecification =
        serde_json::from_str(r#"{"anchor":"map","position":[1.5,90,90]}"#)
            .expect("light should parse");
    let metadata = AtmosphereLayerMetadata::from_view(&camera(), &light, 1.0, 0.75)
        .expect("metadata should be valid");

    assert_eq!(metadata.radius_blend_padding[1], 0.75);
    assert!(metadata.radius_blend_padding[0] > 0.0);
    assert!(metadata.globe_position[..3]
        .iter()
        .all(|value| value.is_finite()));
    assert!(metadata.sun_direction[..3]
        .iter()
        .all(|value| value.is_finite()));
    assert!(metadata
        .inverse_projection
        .iter()
        .flatten()
        .all(|value| value.is_finite()));
}

#[test]
fn disabled_metadata_cannot_contribute_color() {
    let metadata = AtmosphereLayerMetadata::disabled();
    assert_eq!(metadata.radius_blend_padding, [1.0, 0.0, 0.0, 0.0]);
}
