{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-25.05";
    flake-utils.url = "github:numtide/flake-utils";
    pre-commit-hooks.url = "github:cachix/pre-commit-hooks.nix";
    crane.url = "github:ipetkov/crane";

    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.rust-analyzer-src.follows = "";
    };
  };

  outputs = { self, nixpkgs, flake-utils, pre-commit-hooks, fenix, crane }:
    flake-utils.lib.eachSystem [ "x86_64-linux" "aarch64-darwin" ] (system:
      let
        version = "0.1.0-alpha";
        pkgs = import nixpkgs { inherit system; overlays = [ ]; };

        craneLib = (crane.mkLib pkgs).overrideToolchain
          fenix.packages.${system}.stable.toolchain;
      in
      {
        checks = { };

        packages = { };

        devShells = {
          default = craneLib.devShell {
            packages = with pkgs; [
              nil
              nixpkgs-fmt
              cargo-watch
              just

              openssl
              pkg-config
              sqlite
              git
            ] ++ pkgs.lib.optionals pkgs.stdenv.isDarwin ([ pkgs.libiconv ]);
          };
        };
      });
}
