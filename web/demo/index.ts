import { startMapLibre } from 'maplibre-rs'

// Polyfill for removed WebGPU limits (wgpu 22.x uses maxInterStageShaderComponents
// which was dropped from the spec in Chrome 131+).
const REMOVED_LIMITS = ['maxInterStageShaderComponents']

if (typeof (globalThis as any).GPUSupportedLimits !== 'undefined') {
  for (const name of REMOVED_LIMITS) {
    const proto = (globalThis as any).GPUSupportedLimits.prototype
    if (!(name in proto)) {
      Object.defineProperty(proto, name, { get() { return 60 } })
    }
  }
}

if (typeof (globalThis as any).GPUAdapter !== 'undefined') {
  const origRequestDevice = (globalThis as any).GPUAdapter.prototype.requestDevice
  ;(globalThis as any).GPUAdapter.prototype.requestDevice = function (desc?: any) {
    if (desc?.requiredLimits) {
      for (const name of REMOVED_LIMITS) {
        delete desc.requiredLimits[name]
      }
    }
    return origRequestDevice.call(this, desc)
  }
}

startMapLibre(undefined, undefined)
