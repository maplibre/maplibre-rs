# Globe projection parity matrix

This audit compares maplibre-rs with MapLibre GL JS commit
`f9a40a5c4462abafd6823d9b6fc623246f31e787`. The upstream render corpus contains 75
non-terrain fixtures and 45 terrain fixtures. Projection behavior is separated from layer and API
features that maplibre-rs does not yet provide for Mercator.

Status meanings:

- **Golden**: upstream style behavior and expected image run in the Rust headless harness; optional
  harness-only metadata may document comparison policy.
- **Structural**: reference math or behavior is implemented and covered by deterministic tests.
- **Baseline blocker**: projection-independent renderer or host API support is absent.
- **Deferred**: terrain work belongs to its own RFC.

## Imported golden suite

The 19-case `render-tests/src/tests/projection-globe` suite runs on Metal/wgpu:

| Area | Imported upstream cases | Result |
| --- | --- | --- |
| Background | `background` | Golden |
| Fill | `fill-planet-{pitched,pole,solid,tiles}`, `fill-translate` | Golden |
| Line | `line-spiral`, `line-translate` | Golden |
| Raster | `raster-{planet,pole,warped}` | Golden |
| Atmosphere | base, zoom, map/viewport light, and four blend cases | Golden |

The default mean-channel threshold is `0.02`. `raster-warped` uses `0.04` because the upstream
fixture explicitly says exact rasterization is irrelevant; the accepted invariant is an untilted,
seam-free checkerboard. `raster-planet` composites white only for comparison because that one
upstream golden bakes opaque white outside the planet while sibling fixtures preserve alpha.

Each attempt renders one initialization frame and compares the following frame. Surface capture is
graph-ordered after translucent rendering, with a unit test that prevents atmosphere readback from
racing the copy pass. The harness reports up to two retries for failed image comparisons, never
retries errors, and removes previous frame files so a missing frame cannot pass.

## Non-terrain render corpus

| GL JS fixture family | Cases | Status | Rust evidence or blocker |
| --- | ---: | --- | --- |
| `background`, `background-opacity` | 2 | 1 Golden, 1 Structural | Curved z0 mesh, physical background/atmosphere pass ordering |
| `background-pattern` | 1 | Baseline blocker | Background patterns are absent in the flat renderer |
| basic fill and translate | 5 | Golden | GeoJSON/MVT subdivision, poles, translations, strict images |
| fill seams and advanced paint | 2+ | Structural / baseline blocker | Shared-edge and clipping tests exist; missing paint paths stay blocked |
| basic line and translate | 2 | Golden | Subdivision, screen width, antimeridian clipping, strict images |
| line pattern/dash/gradient | 8 | Baseline blocker | The corresponding flat paint paths are incomplete |
| `raster-*` | 3 | Golden | Homogeneous texture coordinates, pole meshes, border/interior seam passes |
| image sources | 2 | Baseline blocker | Image-source rendering is absent |
| symbol/text/collision | 22 | Structural / baseline blocker | Anchor projection and horizon/collision tests exist; full line-label placement is absent |
| circles | 5 | Baseline blocker | Circle rendering is not active in the render graph |
| fill extrusion | 2 | Baseline blocker | Fill-extrusion rendering is absent |
| heatmap, hillshade | 2 | Baseline blocker | Flat renderer paths are absent |
| custom layers | 3 | Baseline blocker | No projection-independent custom-layer API exists |
| atmosphere and sky | 10 | 8 Golden, 2 Structural | Physical scattering, style blend, light anchor, zoom, and premultiplied composition |
| antimeridian LOD | 1 | Structural | Pinned covering-tile reference sets for pitch and rotation |
| antimeridian overdraw | 11 | Structural / baseline blocker | Canonical wraps and clipping exist; blocked layer families remain blocked |
| zoom transition | 1 | Structural | The `globe` preset stays vertical perspective through z11 and blends to Mercator by z12, matching the GL JS implementation and its z11 golden |

Family counts overlap because some fixtures cover multiple subsystems. The audited upstream total,
not the table sum, is authoritative.

## Function-level inventory

| GL JS subsystem | Status | Executable Rust contract |
| --- | --- | --- |
| Globe utility math | Structural | radius/circumference, distance, coordinate/vector conversion, orientation, horizon, sphere clamp, ray intersection, zoom adjustment, interpolation |
| Vertical-perspective camera | Structural | matrices and inverses, camera position, clipping, projection/unprojection, screen rays, horizon continuation, occlusion, scale corrections, light transforms |
| Camera helper | Wired (winit) / Structural | virtual-trackball drag pan, pole dial, pan inertia, and the pointer zoom heuristic drive the winit handlers; jump/ease/fly normalized zoom and the bounds scale solver wait for a host camera API |
| Covering tiles | Structural | wraps, antimeridian, variable LOD, frustum/horizon culling, elevation-aware volumes |
| Mesh and subdivision | Structural | raster grids, borders, poles, index safety, polygon/line subdivision, antimeridian clipping |
| Active renderer | Golden | background, fill, line, raster, atmosphere, local GeoJSON/MVT/raster sources |
| Symbol integration | Partial structural | anchors and occlusion exist; layer zoom ranges are honored; glyph rendering, collision, and line placement need the flat symbol pipeline first |
| Runtime map/UI APIs | Baseline blocker | maplibre-rs has no equivalent complete projection transition/event, marker, popup, or custom-layer host API; roll is accepted by the globe camera but the map camera has no roll axis |

Generic operations remain generic: roll, pitch, bearing, padding interpolation, and caller-side
constraints do not need globe-specific implementations. The globe camera-helper module exposes the
projection-specific results needed by a future host camera API instead of inventing that missing API
inside the renderer.

## Terrain corpus

All 45 `projection/globe/terrain` fixtures are **Deferred**. Radial elevation math and
elevation-aware covering volumes are present, but DEM upload, displaced mesh rendering, depth,
picking, and atmosphere/terrain composition are not claimed by this RFC.

## Known upstream behavior

The demotiles Antarctica polygon is cut near longitude -156 at zoom 0 rather than at the
antimeridian, and its pieces meet only approximately. On the globe the pole fill turns that gap into
a thin wedge from the coast toward the pole. MapLibre GL JS renders the same wedge
([maplibre-gl-js#5433](https://github.com/maplibre/maplibre-gl-js/issues/5433), open), so the
Rust renderer matches upstream here and a fix belongs upstream or in the data.

The style specification's documentation describes the `globe` preset as
`["interpolate", ["linear"], ["zoom"], 10, "vertical-perspective", 12, "mercator"]`, while the GL JS
implementation starts the blend at zoom 11 and its `atmosphere-blend/interpolate-to-0.5` golden at
zoom 11 expects a full globe. This renderer follows the implementation and its goldens; the
discrepancy is a documentation question for upstream.

## Trying the globe

`cargo run -p maplibre-demo -- headed --globe` opens the bundled world style on the globe. Tile
sources come from the style: `tiles` templates are used directly and a TileJSON `url` is resolved
when the renderer initializes. Layers that name no source keep the crate default source. Left-drag
pans with GL JS's fixed-bearing versor drag, trackball fall-off and pole dial, and eases out with
GL JS inertia; the wheel zooms around the pointer with the horizon-safe heuristic; right-drag turns
the bearing and tilts; W/A/S/D pan. `--style <file>` loads any style and `--projection` overrides
its projection. `--frames <n>` exits after `n` frames for smoke tests. Debug builds draw the tile
grid projected onto the globe; release builds draw none.

The web demo (`just web-lib build`, then `just web-demo start`) loads the MapLibre demotiles style
on the globe through WebGPU; `?style=<url>` and `?projection=` select another style or projection.
The browser HTTP client fetches from the window as well as from workers so TileJSON sources resolve
before the first frame, and a vector layer whose geometry cannot be indexed no longer aborts the
worker that decoded it.

## Required gates

- `cargo fmt --all -- --check`
- globe unit tests, including the four GL JS drag-control scenarios
- shader validation, render-phase ordering, and headless capture-order tests
- all 19 imported headless goldens in one process
- Mercator tests and workspace checks appropriate to each split PR

An item may move to **Golden** only when its upstream style, assets, camera metadata, and expected
image execute in the Rust harness. A baseline blocker cannot be counted as a globe regression.
