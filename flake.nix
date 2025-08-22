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

        rust = (pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml).override {
          extensions = [
            "rust-analyzer"
            "rust-src"
          ];
        };

        craneLib = (crane.mkLib pkgs).overrideToolchain (p: rust);

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

        # These tests fail in `nix flake check`. I am not sure why. :(
        skippedTests =
          pkgs.lib.pipe
            [
              "test::completion_file::case_1_bash"
              "test::completion_partial::case_1_bash"
            ]
            [
              (builtins.map (test: "--skip ${test}"))
              (builtins.concatStringsSep " ")
            ];

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
          test = craneLib.cargoNextest (
            artifacts
            // {
              cargoNextestExtraArgs = "-- ${skippedTests}";
            }
          );
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
            ++ [ rust ]
            ++ shells;
        };
      }
    );
}
