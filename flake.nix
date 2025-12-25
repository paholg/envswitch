{
  inputs = {
    crane.url = "github:ipetkov/crane";
    flake-utils.url = "github:numtide/flake-utils";
    nixpkgs.url = "nixpkgs/nixos-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      crane,
      flake-utils,
      nixpkgs,
      rust-overlay,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        overlays = [
          (import rust-overlay)
        ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        rustMinimal = pkgs.rust-bin.stable.latest.minimal;
        rustDev = pkgs.rust-bin.stable.latest.default.override {
          extensions = [
            "rust-analyzer"
            "rust-src"
          ];
        };

        craneLib = (crane.mkLib pkgs).overrideToolchain (p: rustMinimal);

        shells = with pkgs; [
          bash
          fish
          zsh
        ];

        commonArgs = {
          src = pkgs.lib.fileset.toSource {
            root = ./.;
            fileset = pkgs.lib.fileset.union (pkgs.lib.fileset.fromSource (craneLib.cleanCargoSource ./.)) (
              pkgs.lib.fileset.fileFilter (file: true) ./src/shell
            );
          };
          strictDeps = true;
          nativeBuildInputs = shells;
        };

        artifacts = commonArgs // {
          cargoArtifacts = craneLib.buildDepsOnly commonArgs;
        };

        envswitch = craneLib.buildPackage (
          artifacts
          // {
            meta.mainProgram = "envswitch";
            doCheck = false;
          }
        );

      in
      {
        checks = {
          clippy = craneLib.cargoClippy (
            artifacts
            // {
              cargoClippyExtraArgs = "-- --deny warnings";
            }
          );
          fmt = craneLib.cargoFmt artifacts;
          test = craneLib.cargoNextest artifacts;
        };
        packages = {
          inherit envswitch;
          default = envswitch;
        };
        devShells.default = pkgs.mkShell {
          packages =
            with pkgs;
            [
              cargo-dist
              cargo-edit
              cargo-nextest
              just
            ]
            ++ [ rustDev ]
            ++ shells;
        };
      }
    );
}
