<h1 align="center">mapr</h1>
<p align="center">
  <img width="200px" alt="Logo" src="https://">
</p>
<div align="center">
  <img width="300px" src="docs/src/figures/mapr-ios.png" alt="preview">
</div>
<div align="center">
  <strong>Native Maps for Web, Mobile and Linux</strong>
</div>
<div align="center">
  A map rendering library written in Rust.
</div>

<div align="center">
  <img src="https://img.shields.io/badge/stability-experimental-orange.svg?style=flat-square" 
      alt="Stability" />
  <a href="https://github.com/maxammann/mapr/actions/workflows/rust.yml">    
    <img src="https://github.com/maxammann/mapr/actions/workflows/rust.yml/badge.svg"
        alt="Build status" /> 
  </a>
  <a href="https://matrix.to/#/#mapr:matrix.org">    
    <img src="https://img.shields.io/static/v1?label=Space&message=%23mapr&color=blue&logo=matrix"
        alt="Build status" />
  </a>
</div>

<div align="center">
  <h3>
    <a href="https://">
      Example
    </a>
    <span> | </span>
    <a href="https://maxammann.org/mapr/docs">
      Book
    </a> | </span>
    <a href="https://maxammann.org/mapr/api-docs/mapr/">
      API
    </a> | </span>
    <a href="https://matrix.to/#/#mapr:matrix.org">
      Chat in Matrix Space
    </a>
  </h3>
</div>

## Description

mapr is a portable and performant vector maps renderer. We aim to support the web, mobile and desktop applications. This
is achieved by the novel [WebGPU](https://www.w3.org/TR/webgpu/) specification. Plenty of native implementations are
already implementing this specification. On the web it is implemented by Firefox, Chrome and Safari. There are also
standalone implementations which directly use Vulkan, OpenGL or Metal as a backend. Those backends allow mapr to run on
mobile and desktop applications.

Rust is used as a Lingua-franka on all platforms. This is made possible by WebAssembly which allows us to use Rust for
web development.

The goal of mapr is to render maps in order to visualize data. Right now the goal of mapr is not to replace existing
vector map renderers like Google Maps, Apple Maps or MapLibre. The current implementation serves as a proof-of-concept
of the used technology stack. It is unclear whether the high-performance requirements of rendering maps using vector
graphics are achievable using the current stack.

## Current Features

* [x] Render single vector tiles
* [x] Render multiple vector tiles
* [x] Runs on Linux, Android, iOS, MacOS, Firefox and Chrome
* [ ] Simple navigations by translating the camera
* [ ] Load and tessellate vector tiles on demand
* [ ] Navigation which "feels good"

### Goals

* Renders [vector tiles](https://docs.mapbox.com/vector-tiles/specification/).
* Runs on:
    * Web via WebAssembly and WebGPU,
    * Linux (Xorg/Wayland) via Vulkan,
    * Android via OpenGL,
    * iOS via Metal.
* Supports the [TileJSON](https://docs.mapbox.com/help/glossary/tilejson/) standard

### Non-Goals

* Rendering any kind of rasterized data

## Repository Layout

```bash
.
├── docs                # Documentation for mapr
├── src                 # The source code of the mapr library
├── libs                # Libraries which will eventually be published as separate crates
│   ├── mbtiles         # Library for extracting .mbtiles files
│   ├── style_spec      # Library for interpreting MapLibre style specifications
│   ├── vector_tile     # Library for parsing vector tile shaders
│   └── wgsl_validate   # Library for validating WGSL shaders
├── apple               # Platform specific files for Apple (iOS and MacOS)
├── web                 # Platform specific files for Web (WebGL and WebGPU)
├── benches             # Benchmarks for specific parts of the library
├── examples            # Examples which can be run
├── test-data           # Geo data which can be used for tests (Usually as .mbtiles)
└── tools
│   ├── build-android   # Script which helps building for Android
│   ├── build-web       # Script which helps building for Web
│   └── extract-region  # Script can extract data from a .mbtiles file
```

## Building & Running

Now, to clone the project:

```bash
git clone git@github.com/maxammann/mapr
```

and then build it for running on a desktop:

```bash
cargo build
```

After that you can run it on your desktop:

```bash
cargo run --example desktop --
```

More information about building for different platforms can be
found [here](https://maxammann.org/mapr-docs/building.html).

## Rust Setup

Install [rustup](https://rustup.rs/) because this is the recommended way of setting up Rust toolchains.

The toolchain will be automatically downloaded when building this project.
See [./rust-toolchain.toml](./rust-toolchain.toml) for more details about the toolchain.

## Documentation

This generates the documentation for this crate and opens the browser. This also includes the documentation of every
dependency.

```bash
cargo doc --open
```

You can also view the up-to-date documentation [here](https://).

