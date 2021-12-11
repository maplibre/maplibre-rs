# Supported Platforms

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
| MacOS          | :question_mark:     | :question_mark:        |                                                                                                                                                                                                                                |
| Android        | :question_mark:     | Vulkan/OpenGL ES/Angle | Not tested, but should work on all devices if [Angle](https://github.com/gfx-rs/wgpu/blob/master/README.md#supported-platforms) is used. [Vulkan](https://developer.android.com/about/dashboards) is not yet supported widely. |
| iOS            | :question_mark:     | Metal                  | Not tested.                                                                                                                                                                                                                    |
| Firefox        | :white_check_mark:  | WebGL/WebGPU           |                                                                                                                                                                                                                                |
| Chrome         | :white_check_mark:  | WebGL/WebGPU           | WebGPU is significantly faster because WASM output is smaller.                                                                                                                                                                 |
| Safari         | :hammer_and_wrench: | WebGL/WebGPU           | Safari does not yet support [Shared Array Buffer](https://caniuse.com/sharedarraybuffer)                                                                                                                                       |
| Mobile Firefox | :ok:                | WebGL/WebGPU           |                                                                                                                                                                                                                                |
| Mobile Chrome  | :ok:                | WebGL                  | [WebGPU](https://caniuse.com/webgpu) is not implemented.                                                                                                                                                                       |
| Mobile Safari  | :hammer_and_wrench: | WebGL                  | [WebGPU](https://caniuse.com/webgpu) is not implemented. Safari does not yet support [Shared Array Buffer](https://caniuse.com/sharedarraybuffer)                                                                              |

:white_check_mark: = First Class Support — :ok: = Best Effort Support — :hammer_and_wrench: = Unsupported, but support
in progress
