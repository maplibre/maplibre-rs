const path = require("path");
const CopyPlugin = require("copy-webpack-plugin");
const WasmPackPlugin = require("@wasm-tool/wasm-pack-plugin");

let dist = path.join(__dirname, 'dist');
module.exports = {
    mode: "development",
    entry: {
        index: "./index.js"
    },
    experiments: {
        syncWebAssembly: true
    },
    performance: {
        maxEntrypointSize: 400000,
        maxAssetSize: 400000000,
    },
    output: {
        path: dist,
        filename: "[name].js"
    },
    devServer: {
        https: true,
        allowedHosts: 'all',
        host: '0.0.0.0',
        static: {
            directory: dist,
        },
        headers: {
            'Cross-Origin-Opener-Policy': 'same-origin',
            'Cross-Origin-Embedder-Policy': 'require-corp'
        },
    },
    plugins: [
        new CopyPlugin({
            patterns: [
                { from: "static", to: "." },
            ],
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
            extraArgs: '--target web -- --features web-webgl -Z build-std=std,panic_abort',

            // Optional array of absolute paths to directories, changes to which
            // will trigger the build.
            // watchDirectories: [
            //   path.resolve(__dirname, "another-crate/src")
            // ],

            // The same as the `--out-dir` option for `wasm-pack`
            outDir: path.resolve(__dirname, './libs/mapr'),

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
            // forceMode: "development",

            // Controls plugin output verbosity, either 'info' or 'error'.
            // Defaults to 'info'.
            // pluginLogLevel: 'info'
        }),
    ]
};