import init, {create_pool_scheduler, run} from "./wasm-pack"
import {Spector} from "spectorjs"
import {WebWorkerMessageType} from "./types"
import {
    bigInt,
    bulkMemory,
    exceptions,
    multiValue,
    mutableGlobals,
    referenceTypes,
    saturatedFloatToInt,
    signExtensions,
    simd,
    tailCall,
    threads
} from "wasm-feature-detect"

// @ts-ignore
import PoolWorker from './pool.worker.js';

const isWebGLSupported = () => {
    try {
        const canvas = document.createElement('canvas')
        canvas.getContext("webgl")
        return true
    } catch (x) {
        return false
    }
}

const checkWasmFeatures = async () => {
    const checkFeature = async function (featureName: string, feature: () => Promise<boolean>) {
        let result = await feature();
        let msg = `The feature ${featureName} returned: ${result}`;
        if (result) {
            console.log(msg);
        } else {
            console.warn(msg);
        }
    }

    await checkFeature("bulkMemory", bulkMemory);
    await checkFeature("exceptions", exceptions);
    await checkFeature("multiValue", multiValue);
    await checkFeature("mutableGlobals", mutableGlobals);
    await checkFeature("referenceTypes", referenceTypes);
    await checkFeature("saturatedFloatToInt", saturatedFloatToInt);
    await checkFeature("signExtensions", signExtensions);
    await checkFeature("simd", simd);
    await checkFeature("tailCall", tailCall);
    await checkFeature("threads", threads);
    await checkFeature("bigInt", bigInt);
}

const alertUser = (message: string) => {
    console.error(message)
    alert(message)
}

const checkRequirements = () => {
    if (!isSecureContext) {
        alertUser("isSecureContext is false!")
        return false
    }

    if (!crossOriginIsolated) {
        alertUser("crossOriginIsolated is false! " +
            "The Cross-Origin-Opener-Policy and Cross-Origin-Embedder-Policy HTTP headers are required.")
        return false
    }

    if (WEBGL) {
        if (!isWebGLSupported()) {
            alertUser("WebGL is not supported in this Browser!")
            return false
        }

        let spector = new Spector()
        spector.displayUI()
    } else {
        if (!("gpu" in navigator)) {
            let message = "WebGPU is not supported in this Browser!"
            alertUser(message)
            return false
        }
    }

    return true
}

const preventDefaultTouchActions = () => {
    document.body.querySelectorAll("canvas").forEach(canvas => {
        canvas.addEventListener("touchstart", e => e.preventDefault())
        canvas.addEventListener("touchmove", e => e.preventDefault())
    })
}
/*
let WORKER_COUNT = 4
const createWorker = (id: number, memory: WebAssembly.Memory) => {
    const worker = new Worker(new URL('./legacy.worker.ts', import.meta.url), {
        type: "module",
    })
    worker.postMessage({type: "init", memory} as WebWorkerMessageType)

    return worker
}

const setupLegacyWebWorker = (schedulerPtr: number, memory: WebAssembly.Memory) => {
    let workers: [number, Worker][] = Array.from(
        new Array(WORKER_COUNT).keys(),
        (id) => [new_thread_local_state(schedulerPtr), createWorker(id, memory)]
    )

    window.schedule_tile_request = (url: string, request_id: number) => {
        const [state, worker] = workers[Math.floor(Math.random() * workers.length)]
        worker.postMessage({
            type: "fetch_tile",
            threadLocalState: state,
            url,
            request_id
        } as WebWorkerMessageType)
    }
}*/

export const startMapLibre = async (wasmPath: string | undefined, workerPath: string | undefined) => {
    await checkWasmFeatures()

    if (!checkRequirements()) {
        return
    }

    preventDefaultTouchActions();

    let MEMORY_PAGES = 16 * 1024

    const memory = new WebAssembly.Memory({initial: 1024, maximum: MEMORY_PAGES, shared: true})
    await init(wasmPath, memory)

    const schedulerPtr = create_pool_scheduler(() => {
        return workerPath ? new PoolWorker(workerPath, {
            type: 'module'
        }) : PoolWorker();
    })

    // setupLegacyWebWorker(schedulerPtr, memory)

    await run(schedulerPtr)
}
