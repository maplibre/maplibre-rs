import init from "./dist/libs/mapr";
import {Spector} from "spectorjs";

const start = async () => {
    let spector = new Spector();
    spector.displayUI();
    let MEMORY = 16 * 1024;
    const memory = new WebAssembly.Memory({initial: 1024, maximum: MEMORY, shared: true});
    const module = await init(undefined, memory);

    const worker = new Worker(new URL('./cache-worker.js', import.meta.url), {
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
