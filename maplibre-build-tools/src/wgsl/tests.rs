use std::{
    collections::HashSet,
    fs, io,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use super::load_wgsl;

struct WgslFixture {
    root: PathBuf,
}

impl WgslFixture {
    fn new() -> io::Result<Self> {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(io::Error::other)?
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "maplibre-wgsl-include-{}-{nonce}",
            std::process::id()
        ));
        fs::create_dir(&root)?;
        Ok(Self { root })
    }

    fn write(&self, name: &str, source: &str) -> io::Result<PathBuf> {
        let path = self.root.join(name);
        fs::write(&path, source)?;
        Ok(path)
    }
}

impl Drop for WgslFixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.root).ok();
    }
}

#[test]
fn expands_relative_includes() {
    let fixture = WgslFixture::new().expect("temporary fixture should be created");
    fixture
        .write("shared.wgsl", "const VALUE: f32 = 1.0;\n")
        .expect("shared shader should be written");
    let entry = fixture
        .write(
            "entry.wgsl",
            "// @include shared.wgsl\nfn read() -> f32 { return VALUE; }\n",
        )
        .expect("entry shader should be written");

    let expanded =
        load_wgsl(&entry, &mut HashSet::new()).expect("relative include should be expanded");

    assert!(expanded.contains("const VALUE: f32 = 1.0;"));
    assert!(expanded.contains("fn read() -> f32"));
}

#[test]
fn rejects_include_cycles() {
    let fixture = WgslFixture::new().expect("temporary fixture should be created");
    let first = fixture
        .write("first.wgsl", "// @include second.wgsl\n")
        .expect("first shader should be written");
    fixture
        .write("second.wgsl", "// @include first.wgsl\n")
        .expect("second shader should be written");

    let error = load_wgsl(Path::new(&first), &mut HashSet::new())
        .expect_err("include cycle should be rejected");

    assert_eq!(error.kind(), io::ErrorKind::InvalidData);
}
