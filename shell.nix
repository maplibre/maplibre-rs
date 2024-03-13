# This nix-shell only supports macOS right now. Soon I will also add support for Linux
# The repository supports direnv (https://direnv.net/). If your IDE supports direnv,
# then you do not need to care about dependencies.

{ pkgs ? import <nixpkgs> { } }:
with pkgs;
let
  unstable = import
    (builtins.fetchTarball {
      url = "https://github.com/NixOS/nixpkgs/archive/075dce259f6ced5cee1226dd76474d0674b54e64.tar.gz";
    })
    { };
in
pkgs.mkShell {
  nativeBuildInputs = [
    # Tools
    unstable.rustup
    unstable.just
    unstable.nodejs
    unstable.mdbook
    unstable.wasm-bindgen-cli
    unstable.tracy
    unstable.nixpkgs-fmt # To format this file: nixpkgs-fmt *.nix
    # System dependencies
    unstable.flatbuffers
    unstable.protobuf

    unstable.xorg.libXrandr
    unstable.xorg.libXi
    unstable.xorg.libXcursor
    unstable.xorg.libX11
    unstable.libxkbcommon
    unstable.sqlite
    unstable.wayland
    unstable.pkg-config
  ]
  ++ lib.optionals stdenv.isDarwin [
    unstable.libiconv
    pkgs.darwin.apple_sdk.frameworks.ApplicationServices
    pkgs.darwin.apple_sdk.frameworks.CoreVideo
    pkgs.darwin.apple_sdk.frameworks.AppKit
  ];
  shellHook = ''
    # Vulkan
    export LD_LIBRARY_PATH="$LD_LIBRARY_PATH:${ pkgs.lib.makeLibraryPath [ pkgs.vulkan-loader ] }";
    # EGL
    export LD_LIBRARY_PATH="$LD_LIBRARY_PATH:${ pkgs.lib.makeLibraryPath [ pkgs.libglvnd ] }";
  '';

}
