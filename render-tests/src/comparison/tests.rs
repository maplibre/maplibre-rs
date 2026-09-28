#![allow(clippy::expect_used, clippy::panic)]
use super::*;

#[test]
fn readback_normalization_preserves_alpha_and_recovers_straight_colors() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let path = directory.path().join("frame.png");
    let pixels = [0, 0, 0, 0, 50, 25, 0, 128, 20, 40, 60, 255];
    RgbaImage::from_raw(3, 1, pixels.to_vec())
        .expect("image dimensions")
        .save(&path)
        .expect("write readback");
    unpremultiply(&path).expect("normalize readback");
    assert_eq!(
        image::open(&path)
            .expect("normalized image")
            .to_rgba8()
            .as_raw(),
        &[0, 0, 0, 0, 100, 50, 0, 128, 20, 40, 60, 255]
    );
    composite_opaque_background(&path, [255, 255, 255]).expect("composite on white");
    assert_eq!(
        image::open(&path)
            .expect("composited image")
            .to_rgba8()
            .as_raw(),
        &[255, 255, 255, 255, 177, 152, 127, 255, 20, 40, 60, 255]
    );
}

#[test]
fn image_comparison_measures_every_channel_and_marks_changed_pixels() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let diff = directory.path().join("diff.png");
    let expected = RgbaImage::from_pixel(2, 1, Rgba([20, 40, 60, 255]));
    let mut actual = expected.clone();
    actual.put_pixel(1, 0, Rgba([30, 20, 60, 250]));
    let difference = compare_equal_dimensions(&actual, &expected, &diff).expect("compare");
    assert!((difference - 35.0 / (8.0 * 255.0)).abs() < f64::EPSILON);
    let image = image::open(diff).expect("diff image").to_rgba8();
    assert_eq!(image.get_pixel(0, 0).0, [0, 0, 0, 0]);
    assert_eq!(image.get_pixel(1, 0).0, [255, 0, 0, 20]);
}

#[tokio::test(flavor = "multi_thread")]
async fn translucent_fill_preserves_color_and_alpha_in_gpu_readback() {
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/tests/fill-color/opacity");
    let directory = tempfile::tempdir().expect("temporary render fixture");
    let mut style: serde_json::Value =
        serde_json::from_slice(&std::fs::read(fixture.join("style.json")).expect("fixture style"))
            .expect("valid style");
    let paint = &mut style["layers"][0]["paint"];
    paint["fill-color"] = serde_json::json!("rgba(128,0,0,0.5)");
    paint
        .as_object_mut()
        .expect("paint object")
        .remove("fill-opacity");
    std::fs::write(
        directory.path().join("style.json"),
        serde_json::to_vec(&style).expect("serialize style"),
    )
    .expect("write style");
    let mut expected = image::open(fixture.join("expected.png"))
        .expect("fixture image")
        .to_rgba8();
    for pixel in expected.pixels_mut().filter(|pixel| pixel.0[3] > 0) {
        pixel.0[0] = 128;
    }
    expected
        .save(directory.path().join("expected.png"))
        .expect("write expected");

    let outcome = crate::run_test_inner(directory.path()).await;
    assert!(
        matches!(outcome, crate::TestResult::Pass { .. }),
        "{outcome:?}"
    );
    let actual = image::open(directory.path().join("actual.png"))
        .expect("rendered image")
        .to_rgba8();
    let pixel = actual.get_pixel(32, 32).0;
    for (actual, expected) in pixel.into_iter().zip([128, 0, 0, 128]) {
        assert!(actual.abs_diff(expected) <= 1, "center pixel: {pixel:?}");
    }
}
