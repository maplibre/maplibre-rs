# Related Resources

## Talks

* [2022-04-13-World-in-Vectors](https://docs.google.com/presentation/d/e/2PACX-1vRsi-sGsqwUXEIQDClaZF4BH2RgjufQQ-yxFDWeOGrm0EbIf4H4lFY3U4at4cAIlxSTWi4XyF2LKjRu/pub)

## Related Projects

* [Tangram Renderer](https://github.com/tangrams/tangram/)
* [Tilezen/Nextzen](https://www.nextzen.org/)
* [Protomaps: New Maps Stack](https://protomaps.com/)
* [Harp GL](https://www.harp.gl/)

## GIS
* [Google Maps Projection](https://www.maptiler.com/google-maps-coordinates-tile-bounds-projection)
* [Grid Calculation Examples](https://gist.github.com/maptiler/fddb5ce33ba995d5523de9afdf8ef118)
* [Slippy map tilenames](https://wiki.openstreetmap.org/wiki/Slippy_map_tilenames) (also known as XYZ) 
* [TMS](https://wiki.osgeo.org/wiki/Tile_Map_Service_Specification#TileMap_Diagram)
* [Mapbox Adaptive Projections](https://www.mapbox.com/blog/adaptive-projections)
* [Bing Map Tile System](https://docs.microsoft.com/en-us/bingmaps/articles/bing-maps-tile-system)
* [Bing: Understanding Scale and Resolution](https://docs.microsoft.com/en-us/bingmaps/articles/understanding-scale-and-resolution)

## WebAssembly and WebWorkers

Specs:

* [Thread/Atomics Proposal for WASM](https://webassembly.github.io/threads/core/bikeshed/#atomic-memory-instructions%E2%91%A2)

Projects:

* [Experiment with shared memory](https://github.com/Ciantic/rust-shared-wasm-experiments) and [the idea behind it](https://github.com/rustwasm/wasm-bindgen/issues/2225)
* [Shared channel](https://github.com/wasm-rs/shared-channel)
* [Bridge for async executors](https://docs.rs/async_executors/latest/async_executors/)
* [Rayon for WebAssembly](https://github.com/GoogleChromeLabs/wasm-bindgen-rayon)
* [wasm-mt: postMessage message passing](https://github.com/w3reality/wasm-mt)
* 
Articles:

* [WebAssembly Threads (official)](https://web.dev/webassembly-threads/)
* [Multithreading Rust and Wasm 2018](https://rustwasm.github.io/2018/10/24/multithreading-rust-and-wasm.html)
* [postMessage Performance](https://surma.dev/things/is-postmessage-slow/)
* [A practical guide to WebAssembly memory](https://radu-matei.com/blog/practical-guide-to-wasm-memory/)

Examples:
* [WASM in a WebWorker](https://rustwasm.github.io/wasm-bindgen/examples/wasm-in-web-worker.html)
* [Building for Shared Memory](https://github.com/rustwasm/wasm-bindgen/blob/main/examples/raytrace-parallel/build.sh)
* [Parallel Raytracing](https://rustwasm.github.io/docs/wasm-bindgen/examples/raytrace.html)

## Rendering

Specs:

* [WebGPU Spec](https://gpuweb.github.io/gpuweb/)
* [WGSL Spec](https://gpuweb.github.io/gpuweb/wgsl/)
* [WGSL Struct Alignment](https://gpuweb.github.io/gpuweb/wgsl/#alignment-and-size)
* [Mismatches Stencil Test](https://github.com/gpuweb/gpuweb/blob/main/design/Pipelines.md#depth-stencil-state)

Articles:

* [Life of a Tile (MapLibre)](https://github.com/maplibre/maplibre-gl-js/blob/main/docs/life-of-a-tile.md)

Tutorials:

* [Stencil Testing](https://learnopengl.com/Advanced-OpenGL/Stencil-testing)
* [Camera](https://learnopengl.com/Getting-started/Camera)
* [Writing an efficient Vulkan renderer](https://zeux.io/2020/02/27/writing-an-efficient-vulkan-renderer/)

Examples:

* [Stencil Mask Example](https://github.com/ruffle-rs/ruffle/blob/master/render/wgpu/src/pipelines.rs#L330)
* [WGPU Examples](https://github.com/gfx-rs/wgpu/blob/ad0c8d4f781aaf9907b5f3a90bc7d00a13c51153/wgpu/examples/README.md)

## Maths

Articles:
* [Magnificent Matrix](https://ncase.me/matrix/)

Examples:

* [Dolly Camera](https://github.com/h3r2tic/dolly)

## Font Rendering

Specs:

* [MapBox Glyphs Spec](https://github.com/mapbox/node-fontnik/blob/master/proto/glyphs.proto)

Articles:

* [Signed distance function](https://en.wikipedia.org/wiki/Signed_distance_function)

Projects:

* [Mapbox fontnik](https://github.com/mapbox/node-fontnik/)
* [TinySDK (JS)](https://github.com/mapbox/tiny-sdf)
* [RustType](https://github.com/redox-os/rusttype)
* [SDF Render](https://docs.rs/sdf_glyph_renderer/latest/sdf_glyph_renderer/)
* [pbf_font_tools](https://github.com/stadiamaps/pbf_font_tools)
* [Multi-channel signed distance field generator](https://github.com/Chlumsky/msdfgen)
  * [MSDF font atlas generator ](https://github.com/Chlumsky/msdf-atlas-gen)

## Tessellation

Projects:

* [earcutr](https://github.com/donbright/earcutr)
* [Line Tessellation in MapLibre](https://github.com/maplibre/maplibre-gl-js/blob/main/src/data/bucket/line_bucket.ts)

## Specifications

* [TileJSON](https://github.com/mapbox/tilejson-spec)


## Render Graphs

* https://github.com/metal-by-example/simple-instancing/blob/master/MetalSimpleInstancing/Renderer.swift
* https://github.com/troughton/Substrate
* https://de.slideshare.net/DICEStudio/framegraph-extensible-rendering-architecture-in-frostbite
* http://themaister.net/blog/2017/08/15/render-graphs-and-vulkan-a-deep-dive/
* http://themaister.net/blog/2017/08/15/render-graphs-and-vulkan-a-deep-dive/

## Animation

* https://crates.io/crates/pareen
* https://crates.io/crates/keyframe