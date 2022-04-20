use wgpu_for_fontrendering::run;

fn run_gui() {
    pollster::block_on(run());
}

fn main() {
    run_gui();
}
