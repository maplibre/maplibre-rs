use std::{env, fs};
use std::path::{Path, PathBuf};

use mapr_utils::mbtiles::extract;
use wgsl_validate::validate_project_wgsl;

fn main() {
    validate_project_wgsl();

    let root_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let out_dir = env::var("OUT_DIR").unwrap();

    let out = PathBuf::from(Path::new(&out_dir).join("munich-tiles"));
    fs::remove_dir_all(&out);
    //let out = PathBuf::from(Path::new(&root_dir).join("test-data/munich-tiles"));
    let source = Path::new(&root_dir).join("test-data/maptiler-osm-2017-07-03-v3.6.1-germany_munich.mbtiles");

    extract(source,
            out,
    ).unwrap();


}
