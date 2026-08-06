{
  description = "maplibre-rs development shell";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  };

  outputs = { self, nixpkgs }:
    let
      supportedSystems = [ "x86_64-linux" "aarch64-linux" "x86_64-darwin" "aarch64-darwin" ];
      forAllSystems = nixpkgs.lib.genAttrs supportedSystems;
    in
    {
      devShells = forAllSystems (system:
        let
          pkgs = import nixpkgs { inherit system; };
        in
        {
          default = (pkgs.mkShell.override {
            # We are using the host clang on macOS; the Nix clang adds a flag that breaks cross compilation:
            # https://github.com/NixOS/nixpkgs/blob/362cb82b75394680990cbe89f40fe65d35f66617/pkgs/build-support/cc-wrapper/default.nix#L490
            # It caused: clang-15: error: invalid argument '-mmacos-version-min=11.0' not allowed with '-miphoneos-version-min=7.0'
            stdenv = if pkgs.stdenv.isDarwin then pkgs.stdenvNoCC else pkgs.llvmPackages_18.stdenv;
          }) {
            nativeBuildInputs = [
              # Tools
              pkgs.rustup
              pkgs.just
              pkgs.nodejs
              pkgs.mdbook
              pkgs.wasm-bindgen-cli_0_2_126 # Also update in Cargo.toml and CI scripts
              pkgs.cargo-criterion
              pkgs.nixpkgs-fmt
              # System dependencies
              pkgs.flatbuffers
              pkgs.protobuf
              pkgs.jdk17
              pkgs.sqlite
              pkgs.pkg-config
            ] ++ pkgs.lib.optionals pkgs.stdenv.isLinux [
              pkgs.tracy-x11
              pkgs.renderdoc
              pkgs.xorg.libXrandr
              pkgs.xorg.libXi
              pkgs.xorg.libXcursor
              pkgs.xorg.libX11
              pkgs.libxkbcommon
              pkgs.wayland
            ];
            shellHook = ''
              export LD_LIBRARY_PATH="$LD_LIBRARY_PATH:${pkgs.lib.makeLibraryPath [
                pkgs.libxkbcommon
                pkgs.vulkan-loader
                pkgs.libglvnd
              ]}";
            '';
          };
        });
    };
}
