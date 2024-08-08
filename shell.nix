# This nix-shell supports macOS and Linux.
# The repository supports direnv (https://direnv.net/). If your IDE supports direnv,
# then you do not need to care about dependencies.

{ pkgs ? import <nixpkgs> {
    overlays = [];
  }
}:
with pkgs;
let
  unstable = import
    (builtins.fetchTarball {
      url = "https://github.com/NixOS/nixpkgs/archive/cb9a96f23c491c081b38eab96d22fa958043c9fa.tar.gz"; # Ger from here: https://github.com/NixOS/nixpkgs/tree/nixos-unstable
    })
    { };
in
(pkgs.mkShell.override {
  # Wew are using the host clang on macOS; the Nix clang adds a clag that breaks cross compilation here:
  # https://github.com/NixOS/nixpkgs/blob/362cb82b75394680990cbe89f40fe65d35f66617/pkgs/build-support/cc-wrapper/default.nix#L490
  # It caused this error during the compilation of ring: clang-15: error: invalid argument '-mmacos-version-min=11.0' not allowed with '-miphoneos-version-min=7.0'
  stdenv = if stdenv.isDarwin then stdenvNoCC else llvmPackages_16.stdenv;
}) {
  nativeBuildInputs = [
    # Tools
    rustup
    unstable.just
    unstable.nodejs
    unstable.mdbook
    (pkgs.wasm-bindgen-cli.override {
      version = "0.2.92"; # This needs to match the wasm-bindgen version of the web module
      hash = "sha256-1VwY8vQy7soKEgbki4LD+v259751kKxSxmo/gqE6yV0=";
      cargoHash = "sha256-aACJ+lYNEU8FFBs158G1/JG8sc6Rq080PeKCMnwdpH0=";
    })
    pkgs.cargo-criterion
    unstable.nixpkgs-fmt # To format this file: nixpkgs-fmt *.nix
    # System dependencies
    unstable.flatbuffers
    unstable.protobuf

    unstable.tracy-x11
    unstable.renderdoc

    pkgs.jdk17

    unstable.sqlite
    unstable.pkg-config
  ] ++ lib.optionals stdenv.isLinux [
    unstable.xorg.libXrandr
    unstable.xorg.libXi
    unstable.xorg.libXcursor
    unstable.xorg.libX11
    unstable.libxkbcommon
    unstable.wayland
  ];
  shellHook = ''
    export LD_LIBRARY_PATH="$LD_LIBRARY_PATH:${ pkgs.lib.makeLibraryPath [
        unstable.libxkbcommon
        # Vulkan
        pkgs.vulkan-loader
         # EGL
        pkgs.libglvnd
    ] }";
  '';
}
