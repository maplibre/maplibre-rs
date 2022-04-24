const path = require("path");
const webpack = require("webpack");
const HtmlWebpackPlugin = require('html-webpack-plugin');
const CopyPlugin = require("copy-webpack-plugin");

let dist = path.join(__dirname, 'dist/');
module.exports = (env) => ({
    mode: "development",
    entry: {
        main: "./index.ts",
    },
    experiments: {
        //syncWebAssembly: true
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
                test: /\.tsx?$/,
                use: 'ts-loader',
                exclude: /node_modules/,
            },
        ],
    },
    resolve: {
        extensions: ['.tsx', '.ts', '.js'],
    },
    plugins: [
        new CopyPlugin({
            patterns: [
                { from: "*.wasm", to: "[path][name][ext]", context: 'node_modules/maplibre_rs/dist/maplibre-rs/' },
                { from: "*.maplibre-rs.js", to: "[path][name][ext]", context: 'node_modules/maplibre_rs/dist/maplibre-rs/' },
            ],
        }),
        new webpack.DefinePlugin({
            WEBGL: !!env.webgl
        }),
        new HtmlWebpackPlugin({
            title: 'maplibre demo',
        }),
    ]
});