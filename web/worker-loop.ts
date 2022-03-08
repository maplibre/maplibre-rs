import init from "./dist/libs/mapr";

let initialized = false;

onmessage = async message => {
    let data = message.data;

    if (data.type === "init") {
        if (initialized) {
            return;
        }
        initialized = true;
        const module = await init(undefined, data.memory);
        let workflowPtr = data.workflowPtr;
        console.log("Started WorkerLoop: " + workflowPtr)
        module.run_worker_loop(workflowPtr);
    }
};