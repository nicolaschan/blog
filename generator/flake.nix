{
  description = "static site generator";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };
        rust = pkgs.rust-bin.stable.latest.default;

        generator = pkgs.rustPlatform.buildRustPackage {
          pname = "nicolaschan-generator";
          version = "0.1.0";
          src = ./.;
          cargoLock.lockFile = ./Cargo.lock;
        };

        site = pkgs.stdenv.mkDerivation {
          pname = "nicolaschan-site";
          version = "0.1.0";
          src = ./..;

          buildInputs = [ generator ];

          buildPhase = ''
            cd generator
            ${generator}/bin/nicolaschan-generator
          '';

          installPhase = ''
            cp -r $src/dist $out || cp -r ../dist $out
          '';
        };
      in
      {
        packages = {
          default = site;
          inherit generator site;
        };

        devShells.default = pkgs.mkShell {
          buildInputs = [
            rust
            pkgs.rust-analyzer
            pkgs.simple-http-server
          ];

          RUST_SRC_PATH = "${rust}/lib/rustlib/src/rust/library";

          shellHook = ''
            alias serve="simple-http-server -p 8000 -i ../dist"
          '';
        };
      }
    );
}
