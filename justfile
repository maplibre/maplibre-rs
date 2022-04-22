#!/usr/bin/env just --justfile
# ^ A shebang isn't required, but allows a justfile to be executed
#   like a script, with `./justfile test`, for example.

export RUSTUP_TOOLCHAIN := "nightly-2022-04-04-x86_64-unknown-linux-gnu"

test:
  cargo test

install-clippy:
  rustup component add clippy

clippy: install-clippy
  cargo clippy --all-targets --all-features --no-deps

install-rustfmt:
  rustup component add rustfmt

fmt: install-rustfmt
  cargo fmt --all --

fmt-check: install-rustfmt
  cargo fmt --all -- --check

nightly-toolchain:
  rustup install $RUSTUP_TOOLCHAIN
  rustup component add rust-src --toolchain $RUSTUP_TOOLCHAIN

webpack-webgl-production: nightly-toolchain
  cd web/web && npm install && npm run webgl-production-build

webpack-production: nightly-toolchain
  cd web/web && npm install && npm run production-build

# TODO
# wasm-pack-webgl: nightly-toolchain
#   ./wasm-pack-v0.10.1-x86_64-unknown-linux-musl/wasm-pack build . \
#     --release --target web --out-dir web/dist/maplibre-rs -- \
#     --features "web-webgl" -Z build-std=std,panic_abort
#
# wasm-pack: nightly-toolchain
#   ./wasm-pack-v0.10.1-x86_64-unknown-linux-musl/wasm-pack build . \
#     --release --target web --out-dir web/dist/maplibre-rs -- \
#     -Z build-std=std,panic_abort
#
# build-web-webgl: nightly-toolchain
#   cargo build --features web-webgl --target wasm32-unknown-unknown -Z build-std=std,panic_abort
#
# build-web: nightly-toolchain
#   cargo build --features "" --target wasm32-unknown-unknown -Z build-std=std,panic_abort
#
# wasm-bindgen:
#   cargo install wasm-bindgen-cli
#   # TODO: Untested: --reference-types
#   wasm-bindgen --target web --out-dir web/dist/maplibre-rs-plain-bindgen target/wasm32-unknown-unknown/debug/maplibre.wasm
#
# build-wasm-bindgen: build-web wasm-bindgen
#
# build-wasm-bindgen-webgpu: build-web wasm-bindgen
# TODO
#profile-bench:
# cargo flamegraph --bench render -- --bench

install-cargo-apk:
  cargo install cargo-apk

run-apk: nightly-toolchain install-cargo-apk
  cargo apk run -p android --lib -Zbuild-std

build-apk: nightly-toolchain install-cargo-apk
  cargo apk build -p android --lib -Zbuild-std

# language=bash
print-android-env:
  echo "ANDROID_HOME: $ANDROID_HOME"
  echo "ANDROID_SDK_ROOT: $ANDROID_SDK_ROOT"
  echo "ANDROID_NDK_ROOT: $ANDROID_NDK_ROOT"

# language=bash
extract-tiles:
  #!/usr/bin/env bash
  set -euxo pipefail
  if ! command -v tilelive-copy &> /dev/null
  then
    echo "tilelive-copy could not be found. Install it with 'yarn global add @mapbox/tilelive @mapbox/mbtiles'"
    exit 1
  fi
  # Bounds copied from https://boundingbox.klokantech.com/
  tilelive-copy \
    --minzoom=12 --maxzoom=12 \
    --bounds="11.395769,48.083436,11.618242,48.220866" \
    test-data/europe_germany-2020-02-13-openmaptiles-v3.12.1.mbtiles test-data/munich-12.mbtiles
  tilelive-copy \
    --minzoom=15 --maxzoom=15 \
    --bounds="11.395769,48.083436,11.618242,48.220866" \
    test-data/europe_germany-2020-02-13-openmaptiles-v3.12.1.mbtiles test-data/munich-15.mbtiles
