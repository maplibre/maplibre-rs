import init from "./dist/libs/mapr";
// @ts-ignore
import {Spector} from "spectorjs";

declare var WEBGL: boolean;

const isWebGLSupported = () => {
    try {
        const canvas = document.createElement('canvas');
        canvas.getContext("webgl");
        return true;
    } catch (x) {
        return false;
    }
}

const start = async () => {
    if (!isSecureContext) {
        let message = "isSecureContext is false!";
        console.error(message)
        alert(message)
        return;
    }

    if (!crossOriginIsolated) {
        let message = "crossOriginIsolated is false! " +
            "The Cross-Origin-Opener-Policy and Cross-Origin-Embedder-Policy HTTP headers are required.";
        console.error(message)
        alert(message)
        return;
    }

    if (WEBGL) {
        if (!isWebGLSupported()) {
            console.error("WebGL is not supported in this Browser!")
            return;
        }

        let spector = new Spector();
        spector.displayUI();
    } else {
        if (!("gpu" in navigator)) {
            console.error("WebGPU is not supported in this Browser!")
            return;
        }
    }

    if ('serviceWorker' in navigator) {
        window.addEventListener('load', () => {
            navigator.serviceWorker.register('/service-worker.js');
        });
    }

    let MEMORY = 16 * 1024;
    const memory = new WebAssembly.Memory({initial: 1024, maximum: MEMORY, shared: true});
    const module = await init(undefined, memory);

    const worker = new Worker(new URL('./cache-worker.ts', import.meta.url), {
        type: "module",
    });

    let cache_address = module.create_cache();

    console.log("Starting cache-worker")
    worker.postMessage({type: "init", memory, cache_address});

    /*    worker.onmessage = (e) => {
            console.log(e)
        }*/

    await module.run(cache_address);
}

start().then(r => console.log("started via wasm"));