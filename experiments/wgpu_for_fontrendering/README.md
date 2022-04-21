# Fontrendering experiment

This is a standalone wgpu/winit application which serves as an experimentation platform for fontrendering.
Currently, the approach from [this article](https://medium.com/@evanwallace/easy-scalable-text-rendering-on-the-gpu-c3f4d782c5ac) is being reimplemented with the goal of measuring its performance.

## Build setup
This is a separate project from the maplibre-rs, therefore it is excluded from the maplibre-rs workspace and defines its own workspace.

> Running on Mac did not work with a simple `cargo run` (linker error) but with `cargo run --target aarch64-apple-darwin` (on a M1) it worked fine.