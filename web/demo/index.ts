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

// `?style=<url>` loads any style; `?projection=globe|mercator|vertical-perspective` overrides its
// projection. Without a style URL the MapLibre demotiles world style is shown on the globe.
const params = new URLSearchParams(location.search)
const styleUrl = params.get('style') ?? 'https://demotiles.maplibre.org/style.json'
const projection = params.get('projection') ?? 'globe'

const loadStyle = async (): Promise<string | undefined> => {
  try {
    const response = await fetch(styleUrl)
    if (!response.ok) {
      throw new Error(`${response.status} ${response.statusText}`)
    }
    const style = await response.json()
    style.projection = { type: projection }
    return JSON.stringify(style)
  } catch (error) {
    console.error(`Cannot load style ${styleUrl}, falling back to the built-in style`, error)
    return undefined
  }
}

loadStyle().then((styleJson) => startMapLibre(undefined, undefined, styleJson))
