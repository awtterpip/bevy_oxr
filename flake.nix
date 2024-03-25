{
  description = "bevy xr flake";
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    # eachDefaultSystem and other utility functions
    utils.url = "github:numtide/flake-utils";
    # Replacement for rustup
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = {
    self,
    nixpkgs,
    utils,
    fenix,
  }:
  # This helper function abstracts over the host platform.
  # See https://github.com/numtide/flake-utils#eachdefaultsystem--system---attrs
    utils.lib.eachDefaultSystem (
      system: let
        pkgs = nixpkgs.legacyPackages.${system};
        # Brings in the rust toolchain from the standard file
        # that rustup/cargo uses.
        rustToolchain = fenix.packages.${system}.fromToolchainFile {
          file = ./rust-toolchain.toml;
          sha256 = "sha256-+syqAd2kX8KVa8/U2gz3blIQTTsYYt3U63xBWaGOSc8=";
        };
        rustPlatform = pkgs.makeRustPlatform {
          inherit (rustToolchain) cargo rustc;
        };
      in
        # See https://nixos.wiki/wiki/Flakes#Output_schema
        {
          # `nix develop` pulls all of this in to become your shell.
          devShells.default = pkgs.mkShell rec {
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

            # see https://github.com/NixOS/nixpkgs/blob/95b81c96f863ca8911dffcda45d1937efcd66a4b/pkgs/games/jumpy/default.nix#L60C5-L60C38
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
