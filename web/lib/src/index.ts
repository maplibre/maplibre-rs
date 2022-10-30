import * as maplibre from "./wasm/maplibre"
import {Spector} from "spectorjs"
import {checkRequirements, checkWasmFeatures} from "./browser";
import {preventDefaultTouchActions} from "./canvas";
// @ts-ignore esbuild plugin is handling this
import MultithreadedPoolWorker from './multithreaded/multithreaded-pool.worker.js';
// @ts-ignore esbuild plugin is handling this
import PoolWorker from './singlethreaded/pool.worker.js';

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

    if (MULTITHREADED) {
        const MEMORY = 209715200; // 200MB
        const PAGES = 64 * 1024;

        const memory = new WebAssembly.Memory({initial: 1024, maximum: MEMORY / PAGES, shared: true})
        await maplibre.default(wasmPath, memory)

        let init_result = await maplibre.init_maplibre(() => {
            return workerPath ? new Worker(workerPath, {
                type: 'module'
            }) : MultithreadedPoolWorker();
        });
        maplibre.run(init_result)
    } else {
        const memory = new WebAssembly.Memory({initial: 1024, shared: false})
        await maplibre.default(wasmPath, memory);

        let callbacks: {worker_callback?: (message: MessageEvent) => void} = {}

        let init_result = await maplibre.init_maplibre(() => {
            let worker: Worker = workerPath ? new Worker(workerPath, {
                type: 'module'
            }) : PoolWorker();

            worker.onmessage = (message: MessageEvent) => {
                callbacks.worker_callback(message)
            }

            return worker;
        })

        // MAJOR FIXME: cloning is not possible let clonedMap = maplibre.clone_map(init_result)

        /*callbacks.worker_callback = (message) => {
            let tag = message.data[0];
            let data = new Uint8Array(message.data[1]);

            // @ts-ignore TODO unsync_main_entry may not be defined
            maplibre.singlethreaded_main_entry(clonedMap, tag, data)
        }*/

        maplibre.run(init_result)
    }
}
