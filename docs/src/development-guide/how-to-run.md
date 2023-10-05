# Running maplibre-rs demos on various platforms

During development, you will want to run the maplibre demos on your local machine to test out your changes.
There are multiple demos of maplibre-rs for different targets. Some targets have prerequisites
depending on your operating system.

* **maplibre-demo** - targets Windows, macOS and Linux, it is built directly with cargo.
* **apple** - targets iOS and macOS and relies on the xcode IDE.
* **android** - targets Android devices and builds in Android Studio.
* **web** - targets the web using a WASM binary.
* **maplibre-headless** - *TBD*

All the targets below require you to install [rustup](https://rustup.rs/) to manage your Rust toolchain.

> __Note__: Make sure you have selected the right toolchain target within rustup. You can use `rustup show` to see your
> active toolchain. If you want to change the target of the build manually, use the cargo `--target` parameter.

## Maplibre-demo

### Linux/macOS

The build for desktop is very simple, you just have to run the following command from the root of the
maplibre-rs project:

```bash
cargo run -p maplibre-demo
```

### Windows

Windows has two additional prerequisites to be able to run. You will need CMake, Visual Studio C++ build tools and the
sqlite3 library.

Install [CMake](https://cmake.org/download/) and add it to your path environment variables.

For the C++ build tools, download the [Visual Studio 2022 Build tools](https://visualstudio.microsoft.com/downloads/)
from the Microsoft website. After the download, while installing the Build tools, make sure that you select the
*C++ build tools*.

To install sqlite3 you need to build the sqlite3.lib manually with the following
[steps](https://gist.github.com/zeljic/d8b542788b225b1bcb5fce169ee28c55). This will generate a .lib file that
you will have to add to the SQLITE3_LIB_DIR environment variable.

Restart your shell to make sure you are using up-to-date environment variables.

Finally, the command below should execute successfully:

```bash
cargo run -p maplibre-demo
```

## Android

Start by installing the 
[Android Studio IDE](https://developer.android.com/studio?gclid=CjwKCAjwj42UBhAAEiwACIhADmF7uHXnEHGnmOgFnjp0Z6n-TnBvutC5faGA89lwouMIXiR6OXK4hBoCq78QAvD_BwE&gclsrc=aw.ds).

Make sure the NDK is installed. The Native Development Kit (NDK) is a set of tools that allows 
you to use C and C++ code with Android. You have to install manually the version that is used in 
`./android/gradle/lib/build.gradle`.

```
ANDROID STUDIO -> tools -> SDK manager -> SDK tools -> tick show package details -> ndk (side by side)
```

Open the project within `./android/gradle` and create a new virtual device with the device manager. Minimum SDK version
should be 21. This was tested on an x86_64 emulator. Finally, run the demo configuration. It should open your virtual device and 
run the maplibre-rs Android demo on it. Alternatively you can also run it on your own Android device.

> Note: If you are building for an x86 Android device, you probably need to install the following target using  
> rustup with the following command `rustup target add i686-linux-android`.

> Note: Android is configured to support OpenGL ES 3.1 (This API specification is supported by Android 5.0 (API level 21) and higher).
> Your Android device is required to support OpenGL ES 3.1 at least. There are some issues 
> [here](https://stackoverflow.com/questions/40797975/android-emulator-and-opengl-es3-egl-bad-config) and 
> [here](https://www.reddit.com/r/Arcore/comments/8squbo/opengl_es_31_is_required_for_android_emulator_to/) that
> discuss configuration of Android Studio for OpenGL ES 3.1 support in emulators.
 
## Apple

Apple builds rely on the [XCode IDE](https://apps.apple.com/us/app/xcode/id497799835?ls=1&mt=12).
Start by installing XCode and open the project within `./apple/xcode`.

> Cargo is used in to build the maplibre library in the build phases of the XCode project configuration.

### iOS

You can use XCode to run on a iOS Simulator or a real device. Install a simulator in XCode.
Version 9 is the minimum version supported theoretically.

Select the scheme called *example (iOS)* and click on run. This will start the iOS application.

### macOS

As you might have seen in the maplibre-demo section, you can build Unix executables directly with Cargo.
In order to build a proper macOS application (in OSX terminology) you have to use the `./apple/xcode` project.

Open the project from the folder `./apple/xcode` with XCode. Select the scheme called *example (macOS)* and
click on run. This will start the macOS application. 

> The minimum target OSX version for the macOS build is defined inside *Build settings -> Deployment -> macOS deployment target*.
> If you are using a lower version of OSX, you will not be able to run the application on your computer.

## Web (WebGL, WebGPU)

You need to first build the library for WebGL or WebGPU. Optionally, you can also enabled multi-threading support,
which requires that the library is used in a secure environment: 
[isSecureContext](https://developer.mozilla.org/en-US/docs/Web/API/isSecureContext)
and [crossOriginIsolated](https://developer.mozilla.org/en-US/docs/Web/API/crossOriginIsolated). 
The demo runs this such an environment.

If you have a browser which already supports a recent version of the WebGPU specification then you can build the library
with WebGPU:

```bash
just web-lib build # WebGPU
```

If not, then you must enable WebGL support:


```bash
just web-lib build --webgl # WebGL
just web-lib build --webgl --multithreaded # WebGL + multithreaded
```

Instead of building it is also possible to watch for changes. The same flags as with `web-lib build` are supported:

```bash
just web-lib watch --webgl
```

After building the library you can run the demo server:

```bash
just web-demo start
```
