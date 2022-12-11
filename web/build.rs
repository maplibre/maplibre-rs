use std::{env, fs, path::Path};

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();

    println!("cargo:rerun-if-changed=./flatbuffer");

    let flatbuffer = fs::read_dir("./flatbuffer")
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .collect::<Vec<_>>();

    flatc_rust::run(flatc_rust::Args {
        inputs: &flatbuffer
            .iter()
            .map(|buf| buf.as_path())
            .collect::<Vec<_>>(),
        out_dir: Path::new(&out_dir),
        extra: &[
            "--include-prefix",
            "platform::singlethreaded::transferables",
        ],
        ..Default::default()
    })
    .expect("flatc");
}
