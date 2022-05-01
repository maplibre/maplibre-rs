# Android

## Gradle Project Setup

In order to package an Android `.aar` archive we use
the [rust-android-gradle](https://github.com/mozilla/rust-android-gradle).
Except some customisations for the latest NDK toolchain release everything worked flawlessly.

## JNI

There is no way right now to automatically generate JNI stubs for Rust. A manual example is available in the android
crate of maplibre-rs.

## Single NativeActivity

Right now `winit` only allows the usage of a `NativeActivity`. This means the application needs to run in fullscreen.
This native activity is referenced in the ´AndroidManifest.xml` by defining the name of the shared library.
[Tracking Issue](https://github.com/maplibre/maplibre-rs/issues/28)
