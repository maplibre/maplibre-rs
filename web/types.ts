export type WebWorkerMessageType = {
    type: 'init',
    memory: WebAssembly.Memory
} | {
    type: 'fetch_tile',
    tessellatorState: number,
    url: string,
    request_id: number,
}

