{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = {
        nixpkgs.follows = "nixpkgs";
      };
    };
  };
  outputs =
    inputs@{
      self,
      nixpkgs,
      flake-utils,
      rust-overlay,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        rustToolchain = pkgs.pkgsBuildHost.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
      in
      with pkgs;
      {
        formatter = nixfmt-tree;

        packages = rec {
          nixos-control = rustPlatform.buildRustPackage {
            pname = "nixos-control";
            version = "0.1.0";
            src = self;
            cargoLock.lockFile = ./Cargo.lock;
            nativeBuildInputs = [ pkg-config ];
            buildInputs = [ openssl ];
          };
          default = nixos-control;
        };

        apps.default = flake-utils.lib.mkApp {
          drv = self.packages.${system}.nixos-control;
          exePath = "/bin/nixos-control";
        };

        checks.package = self.packages.${system}.nixos-control;

        devShells.default = mkShell {
          packages = [
            rustToolchain
            rust-analyzer
            cargo-edit
            cargo-watch
            pkg-config
            lld
            clang
            just
            openssl
          ];
        };
      }
    )
    // {
      nixosModules.default =
        moduleArgs@{ pkgs, ... }:
        import ./nix/modules (
          moduleArgs
          // {
            package = self.packages.${pkgs.stdenv.hostPlatform.system}.default;
          }
        );
      nixosModules.nixos-control = self.nixosModules.default;
    };
}
