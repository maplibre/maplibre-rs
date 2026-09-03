- Feature Name: `globe_projection`
- Start Date: 2026-09-01
- RFC PR: (to be assigned)
- maplibre-rs Issue: [maplibre/maplibre-rs#139](https://github.com/maplibre/maplibre-rs/issues/139)

# Summary

Add a globe projection to maplibre-rs with the same observable style, camera, rendering, and query
behavior as MapLibre GL JS. The implementation keeps Web Mercator as the default,
uses projection-specific camera and GPU data, subdivides flat tile geometry before projecting it
onto a sphere, and treats the MapLibre GL JS globe test corpus as the compatibility contract.

Terrain is a follow-up RFC and implementation. This RFC keeps elevation-aware interfaces and radial
geometry semantics so terrain can be added without replacing the globe projection contract.

# Motivation

Globe rendering is required for world-scale maps where a flat Web Mercator view communicates the
wrong spatial relationship, duplicates the world, or cannot represent the poles. Compatibility
with MapLibre GL JS matters because applications should be able to share styles and expected camera
behavior across the JavaScript and Rust renderers.

The expected outcome is not a globe-shaped demonstration. Globe completion supports these
functional areas for layer types that maplibre-rs renders:

- projection selection and animated globe-to-Mercator transition;
- camera projection, unprojection, bounds fitting, panning, zooming, and inertia;
- globe-aware tile covering, wrapping, culling, level of detail, and pole geometry;
- vector fill and line, raster, symbol, and background rendering;
- antimeridian clipping, horizon occlusion, desktop queries, and cursor interaction;
- atmosphere and sky rendering;

Circle, heatmap, hillshade, fill-extrusion, image-source, custom-layer, marker, and popup parity are
blocked by missing or incomplete projection-independent renderer/API support. They remain in the
compatibility inventory and become globe requirements when their flat-renderer counterparts land.

## Implementation status

A reference implementation accompanies this RFC as a single pull request of one commit per area:

| Area | Status | Evidence |
| --- | --- | --- |
| Style projection model | Complete | Mercator, globe, vertical perspective, explicit transitions, interpolate, and step tests |
| Globe mathematics and camera | Complete | Coordinate, orientation, horizon, ray/sphere, screen round-trip, and precision tests |
| Tile covering and LOD | Complete | Frustum, wrap, antimeridian, pitch, rotation, and elevation-bound fixtures |
| Raster/background meshes | Complete | Subdivided grids, borders, poles, stencil ordering, and z0 background mesh tests |
| Fill and line projection | Complete for supported paint paths | Seven imported fill/line goldens plus shared WGSL and CPU subdivision tests |
| Raster projection | Complete | Three imported goldens, homogeneous sampling, poles, and two-pass seam ownership |
| Symbols | Partial | Anchor projection, horizon culling, collision opacity, and antimeridian tests; tangent-aligned line labels remain |
| Atmosphere | Complete for the active style model | Eight imported goldens cover physical scattering, blend, zoom, map/viewport lights, and graph-ordered headless capture |
| Interaction and queries | Wired in winit | versor drag with silhouette fallback and pole dial, pan inertia, pointer zoom heuristic, keyboard pan, hit and horizon queries; jump/ease/fly and the bounds solver remain library functions without a host camera API |
| GL JS render corpus | 19 Golden | Exact upstream styles, local assets, camera metadata, and expected images run in one Metal/wgpu suite |
| Host UI/custom-layer integration | Partial | the desktop and web demos load globe styles and fetch style-declared tile sources, layers honor their zoom ranges, and the debug tile grid follows the projection; maplibre-rs still lacks the projection-independent camera animation, event, marker, popup, and custom-layer host API |
| Terrain | Deferred | Elevation-aware math exists; DEM sampling, depth, picking, and render integration are outside this RFC |

The detailed compatibility matrix is maintained in
[`globe-projection-parity.md`](globe-projection-parity.md).

# Guide-level explanation

A style selects the globe by adding a root projection object:

```json
{
  "version": 8,
  "projection": { "type": "globe" },
  "sources": {},
  "layers": []
}
```

Styles without `projection` continue to use Web Mercator. Loading a globe style changes both the
rendering projection and map interaction. Coordinates and public camera options remain longitude,
latitude, zoom, bearing, pitch, roll, and field of view; callers do not supply sphere coordinates.

At world scale, tiles are curved over a sphere and geometry behind the horizon is hidden. The
`globe` preset blends from vertical perspective at zoom 11 to the flat Mercator path at zoom 12,
as MapLibre GL JS implements it. This preserves familiar
street-level behavior and permits the flat path to remain the optimized high-zoom implementation.

Future terrain elevation extends radially from the globe center. A positive elevation therefore
increases the sphere radius at a location instead of adding to a planar Z axis. This RFC implements
the radial math and elevation-aware bounds, not DEM sampling or terrain picking.

Projection selection is a style concern, while projection state is renderer-owned. Plugins and
custom layers receive explicit projection data for the current frame instead of reading mutable
global state.

# Reference-level explanation

## Compatibility boundary

The compatibility target is observable MapLibre GL JS behavior rather than its TypeScript class
layout or WebGL resource model. Rust and WGSL code is written for maplibre-rs and wgpu. Mathematical
identities, test scenarios, expected images, and engineering rationale may be adapted under the
MapLibre GL JS BSD-3-Clause license with attribution retained where required.

The implementation must not regress Mercator rendering. Projection-specific work is selected by an
explicit projection value and uses the existing path when that value is absent or Mercator.

## Projection model

`ProjectionType` is the style-facing choice. Renderer code uses a projection contract with the
following responsibilities:

- identify the projection and active shader variant;
- report whether geometry subdivision is required;
- produce per-frame camera data and per-tile projection data;
- select or construct projection-specific tile meshes;
- report transition state and whether a transition remains active;
- release GPU resources owned by the projection.

The globe projection composes a Mercator implementation and a vertical-perspective sphere
implementation. The transition state selects either implementation at the endpoints and provides
both projection matrices during interpolation. Endpoint selection avoids globe mesh and shader cost
when the transition has fully reached Mercator.

The projection object owns immutable or GPU-heavy projection resources. Camera state belongs to a
separate transform so transforms remain cloneable for symbol placement and queries.

## Coordinate systems and precision

Source geometry remains in tile-local Web Mercator coordinates. For tile `(x, y, z)` and local
coordinate `(u, v)`, the renderer first computes normalized Mercator coordinates and then maps them
to longitude and latitude on a unit sphere.

The sphere axes match MapLibre GL JS:

- positive Y points north;
- longitude and latitude zero map to positive Z;
- positive X follows increasing longitude at the prime meridian.

The inverse Web Mercator latitude calculation uses the tangent-half-angle rational identities. The
shader must not subtract π/2 after evaluating an angle near π/2 because that loses float32 precision
near the equator on affected GPUs. CPU and WGSL implementations share numerical parity tests.

Earth radius is `6_371_008.8` metres for elevation scaling. Position calculations use f64 on the
CPU and projection-relative f32 data on the GPU. Per-tile Mercator origin and scale are supplied
separately so large world coordinates are not rounded before projection.

## Tile meshes and geometry subdivision

A four-corner tile cannot approximate a curved globe. Geometry is subdivided in tile space before
the vertex shader projects it. Subdivision has three paths:

- regular grid meshes for raster, hillshade, background, and stencil use;
- polygon subdivision that preserves original edges, winding, holes, and feature metadata;
- line subdivision that preserves joins, caps, distance metrics, and antimeridian clipping inputs.

Subdivision granularity is projection- and usage-specific. Neighboring tiles generate identical
coordinates along shared edges. Border vertices may extend slightly outside the tile for raster
filtering, while antimeridian clipping prevents those borders from drawing twice.

The north and south Web Mercator edge tiles receive explicit pole vertices when the draw operation
allows poles. Pole sentinels are converted to exact unit vectors in the shader. Pole geometry is not
added to line paths or operations where it would change feature semantics.

Generated meshes are cached by canonical tile, granularity, border policy, pole policy, and mesh
usage. The cache is owned by the projection and released with it.

## GPU projection data and shaders

Each projected draw receives:

- globe projection matrix;
- Mercator fallback matrix for transition rendering;
- normalized Mercator tile origin and local-coordinate scale;
- horizon clipping plane;
- transition value;
- antimeridian clipping policy;
- viewport and camera data already required by the layer shader.

Shared WGSL projection functions provide:

- tile coordinate to unit-sphere conversion;
- local tangent basis construction;
- radial elevation;
- line-width and circle-radius correction by latitude;
- globe/Mercator interpolation;
- horizon depth or fragment clipping;
- antimeridian fragment clipping;
- 3D projection that preserves depth for extrusions and custom geometry.

Layer shaders call the shared projection functions instead of embedding separate globe formulas.
Mercator shaders remain a distinct fast path. Pipeline specialization selects the projection
variant; a per-vertex branch is not used once a transition endpoint is reached.

## Camera and transform

The vertical-perspective transform implements the complete map transform contract:

- projection and inverse-projection matrices;
- camera position, near and far planes, pixel scale, and pixels per metre;
- longitude/latitude projection and screen unprojection;
- camera frustum and horizon clipping plane;
- bounds, camera altitude, camera longitude/latitude, and depth queries;
- location occlusion and screen-surface visibility;
- custom-layer projection data and light-direction transformation.

The globe transform delegates to Mercator or vertical perspective according to transition state and
keeps the two transforms synchronized. Terrain-driven center and zoom recalculation must not change
the geographic center merely because terrain becomes available.

Screen picking uses ray-sphere intersection. Pixels outside the planet are rejected for feature
queries. Drag panning switches before the silhouette to a slope-matched virtual trackball curve
that saturates short of the far side; ordinary screen-to-location queries clamp to the horizon.

## Camera interaction

Globe interaction uses quaternion orientation to avoid singular behavior at the poles. The camera
helper covers pan inertia, drag pan, combined roll/pitch/bearing/zoom controls, jump, ease, fly, and
bounds fitting.

Dragging keeps the grabbed surface location under the pointer while a ray intersects the globe. A
bounded fallback takes over near and outside the silhouette so panning remains continuous. A smooth
pole dial blends longitude from the quaternion swing to the cursor sweep within the last latitude
band, preserving bearing without stalling or reversing at a pole.

Zoom compensates for the latitude-dependent globe scale. Ease and fly interpolation use a globe
path whose apparent angular velocity is stable across latitude and the antimeridian.

## Tile covering and culling

Globe tile covering supplies projection-specific implementations for:

- distance from the camera center to a tile with X wrapping;
- wrap selection and optional world copies;
- variable zoom selection;
- convex tile bounding volumes including elevation;
- frustum and horizon culling;
- level-of-detail selection near the antimeridian and poles.

Bounding volumes enclose every subdivided surface point and the requested elevation range. Culling
must be conservative: it may retain a hidden tile but must not remove a visible one.

## Layer behavior

The projection path is shared, but layer semantics require separate acceptance tests:

- fills and outlines preserve seams, translations, patterns, gradients, and opacity;
- lines preserve screen-space width, dashes, patterns, gradients, joins, and antimeridian behavior;
- circles correct map-aligned radius and pitch behavior;
- symbols project anchors and glyphs, rotate tangent-aligned text, and reject horizon-occluded
  collision candidates even when overlap is allowed;
- raster and image sources use subdivided meshes and correct texture interpolation;
- hillshade and heatmap render in globe space;
- fill extrusions use radial elevation, tangent-space normals, lighting, and correct depth;
- backgrounds either draw tile meshes or the globe/atmosphere background path as required;
- debug, collision, and stencil passes use the same projection and clipping contract.

Queries, markers, and popups use transform occlusion results. A result behind the globe is absent or
faded according to the corresponding public API behavior.

## Atmosphere and sky

Atmosphere renders around the globe using camera position, globe radius, inverse projection, sun or
light direction, and atmosphere blend. It composes with sky, background opacity, and terrain. The
atmosphere pass runs only while globe rendering is active. Headless surface capture depends on the
translucent pass so golden images cannot be read before atmosphere composition completes.

## Terrain integration boundary

Globe and terrain will share the same subdivided surface. DEM elevation is sampled before radial
projection. A terrain implementation must provide globe-aware picking, tile bounds, depth,
fog/sky composition, and pole handling. Globe depth-prepass behavior must change when terrain
supplies the depth surface, preventing two competing globe depth representations.

The initial globe implementation must preserve a projection-independent terrain interface so a
terrain implementation is not embedded in the globe transform.

## Public and custom-layer behavior

Style loading accepts `globe` and preserves Mercator when projection is absent. Runtime projection
changes update transitions and emit the same category of projection transition event exposed by the
host API. Globe controls report availability from the active projection.

Custom layers receive matrices for globe and Mercator fallback, transition state, tile Mercator
coordinates, and clipping information. Existing custom layers that do not request globe matrices
continue to receive Mercator-compatible data.

## Error handling and resource ownership

Invalid projection style values fail style deserialization with contextual typed errors. Mesh or
buffer creation failures propagate through renderer initialization or upload errors. Projection GPU
resources are instance-owned; module-level mutable state is not introduced.

## Test and parity contract

The compatibility gate has four levels:

1. Pure math tests compare coordinate, orientation, horizon, zoom, interpolation, and ray/sphere
   invariants with MapLibre GL JS fixtures.
2. Transform tests cover matrices, projection/unprojection, bounds, occlusion, covering tiles,
   camera operations, and terrain picking.
3. Shader and mesh tests cover subdivision, shared edges, poles, antimeridian clipping, radial
   elevation, transition endpoints, and GPU precision-sensitive coordinates.
4. Render tests run 19 MapLibre GL JS `projection/globe` styles and exact expected images
   through the maplibre-rs headless renderer. The default normalized mean-channel tolerance is
   `0.02`; any exception is fixture-local and documents the upstream invariant. Terrain fixtures
   belong to the terrain follow-up.

Every audited render case has a matrix entry recording support, required renderer capability, and
the maplibre-rs evidence or blocker. A case marked implemented must run in the parity gate; blocked
cases cannot be silently relabeled as passing. The harness must apply test camera metadata,
projection transitions, source loading, and all style operations used by the selected corpus.

Mercator render tests run in the same CI job to guard the default path. Native and WebAssembly builds
must exercise the shared WGSL projection code.

## Delivery plan

Implementation is split into independently reviewable changes:

1. RFC and executable parity manifest.
2. Style projection model, pure globe mathematics, and vertical-perspective camera.
3. Subdivided meshes, covering tiles, poles, antimeridian clipping, and projection GPU data.
4. Active renderer integration for background, fill, line, raster, and atmosphere.
5. Camera-helper projection core and transform queries.
6. Imported GL JS golden harness and fixture-local comparison policy.
7. Symbol completion when the flat renderer supports the required placement behavior.
8. Runtime projection events, markers/popups, and custom layers when host APIs exist.
9. Native/WebAssembly performance and regression audit.

Terrain sampling, picking, radial elevation, depth, and globe-with-terrain render parity are planned
in the terrain RFC rather than this delivery sequence.

Each change includes the guardrail test that proves its behavior. A later change may depend on an
earlier one, but must not combine unrelated cleanup or resolve pre-existing repository warnings.

## MapLibre GL JS functional inventory

The parity audit covers the following MapLibre GL JS globe subsystems.

| Subsystem | Required behavior |
| --- | --- |
| Globe projection | transition state, shader and mesh selection, lifecycle, property evaluation |
| Globe transform | full transform delegation, synchronization, projection data, occlusion, queries |
| Vertical-perspective transform | matrices, camera state, clipping, covering, screen mapping, terrain raycast |
| Globe mathematics | coordinate conversion, vectors, orientation, pan, horizon, zoom, interpolation, raycast |
| Camera helpers | inertia, map controls, jump, ease, fly, bounds fitting |
| Tile covering | distance, wrap, variable zoom, bounding volumes, elevation, culling |
| Mesh generation | regular grids, borders, poles, polygon/line subdivision, winding, clipping |
| Shared shaders | sphere projection, elevation, transition, clipping, scale correction, 3D depth |
| Render integration | depth, raster, vector, symbols, atmosphere, sky, terrain, custom layers |
| UI and queries | controls, events, markers, popups, feature queries, world-copy behavior |

Pure globe functions tracked for parity are: globe circumference and radius; globe distance;
Mercator-to-angular conversion; angular-to-vector conversion; tile-to-sphere projection;
surface-vector conversion in both directions; orientation conversion in both directions; grabbed-
point panning; horizon circle extraction; sphere clamping; zoom adjustment; degrees per pixel;
globe pan center; globe interpolation; and ray-sphere intersection.

Transform-specific functions tracked for parity are: projection data; location occlusion; light
direction; pixel, circle, and pitched-text correction; tile-coordinate projection; fog matrix;
visible unwrapped coordinates; camera frustum; clipping plane; covering provider; zoom/center
recalculation; maximum pitch scale; camera point, altitude, and location; location depth; cache
population; bounds; camera-from-location; location-at-point; location-to-screen; screen-to-Mercator;
terrain screen picking; screen-to-location; surface hit testing; ray direction; custom-layer data;
and the fast-path matrix.

Mesh functions tracked for parity are: regular tile mesh creation; buffered mesh upload; polygon
subdivision; vertex-line subdivision; winding correction; scanline triangulation; pole quad
generation; pole fill; border clipping; and index conversion.

# Drawbacks

Globe parity substantially expands camera, mesh, shader, and test complexity. Subdivision increases
CPU work, vertex counts, GPU memory, and pipeline variants. Image parity across GPU backends requires
carefully chosen tolerances without hiding real seams or clipping errors.

Following MapLibre GL JS behavior also constrains some design freedom. The transition to Mercator,
axis convention, pole treatment, and interaction feel become compatibility requirements even when a
different design might be simpler in isolation.

# Rationale and alternatives

A projection contract is preferred over inserting globe conditionals throughout existing renderer
systems. It keeps Mercator fast, makes projection resource ownership explicit, and creates a place
for projection-specific meshes and shader data.

CPU-projecting every vertex was considered. It simplifies WGSL but prevents efficient camera and
elevation updates, duplicates projected buffers, and loses the precision strategy used by the
reference implementation. GPU projection with CPU subdivision is preferred.

Rendering a standalone sphere textured with a flat map was considered. It cannot preserve vector
layer semantics, symbol placement, feature queries, extrusion geometry, or terrain and therefore
does not meet the compatibility goal.

Shipping globe without the transition, interactions, or terrain was considered. Those pieces may be
delivered as intermediate steps of the implementation, but they are not an acceptable definition of feature
completion because the shared style and API would claim behavior the renderer does not provide.

# Prior art

MapLibre GL JS is the primary behavioral reference. Its implementation separates projection resource
selection, a globe transform, a vertical-perspective transform, camera helpers, globe covering-tile
details, shared projection shaders, mesh subdivision, and layer integrations. Its history is useful
for understanding why precision workarounds, pole meshes, antimeridian clipping, transition depth,
and terrain-specific paths exist.

The implementation and fixture audit are pinned to GL JS commit
`f9a40a5c4462abafd6823d9b6fc623246f31e787`. Relevant references include:

- [globe developer guide](https://maplibre.org/maplibre-gl-js/docs/book/developer-guides/globe/)
- [pinned globe projection source](https://github.com/maplibre/maplibre-gl-js/tree/f9a40a5c4462abafd6823d9b6fc623246f31e787/src/geo/projection)
- [globe shader source](https://github.com/maplibre/maplibre-gl-js/blob/main/src/shaders/glsl/_projection_globe.vertex.glsl)
- [tile subdivision source](https://github.com/maplibre/maplibre-gl-js/blob/main/src/render/subdivision.ts)
- [globe render tests](https://github.com/maplibre/maplibre-gl-js/tree/main/test/integration/render/tests/projection/globe)
- [initial globe implementation history](https://github.com/maplibre/maplibre-gl-js/pull/3963)
- [globe and terrain integration history](https://github.com/maplibre/maplibre-gl-js/pull/4977)

The implementation history is cited in the RFC and change descriptions. Source comments describe
current invariants rather than historical pull request numbers.

# Unresolved questions

- Should runtime projection changes be part of the first stable Rust API or initially style-only?
- Should the projection contract be public for plugins immediately or remain crate-private until
  globe parity stabilizes?
- What cache budget should projection-specific meshes use on mobile and WebAssembly?
- Which existing maplibre-rs camera APIs need additive results for failed globe intersections?

These questions affect API stability or measurable test policy and should be resolved during RFC
review. They do not remove any functional area from the compatibility target.

# Future possibilities

The projection contract can support additional projections after globe parity is stable. Adaptive
subdivision based on screen-space error may reduce geometry at high zoom. Compute-driven mesh
generation could move subdivision off the CPU. Ellipsoidal earth models may improve specialized
measurement use cases, but the rendered compatibility sphere remains the default.

Projection-independent terrain and camera contracts also make it possible to share more rendering
infrastructure with native MapLibre implementations while retaining a wgpu backend.
