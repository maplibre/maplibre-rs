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
    time::Instant,
};

use image::{ImageBuffer, Rgba, RgbaImage};
use maplibre::{
    coords::{WorldTileCoords, ZoomLevel},
    headless::{create_headless_renderer, map::HeadlessMap, HeadlessPlugin},
    platform::run_multithreaded,
    plugin::Plugin,
    render::RenderPlugin,
    style::{
        layer::StyleLayer,
        source::{GeoJsonData, Source},
        Style,
    },
    vector::{DefaultVectorTransferables, VectorPlugin},
};
use serde_json::Value;

// ---------------------------------------------------------------------------
// Paths — all relative to the workspace root (where `cargo run` is executed)
// ---------------------------------------------------------------------------

fn workspace_tests_dir() -> PathBuf {
    PathBuf::from("render-tests/src/tests")
}

fn workspace_templates_dir() -> PathBuf {
    PathBuf::from("render-tests/src/templates")
}

// ---------------------------------------------------------------------------
// Test metadata
// ---------------------------------------------------------------------------

#[derive(Debug)]
struct TestMeta {
    width: u32,
    height: u32,
}

impl Default for TestMeta {
    fn default() -> Self {
        Self {
            width: 512,
            height: 512,
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
    }
}

// ---------------------------------------------------------------------------
// Single test
// ---------------------------------------------------------------------------

#[derive(Debug)]
struct TestOutcome {
    id: String,
    result: TestResult,
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

    let result = run_test_inner(&test_dir).await;
    TestOutcome { id, result }
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
    let (kernel, renderer) = create_headless_renderer(meta.width, meta.height, None).await;

    let plugins: Vec<Box<dyn Plugin<_>>> = vec![
        Box::new(RenderPlugin::default()),
        Box::new(maplibre::background::BackgroundPlugin::default()),
        Box::new(VectorPlugin::<DefaultVectorTransferables>::default()),
        Box::new(HeadlessPlugin::new(true)),
    ];

    let mut map = match HeadlessMap::new(style.clone(), renderer, kernel, plugins) {
        Ok(m) => m,
        Err(e) => return TestResult::Error(format!("HeadlessMap creation failed: {e:?}")),
    };

    // ---- Process GeoJSON sources ----
    let target_coords = WorldTileCoords::from((0, 0, ZoomLevel::default()));
    let mut all_layers = Vec::new();

    for (source_name, source) in &style.sources {
        let Source::GeoJson(geojson_source) = source else {
            continue;
        };

        let geojson_value = match &geojson_source.data {
            GeoJsonData::Inline(v) => v.clone(),
            GeoJsonData::Url(_url) => {
                log::warn!(
                    "URL GeoJSON source '{}' not supported in test harness",
                    source_name
                );
                continue;
            }
        };

        let matching_layers: Vec<StyleLayer> = style
            .layers
            .iter()
            .filter(|l| l.source.as_deref() == Some(source_name.as_str()))
            .cloned()
            .collect();

        if matching_layers.is_empty() {
            continue;
        }

        let mut layers = map.process_geojson(
            &geojson_value,
            source_name,
            matching_layers,
            target_coords,
            false,
        );
        all_layers.append(&mut layers);
    }

    // ---- Render ----
    map.render_tile(all_layers);

    // HeadlessPlugin writes "frame_0.png" to the CWD (workspace root).
    let frame_path = PathBuf::from("frame_0.png");
    if !frame_path.exists() {
        return TestResult::Error("Renderer did not produce frame_0.png".to_string());
    }
    if let Err(e) = std::fs::rename(&frame_path, &actual_path) {
        let _ = std::fs::copy(&frame_path, &actual_path);
        let _ = std::fs::remove_file(&frame_path);
        let _ = e;
    }

    // ---- Compare with expected.png ----
    if !expected_path.exists() {
        return TestResult::Error(format!(
            "expected.png not found: {}",
            expected_path.display()
        ));
    }

    match compare_and_diff(&actual_path, &expected_path, &diff_path) {
        Ok(diff) if diff < 0.02 => TestResult::Pass { diff },
        Ok(diff) => TestResult::Fail { diff },
        Err(e) => TestResult::Error(format!("Image comparison failed: {e}")),
    }
}

/// Compare two images, write a diff PNG, and return the normalised mean diff in [0,1].
fn compare_and_diff(
    actual_path: &Path,
    expected_path: &Path,
    diff_path: &Path,
) -> Result<f64, String> {
    let actual = image::open(actual_path)
        .map_err(|e| format!("Cannot open actual: {e}"))?
        .to_rgba8();
    let expected = image::open(expected_path)
        .map_err(|e| format!("Cannot open expected: {e}"))?
        .to_rgba8();

    // If dimensions differ, produce a plain red diff and report max diff.
    if actual.dimensions() != expected.dimensions() {
        let (aw, ah) = actual.dimensions();
        let (ew, eh) = expected.dimensions();
        let diff_img: RgbaImage =
            ImageBuffer::from_pixel(aw.max(ew), ah.max(eh), Rgba([255u8, 0, 0, 255]));
        let _ = diff_img.save(diff_path);
        return Err(format!(
            "Dimension mismatch: actual {aw}x{ah} vs expected {ew}x{eh}"
        ));
    }

    let (w, h) = actual.dimensions();
    let mut diff_img: RgbaImage = ImageBuffer::new(w, h);
    let mut total_diff: u64 = 0;

    for (x, y, a_px) in actual.enumerate_pixels() {
        let e_px = expected.get_pixel(x, y);
        let channel_diffs: Vec<u8> = a_px
            .0
            .iter()
            .zip(e_px.0.iter())
            .map(|(a, e)| (*a as i32 - *e as i32).unsigned_abs() as u8)
            .collect();

        let max_ch = *channel_diffs.iter().max().unwrap_or(&0);
        total_diff += channel_diffs.iter().map(|&d| d as u64).sum::<u64>();

        // Diff pixel: red tint proportional to difference, green for same pixels
        let diff_px = if max_ch == 0 {
            Rgba([0u8, 0, 0, 0]) // transparent (same)
        } else {
            Rgba([255u8, 0, 0, max_ch])
        };
        diff_img.put_pixel(x, y, diff_px);
    }

    let _ = diff_img.save(diff_path);

    let n = w as u64 * h as u64 * 4;
    Ok(total_diff as f64 / (n as f64 * 255.0))
}

// ---------------------------------------------------------------------------
// Test discovery
// ---------------------------------------------------------------------------

fn collect_tests(test_root: &Path) -> Vec<PathBuf> {
    let mut tests = Vec::new();

    // Walk `test_root` up to `max_depth` to find directories containing `style.json`
    for entry in walkdir::WalkDir::new(test_root)
        .min_depth(1)
        .max_depth(5)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if entry.file_name() == "style.json" {
            if let Some(parent) = entry.path().parent() {
                // Ignore the `projection` tests because Maplibre-RS does not yet support Globe projection fully,
                // and the NaN coordinate transformations crash `lyon_path` during full test runs.
                if !parent.components().any(|c| c.as_os_str() == "projection") {
                    tests.push(parent.to_path_buf());
                }
            }
        }
    }

    tests.sort();
    tests
}

// ---------------------------------------------------------------------------
// HTML report generation
// ---------------------------------------------------------------------------

fn generate_report(outcomes: &[TestOutcome], templates_dir: &Path) {
    let report_template_path = templates_dir.join("report_template.html");
    let item_template_path = templates_dir.join("result_item_template.html");
    let results_html_path = templates_dir.join("results.html");
    let failed_ids_path = templates_dir.join("results-failed-caseIds.txt");
    let errored_ids_path = templates_dir.join("results-errored-caseIds.txt");

    let report_template = std::fs::read_to_string(&report_template_path)
        .unwrap_or_else(|_| "<html><body>${resultData}</body></html>".to_string());
    let item_template = std::fs::read_to_string(&item_template_path).unwrap_or_default();

    let mut failed_items = String::new();
    let mut errored_items = String::new();
    let mut failed_ids = Vec::new();
    let mut errored_ids = Vec::new();

    for outcome in outcomes {
        let test_id = format!("tests/{}", outcome.id);

        // Relative image paths from templates/ to tests/
        let rel_actual = format!("../tests/{}/actual.png", outcome.id);
        let rel_expected = format!("../tests/{}/expected.png", outcome.id);
        let rel_diff = format!("../tests/{}/diff.png", outcome.id);

        let item_html = format!(
            r#"<div class="test">
  <h2>{id}</h2>
  <div class="imagewrap">
    <div><p>Actual</p><img src="{actual}" data-alt-src="{expected}"></div>
    <div><p>Expected</p><img src="{expected}"></div>
    <div class="diff"><p>Diff</p><img src="{diff}"></div>
  </div>
  <p>{status}</p>
</div>"#,
            id = outcome.id,
            actual = rel_actual,
            expected = rel_expected,
            diff = rel_diff,
            status = match &outcome.result {
                TestResult::Pass { diff } => format!("PASS (diff={diff:.4})"),
                TestResult::Fail { diff } => format!("FAIL (diff={diff:.4})"),
                TestResult::Error(msg) => format!("ERROR: {msg}"),
            },
        );

        match &outcome.result {
            TestResult::Fail { .. } => {
                failed_items.push_str(&item_html);
                failed_ids.push(test_id);
            }
            TestResult::Error(_) => {
                errored_items.push_str(&item_html);
                errored_ids.push(test_id);
            }
            TestResult::Pass { .. } => {}
        }
    }

    // Fill in item template
    let result_data = item_template
        .replace("${failedItemsLength}", &failed_ids.len().to_string())
        .replace("${failedItems}", &failed_items)
        .replace("${erroredItemsLength}", &errored_ids.len().to_string())
        .replace("${erroredItems}", &errored_items);

    // Fill in report template
    let passed = outcomes
        .iter()
        .filter(|o| matches!(o.result, TestResult::Pass { .. }))
        .count();
    let all_passed_banner = if failed_ids.is_empty() && errored_ids.is_empty() {
        format!(
            r#"<h1 style="color: green">All {} tests passed!</h1>"#,
            passed
        )
    } else {
        format!(
            r#"<p class="stats">{} passed / {} failed / {} errored out of {} total</p>"#,
            passed,
            failed_ids.len(),
            errored_ids.len(),
            outcomes.len()
        )
    };

    let full_html = if result_data.is_empty() {
        report_template.replace("${resultData}", &all_passed_banner)
    } else {
        report_template.replace(
            "${resultData}",
            &format!("{all_passed_banner}\n{result_data}"),
        )
    };

    let _ = std::fs::write(&results_html_path, &full_html);
    let _ = std::fs::write(&failed_ids_path, failed_ids.join("\n"));
    let _ = std::fs::write(&errored_ids_path, errored_ids.join("\n"));

    println!("\nReport written to: {}", results_html_path.display());
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

fn main() {
    env_logger::init();
    let args: Vec<String> = std::env::args().collect();

    let test_root = if args.len() > 1 {
        PathBuf::from(&args[1])
    } else {
        workspace_tests_dir()
    };

    if !test_root.exists() {
        eprintln!(
            "Test directory not found: {}\nUsage: cargo run -p render-tests [test-dir]",
            test_root.display()
        );
        std::process::exit(1);
    }

    let tests = collect_tests(&test_root);
    if tests.is_empty() {
        eprintln!("No tests found in {}", test_root.display());
        std::process::exit(1);
    }

    println!(
        "Running {} render tests from {}",
        tests.len(),
        test_root.display()
    );
    println!("{:-<70}", "");

    let mut outcomes: Vec<TestOutcome> = Vec::new();

    for test_dir in &tests {
        let name = test_dir
            .strip_prefix(&test_root)
            .unwrap_or(test_dir)
            .display()
            .to_string();

        let start = Instant::now();
        let outcome = run_multithreaded(run_test(test_dir.clone()));
        let elapsed = start.elapsed();

        let tag = match &outcome.result {
            TestResult::Pass { diff } => format!("PASS  (diff={diff:.4})"),
            TestResult::Fail { diff } => format!("FAIL  (diff={diff:.4})"),
            TestResult::Error(msg) => format!("ERR   {msg}"),
        };

        println!("  {tag}  {name}  ({elapsed:.1?})");

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

    println!("{:-<70}", "");
    println!(
        "Results: {} passed, {} failed, {} errors  (total {})",
        passed,
        failed,
        errored,
        outcomes.len()
    );

    generate_report(&outcomes, &workspace_templates_dir());

    if failed > 0 || errored > 0 {
        std::process::exit(1);
    }
}
