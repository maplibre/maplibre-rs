import {build} from 'esbuild';
import metaUrlPlugin from '@chialab/esbuild-plugin-meta-url';
import inlineWorker from 'esbuild-plugin-inline-worker';
import yargs from "yargs";
import process from "process";
import chokidar from "chokidar";
import {spawnSync} from "child_process"
import {dirname} from "path";
import {fileURLToPath} from "url";

let argv = yargs(process.argv.slice(2))
    .option('watch', {
        type: 'boolean',
        description: 'Enable watching'
    })
    .option('release', {
        type: 'boolean',
        description: 'Release mode'
    })
    .option('webgl', {
        type: 'boolean',
        description: 'Enable webgl'
    })
    .option('multithreaded', {
        type: 'boolean',
        description: 'Enable multithreaded support'
    })
    .option('esm', {
        type: 'boolean',
        description: 'Enable esm'
    })
    .option('cjs', {
        type: 'boolean',
        description: 'Enable cjs'
    })
    .option('iife', {
        type: 'boolean',
        description: 'Enable iife'
    })
    .parse()

let esm = argv.esm;
let iife = argv.iife;
let cjs = argv.cjs;
let release = argv.release;
let multithreaded = argv.multithreaded;

if (!esm && !iife && !cjs) {
    console.warn("Enabling ESM bundling as no other bundle is enabled.")
    esm = true;
}

let webgl = argv.webgl;

if (webgl) {
    console.log("WebGL support enabled.")
}

if (multithreaded) {
    console.log("multithreaded support enabled.")
}

let baseConfig = {
    platform: "browser",
    bundle: true,
    assetNames: "assets/[name]",
    define: {
        WEBGL: `${webgl}`,
        MULTITHREADED: `${multithreaded}`
    },
}

let config = {
    ...baseConfig,
    entryPoints:['src/index.ts'],
    incremental: argv.watch,
    plugins: [
        inlineWorker({
            ...baseConfig,
            format: "cjs",
            target: 'es2022',
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
        "--declaration",
        "--emitDeclarationOnly"
    ], {
        cwd: '.',
        stdio: 'inherit',
    });

    if (child.status !== 0) {
        throw new Error("Failed to execute tsc")
    }
}

// TODO: Do not continue if one step fails
const wasmPack = () => {
    let outDirectory = `${getLibDirectory()}/src/wasm`;
    let profile = release ? "wasm-release" : "wasm-dev"

    // language=toml
    let multithreaded_config = `target.wasm32-unknown-unknown.rustflags = [
        # Enables features which are required for shared-memory
        "-C", "target-feature=+atomics,+bulk-memory,+mutable-globals",
        # Enables the possibility to import memory into wasm.
        # Without --shared-memory it is not possible to use shared WebAssembly.Memory.
        # Set maximum memory to 200MB
        "-C", "link-args=--shared-memory --import-memory --max-memory=209715200"
    ]`

    let cargo = spawnSync('cargo', [
        ...(multithreaded ? ["--config", multithreaded_config] : []),
        "build",
        "-p", "web", "--lib",
        "--target", "wasm32-unknown-unknown",
        "--profile", profile,
        "--features", `${webgl ? "web-webgl," : ""}`,
        ...(multithreaded ? ["-Z", "build-std=std,panic_abort"] : []),
    ], {
        cwd: '.',
        stdio: 'inherit',
    });

    if (cargo.status !== 0) {
        throw new Error("Failed to execute cargo build")
    }

    let wasmbindgen = spawnSync('wasm-bindgen', [
        `${getProjectDirectory()}/target/wasm32-unknown-unknown/${profile}/web.wasm`,
        "--out-name", "maplibre",
        "--out-dir", outDirectory,
        "--typescript",
        "--target", "web",
        "--debug",
    ], {
        cwd: '.',
        stdio: 'inherit',
    });

    if (wasmbindgen.status !== 0) {
        throw new Error("Failed to execute wasm-bindgen")
    }

    if (release) {
        console.log("Running wasm-opt")
        let wasmOpt = spawnSync('npm', ["exec",
            "wasm-opt", "--",
            `${outDirectory}/maplibre_bg.wasm`,
            "-o", `${outDirectory}/maplibre_bg.wasm`,
            "-O"
        ], {
            cwd: '.',
            stdio: 'inherit',
        });

        if (wasmOpt.status !== 0) {
            throw new Error("Failed to execute wasm-opt")
        }
    }
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
        try {
            console.log(`Updating: ${path}`)
            if (path.endsWith(".rs")) {
                console.log("Rebuilding Rust...")
                wasmPack();
            }

            console.log("Rebuilding...")
            await result.rebuild();

            console.log("Emitting TypeScript types...")
            emitTypeScript();
        } catch (e) {
            console.error("Error while updating:")
            console.error(e)
        }
    }

    console.log("Watching...")
    watcher
        .on('ready', () => console.log('Initial scan complete. Ready for changes'))
        .on('add', update)
        .on('change', update)
        .on('unlink', update);
}

const esbuild = async (name, globalName = undefined) => {
    let result = await build({...config, format: name, globalName, outfile: `dist/esbuild-${name}/module.js`,});

    if (argv.watch) {
        console.log("Watching is enabled.")
        await watchResult(result)
    }
}

const start = async () => {
    try {
    console.log("Creating WASM...")
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
    } catch (e) {
        console.error("Failed to start building: " + e.message)
        process.exit(1)
    }
}

const _ = start()
