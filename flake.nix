{
  description = "A Nix-flake-based Rust development environment";

  inputs = {
    nixpkgs.url = "https://flakehub.com/f/NixOS/nixpkgs/0.1"; # unstable Nixpkgs
    fenix = {
      url = "https://flakehub.com/f/nix-community/fenix/0.1";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    crane.url = "github:ipetkov/crane";
  };

  outputs =
    { self, ... }@inputs:

    let
      supportedSystems = [
        "x86_64-linux"
        "aarch64-linux"
        "x86_64-darwin"
        "aarch64-darwin"
      ];
      forEachSupportedSystem =
        f:
        inputs.nixpkgs.lib.genAttrs supportedSystems (
          system:
          f {
            inherit system;
            pkgs = import inputs.nixpkgs {
              inherit system;
              overlays = [
                inputs.self.overlays.default
              ];
            };
          }
        );
      gpuiRuntimeLibraries =
        pkgs: with pkgs; [
          fontconfig
          libxkbcommon
          wayland
          vulkan-loader
        ];
      mkPackage =
        { pkgs, ... }:
        let
          craneLib = (inputs.crane.mkLib pkgs).overrideToolchain pkgs.rustToolchain;
        in
        craneLib.buildPackage {
          pname = "gpui-demo-input";
          version = "0.1.0";
          src = craneLib.cleanCargoSource ./.;
          strictDeps = true;

          nativeBuildInputs = with pkgs; [
            pkg-config
          ];

          buildInputs = gpuiRuntimeLibraries pkgs;

          NIX_LDFLAGS = "-rpath ${pkgs.lib.makeLibraryPath (gpuiRuntimeLibraries pkgs)}";

          dontPatchELF = true;
        };
    in
    {
      overlays.default = final: prev: {
        rustToolchain =
          with inputs.fenix.packages.${prev.stdenv.hostPlatform.system};
          combine (
            with stable;
            [
              clippy
              rustc
              cargo
              rustfmt
              rust-src
            ]
          );
      };

      devShells = forEachSupportedSystem (
        { pkgs, system }:
        {
          default = pkgs.mkShell {
            packages =
              (with pkgs; [
                rustToolchain
                openssl
                pkg-config
                cargo-deny
                cargo-edit
                cargo-watch
                rust-analyzer
                self.formatter.${system}
              ])
              ++ gpuiRuntimeLibraries pkgs;

            dontPatchELF = true;

            env = {
              # Required by rust-analyzer
              RUST_SRC_PATH = "${pkgs.rustToolchain}/lib/rustlib/src/rust/library";

              LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath (gpuiRuntimeLibraries pkgs);
            };
          };
        }
      );

      packages = forEachSupportedSystem (
        { pkgs, system }:
        pkgs.lib.optionalAttrs (system == "x86_64-linux") (
          let
            package = mkPackage { inherit pkgs; };
          in
          {
            default = package;
            gpui-demo-input = package;
          }
        )
      );

      formatter = forEachSupportedSystem ({ pkgs, ... }: pkgs.nixfmt);
    };
}
