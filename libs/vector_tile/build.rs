use protobuf_codegen_pure::Customize;
use std::path::PathBuf;

fn main() {
    let out_path = PathBuf::from("src/protos");
    protobuf_codegen_pure::Codegen::new()
        .customize(Customize {
            //carllerche_bytes_for_bytes: Some(true),
            //carllerche_bytes_for_string: Some(true),
            lite_runtime: Some(true),
            ..Default::default()
        })
        .out_dir(out_path)
        .inputs(&["spec/2.1/vector_tile.proto"])
        .include("spec/2.1")
        .run()
        .expect("Codegen failed.");
}
