import * as maplibre from "../wasm/maplibre"

type MessageData = {type: 'wasm_init', module: WebAssembly.Module, memory: WebAssembly.Memory}
    | {type: 'pool_call', work_ptr: number}

let initialised: Promise<maplibre.InitOutput> = null

onmessage = async (message: MessageEvent<MessageData>) => {

    if (initialised) {
        // This will queue further commands up until the module is fully initialised:
        await initialised;
    }

    const type = message.data.type;
    if (type === 'wasm_init') {
        const data = message.data;
        const module = data.module;
        const memory = data.memory;
        initialised = maplibre.default(module, memory).catch(err => {
            // Propagate to main `onerror`:
            setTimeout(() => {
                throw err;
            });
            // Rethrow to keep promise rejected and prevent execution of further commands:
            throw err;
        });
    } else if (type === 'pool_call') {
        const work_ptr = message.data.work_ptr; // because memory is shared, this pointer is valid in the memory of the main thread and this worker thread

        const process_data: (msg: any) => Promise<void> = maplibre["multithreaded_process_data"]

        if (!process_data) {
            throw Error("multithreaded_worker_entry is not defined. Maybe the Rust build used the wrong build configuration.")
        }

        await process_data(work_ptr);
    }
}