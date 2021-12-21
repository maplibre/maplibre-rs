# Supported Platforms

For development the following platforms are recommended:

* Linux X11/Wayland
* MacOS
* Firefox Nightly/Chrome Canary with WebGPU (Because WebGPU is a living spec, sometimes a bleeding-edge browser release
  is required)

## Short-term Obstacles

| Platform       | Obstacles                                                                                                                                                |
|----------------|----------------------------------------------------------------------------------------------------------------------------------------------------------|
| Linux X11      |                                                                                                                                                          |
| Linux Wayland  |                                                                                                                                                          |
| Windows        |                                                                                                                                                          |
| MacOS          |                                                                                                                                                          |
| Android        | * Unable to get window size before resume                                                                                                                |
| iOS            | * Touches are crashing the app on real devices <br/> * Instanced indices drawing is not supported <br/> * Drawing zero-length indices is prohibited<br/> |
| Firefox        | * \[\[block\]\] is still present (WebGPU) <br/> * 2D Texture initialisation not working (WebGL)                                                          |
| Chrome         | * \[\[block\]\] is still present (WebGPU) <br/> * 2D Texture initialisation not working (WebGL)                                                          |
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

| Platform       | Linux & Android     | Graphics API           | Note                                                                                                                                                                                                                           |
|----------------|---------------------|------------------------|--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| Linux X11      | :white_check_mark:  | Vulkan                 |                                                                                                                                                                                                                                |
| Linux Wayland  | :white_check_mark:  | Vulkan                 |                                                                                                                                                                                                                                |
| Windows        | :question_mark:     | :question_mark:        |                                                                                                                                                                                                                                |
| MacOS          | :white_check_mark:  | :question_mark:        |                                                                                                                                                                                                                                |
| Android        | :white_check_mark:  | Vulkan/OpenGL ES/Angle | Not tested, but should work on all devices if [Angle](https://github.com/gfx-rs/wgpu/blob/master/README.md#supported-platforms) is used. [Vulkan](https://developer.android.com/about/dashboards) is not yet supported widely. |
| iOS            | :white_check_mark:  | Metal                  | Not tested.                                                                                                                                                                                                                    |
| Firefox        | :white_check_mark:  | WebGL/WebGPU           |                                                                                                                                                                                                                                |
| Chrome         | :white_check_mark:  | WebGL/WebGPU           | WebGPU is significantly faster because WASM output is smaller.                                                                                                                                                                 |
| Safari         | :hammer_and_wrench: | WebGL/WebGPU           | Safari does not yet support [Shared Array Buffer](https://caniuse.com/sharedarraybuffer)                                                                                                                                       |
| Mobile Firefox | :ok:                | WebGL/WebGPU           |                                                                                                                                                                                                                                |
| Mobile Chrome  | :ok:                | WebGL                  | [WebGPU](https://caniuse.com/webgpu) is not implemented.                                                                                                                                                                       |
| Mobile Safari  | :hammer_and_wrench: | WebGL                  | [WebGPU](https://caniuse.com/webgpu) is not implemented. Safari does not yet support [Shared Array Buffer](https://caniuse.com/sharedarraybuffer)                                                                              |

:white_check_mark: = First Class Support — :ok: = Best Effort Support — :hammer_and_wrench: = Unsupported, but support
in progress
