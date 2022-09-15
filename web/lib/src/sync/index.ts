import {run} from "../wasm/maplibre"
import {Spector} from "spectorjs"
import {checkRequirements, checkWasmFeatures} from "../browser";
import init from "../wasm/maplibre";
import {preventDefaultTouchActions} from "../canvas";
// @ts-ignore esbuild plugin is handling this
import PoolWorker from './multithreaded-pool.worker.js';

const initializeSharedModule = async (wasmPath) => {
    let MEMORY_PAGES = 16 * 1024

    const memory = new WebAssembly.Memory({initial: 1024, maximum: MEMORY_PAGES, shared: true})
    // @ts-ignore
    await init(wasmPath, memory)
}

export const startMapLibre = async (wasmPath: string | undefined, workerPath: string | undefined) => {
    await checkWasmFeatures()

    let message = checkRequirements();
    if (message) {
        console.error(message)
        alert(message)
        return
    }

    if (WEBGL) {
        let spector = new Spector()
        spector.displayUI()
    }

    preventDefaultTouchActions();

    await initializeSharedModule(wasmPath);

    await run(() => {
        return workerPath ? new Worker(workerPath, {
            type: 'module'
        }) : PoolWorker();
    })
}
