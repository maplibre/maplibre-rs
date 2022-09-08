import {build} from 'esbuild';
import metaUrlPlugin from '@chialab/esbuild-plugin-meta-url';
import inlineWorker from 'esbuild-plugin-inline-worker';
import yargs from "yargs";
import chokidar from "chokidar";
import {spawnSync} from "child_process"
import {unlink} from "fs";
import {dirname} from "path";
import {fileURLToPath} from "url";

let argv = yargs(process.argv.slice(2))
    .option('watch', {
        alias: 'w',
        type: 'boolean',
        description: 'Enable watching'
    })
    .option('webgl', {
        alias: 'g',
        type: 'boolean',
        description: 'Enable webgl'
    })
    .option('esm', {
        alias: 'e',
        type: 'boolean',
        description: 'Enable esm'
    })
    .option('cjs', {
        alias: 'c',
        type: 'boolean',
        description: 'Enable cjs'
    })
    .option('iife', {
        alias: 'i',
        type: 'boolean',
        description: 'Enable iife'
    })
    .parse()

let esm = argv.esm;
let iife = argv.iife;
let cjs = argv.cjs;

if (!esm && !iife && !cjs) {
    console.warn("Enabling ESM bundling as no other bundle is enabled.")
    esm = true;
}

let webgl = argv.webgl;

if (webgl) {
    console.log("WebGL support enabled.")
}

let baseSettings = {
    entryPoints: ['src/index.ts'],
    bundle: true,
    platform: "browser",
    assetNames: "assets/[name]",
    define: {"WEBGL": `${webgl}`},
    incremental: argv.watch,
    plugins: [
        inlineWorker({
            format: "cjs", platform: "browser",
            target: 'es2022',
            bundle: true,
            assetNames: "assets/[name]",
        }),
        metaUrlPlugin()
    ],
};

/***
 * @returns {string} The path to the project
 */
const getProjectDirectory = () => {
    return `${getWebDirectory()}/..`
}

/***
 * @returns {string} The path to <project>/web
 */
const getWebDirectory = () => {
    return `${getLibDirectory()}/..`
}

/***
 * @returns {string} The path to <project>/web/lib
 */
const getLibDirectory = () => {
    return dirname(fileURLToPath(import.meta.url))
}

const emitTypeScript = () => {
    let outDirectory = `${getLibDirectory()}/dist/ts`;

    let child = spawnSync('npm', ["exec",
        "tsc",
        "--",
        "-m", "es2022",
        "-outDir", outDirectory,
        "--emitDeclarationOnly"

    ], {
        cwd: '.',
        stdio: 'inherit',
    });

    if (child.status !== 0) {
        console.error("Failed to execute tsc")
        process.exit(1)
    }
}

const wasmPack = () => {
    let outDirectory = `${getLibDirectory()}/src/wasm-pack`;

    let child = spawnSync('npm', ["exec",
        "wasm-pack","--",
        "build",
        "--out-name", "index",
        "--out-dir", outDirectory,
        getWebDirectory(),
        "--target", "web",
        "--",
        "--features", `${webgl ? "web-webgl" : ""}`,
        "-Z", "build-std=std,panic_abort"
    ], {
        cwd: '.',
        stdio: 'inherit',
    });

    if (child.status !== 0) {
        console.error("Failed to execute wasm-pack")
    }

    // Having package.json within another npm package is not supported. Remove that.
    unlink(`${getLibDirectory()}/src/wasm-pack/package.json`, (err) => {
        if (err) throw err;
    })
}

const watchResult = async (result) => {
    const watcher = chokidar.watch(['**/*.ts', '**/*.js', '**/*.rs'], {
        cwd: getProjectDirectory(),
        ignored: /dist|node_modules|target/,
        ignoreInitial: true,
        disableGlobbing: false,
        followSymlinks: false,
    });

    const update = async (path) => {
        console.log(`Updating: ${path}`)
        if (path.endsWith(".rs")) {
            console.log("Rebuilding Rust...")
            wasmPack();
        }

        console.log("Rebuilding...")
        await result.rebuild();

        console.log("Emitting TypeScript types...")
        emitTypeScript();
    }

    console.log("Watching...")
    watcher
        .on('ready', () => console.log('Initial scan complete. Ready for changes'))
        .on('add', update)
        .on('change', update)
        .on('unlink', update);
}

const esbuild = async (name, globalName = undefined) => {
    let result = await build({...baseSettings, format: name, globalName, outfile: `dist/esbuild-${name}/module.js`,});

    if (argv.watch) {
        console.log("Watching is enabled.")
        await watchResult(result)
    }
}

const start = async () => {
    console.log("Running wasm-pack...")
    wasmPack();

    if (esm) {
        console.log("Building esm bundle...")
        await esbuild("esm")
    }

    if (cjs) {
        console.log("Building cjs bundle...")
        await esbuild("cjs")
    }

    if (iife) {
        console.log("Building iife bundle...")
        await esbuild("iife", "maplibre")
    }

    console.log("Emitting TypeScript types...")
    emitTypeScript();
}

const _ = start()
