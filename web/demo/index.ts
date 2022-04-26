import { startMapLibre } from 'maplibre-rs'

if (process.env.CJS) {
    // When bundling a CJS library, webpack can not know where to find the wasm file or the WebWorker. So we need to
    // find it manually and then pass it down.
    const maplibreWasm = require('file-loader!maplibre_rs/dist/esbuild-cjs/assets/index_bg.wasm')
    startMapLibre(maplibreWasm.default, undefined)
} else {
    startMapLibre(undefined, undefined)
}
