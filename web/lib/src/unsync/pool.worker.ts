import init, {unsync_worker_entry} from "../wasm/maplibre"

onmessage = async message => {
    let MEMORY_PAGES = 16 * 1024
    const memory = new WebAssembly.Memory({initial: 1024, maximum: MEMORY_PAGES, shared: false})
    const initialised = init(message.data[0], memory).catch(err => {
        // Propagate to main `onerror`:
        setTimeout(() => {
            throw err;
        });
        // Rethrow to keep promise rejected and prevent execution of further commands:
        throw err;
    });

    self.onmessage = async message => {
        // This will queue further commands up until the module is fully initialised:
        await initialised;
        let procedure_ptr = message.data[0];
        let input = message.data[1];
        await unsync_worker_entry(procedure_ptr, input);
    };
}
