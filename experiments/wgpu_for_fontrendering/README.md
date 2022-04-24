# GPU-based font rendering experiment

This is a standalone wgpu/winit application which serves as an experimentation platform for font rendering on the GPU.

> The goal is not (yet) to provide a fully fleshed out gpu-based font rendering library but rather see w


## Current Approach:
We can render arbitrary text with a .ttf font under arbitrary 3-d transformations on the GPU:
![](./doc/perspective_transform.png)


### Algorithm
[Lightweight bezier curve rendering](https://medium.com/@evanwallace/easy-scalable-text-rendering-on-the-gpu-c3f4d782c5ac) was reimplemented:

* Convert ttf outlines into triangle meshes (cpu side, once)
* Use winding order trick to produce correct glyph shape
    - First pass: overdraw pixels into a texture
    - Second pass: render texture but only the pixels that were drawn an uneven number of times
![](./doc/animation.gif)
Animation taken from [here](https://medium.com/@evanwallace/easy-scalable-text-rendering-on-the-gpu-c3f4d782c5ac).
* Quadratic curve segments get two triangles, one with special uv coordinates to enable [simple curve evaluation in the fragment shader](https://developer.nvidia.com/gpugems/gpugems3/part-iv-image-effects/chapter-25-rendering-vector-art-gpu)

A rough overview of the setup and render routine:
![](./doc/overview.png)

### Performance
* Parsing/tesselation of glyphs is done with the help of `ttfparser` -> currently unoptimized. With glyph caching this should be ok
* Rendering times degrade massively with number of glyphs, but is independent of screen resolution
![](./doc/benchmark_2022-04-24.png)

### Issues

The main issue with this approach (besides performance) is that the trick with using overdrawing of pixels to decide whether to fill them or not produces artifacts when two separate glyphs overlap in screen space:
![](./doc/overlapping_problem.png)

### TODOs
* Figure out, whether the problem with overlapping glyphs can be fixed
* Cache glyph meshes, so they are not recreated whenever they appear in a word and render them as instances

## Build setup
This is a separate project from the maplibre-rs, therefore it is excluded from the maplibre-rs workspace and defines its own workspace.

> Running on Mac did not work with a simple `cargo run` (linker error) but with `cargo run --target aarch64-apple-darwin` (on a M1) it worked fine.