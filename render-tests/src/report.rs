//! HTML and case-list report output.

use std::path::{Path, PathBuf};

use super::{TestOutcome, TestResult};

pub(super) fn generate_report(
    outcomes: &[TestOutcome],
    templates_dir: &Path,
) -> Result<PathBuf, String> {
    let report_template_path = templates_dir.join("report_template.html");
    let item_template_path = templates_dir.join("result_item_template.html");
    let results_path = templates_dir.join("results.html");
    let failed_ids_path = templates_dir.join("results-failed-caseIds.txt");
    let errored_ids_path = templates_dir.join("results-errored-caseIds.txt");
    let report_template = std::fs::read_to_string(report_template_path)
        .unwrap_or_else(|_| "<html><body>${resultData}</body></html>".to_string());
    let item_template = std::fs::read_to_string(item_template_path).unwrap_or_default();
    let (failed_items, errored_items, failed_ids, errored_ids) = report_items(outcomes);
    let result_data = item_template
        .replace("${failedItemsLength}", &failed_ids.len().to_string())
        .replace("${failedItems}", &failed_items)
        .replace("${erroredItemsLength}", &errored_ids.len().to_string())
        .replace("${erroredItems}", &errored_items);
    let banner = report_banner(outcomes, failed_ids.len(), errored_ids.len());
    let full_html = if result_data.is_empty() {
        report_template.replace("${resultData}", &banner)
    } else {
        report_template.replace("${resultData}", &format!("{banner}\n{result_data}"))
    };
    std::fs::write(&results_path, full_html)
        .map_err(|error| format!("Cannot write {}: {error}", results_path.display()))?;
    std::fs::write(&failed_ids_path, failed_ids.join("\n"))
        .map_err(|error| format!("Cannot write {}: {error}", failed_ids_path.display()))?;
    std::fs::write(&errored_ids_path, errored_ids.join("\n"))
        .map_err(|error| format!("Cannot write {}: {error}", errored_ids_path.display()))?;
    Ok(results_path)
}

fn report_items(outcomes: &[TestOutcome]) -> (String, String, Vec<String>, Vec<String>) {
    let mut failed_items = String::new();
    let mut errored_items = String::new();
    let mut failed_ids = Vec::new();
    let mut errored_ids = Vec::new();
    for outcome in outcomes {
        let item = report_item(outcome);
        let test_id = format!("tests/{}", outcome.id);
        match outcome.result {
            TestResult::Fail { .. } => {
                failed_items.push_str(&item);
                failed_ids.push(test_id);
            }
            TestResult::Error(_) => {
                errored_items.push_str(&item);
                errored_ids.push(test_id);
            }
            TestResult::Pass { .. } => {}
        }
    }
    (failed_items, errored_items, failed_ids, errored_ids)
}

fn report_item(outcome: &TestOutcome) -> String {
    let expected = format!("../tests/{}/expected.png", outcome.id);
    let status = match &outcome.result {
        TestResult::Pass { diff } => format!("PASS (diff={diff:.4})"),
        TestResult::Fail { diff } => format!("FAIL (diff={diff:.4})"),
        TestResult::Error(message) => format!("ERROR: {message}"),
    };
    format!(
        r#"<div class="test">
  <h2>{id}</h2>
  <div class="imagewrap">
    <div><p>Actual</p><img src="../tests/{id}/actual.png" data-alt-src="{expected}"></div>
    <div><p>Expected</p><img src="{expected}"></div>
    <div class="diff"><p>Diff</p><img src="../tests/{id}/diff.png"></div>
  </div>
  <p>{status}</p>
</div>"#,
        id = outcome.id,
    )
}

fn report_banner(outcomes: &[TestOutcome], failed: usize, errored: usize) -> String {
    let passed = outcomes
        .iter()
        .filter(|outcome| matches!(outcome.result, TestResult::Pass { .. }))
        .count();
    if failed == 0 && errored == 0 {
        format!(r#"<h1 style="color: green">All {passed} tests passed!</h1>"#)
    } else {
        format!(
            r#"<p class="stats">{passed} passed / {failed} failed / {errored} errored out of {} total</p>"#,
            outcomes.len()
        )
    }
}
