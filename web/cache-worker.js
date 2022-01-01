import init from "./dist/libs/mapr";

let initialized = false;

onmessage = async m => {
    let msg = m.data;

    if (msg.type === "init") {
        if (initialized) {
            return;
        }
        initialized = true;
        const module = await init(undefined, msg.memory);
        console.log("Started cache-worker: " + msg.cache_address)

        module.run_cache_loop(msg.cache_address);
    }
};