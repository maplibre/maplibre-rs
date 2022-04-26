# Supported Platforms

For development the following platforms are recommended:

* Linux X11/Wayland
* MacOS
* Latest Firefox Nightly/Chrome Canary with WebGPU (Because WebGPU is a living spec, sometimes a bleeding-edge browser
  release is required)

## Short-term Obstacles

| Platform       | Obstacles                                                                                                                                                |
|----------------|----------------------------------------------------------------------------------------------------------------------------------------------------------|
| Linux X11      |                                                                                                                                                          |
| Linux Wayland  |                                                                                                                                                          |
| Windows        |                                                                                                                                                          |
| MacOS          |                                                                                                                                                          |
| Android        | * Unable to get window size before resume                                                                                                                |
| iOS            | * Touches are crashing the app on real devices <br/> * Instanced indices drawing is not supported <br/> * Drawing zero-length indices is prohibited<br/> |
| Firefox        | * Shared Memory is currently not working because it a parallel web worker corrupts memory                                                                |
| Chrome         |                                                                                                                                                          |
| Safari         |                                                                                                                                                          |
| Mobile Firefox |                                                                                                                                                          |
| Mobile Chrome  |                                                                                                                                                          |
| Mobile Safari  |                                                                                                                                                          |

## Long-term Goals

[WebGPU](https://caniuse.com/webgpu) is not enabled by default for all platforms.

WebGPU Status:

* [Firefox](https://github.com/gpuweb/gpuweb/wiki/Implementation-Status#firefox-and-servo)
* [Chrome](https://chromestatus.com/feature/6213121689518080)
* [WebKit](https://webkit.org/status/#specification-webgpu)

| Platform       | Linux & Android | Graphics API           | Note                                                                                                                                                                                                                           |
|----------------|-----------------|------------------------|--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| Linux X11      | ✅               | Vulkan                 |                                                                                                                                                                                                                                |
| Linux Wayland  | ✅               | Vulkan                 |                                                                                                                                                                                                                                |
| Windows        | ❓               | ❓                      |                                                                                                                                                                                                                                |
| MacOS          | ✅               | ❓                      |                                                                                                                                                                                                                                |
| Android        | ✅               | Vulkan/OpenGL ES/Angle | Not tested, but should work on all devices if [Angle](https://github.com/gfx-rs/wgpu/blob/master/README.md#supported-platforms) is used. [Vulkan](https://developer.android.com/about/dashboards) is not yet supported widely. |
| iOS            | ✅               | Metal                  | Not tested.                                                                                                                                                                                                                    |
| Firefox        | ✅               | WebGL/WebGPU           |                                                                                                                                                                                                                                |
| Chrome         | ✅               | WebGL/WebGPU           | WebGPU is significantly faster because WASM output is smaller.                                                                                                                                                                 |
| Safari         | 🛠️             | WebGL/WebGPU           | Safari does not yet support [Shared Array Buffer](https://caniuse.com/sharedarraybuffer)                                                                                                                                       |
| Mobile Firefox | 🆗              | WebGL/WebGPU           |                                                                                                                                                                                                                                |
| Mobile Chrome  | 🆗              | WebGL                  | [WebGPU](https://caniuse.com/webgpu) is not implemented.                                                                                                                                                                       |
| Mobile Safari  | 🛠️             | WebGL                  | [WebGPU](https://caniuse.com/webgpu) is not implemented. Safari does not yet support [Shared Array Buffer](https://caniuse.com/sharedarraybuffer)                                                                              |

✅ = First Class Support — 🆗= Best Effort Support — 🛠️  = Unsupported, but support
in progress
