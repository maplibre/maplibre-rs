import init, {worker_entry} from "../wasm/maplibre"

const initializeExisting = async (module: string) => {
    await init(module)
}

onmessage = async message => {
    const initialised = initializeExisting(message.data[0]).catch(err => {
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
        await worker_entry();
    };
}
