import init from "./dist/libs/mapr";

const start = async () => {
    const memory = new WebAssembly.Memory({initial: 1024, maximum: 10 * 1024, shared: true});
    const init_output = await init(undefined, memory);

    const fetch_worker = new Worker(new URL('./fetch-worker.js', import.meta.url), {
        type: "module",
    });

    fetch_worker.postMessage({type: "init", memory, address: init_output.test_alloc()});

    fetch_worker.onmessage = (e) => {
        console.log(e)
    }

    await init_output.run();
}

start();