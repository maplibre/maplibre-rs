use std::path::PathBuf;

fn main() {
    let out_path = PathBuf::from("src/protos");
    protobuf_codegen_pure::Codegen::new()
        .out_dir(out_path)
        .inputs(&["spec/2.1/vector_tile.proto"])
        .include("spec/2.1")
        .run()
        .expect("Codegen failed.");
}
