{
  inputs = {
    fenix.url = "github:nix-community/fenix";
    flake-utils.url = "github:numtide/flake-utils";
    naersk.url = "github:nix-community/naersk";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
  };

  outputs = {
    self,
    fenix,
    flake-utils,
    naersk,
    nixpkgs,
  }:
    flake-utils.lib.eachDefaultSystem (
      system: let
        pkgs = nixpkgs.legacyPackages.${system};
        rustToolchain = with fenix.packages.${system};
          combine [
            (stable.withComponents [
              "rustc"
              "cargo"
              "rustfmt"
              "clippy"
              "rust-src"
            ])

            targets.wasm32-unknown-unknown.stable.rust-std
          ];
        rustPlatform = pkgs.makeRustPlatform {
          inherit (rustToolchain) cargo rustc;
        };
      in {
        devShells.default = pkgs.mkShell rec {
          # build dependencies
          nativeBuildInputs = with pkgs; [
            rustToolchain
            pkg-config

            # Common cargo tools we often use
            cargo-deny
            cargo-expand
            cargo-binutils

            # cmake for openxr
            cmake
          ];

          # runtime dependencies
          buildInputs =
            [
              pkgs.zstd
            ]
            ++ pkgs.lib.optionals pkgs.stdenv.isLinux (with pkgs; [
              # bevy dependencies
              udev
              alsa-lib
              # vulkan
              vulkan-loader
              vulkan-headers
              vulkan-tools
              vulkan-validation-layers
              # x11
              xorg.libX11
              xorg.libXcursor
              xorg.libXi
              xorg.libXrandr
              # wayland
              libxkbcommon
              wayland
              # xr
              openxr-loader
              libGL
            ])
            ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
              pkgs.darwin.apple_sdk.frameworks.Cocoa
              rustPlatform.bindgenHook
              # # This is missing on mac m1 nix, for some reason.
              # # see https://stackoverflow.com/a/69732679
              pkgs.libiconv
            ];

          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath buildInputs;
          # this is most likely not needed. for some reason shadows flicker without it.
          AMD_VULKAN_ICD = "RADV";
        };
        # This only formats the nix files.
        formatter = pkgs.nixpkgs-fmt;
      }
    );
}
