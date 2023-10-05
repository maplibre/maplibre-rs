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

        // WARNING: Do not modify data passed from Rust!
        const procedure_ptr = message.data[0];
        const input = message.data[1];

        const worker_entry = maplibre["singlethreaded_worker_entry"];

        if (!worker_entry) {
            throw Error("singlethreaded_worker_entry is not defined. Maybe the Rust build used the wrong build configuration.")
        }

        await worker_entry(procedure_ptr, input);
    };
}
