# Building for various Platforms

## Desktop

The build for desktop is very simple. You just have to run the following:

```bash
cargo build -p maplibre-demo
```

You can use the `--release` parameter if you want to be in release mode instead of debug.

If you want to run the application:

```bash
cargo run -p maplibre-demo
```

> __Note__: Make sure you have selected the right toolchain target within rustup. You can use `rustup show` to see your
> active toolchain. If you want to change the target of the build manually, use the cargo `--target` parameter.


## Android

You should make sure that a recent Android NDK is installed. You will need to set the `ANDROID_NDK_ROOT` variable
to something like this:

```bash
export ANDROID_NDK_ROOT=$HOME/android-sdk/ndk/23.1.7779620/
```

After that you can run the build the library:

``bash
just build-android
``

## iOS

In order to run this app on iOS you have to open the Xcode project at `./apple/xcode`.
You can then run the app on an iOS Simulator or a real device. During the Xcode build process cargo is used to build
a static library for the required architecture.

## MacOS

You can build Unix Executables for MacOS, as explained in the first section, with one of the following toolchain targets:

* **x86_64-apple-darwin** for Intel's x86-64 processors
* **aarch64-apple-darwin** for ARM64 processors

Use one of the following commands to build for your desired target architecture:

```bash
cargo build -p maplibre-demo --target x86_64-apple-darwin
cargo build -p maplibre-demo --target aarch64-apple-darwin
```

If you want to build a proper MacOS application (in OSX terminology), you will need to use the XCode project
in the folder `./apple/xcode/`.

Install [XCode](https://apps.apple.com/us/app/xcode/id497799835?ls=1&mt=12) and [rustup](https://rustup.rs/).
Then open the project from the folder `./apple/xcode` with XCode. Select the scheme called *example(macOS)* and
click on *Product -> Build for -> Running*. This will build the MacOS application for the version of OSX defined
in the Build Settings. The XCode project is configured to automatically compile the Rust library with the correct target
in the *Cargo Build* build phases configuration.

If you want to run the project from XCode, you need to make sure that you have selected the version of OSX which
corresponds to your system. Otherwise, XCode will tell you that the app is incompatible with the current version of macOS.
In order to change that, go into *Build settings -> Deployment -> MacOS deployment target* and select your OSX version.
Finally, you can click on the run button to start the application.

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