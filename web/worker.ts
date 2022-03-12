import init, {InitOutput, tessellate_layers} from "./dist/libs/mapr"
import {WebWorkerMessageType} from "./types"

let module: Promise<InitOutput> = null

onmessage = async message => {
    let messageData: WebWorkerMessageType = message.data
    console.dir(messageData)

    switch (messageData.type) {
        case "init":
            if (module != null) {
                return
            }
            module = init(undefined, messageData.memory)
            break
        case "fetch_tile":
            let {tessellatorState, url, request_id} = messageData
            await module

            console.log("Fetching from " + self.name)

            let result = await fetch(url)
            let buffer = await result.arrayBuffer()

            tessellate_layers(tessellatorState, request_id, new Uint8Array(buffer))
            break
        default:
            console.warn("WebWorker received unknown message!")
            break
    }
}