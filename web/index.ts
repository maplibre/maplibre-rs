import init from "./dist/libs/mapr"
// @ts-ignore
import {Spector} from "spectorjs"
import {WebWorkerMessageType} from "./types"

declare var WEBGL: boolean

const isWebGLSupported = () => {
    try {
        const canvas = document.createElement('canvas')
        canvas.getContext("webgl")
        return true
    } catch (x) {
        return false
    }
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


const start = async () => {
    if (!checkRequirements()) {
        return
    }

    if ('serviceWorker' in navigator) {
        window.addEventListener('load', () => {
            navigator.serviceWorker.register(new URL('./service-worker.ts', import.meta.url))
        })
    }

    document.body.querySelectorAll("canvas").forEach(canvas => {
        canvas.addEventListener("touchstart", e => e.preventDefault())
        canvas.addEventListener("touchmove", e => e.preventDefault())
    })

    let MEMORY_PAGES = 16 * 1024
    const memory = new WebAssembly.Memory({initial: 1024, maximum: MEMORY_PAGES, shared: true})
    const module = await init(undefined, memory)

    const worker = new Worker(new URL('./worker.ts', import.meta.url), {
        type: "module",
    })

    worker.postMessage({type: "init", memory} as WebWorkerMessageType)

    let workflowPtr = module.create_workflow()
    worker.postMessage({type: "run_worker_loop", workflowPtr: workflowPtr} as WebWorkerMessageType)

    await module.run(workflowPtr)
}

start().then(() => console.log("started via wasm"))
