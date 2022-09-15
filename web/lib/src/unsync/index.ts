import init, {run, create_map, clone_map, unsync_main_entry} from "../wasm/maplibre"
import {Spector} from "spectorjs"
import {checkRequirements, checkWasmFeatures} from "../browser";
import {preventDefaultTouchActions} from "../canvas";
// @ts-ignore esbuild plugin is handling this
import PoolWorker from './pool.worker.js';

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
    let MEMORY_PAGES = 16 * 1024
    const memory = new WebAssembly.Memory({initial: 1024, maximum: MEMORY_PAGES, shared: false})
    await init(wasmPath, memory);

    let callback = [undefined]

    let map = await create_map(() => {
        let worker = workerPath ? new Worker(workerPath, {
            type: 'module'
        }) : PoolWorker();

        worker.onmessage =  (message) => {
            callback[0](message)
        }

        return worker;
    })

    let clonedMap = clone_map(map)

    callback[0] = (message) => {
        unsync_main_entry(clonedMap, message.data[0], new Uint8Array(message.data[1]))
    }

    run(map)
}
