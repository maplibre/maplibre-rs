import init, {create_pool_scheduler, create_scheduler, new_tessellator_state, run} from "./dist/libs/mapr"
import {Spector} from "spectorjs"
import {WebWorkerMessageType} from "./types"

declare global {
    interface Window {
        schedule_tile_request: (url: string, request_id: number) => void;
        newWorker: () => void;
    }
}

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

const preventDefaultTouchActions = () => {
    document.body.querySelectorAll("canvas").forEach(canvas => {
        canvas.addEventListener("touchstart", e => e.preventDefault())
        canvas.addEventListener("touchmove", e => e.preventDefault())
    })
}

const registerServiceWorker = () => {
    if ('serviceWorker' in navigator) {
        window.addEventListener('load', () => {
            navigator.serviceWorker.register(new URL('./service-worker.ts', import.meta.url)).catch(() => {
                console.error("Failed to register service worker");
            })
        })
    }
}


const start = async () => {
    if (!checkRequirements()) {
        return
    }

    registerServiceWorker()

    preventDefaultTouchActions();

    let MEMORY_PAGES = 16 * 1024

    const memory = new WebAssembly.Memory({initial: 1024, maximum: MEMORY_PAGES, shared: true})
    await init(undefined, memory)
    /*const schedulerPtr = create_pool_scheduler(() => {
        console.log("spawni")
        return new Worker(new URL('./pool_worker.ts', import.meta.url), {
            type: 'module'
        });
    })*/

    const schedulerPtr = create_scheduler()

    let WORKER_COUNT = 4
    const createWorker = (id: number) => {
        const worker = new Worker(new URL('./worker.ts', import.meta.url), {
            type: "module",
            name: `worker_${id}`
        })
        worker.postMessage({type: "init", memory} as WebWorkerMessageType)

        return worker
    }

    let workers: [number, Worker][] = Array.from(
        new Array(WORKER_COUNT).keys(),
        (id) => [new_tessellator_state(schedulerPtr), createWorker(id)]
    )

    window.schedule_tile_request = (url: string, request_id: number) => {
        const [tessellatorState, worker] = workers[Math.floor(Math.random() * workers.length)]
        worker.postMessage({
            type: "fetch_tile",
            tessellatorState: tessellatorState,
            url,
            request_id
        } as WebWorkerMessageType)
    }

    await run(schedulerPtr)
}

start().then(() => console.log("started via wasm"))


