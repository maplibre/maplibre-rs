mod fps_meter;
mod platform;
mod render;
mod io;
mod setup;

#[cfg(target_arch = "wasm32")]
mod web;

#[cfg(target_arch = "aarch64")]
mod apple;

#[cfg(target_os = "android")]
mod android;
