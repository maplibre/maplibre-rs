import init, {sync_worker_entry} from "../wasm/maplibre"

onmessage = async message => {
    const initialised = init(message.data[0], message.data[1]).catch(err => {
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
        await sync_worker_entry(message.data);
    };
}