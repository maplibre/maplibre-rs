import {build} from 'esbuild';
import metaUrlPlugin from '@chialab/esbuild-plugin-meta-url';
import inlineWorker from 'esbuild-plugin-inline-worker';
import yargs from "yargs";
import process from "process";
import {spawnSync} from "child_process"
import {dirname} from "path";
import {fileURLToPath} from "url";

let argv = yargs(process.argv.slice(2))
    .strict(true)
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
    minify: release,
    assetNames: "assets/[name]",
    define: {
        WEBGL: `${webgl}`,
        MULTITHREADED: `${multithreaded}`
    },
}

let config = {
    ...baseConfig,
    entryPoints: ['src/index.ts'],
    plugins: [
        inlineWorker({
            ...baseConfig,
            format: "cjs",
            target: 'es2022',
            // workerName: 'worker' Supported when the follow commit is released: https://github.com/mitschabaude/esbuild-plugin-inline-worker/commit/d1aaffc721a62a3fe33f59f8f69b462c7dd05f45
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

    let child = spawnTool('npm', ["exec",
        "tsc",
        "--",
        "-m", "es2022",
        "-outDir", outDirectory,
        "--declaration",
        "--emitDeclarationOnly"
    ]);

    if (child.status !== 0) {
        throw new Error("Failed to execute tsc")
    }
}

const spawnTool = (program, args) => {
    console.debug(`Executing: ${program} ${args.join(" ")}`)
    return spawnSync(program, args, {
        cwd: '.',
        stdio: 'inherit',
    })
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

    spawnTool('cargo', ["--version"])

    let cargo = spawnTool('cargo', [
        ...(multithreaded ? ["--config", multithreaded_config] : []),
        "build",
        "-p", "web", "--lib",
        "--target", "wasm32-unknown-unknown",
        "--profile", profile,
        "--features", `${webgl ? "web-webgl," : ""}`,
        ...(multithreaded ? ["-Z", "build-std=std,panic_abort"] : []),
    ]);

    if (cargo.status !== 0) {
        throw new Error("Failed to execute cargo build")
    }

    let wasmbindgen = spawnTool('wasm-bindgen', [
        `${getProjectDirectory()}/target/wasm32-unknown-unknown/${profile}/web.wasm`,
        "--out-name", "maplibre",
        "--out-dir", outDirectory,
        "--typescript",
        "--target", "web",
        "--debug",
    ]);

    if (wasmbindgen.status !== 0) {
        throw new Error("Failed to execute wasm-bindgen")
    }

    if (release) {
        console.log("Running wasm-opt")
        let wasmOpt = spawnTool('npm', ["exec",
            "wasm-opt", "--",
            `${outDirectory}/maplibre_bg.wasm`,
            "-o", `${outDirectory}/maplibre_bg.wasm`,
            "-O"
        ]);

        if (wasmOpt.status !== 0) {
            throw new Error("Failed to execute wasm-opt")
        }
    }
}

const esbuild = async (name, globalName = undefined) => {
    let result = await build({...config, format: name, globalName, outfile: `dist/esbuild-${name}/module.js`,});
    console.log(result.errors.length === 0 ? "No errors." : "Found errors.")
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
