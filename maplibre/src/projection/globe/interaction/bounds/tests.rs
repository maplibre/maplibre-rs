#![allow(clippy::panic)]

use cgmath::{Matrix4, Vector3};

use super::{lesser_non_negative, solve_vector_scale, ClipDimension};

#[test]
fn solves_scale_for_projected_x_edge() {
    let projection = Matrix4::new(
        2.0, 0.0, 0.0, 0.0, //
        0.0, 2.0, 0.0, 0.0, //
        0.0, 0.0, 1.0, 1.0, //
        0.0, 0.0, 0.0, 2.0,
    );
    let scale = solve_vector_scale(
        Vector3::new(1.0, 0.0, 1.0),
        Vector3::new(0.0, 0.0, 1.0),
        projection,
        ClipDimension::X,
        0.5,
    );

    assert!(scale.is_some_and(|value| (value - 0.75).abs() < 1e-9));
}

#[test]
fn rejects_degenerate_solution_and_negative_candidate() {
    assert_eq!(
        solve_vector_scale(
            Vector3::unit_x(),
            Vector3::unit_x(),
            Matrix4::from_scale(1.0),
            ClipDimension::Y,
            0.0,
        ),
        None
    );
    assert_eq!(lesser_non_negative(2.0, Some(-1.0)), 2.0);
    assert_eq!(lesser_non_negative(2.0, Some(1.5)), 1.5);
}
