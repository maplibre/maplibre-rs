import { build } from 'esbuild';
import metaUrlPlugin from '@chialab/esbuild-plugin-meta-url';
import inlineWorker from 'esbuild-plugin-inline-worker';
import envPlugin from '@chialab/esbuild-plugin-env';

let baseSettings = {
    entryPoints: ['src/index.ts'],
    bundle: true,
    platform: "browser",
    assetNames: "assets/[name]",
    plugins: [
        inlineWorker({
            format: "cjs", platform: "browser",
            target: 'es2022',
            bundle: true,
            assetNames: "assets/[name]",
        }),
        metaUrlPlugin(),
        envPlugin()
    ],
};

const start = async() => {
    await build({...baseSettings, format: "esm", outfile: "dist/esbuild-esm/module.js",});
    await build({...baseSettings, format: "cjs", outfile: "dist/esbuild-cjs/main.js",});
    await build({...baseSettings, format: "iife", outfile: "dist/esbuild-iffe/main.js", globalName: "maplibre"});
}

start()