import {
    bigInt,
    bulkMemory,
    exceptions,
    multiValue,
    mutableGlobals,
    referenceTypes,
    saturatedFloatToInt,
    signExtensions,
    simd,
    tailCall,
    threads
} from "wasm-feature-detect"


export const checkRequirements = () => {
    if (MULTITHREADED) {
        if (!isSecureContext) {
            return "isSecureContext is false!"
        }

        if (!crossOriginIsolated) {
            return "crossOriginIsolated is false! " +
                "The Cross-Origin-Opener-Policy and Cross-Origin-Embedder-Policy HTTP headers are required."
        }
    }

    if (WEBGL) {
        if (!isWebGLSupported()) {
            return "WebGL is not supported in this Browser!"
        }
    } else {
        if (!("gpu" in navigator)) {
            return "WebGPU is not supported in this Browser!"
        }
    }

    return null
}

export const isWebGLSupported = () => {
    try {
        const canvas = document.createElement('canvas')
        canvas.getContext("webgl")
        return true
    } catch (x) {
        return false
    }
}

export const checkWasmFeatures = async () => {
    const checkFeature = async function (featureName: string, feature: () => Promise<boolean>) {
        let result = await feature();
        let msg = `The feature ${featureName} returned: ${result}`;
        if (result) {
            console.log(msg);
        } else {
            console.warn(msg);
        }
    }

    await checkFeature("bulkMemory", bulkMemory);
    await checkFeature("exceptions", exceptions);
    await checkFeature("multiValue", multiValue);
    await checkFeature("mutableGlobals", mutableGlobals);
    await checkFeature("referenceTypes", referenceTypes);
    await checkFeature("saturatedFloatToInt", saturatedFloatToInt);
    await checkFeature("signExtensions", signExtensions);
    await checkFeature("simd", simd);
    await checkFeature("tailCall", tailCall);
    await checkFeature("threads", threads);
    await checkFeature("bigInt", bigInt);
}
