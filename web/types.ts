export type WebWorkerMessageType = { type: 'init', memory: WebAssembly.Memory } | {type: 'run_worker_loop', workflowPtr: number}
