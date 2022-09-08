import init from "./wasm/maplibre";

export const initialize = async (wasmPath: string) => {
    if (MULTITHREADED) {
        await initializeSharedModule(wasmPath)
    } else {
        // @ts-ignore
        await init(wasmPath)
    }
}

export const initializeExisting = async (module: string, memory?: string) => {
    if (MULTITHREADED) {
        // @ts-ignore
        await init(module, memory)
    } else {
        // @ts-ignore
        await init(module)
    }
}

const initializeSharedModule = async (wasmPath) => {
    let MEMORY_PAGES = 16 * 1024

    const memory = new WebAssembly.Memory({initial: 1024, maximum: MEMORY_PAGES, shared: true})
    // @ts-ignore
    await init(wasmPath, memory)
}
