# Debugging

* log crate



## GPU Debugging

* For WebGL there is SpectorJS is enabled by default right now. For debugging on a desktop environment you can use
  [RenderDoc](https://renderdoc.org/).

## Frame Profiling

maplibre-rs is set up to use the Tracy profiler (https://github.com/wolfpld/tracy). It's mainly designed for C++ but has some Rust support.

The connection to Rust uses a project that connects to the tracing crate (https://github.com/nagisa/rust_tracy_client). This uses a set of three crates (tracing-tracy, tracy-client, tracy-client-sys).

Unfortunately, the Tracy project does not use semantic versioning, whereas tracing-tracy, tracy-client, tracy-client sys do.

The current version of the Rust client is at v0.8.1 of Tracy. See the correlating versions in the table below (original at https://github.com/nagisa/rust_tracy_client#version-support-table):

| Tracy  | tracy-client-sys | tracy-client | tracing-tracy |
|--------|------------------|--------------|---------------|
| 0.7.1  | 0.9.0            | 0.8.0        | 0.2.0         |
| 0.7.3  | 0.10.0           | 0.9.0        | 0.3.0         |
| 0.7.4  | 0.11.0           | 0.10.0       | 0.4.0         |
| 0.7.5  | 0.12.0           | 0.11.0       | 0.5.0         |
| 0.7.6  | 0.13.0, 0.14.0   | 0.12.*       | 0.6.*         |
| v0.7.7 | 0.15.0           | 0.12.*       | 0.6.*         |
| v0.7.8 | 0.16.0           | 0.12.*       | 0.6.*         |
| v0.7.8 | 0.16.0           | 0.12.*       | 0.7.*         |
| v0.7.8 | 0.16.0           | 0.12.*       | 0.8.*         |
| v0.8.1 | 0.17.*           | 0.13.*       | 0.9.*         |
| v0.8.1 | 0.17.*           | 0.14.*       | 0.10.*        |
