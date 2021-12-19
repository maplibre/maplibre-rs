use std::{env, fs};
use std::path::{Path, PathBuf};

use mapr_utils::mbtiles::extract;
use wgsl_validate::validate_project_wgsl;

fn main() {
    validate_project_wgsl();

    let root_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let out_dir = env::var("OUT_DIR").unwrap();

    let out = PathBuf::from(Path::new(&out_dir).join("extracted-tiles"));
    if out.exists() && out.is_dir() {
        fs::remove_dir_all(&out).unwrap()
    }
    let source = Path::new(&root_dir).join("test-data/munich-12.mbtiles");

    // Pack tiles around Maxvorstadt (100 tiles in each direction)
    extract(source,
            out,
            12,
            (2179 - 100)..(2179 + 100),
            (1421 - 100)..(1421 + 100),
    ).unwrap();
}
