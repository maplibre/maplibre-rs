//! Workspace-relative paths used by the render harness.

use std::path::PathBuf;

use maplibre::coords::WorldTileCoords;

pub(super) fn workspace_tests_dir() -> PathBuf {
    PathBuf::from("render-tests/src/tests")
}

pub(super) fn workspace_templates_dir() -> PathBuf {
    PathBuf::from("render-tests/src/templates")
}

pub(super) fn local_tile_path(template: &str, coords: WorldTileCoords) -> Result<PathBuf, String> {
    let relative = template
        .strip_prefix("local://tiles/")
        .ok_or_else(|| format!("Unsupported tile URL in render harness: {template}"))?;
    let relative = relative
        .replace("{z}", &u8::from(coords.z).to_string())
        .replace("{x}", &coords.x.to_string())
        .replace("{y}", &coords.y.to_string());
    Ok(PathBuf::from("render-tests/src/assets/tiles").join(relative))
}

pub(super) fn local_data_path(url: &str) -> Result<PathBuf, String> {
    let relative = url
        .strip_prefix("local://data/")
        .ok_or_else(|| format!("Unsupported GeoJSON URL in render harness: {url}"))?;
    Ok(PathBuf::from("render-tests/src/assets/data").join(relative))
}

pub(super) fn collect_tests(test_root: &std::path::Path) -> Vec<PathBuf> {
    let mut tests = walkdir::WalkDir::new(test_root)
        .min_depth(1)
        .max_depth(5)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_name() == "style.json")
        .filter_map(|entry| entry.path().parent().map(std::path::Path::to_path_buf))
        .filter(|parent| !parent.ends_with("projection/perspective"))
        .collect::<Vec<_>>();
    tests.sort();
    tests
}
