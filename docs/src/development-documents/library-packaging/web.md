# Web

This document describes issues and challenges when packaging maplibre-rs as a npm package.

## Required Formats

### ESM

The ESM module format is the standard nowadays which should be followed. If a JS bundler encounters an ESM
module it can resolve WebAssembly files or WebWorkers dynamically.
The following syntax is used to resolve referenced WebWorkers:

```ts
new Worker(new URL("./multithreaded-pool.worker.ts", import.meta.url), {
    type: 'module'
});
```

Similarly, the following works:

```ts
new URL('index_bg.wasm', import.meta.url);
```

### IIFE (immediately-invoked function expression)

> This format is used when including maplibre-rs in a `<script>` tag. The library is "written" onto the window/global
> object. This allows quick prototyping/playgrounds/experiments using maplibre-rs.

In order to support this we need to create a bundle which works on any modern browser. Additionally, a WASM file and
WebWorker needs to be deployed at a predictable path, because there is no bundler active which manages assets. Users of
these libraries have to specify where WASM or non-inlined WebWorkers are located.

Both assets could be inlined theoretically. This is common for WebWorkers, but not for WASM files.

### UMD

> UMD modules are needed when creating a library which should run in Node as well as browsers. This is not a use case
> for maplibre-rs. If we support node, then we probably would ship a separate package called "maplibre-rs-node" which
> bundles to CJS directly.

### CJS/CommonJS

> Not needed for the browser build of maplibre-rs, possibly needed when supporting Node

With a CommonJS module its is not possible for bundlers to dynamically resolve WebWorkers or WASM files.

The `import.meta.url` token can not exist in a CommonJS module. Therefore, bundlers which encounter a CommonJS module
have to use a different mechanism of resolving files.

Generally, we do not need to support CommonJS, because we are not targeting Node with maplibre-rs. It's properly good to
support it as a fallback though, for bundlers which can not deal with ESM modules yet.
This is for example true for test runners like Jest which require that dependencies are available as CJS module.

## wasm-pack output

wasm-pack can output [multiple formats](https://rustwasm.github.io/docs/wasm-pack/commands/build.html#target). The `web`
and `bundler` outputs offer the most modular modules.
Unfortunately, the
function [wasm_bindgen::module()](https://docs.rs/wasm-bindgen/0.2.80/src/wasm_bindgen/lib.rs.html#1208-1217)
is only supported in `web` and `no-modules`. We currently are using this in order to send loaded instances
of `WebAssembly.Module` to WebWorkers. `nodejs` should not be used because MapLibre does not target Node.
Therefore, we should stick to the `web` output format.

## Required Features

* WASM Bundling: Make the WASM binary available to users of the maplibre-rs library
* WebWorker Bundling: Make the WebWorker available to users of the maplibre-rs library. This could also be achived by inlining.
* WebWorker Inlining: Inline the WebWorker bundle in the library bundle as a string.
* Predictable Paths: Without predictable paths, it's difficult for users to reference the wasm file directly from the `node_modules` directory if requried.


## Bundler Feature Comparison

| Bundler       | *ESM* | *IIFE* | CJS | UMD | *WebWorker Inlining* | Web Worker Bundling | *WASM Bundling* | *Predictable Paths* | Inlining Environment Variables |
|---------------|-------|--------|-----|-----|----------------------|---------------------|-----------------|---------------------|--------------------------------|
| Babel 1)      | ✅     | ❌      | ❌   | ❌   | ❌                    | ❌                   | ❌               | ✅                   | ✅                              |
| TypeScript 1) | ✅     | ❌      | ❌   | ❌   | ❌                    | ❌                   | ❌               | ✅                   | ❌                              |
| Webpack       | ❌ 4)  | ❓      | ❌   | ❓   | ❌ 2)                 | ✅                   | ✅               | ❓                   | ✅                              |
| Parcel        | ✅     | ❌      | ✅   | ❌   | 🛠️ 3)               | ✅                   | ✅               | ❌ 5)                | ✅                              |
| ESBuild       | ✅     | ✅      | ✅   | ❌   | ✅ 6)                 | ❓                   | ✅ 6)            | ✅                   | ✅                              |
| Rollup        | ❓     | ❓      | ❓   | ❓   | ❓                    | ❓                   | ❓               | ❓                   | ✅                              |

Features in ***italic***s are required for maplibre-rs.

> 1) Technically not a bundler but can be used to emit ES modules
> 2) Was Supported in Webpack 4, but currently is not supported
> 3) https://github.com/parcel-bundler/parcel/issues/8004
> 4) As of the time of writing Webpack can not output ESM libraries
> 5) Plugins exist, but they don't work reliably
> 6) Plugins exist, and work reliably

### ESBuild

ESBuild supports CJS, ESM and IIFI modules equally well. Plugins exist for WebWorker inlining and resolving assets
through `import.meta.url`. The plugin quality seems to be superior compared to Parcel. It is also very fast compared to
all other bundlers.

* IIFI: The esbuild bundler translates to `new URL('index_bg.wasm', import.meta.url);` to
  ```js
  var __currentScriptUrl__ = document.currentScript && document.currentScript.src || document.baseURI;
  new URL("./assets/index_bg.wasm?emit=file", __currentScriptUrl__);
  ```

See config in `web/lib/build.mjs` for an example usage.

### Babel & TypeScript

Babel and TypeScript both can produce ESM modules, but they **fail with transforming references within the source code**
like `new URL("./multithreaded-pool.worker.ts", import.meta.url)`. There exist some Babel plugins, but none of them is stable.
Therefore, we actually need a proper bundler which supports outputting ESM modules.
The only stable solution to this is Parcel. Parcel also has good documentation around the bundling of WebWorkers.

### WebPack

WebPack supports older module formats like CommonJS or UMD very well. It falls short when bundling the format ESM
format which is not yet stable. It also does not support inlining WebWorkers in version 5. The wasm-pack plugin
for WebPack makes including Cargo projects easy.

* CJS: Webpack translates `new URL('index_bg.wasm', import.meta.url);` to something that is equivalent to `'./index_bg.wasm'`
  . It just expects that assets are resolvable from the current file.

Example scripts for `package.json`:

```json
{
  "scripts": {
    "webpack": "webpack --mode=development",
    "webpack-webgl": "npm run build -- --env webgl",
    "webpack-production": "webpack --mode=production",
    "webpack-webgl-production": "npm run production-build -- --env webgl"
  }
}
```

Example config:

```js
const path = require("path");
const webpack = require("webpack");
const WasmPackPlugin = require("@wasm-tool/wasm-pack-plugin");

let dist = path.join(__dirname, 'dist/maplibre-rs');
module.exports = (env) => ({
    mode: "development",
    entry: "./src/index.ts",
    experiments: {
        syncWebAssembly: true,
    },
    performance: {
        maxEntrypointSize: 400000,
        maxAssetSize: 400000000,
    },
    output: {
        path: dist,
        filename: "maplibre-rs.js",
        library: {
            name: 'maplibre_rs',
            type: 'umd',
        },
    },
    module: {
        rules: [
            {
                test: /\.ts$/,
                exclude: /node_modules/,
                use: [
                    {
                        loader: 'ts-loader',
                        options: {}
                    }
                ]
            },
        ],
    },
    resolve: {
        extensions: ['.ts', '.js'],
    },
    plugins: [
        new webpack.DefinePlugin({
            'process.env.WEBGL': !!env.webgl
        }),
        new WasmPackPlugin({
            crateDirectory: path.resolve(__dirname, '../'),

            // Check https://rustwasm.github.io/wasm-pack/book/commands/build.html for
            // the available set of arguments.
            //
            // Optional space delimited arguments to appear before the wasm-pack
            // command. Default arguments are `--verbose`.
            //args: '--log-level warn',
            // Default arguments are `--typescript --target browser --mode normal`.
            extraArgs: ` --target web -- . -Z build-std=std,panic_abort ${env.webgl ? '--features web-webgl' : ''} ${env.tracing ? '--features trace' : ''}`,

            // Optional array of absolute paths to directories, changes to which
            // will trigger the build.
            // watchDirectories: [
            //   path.resolve(__dirname, "another-crate/src")
            // ],

            // The same as the `--out-dir` option for `wasm-pack`
            outDir: path.resolve(__dirname, 'src/wasm-pack'),

            // The same as the `--out-name` option for `wasm-pack`
            // outName: "index",

            // If defined, `forceWatch` will force activate/deactivate watch mode for
            // `.rs` files.
            //
            // The default (not set) aligns watch mode for `.rs` files to Webpack's
            // watch mode.
            // forceWatch: true,

            // If defined, `forceMode` will force the compilation mode for `wasm-pack`
            //
            // Possible values are `development` and `production`.
            //
            // the mode `development` makes `wasm-pack` build in `debug` mode.
            // the mode `production` makes `wasm-pack` build in `release` mode.
            // forceMode: "production",

            // Controls plugin output verbosity, either 'info' or 'error'.
            // Defaults to 'info'.
            // pluginLogLevel: 'info'
        }),
    ]
});
```

### Parcel

Parcel supports CommonJS and ESM modules equally good. The documentation about `import.meta.url` is very good. In other
bundlers documentations around this feature is missing. In the latest Parcel version inlining WebWorkers is not working.

* CJS: The Parcel bundler translates to `new URL('index_bg.wasm', import.meta.url);`
  to `new URL("index_bg.wasm", "file:" + __filename);`
  While depending on `file:` and `filename` works for NodeJS, it is unsupported in the browser.

Example scripts for `package.json`:

```json
{
  "scripts": {
    "parcel": "npm run clean && npm run wasm-pack && WEBGL=false parcel build --no-cache src/index.ts",
    "parcel-webgl": "npm run clean && FEATURES=web-webgl npm run wasm-pack && WEBGL=true parcel build --no-cache src/index.ts"
  }
}
```

Example config in `package.json:

```json
{
  "module": "dist/parcel-esm/module.js",
  "main": "dist/parcel-cjs/main.js",
  "types": "dist/parcel/types.d.ts",
  "targets": {
    "main": {
      "distDir": "./dist/parcel-cjs",
      "context": "browser",
      "outputFormat": "commonjs"
    },
    "module": {
      "distDir": "./dist/parcel-esm",
      "context": "browser",
      "outputFormat": "esmodule"
    }
  },
  "@parcel/transformer-js": {
    "inlineFS": false,
    "inlineEnvironment": [
      "WEBGL"
    ]
  }
}
```

### Rollup

Not yet evaluated
