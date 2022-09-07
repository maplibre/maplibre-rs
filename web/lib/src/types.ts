export type WebWorkerMessageType = {
    type: 'init',
    memory: WebAssembly.Memory
} | {
    type: 'fetch_tile',
    threadLocalState: number,
    url: string,
    request_id: number,
}
