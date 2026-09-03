//! Image normalization and comparison for render tests.

use std::path::Path;

use image::{ImageBuffer, Rgba, RgbaImage};

pub(super) fn composite_opaque_background(path: &Path, background: [u8; 3]) -> Result<(), String> {
    let mut image = image::open(path)
        .map_err(|error| format!("Cannot open actual for compositing: {error}"))?
        .to_rgba8();
    for pixel in image.pixels_mut() {
        let alpha = u16::from(pixel.0[3]);
        let inverse_alpha = 255 - alpha;
        for (channel, background_channel) in pixel.0[..3].iter_mut().zip(background) {
            let blended =
                u16::from(*channel) * alpha + u16::from(background_channel) * inverse_alpha;
            *channel = (blended / 255) as u8;
        }
        pixel.0[3] = 255;
    }
    image
        .save(path)
        .map_err(|error| format!("Cannot save composited actual: {error}"))
}

/// Writes a diff PNG and returns normalized mean channel difference in `[0, 1]`.
pub(super) fn compare_and_diff(
    actual_path: &Path,
    expected_path: &Path,
    diff_path: &Path,
) -> Result<f64, String> {
    let actual = image::open(actual_path)
        .map_err(|error| format!("Cannot open actual: {error}"))?
        .to_rgba8();
    let expected = image::open(expected_path)
        .map_err(|error| format!("Cannot open expected: {error}"))?
        .to_rgba8();
    if actual.dimensions() != expected.dimensions() {
        let (actual_width, actual_height) = actual.dimensions();
        let (expected_width, expected_height) = expected.dimensions();
        let diff = ImageBuffer::from_pixel(
            actual_width.max(expected_width),
            actual_height.max(expected_height),
            Rgba([255_u8, 0, 0, 255]),
        );
        diff.save(diff_path)
            .map_err(|error| format!("Cannot save dimension-mismatch diff: {error}"))?;
        return Err(format!(
            "Dimension mismatch: actual {actual_width}x{actual_height} vs expected {expected_width}x{expected_height}"
        ));
    }
    compare_equal_dimensions(&actual, &expected, diff_path)
}

fn compare_equal_dimensions(
    actual: &RgbaImage,
    expected: &RgbaImage,
    diff_path: &Path,
) -> Result<f64, String> {
    let (width, height) = actual.dimensions();
    let mut diff_image = RgbaImage::new(width, height);
    let mut total_diff = 0_u64;
    for (x, y, actual_pixel) in actual.enumerate_pixels() {
        let expected_pixel = expected.get_pixel(x, y);
        let channel_diffs = actual_pixel
            .0
            .iter()
            .zip(expected_pixel.0.iter())
            .map(|(actual, expected)| (*actual as i32 - *expected as i32).unsigned_abs() as u8)
            .collect::<Vec<_>>();
        let max_channel = channel_diffs.iter().copied().max().unwrap_or(0);
        total_diff += channel_diffs
            .iter()
            .map(|difference| u64::from(*difference))
            .sum::<u64>();
        let diff_pixel = if max_channel == 0 {
            Rgba([0_u8, 0, 0, 0])
        } else {
            Rgba([255_u8, 0, 0, max_channel])
        };
        diff_image.put_pixel(x, y, diff_pixel);
    }
    diff_image
        .save(diff_path)
        .map_err(|error| format!("Cannot save image diff: {error}"))?;
    let channel_count = u64::from(width) * u64::from(height) * 4;
    Ok(total_diff as f64 / (channel_count as f64 * 255.0))
}
