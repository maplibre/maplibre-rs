#![allow(clippy::expect_used, clippy::panic)]

use super::{InterpolationCurve, NamedProjection, ProjectionStop};
use crate::projection::ProjectionType;

fn round_trip(json: &str) -> ProjectionType {
    let projection: ProjectionType =
        serde_json::from_str(json).expect("projection should deserialize");
    let encoded = serde_json::to_string(&projection).expect("projection should serialize");
    serde_json::from_str(&encoded).expect("serialized projection should deserialize")
}

#[test]
fn explicit_transition_interpolates_named_projections() {
    let projection = round_trip(r#"["vertical-perspective","mercator",0.25]"#);

    assert_eq!(projection.globe_transition(0.0), 0.75);
}

#[test]
fn multi_stop_interpolation_selects_the_adjacent_interval() {
    let projection = round_trip(
        r#"["interpolate",["linear"],["zoom"],0,"mercator",2,"vertical-perspective",4,"mercator"]"#,
    );

    assert_eq!(projection.globe_transition(-1.0), 0.0);
    assert_eq!(projection.globe_transition(1.0), 0.5);
    assert_eq!(projection.globe_transition(2.0), 1.0);
    assert_eq!(projection.globe_transition(3.0), 0.5);
    assert_eq!(projection.globe_transition(5.0), 0.0);
}

#[test]
fn step_expression_changes_projection_at_each_stop() {
    let projection = round_trip(
        r#"["step",["zoom"],"vertical-perspective",5,"mercator",7,"vertical-perspective"]"#,
    );

    assert_eq!(projection.globe_transition(4.9), 1.0);
    assert_eq!(projection.globe_transition(5.0), 0.0);
    assert_eq!(projection.globe_transition(7.0), 1.0);
}

#[test]
fn supported_interpolation_curves_are_bounded() {
    for curve in [
        InterpolationCurve::Exponential(2.0),
        InterpolationCurve::CubicBezier(0.42, 0.0, 0.58, 1.0),
    ] {
        let projection = ProjectionType::interpolate(
            curve,
            vec![
                ProjectionStop::new(0.0, NamedProjection::Mercator),
                ProjectionStop::new(1.0, NamedProjection::VerticalPerspective),
            ],
        )
        .expect("curve should be valid");
        let transition = projection.globe_transition(0.5);

        assert!(transition > 0.0 && transition < 1.0);
    }
}

#[test]
fn invalid_programmatic_expressions_are_rejected() {
    assert!(ProjectionType::transition(
        NamedProjection::Mercator,
        NamedProjection::VerticalPerspective,
        f64::NAN,
    )
    .is_err());
    assert!(ProjectionType::interpolate(
        InterpolationCurve::Linear,
        vec![
            ProjectionStop::new(2.0, NamedProjection::Mercator),
            ProjectionStop::new(1.0, NamedProjection::VerticalPerspective),
        ],
    )
    .is_err());
}

#[test]
fn invalid_expression_shapes_are_rejected() {
    for json in [
        r#"["interpolate",["linear"],["pitch"],0,"mercator",1,"vertical-perspective"]"#,
        r#"["step",["zoom"],"mercator"]"#,
        r#"["mercator","vertical-perspective",2]"#,
        r#"["unknown",["zoom"],"mercator",1,"vertical-perspective"]"#,
    ] {
        assert!(
            serde_json::from_str::<ProjectionType>(json).is_err(),
            "{json}"
        );
    }
}
