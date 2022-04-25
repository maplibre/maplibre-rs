const path = require("path");
const HtmlWebpackPlugin = require('html-webpack-plugin');
const CopyPlugin = require("copy-webpack-plugin");

let dist = path.join(__dirname, 'dist/');
module.exports = (_env) => ({
    mode: "development",
    entry: {
        main: "./index.ts",
    },
    experiments: {
        asyncWebAssembly: true
    },
    performance: {
        maxEntrypointSize: 400000,
        maxAssetSize: 400000000,
    },
    output: {
        path: dist,
        filename: "[name].[fullhash].js"
    },
    devServer: {
        server: {
            type: 'http',
        },
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
    module: {
        rules: [
            {
                test: /\.ts$/,
                use: 'ts-loader',
                exclude: /node_modules/,
            },
        ],
    },
    resolve: {
        extensions: ['.ts', '.js'],
    },
    plugins: [
        /*new CopyPlugin({
            patterns: [
                // webpack
                //{ from: "*.wasm", to: "[path][name][ext]", context: 'node_modules/maplibre_rs/dist/maplibre-rs/' },
                //{ from: "*.maplibre-rs.js", to: "[path][name][ext]", context: 'node_modules/maplibre_rs/dist/maplibre-rs/' },
                // parcel
                {from: "*.wasm", to: "[path]maplibre[ext]", context: 'node_modules/maplibre_rs/dist/parcel-cjs/'},
                {from: "*worker*", to: "[path]worker[ext]", context: 'node_modules/maplibre_rs/dist/parcel-cjs/'},
            ],
        }),*/
        new HtmlWebpackPlugin({
            title: 'maplibre demo',
        }),
    ]
});