import init, {create_scheduler, run} from "../wasm/maplibre"
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

    const schedulerPtr = create_scheduler(() => {
        let worker = workerPath ? new Worker(workerPath, {
            type: 'module'
        }) : PoolWorker();

        let memories =  []

        worker.onmessage = (message) => {
            console.warn("new message");
            //let uint8Array = new Uint8Array(message.data[0], message.data[1]);

            memories.push(message.data[0])


            console.warn(memories.map(v =>  new Uint8Array(v, message.data[1])[0]));
            console.warn(memories[0] == memories[1]);

            worker.postMessage("test")
        }

        return worker;
    })

    await run(schedulerPtr)
}
