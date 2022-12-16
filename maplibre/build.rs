//! # Build
//!
//! This script is built and executed just before building the package.
//! It will validate the WGSL (WebGPU Shading Language) shaders and embed static files.

use std::{fs, path::PathBuf};

use maplibre_build_tools::wgsl::validate_project_wgsl;

#[cfg(feature = "embed-static-tiles")]
fn embed_tiles_statically() {
    use std::{env, path::Path};

    use maplibre_build_tools::mbtiles::extract;

    const MUNICH_X: u32 = 17425;
    const MUNICH_Y: u32 = 11365;
    const MUNICH_Z: u8 = 15;

    /// Tiles which can be used by StaticTileFetcher.
    fn clean_static_tiles() -> std::path::PathBuf {
        let out_dir = std::env::var("OUT_DIR").unwrap();

        let out = std::path::Path::new(&out_dir).join("extracted-tiles");

        if out.exists() && out.is_dir() {
            std::fs::remove_dir_all(&out).unwrap()
        }

        out
    }

    let out = clean_static_tiles();

    let root_dir = env::var("CARGO_MANIFEST_DIR").unwrap();

    let source = Path::new(&root_dir).join(format!("../test-data/munich-{}.mbtiles", MUNICH_Z));

    if source.exists() {
        println!("cargo:rustc-cfg=static_tiles_found");
        // Pack tiles around Munich HBF (100 tiles in each direction)
        extract(
            source,
            out,
            MUNICH_Z,
            (MUNICH_X - 2)..(MUNICH_X + 2),
            (MUNICH_Y - 2)..(MUNICH_Y + 2),
        )
        .unwrap();
    } else {
        // Do not statically embed tiles
    }
}

fn generate_protobuf() {
    let proto_paths = fs::read_dir("./proto")
        .unwrap()
        .filter_map(|entry| {
            let entry = entry.ok()?;
            println!(
                "cargo:rerun-if-changed={}",
                entry.path().display().to_string()
            );
            Some(entry.path())
        })
        .collect::<Vec<_>>();

    if !proto_paths.is_empty() {
        prost_build::compile_protos(&proto_paths, &[PathBuf::from("./proto/")]).unwrap();
    }
}

fn main() {
    validate_project_wgsl();

    #[cfg(feature = "embed-static-tiles")]
    embed_tiles_statically();

    generate_protobuf();
}
