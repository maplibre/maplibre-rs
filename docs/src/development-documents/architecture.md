# Architecture

## Rendering Architecture

The big picture of wgpu is as follows:

![](https://raw.githubusercontent.com/gfx-rs/wgpu/8f02b73655aff641361822a8ac0347fc47622b49/etc/big-picture.png)

A simplified version is shown below:

![](./figures/render-stack.drawio.svg)

A further simplified version:

![](./figures/simplified-render-stack.drawio.svg)

Notes:
* wgpu is able to create an interface through which we can reach any device with a GPU.

Notes:
* The ability to use shared memory or the atomic instruction set of WASM comes by enabling compilation features.
* `threads` support here does not introduce threads like we know them from Linux. It introduces 
* [support for atomics](https://github.com/WebAssembly/threads/blob/main/proposals/threads/Overview.md) like
  specified in a working draft to WebAssembly. Threads are simulated using WebWorkers by the browser.
