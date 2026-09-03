# Imported MapLibre GL JS globe fixtures

These render tests are copied from MapLibre GL JS,
`test/integration/render/tests/projection/globe` at commit
`f9a40a5c4462abafd6823d9b6fc623246f31e787`, together with the tile assets they reference under
`render-tests/src/assets`. Each `style.json` names its upstream directory in
`metadata.test.source`, and every `expected.png` is byte-identical to upstream.

MapLibre GL JS is distributed under the BSD-3-Clause license; see
<https://github.com/maplibre/maplibre-gl-js/blob/main/LICENSE.txt> for the full text and copyright
holders. The fixtures and assets are redistributed here under that license.

Harness-local comparison policy lives in each fixture's `metadata.test` object: `max-diff`,
`comparison-background`, and `comparison-note`.
