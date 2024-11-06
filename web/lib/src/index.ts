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
        const MEMORY = 900 * 1024 * 1024; // 900 MB
        const PAGES = 64 * 1024;

        const memory = new WebAssembly.Memory({initial: 1024, maximum: MEMORY / PAGES, shared: true})
        await maplibre.default(wasmPath, memory)

        await maplibre.run_maplibre(() => {
            return workerPath ? new Worker(workerPath, {
                type: 'module',
            }) : MultithreadedPoolWorker();
        });
    } else {
        const memory = new WebAssembly.Memory({initial: 1024, shared: false})
        await maplibre.default(wasmPath, memory);

        await maplibre.run_maplibre((received_ptr: number) => {
            let worker: Worker = workerPath ? new Worker(workerPath, {
                type: 'module',
            }) : PoolWorker();  // Setting a "name" for this webworker is not yet supported, because it needs support from esbuild-plugin-inline-worker

            // Handle messages coming back from the Worker
            worker.onmessage = (message: MessageEvent<[tag: number, buffer: ArrayBuffer]>) => {
                // WARNING: Do not modify data passed from Rust!
                let data = message.data;

                const receive_data: (received_ptr: number, tag: number, buffer: ArrayBuffer) => void = maplibre["singlethreaded_receive_data"];

                if (!receive_data) {
                    throw Error("singlethreaded_main_entry is not defined. Maybe the Rust build used the wrong build configuration.")
                }

                receive_data(received_ptr, data[0], data[1])
            }

            return worker;
        });
    }
}
