{
  inputs = {
    nixpkgs.url = "nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
    crane.url = "github:ipetkov/crane";
  };

  outputs =
    {
      nixpkgs,
      flake-utils,
      rust-overlay,
      crane,
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

        crate = craneLib.buildPackage {
          src = pkgs.lib.fileset.toSource {
            root = ./.;
            fileset = pkgs.lib.fileset.union (pkgs.lib.fileset.fromSource (craneLib.cleanCargoSource ./.)) (
              pkgs.lib.fileset.fileFilter (file: true) ./src/shell
            );
          };
          strictDeps = true;
          # Our completion tests fail when run by nix.
          doCheck = false;
          meta.mainProgram = "envswitch";
        };
      in
      {
        checks = {
          inherit crate;
        };
        packages = {
          default = crate;
          inherit crate;
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
