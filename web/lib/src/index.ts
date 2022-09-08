import {create_pool_scheduler, run} from "./wasm/maplibre"
import {Spector} from "spectorjs"
// @ts-ignore esbuild plugin is handling this
import MultithreadedPoolWorker from './multithreaded-pool.worker.js';
// @ts-ignore esbuild plugin is handling this
import PoolWorker from './pool.worker.js';

import {initialize} from "./module";
import {checkRequirements, checkWasmFeatures} from "./browser";

const preventDefaultTouchActions = () => {
    document.body.querySelectorAll("canvas").forEach(canvas => {
        canvas.addEventListener("touchstart", e => e.preventDefault())
        canvas.addEventListener("touchmove", e => e.preventDefault())
    })
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

    await initialize(wasmPath);

    const schedulerPtr = create_pool_scheduler(() => {
        let CurrentWorker = MULTITHREADED ? MultithreadedPoolWorker : PoolWorker;

        return workerPath ? new Worker(workerPath, {
            type: 'module'
        }) : CurrentWorker();
    })

    await run(schedulerPtr)
}
