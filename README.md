<h1 align="center">mapr</h1>
<p align="center">
  <img width=200px alt="Logo" src="https://">
</p>
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
    <img src="https://img.shields.io/github/workflow/status/maxammann/mapr/Rust?style=flat-square"
        alt="Build status" />
  </a>
</div>

<div align="center">
  <h3>
    <a href="https://">
      Example
    </a>
    <span> | </span>
    <a href="https://maxammann.github.io/mapr">
      Documentation
    </a>
  </h3>
</div>


## Description

TODO


## Features

* None so far

## Goals

* Renders [vector tiles](https://docs.mapbox.com/vector-tiles/specification/).
* Runs on:
  * Web via WebAssembly and WebGPU,
  * Linux (Xorg/Wayland) via Vulkan,
  * Android via OpenGL,
  * iOS via Metal.
* Supports the [TileJSON](https://docs.mapbox.com/help/glossary/tilejson/) standard
* Pimarily 

## Non-Goals

* Rendering any kind of rasterized data

## Building

Now, to clone the project:

```bash
git clone git@github.com/maxammann/mapr
```

and then build it for running on a desktop:

```bash
cargo build
```

### Target: WebGPU

```bash
tools/build-web
cd web
python3 -m http.server
```

### Target: WebGL

```bash
tools/build-web -g
cd web
python3 -m http.server
```

## Running on Linux

Fuzz using three clients:

```bash
cargo run --bin mapr --
```

## Testing

```bash
cargo test
```

## Rust Setup

Install [rustup](https://rustup.rs/).

The toolchain will be automatically downloaded when building this project. See [./rust-toolchain.toml](./rust-toolchain.toml) for more details about the toolchain.

## Documentation

This generates the documentation for this crate and opens the browser. This also includes the documentation of every
dependency.

```bash
cargo doc --open
```

You can also view the up-to-date documentation [here](https://).

