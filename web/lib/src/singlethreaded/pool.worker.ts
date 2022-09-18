import * as maplibre from "../wasm/maplibre"

onmessage = async message => {
    const memory = new WebAssembly.Memory({initial: 1024, shared: false})
    let module = message.data[0];
    const initialised = maplibre.default(module, memory).catch(err => {
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
        // @ts-ignore TODO
        await maplibre.singlethreaded_worker_entry(procedure_ptr, input);
    };
}
