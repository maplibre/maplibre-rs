import * as maplibre from "../wasm/maplibre"

type MessageData = { type: 'wasm_init', module: WebAssembly.Module }
    | { type: 'kernel_config', config: string }
    | { type: 'call', procedure_ptr: number, input: string }

let initialised: Promise<maplibre.InitOutput> = null

onmessage = async (message: MessageEvent<MessageData>) => {

    if (initialised) {
        // This will queue further commands up until the module is fully initialised:
        await initialised;
    }

    const type = message.data.type;
    if (type === 'wasm_init') {
        const data = message.data;
        const memory = new WebAssembly.Memory({initial: 1024, shared: false})
        let module = data.module;
        initialised = maplibre.default(module, memory).catch(err => {
            // Propagate to main `onerror`:
            setTimeout(() => {
                throw err;
            });
            // Rethrow to keep promise rejected and prevent execution of further commands:
            throw err;
        });
    } else if (type === 'call') {
        const data = message.data;
        // WARNING: Do not modify data passed from Rust!
        const procedure_ptr = data.procedure_ptr;
        const input = data.input;

        const process_data: (procedure_ptr: number, input: string) => Promise<void> = maplibre["singlethreaded_process_data"];

        if (!process_data) {
            throw Error("singlethreaded_worker_entry is not defined. Maybe the Rust build used the wrong build configuration.")
        }

        await process_data(procedure_ptr, input);
    } else if (type === 'kernel_config') {
        const data = message.data;

        const set_kernel_config: (config: string) => void = maplibre["set_kernel_config"];

        if (!set_kernel_config) {
            throw Error("set_kernel_config is not defined. Maybe the Rust build used the wrong build configuration.")
        }


        set_kernel_config(data.config)
    }
}
