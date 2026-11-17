{
  description = "Prune old Zellij sessions";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    { nixpkgs, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs {
          inherit system;
        };

        buildInputs = [
          pkgs.openssl
        ];

        nativeBuildInputs = [
          pkgs.rustc
          pkgs.cargo
          pkgs.pkg-config
        ];
      in
      {
        devShells.default = pkgs.mkShell {
          packages = [
            pkgs.zellij
            pkgs.clippy
            pkgs.rustfmt
            pkgs.rust-analyzer
          ]
          ++ buildInputs
          ++ nativeBuildInputs;
        };

        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "zellij-prune";
          version = "0.1.0";

          src = pkgs.lib.cleanSource ./.;

          cargoLock = {
            lockFile = ./Cargo.lock;
          };

          inherit buildInputs nativeBuildInputs;
        };
      }
    );
}
