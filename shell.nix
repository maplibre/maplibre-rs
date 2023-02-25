{ pkgs ? import <nixpkgs> {} }:
  pkgs.mkShell {
    nativeBuildInputs = [
      pkgs.rustup
      pkgs.just
      pkgs.flatbuffers pkgs.protobuf
      pkgs.libiconv
      pkgs.darwin.apple_sdk.frameworks.ApplicationServices
      pkgs.darwin.apple_sdk.frameworks.CoreVideo
      pkgs.darwin.apple_sdk.frameworks.AppKit
    ];
}