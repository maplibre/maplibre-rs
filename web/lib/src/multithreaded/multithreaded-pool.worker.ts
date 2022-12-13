import * as maplibre from "../wasm/maplibre"

onmessage = async message => {
    const initialised = maplibre.default(message.data[0], message.data[1]).catch(err => {
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

        const worker_entry = maplibre["multithreaded_worker_entry"]

        if (!worker_entry) {
            throw Error("multithreaded_worker_entry is not defined. Maybe the Rust build used the wrong build configuration.")
        }

        await worker_entry(message.data);
    };
}
