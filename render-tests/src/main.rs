//! Render test harness for maplibre-rs.
//!
//! Runs render tests from `render-tests/src/tests/`, compares against
//! `expected.png`, writes `actual.png` and `diff.png`, and generates
//! `render-tests/src/templates/results.html`.
//!
//! # Usage
//!
//! ```
//! # Run all tests (from workspace root)
//! cargo run -p render-tests
//!
//! # Run a single test or category
//! cargo run -p render-tests -- render-tests/src/tests/fill-color
//! ```

use std::{
    path::{Path, PathBuf},
    process::ExitCode,
    time::Instant,
};

use maplibre::{
    headless::{create_headless_renderer, map::HeadlessMap, HeadlessPlugin},
    platform::run_multithreaded,
    plugin::Plugin,
    raster::{AvailableRasterLayerData, DefaultRasterTransferables, RasterPlugin},
    render::RenderPlugin,
    style::{
        layer::StyleLayer,
        source::{GeoJsonData, Source},
        Style,
    },
    vector::{DefaultVectorTransferables, VectorPlugin},
};
use serde_json::Value;

mod comparison;
mod paths;
mod report;
mod source_tiles;

use comparison::{compare_and_diff, composite_opaque_background};
use paths::{
    collect_tests, local_data_path, local_tile_path, workspace_templates_dir, workspace_tests_dir,
};
use report::generate_report;
use source_tiles::source_tile_coords;

// ---------------------------------------------------------------------------
// Test metadata
// ---------------------------------------------------------------------------

#[derive(Debug)]
struct TestMeta {
    width: u32,
    height: u32,
    comparison_background: Option<[u8; 3]>,
    max_diff: f64,
}

impl Default for TestMeta {
    fn default() -> Self {
        Self {
            width: 512,
            height: 512,
            comparison_background: None,
            max_diff: 0.02,
        }
    }
}

fn parse_test_meta(style_value: &Value) -> TestMeta {
    let test = style_value
        .pointer("/metadata/test")
        .and_then(|v| v.as_object());

    let Some(test) = test else {
        return TestMeta::default();
    };

    TestMeta {
        width: test.get("width").and_then(|v| v.as_u64()).unwrap_or(512) as u32,
        height: test.get("height").and_then(|v| v.as_u64()).unwrap_or(512) as u32,
        comparison_background: test
            .get("comparison-background")
            .and_then(Value::as_array)
            .and_then(|channels| match channels.as_slice() {
                [red, green, blue] => Some([
                    u8::try_from(red.as_u64()?).ok()?,
                    u8::try_from(green.as_u64()?).ok()?,
                    u8::try_from(blue.as_u64()?).ok()?,
                ]),
                _ => None,
            }),
        max_diff: test.get("max-diff").and_then(Value::as_f64).unwrap_or(0.02),
    }
}

// ---------------------------------------------------------------------------
// Single test
// ---------------------------------------------------------------------------

#[derive(Debug)]
struct TestOutcome {
    id: String,
    result: TestResult,
    attempts: u8,
}

#[derive(Debug)]
enum TestResult {
    Pass { diff: f64 },
    Fail { diff: f64 },
    Error(String),
}

/// Run one test in `test_dir`. Writes `actual.png` and `diff.png` into `test_dir`.
async fn run_test(test_dir: PathBuf) -> TestOutcome {
    let id = test_dir
        .iter()
        .rev()
        .take(2)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect::<PathBuf>()
        .to_string_lossy()
        .into_owned();

    let mut result = run_test_inner(&test_dir).await;
    let mut attempts = 1_u8;
    while matches!(result, TestResult::Fail { .. }) && attempts < 3 {
        attempts = attempts.wrapping_add(1);
        result = run_test_inner(&test_dir).await;
    }
    TestOutcome {
        id,
        result,
        attempts,
    }
}

async fn run_test_inner(test_dir: &Path) -> TestResult {
    let style_path = test_dir.join("style.json");
    let expected_path = test_dir.join("expected.png");
    let actual_path = test_dir.join("actual.png");
    let diff_path = test_dir.join("diff.png");

    // ---- Load & parse style.json ----
    let style_str = match std::fs::read_to_string(&style_path) {
        Ok(s) => s,
        Err(e) => return TestResult::Error(format!("Cannot read style.json: {e}")),
    };

    let style_value: Value = match serde_json::from_str(&style_str) {
        Ok(v) => v,
        Err(e) => return TestResult::Error(format!("Cannot parse style.json: {e}")),
    };

    let meta = parse_test_meta(&style_value);

    let mut style: Style = match serde_json::from_value(style_value) {
        Ok(s) => s,
        Err(e) => return TestResult::Error(format!("Cannot deserialize Style: {e}")),
    };

    for (i, layer) in style.layers.iter_mut().enumerate() {
        layer.index = i as u32 + 1; // Start at 1 to be > 0.0 depth clear
    }

    // ---- Set up headless renderer ----
    let (kernel, renderer) = match create_headless_renderer(meta.width, meta.height, None).await {
        Ok(renderer) => renderer,
        Err(error) => {
            return TestResult::Error(format!("Cannot create headless renderer: {error}"));
        }
    };

    let has_vector_sources = style
        .sources
        .values()
        .any(|source| matches!(source, Source::GeoJson(_) | Source::Vector(_)));
    let has_raster_sources = style
        .sources
        .values()
        .any(|source| matches!(source, Source::Raster(_)));
    let mut plugins: Vec<Box<dyn Plugin<_>>> = vec![
        Box::new(RenderPlugin),
        Box::new(maplibre::background::BackgroundPlugin),
    ];
    if has_vector_sources {
        plugins.push(Box::new(
            VectorPlugin::<DefaultVectorTransferables>::default(),
        ));
    }
    if has_raster_sources {
        plugins.push(Box::new(
            RasterPlugin::<DefaultRasterTransferables>::default(),
        ));
    }
    plugins.push(Box::new(HeadlessPlugin::new(true).preserve_tile_sources()));

    let mut map = match HeadlessMap::new(style.clone(), renderer, kernel, plugins) {
        Ok(m) => m,
        Err(e) => return TestResult::Error(format!("HeadlessMap creation failed: {e:?}")),
    };

    // ---- Process GeoJSON sources ----
    let target_coords = match map.required_tile_coords() {
        Ok(coords) => coords,
        Err(error) => {
            return TestResult::Error(format!("Cannot select source tiles: {error}"));
        }
    };
    let mut all_layers = Vec::new();
    let mut all_raster_layers = Vec::new();

    let projection = style
        .projection
        .as_ref()
        .map_or_else(crate_projection_default, |specification| {
            specification.projection_type.clone()
        });
    for (source_name, source) in &style.sources {
        let matching_layers: Vec<StyleLayer> = style
            .layers
            .iter()
            .filter(|l| l.source.as_deref() == Some(source_name.as_str()))
            .cloned()
            .collect();

        if matching_layers.is_empty() {
            continue;
        }

        match source {
            Source::GeoJson(geojson_source) => {
                let loaded;
                let geojson_value = match &geojson_source.data {
                    GeoJsonData::Inline(value) => value,
                    GeoJsonData::Url(url) => {
                        let path = match local_data_path(url) {
                            Ok(path) => path,
                            Err(error) => return TestResult::Error(error),
                        };
                        let text = match std::fs::read_to_string(&path) {
                            Ok(text) => text,
                            Err(error) => {
                                return TestResult::Error(format!(
                                    "Cannot read GeoJSON {}: {error}",
                                    path.display()
                                ));
                            }
                        };
                        loaded = match serde_json::from_str::<Value>(&text) {
                            Ok(value) => value,
                            Err(error) => {
                                return TestResult::Error(format!(
                                    "Cannot parse GeoJSON {}: {error}",
                                    path.display()
                                ));
                            }
                        };
                        &loaded
                    }
                };
                for coords in &target_coords {
                    let mut layers = match map.process_geojson(
                        geojson_value,
                        source_name,
                        matching_layers.clone(),
                        *coords,
                        projection.clone(),
                    ) {
                        Ok(layers) => layers,
                        Err(error) => {
                            return TestResult::Error(format!(
                                "Cannot process GeoJSON source '{source_name}': {error}"
                            ));
                        }
                    };
                    all_layers.append(&mut layers);
                }
            }
            Source::Vector(vector_source) => {
                let Some(template) = vector_source
                    .tiles
                    .as_ref()
                    .and_then(|templates| templates.first())
                else {
                    return TestResult::Error(format!(
                        "Vector source '{source_name}' has no tile template"
                    ));
                };
                for coords in
                    source_tile_coords(&target_coords, vector_source.minzoom, vector_source.maxzoom)
                {
                    let path = match local_tile_path(template, coords) {
                        Ok(path) => path,
                        Err(error) => return TestResult::Error(error),
                    };
                    let tile_data = match std::fs::read(&path) {
                        Ok(data) => data.into_boxed_slice(),
                        Err(error) => {
                            return TestResult::Error(format!(
                                "Cannot read vector tile {}: {error}",
                                path.display()
                            ));
                        }
                    };
                    for layer in &matching_layers {
                        let mut layers = match map.process_tile_at(
                            tile_data.clone(),
                            layer,
                            coords,
                            projection.clone(),
                        ) {
                            Ok(layers) => layers,
                            Err(error) => {
                                return TestResult::Error(format!(
                                    "Cannot process vector source '{source_name}': {error}"
                                ));
                            }
                        };
                        all_layers.append(&mut layers);
                    }
                }
            }
            Source::Raster(raster_source) => {
                let Some(template) = raster_source
                    .tiles
                    .as_ref()
                    .and_then(|templates| templates.first())
                else {
                    return TestResult::Error(format!(
                        "Raster source '{source_name}' has no tile template"
                    ));
                };
                for coords in
                    source_tile_coords(&target_coords, raster_source.minzoom, raster_source.maxzoom)
                {
                    let path = match local_tile_path(template, coords) {
                        Ok(path) => path,
                        Err(error) => return TestResult::Error(error),
                    };
                    let image = match image::open(&path) {
                        Ok(image) => image.to_rgba8(),
                        Err(error) => {
                            return TestResult::Error(format!(
                                "Cannot read raster tile {}: {error}",
                                path.display()
                            ));
                        }
                    };
                    all_raster_layers.push(AvailableRasterLayerData {
                        coords,
                        source_layer: source_name.clone(),
                        image,
                    });
                }
            }
        }
    }

    // ---- Render ----
    let frame_paths = [PathBuf::from("frame_0.png"), PathBuf::from("frame_1.png")];
    for frame_path in &frame_paths {
        match std::fs::remove_file(frame_path) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => {
                return TestResult::Error(format!("Cannot remove stale headless frame: {error}"));
            }
        }
    }
    if let Err(error) = map.render_source_frames(all_layers, all_raster_layers, 2) {
        return TestResult::Error(format!("Cannot render source tiles: {error}"));
    }

    let frame_path = &frame_paths[1];
    if !frame_path.exists() {
        return TestResult::Error("Renderer did not produce the requested final frame".to_string());
    }
    if let Err(error) = std::fs::rename(frame_path, &actual_path) {
        return TestResult::Error(format!(
            "Cannot move rendered frame into test output: {error}"
        ));
    }
    if let Some(background) = meta.comparison_background {
        if let Err(error) = composite_opaque_background(&actual_path, background) {
            return TestResult::Error(error);
        }
    }

    // ---- Compare with expected.png ----
    if !expected_path.exists() {
        return TestResult::Error(format!(
            "expected.png not found: {}",
            expected_path.display()
        ));
    }

    match compare_and_diff(&actual_path, &expected_path, &diff_path) {
        Ok(diff) if diff < meta.max_diff => TestResult::Pass { diff },
        Ok(diff) => TestResult::Fail { diff },
        Err(e) => TestResult::Error(format!("Image comparison failed: {e}")),
    }
}

fn crate_projection_default() -> maplibre::projection::ProjectionType {
    maplibre::projection::ProjectionType::default()
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

fn run() -> Result<bool, String> {
    let args: Vec<String> = std::env::args().collect();

    let test_root = if args.len() > 1 {
        PathBuf::from(&args[1])
    } else {
        workspace_tests_dir()
    };

    if !test_root.exists() {
        return Err(format!(
            "Test directory not found: {}. Usage: cargo run -p render-tests [test-dir]",
            test_root.display()
        ));
    }

    let tests = collect_tests(&test_root);
    if tests.is_empty() {
        return Err(format!("No tests found in {}", test_root.display()));
    }

    run_multithreaded(run_tests(test_root, tests))
}

async fn run_tests(test_root: PathBuf, tests: Vec<PathBuf>) -> Result<bool, String> {
    tracing::info!(
        "Running {} render tests from {}",
        tests.len(),
        test_root.display()
    );
    tracing::info!("{}", "-".repeat(70));

    let mut outcomes: Vec<TestOutcome> = Vec::new();

    for test_dir in &tests {
        let name = test_dir
            .strip_prefix(&test_root)
            .unwrap_or(test_dir)
            .display()
            .to_string();

        let start = Instant::now();
        let outcome = run_test(test_dir.clone()).await;
        let elapsed = start.elapsed();

        let tag = match &outcome.result {
            TestResult::Pass { diff } => format!("PASS  (diff={diff:.4})"),
            TestResult::Fail { diff } => format!("FAIL  (diff={diff:.4})"),
            TestResult::Error(msg) => format!("ERR   {msg}"),
        };
        let retry_label = if outcome.attempts > 1 {
            format!(" [attempt {}]", outcome.attempts)
        } else {
            String::new()
        };

        tracing::info!("  {tag}  {name}{retry_label}  ({elapsed:.1?})");

        outcomes.push(outcome);
    }

    let passed = outcomes
        .iter()
        .filter(|o| matches!(o.result, TestResult::Pass { .. }))
        .count();
    let failed = outcomes
        .iter()
        .filter(|o| matches!(o.result, TestResult::Fail { .. }))
        .count();
    let errored = outcomes
        .iter()
        .filter(|o| matches!(o.result, TestResult::Error(_)))
        .count();

    tracing::info!("{}", "-".repeat(70));
    tracing::info!(
        "Results: {} passed, {} failed, {} errors  (total {})",
        passed,
        failed,
        errored,
        outcomes.len()
    );

    let report_path = generate_report(&outcomes, &workspace_templates_dir())?;
    tracing::info!("Report written to: {}", report_path.display());

    Ok(failed == 0 && errored == 0)
}

fn main() -> ExitCode {
    tracing_subscriber::fmt::init();
    match run() {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => ExitCode::FAILURE,
        Err(error) => {
            tracing::error!("{error}");
            ExitCode::FAILURE
        }
    }
}
