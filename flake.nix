{
  description = "Building static binaries with musl";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";

    crane.url = "github:ipetkov/crane";

    flake-utils.url = "github:numtide/flake-utils";

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      nixpkgs,
      crane,
      flake-utils,
      rust-overlay,
      ...
    }:
    flake-utils.lib.eachSystem [ "x86_64-linux" ] (
      system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay) ];
        };

        craneLib = (crane.mkLib pkgs).overrideToolchain (
          p:
          p.rust-bin.stable.latest.default.override {
            targets = [ "x86_64-unknown-linux-gnu" ];
          }
        );

        my-crate = craneLib.buildPackage {
          src = craneLib.cleanCargoSource ./.;
          strictDeps = true;

          nativeBuildInputs = with pkgs; [
            pkg-config
          ];

          buildInputs = with pkgs; [
            libxcb
            libxkbcommon
            wayland
            vulkan-loader
          ];

          NIX_LDFLAGS = "-rpath ${
            pkgs.lib.makeLibraryPath [
              pkgs.wayland
              pkgs.vulkan-loader
            ]
          }";

          dontPatchELF = true;
        };
      in
      {
        checks = {
          inherit my-crate;
        };

        packages.default = my-crate;

        devShells.default = craneLib.devShell {
          inputsFrom = [ my-crate ];
          packages = with pkgs; [
            wayland
            vulkan-loader
          ];
          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath (with pkgs; [
            wayland
            vulkan-loader
          ]);
        };
      }
    );
}
