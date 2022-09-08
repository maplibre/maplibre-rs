import init, {create_pool_scheduler, run} from "../wasm/maplibre"
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

    await init(wasmPath);

    const schedulerPtr = create_pool_scheduler(() => {
        return workerPath ? new Worker(workerPath, {
            type: 'module'
        }) : PoolWorker();
    })

    await run(schedulerPtr)
}
