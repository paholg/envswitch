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

        buildInputs = [ ];
        nativeBuildInputs = [ ];

        crate = craneLib.buildPackage {
          inherit buildInputs nativeBuildInputs;
          src = craneLib.cleanCargoSource ./.;
          strictDeps = true;
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
          inherit buildInputs nativeBuildInputs;
          packages = [
            pkgs.cargo-dist
            rust
          ];
        };
      }
    );
}
