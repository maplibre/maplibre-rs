import init, { InitOutput } from "./dist/libs/mapr";
import {WebWorkerMessageType} from "./types";

let module: Promise<InitOutput> = null;

onmessage = async message => {
    let messageData: WebWorkerMessageType = message.data;

    switch (messageData.type) {
        case "init":
            if (module != null) {
                return;
            }
            module = init(undefined, messageData.memory);
            break
        case "run_worker_loop":
            let workflowPtr = messageData.workflowPtr;
            (await module).run_worker_loop(workflowPtr);
            break
        default:
            console.warn("WebWorker received unknown message!")
            break;

    }
};