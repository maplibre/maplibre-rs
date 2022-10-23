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
        // @ts-ignore TODO may not exist
        await maplibre.multithreaded_worker_entry(message.data);
    };
}
