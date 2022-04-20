# Building on various Platforms

## Desktop (Linux)

The setup normal desktop is very simple. You just have to run the following:

```bash
cargo run --example desktop --
```

## Android

First you have to install cargo-apk:

```bash
cargo install cargo-apk
```

Next, you should make sure that a recent Android NDK is installed. You will need to set the `ANDROID_NDK_ROOT` variable
to something like this:

```bash
export ANDROID_NDK_ROOT=$HOME/android-sdk/ndk/23.1.7779620/
```

After that you can run the app on a connected device using:

``bash
cargo apk run --lib
``

## iOS

In order to run this app on iOS you have to open the Xcode project at `./apple/mapr/mapr.xcodeproj`.
You can then run the app on an iOS Simulator or a real device. During the Xcode build process cargo is used to build
a static library for the required architecture.

## Desktop (macOS)

In order to run this app on macOS you have to open the Xcode project at `./apple/mapr/mapr.xcodeproj`.
You can then run the app on a macOS. During the Xcode build process cargo is used to build
a static library for the required architecture.

*Hint: Right now only arm64/aarch64 devices are supported*

## Web (WebGL, WebGPU)

If you have a browser which already supports a recent version of the WebGPU specification then you can start a
development server using the following commands.

```bash
cd web
npm run start
```

If you want to run maplibre-rs with WebGL which is supported on every major browser, then you have to use the following
command.

```bash
npm run webgl-start
```