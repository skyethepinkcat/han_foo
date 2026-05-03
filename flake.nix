{
  inputs = {
    naersk.url = "github:nix-community/naersk/master";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    treefmt-nix.url = "github:numtide/treefmt-nix";
  };

  outputs =
    inputs@{ flake-parts, naersk, ... }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      imports = [
        inputs.treefmt-nix.flakeModule
      ];
      systems = [
        "x86_64-linux"
        "aarch64-linux"
        "aarch64-darwin"
      ];

      perSystem =
        { pkgs, ... }:
        let
          naersk-lib = pkgs.callPackage naersk { };
        in
        {
          treefmt = {
            # Used to find the project root
            projectRootFile = "flake.nix";
            programs = {
              biome.enable = true;
              rustfmt.enable = true;

            };

          };
          packages.default = naersk-lib.buildPackage ./.;

          devShells.default =
            with pkgs;
            mkShell {
              buildInputs = [
                lld
                cargo
                rustc
                rustfmt
                pre-commit
                rust-analyzer
                rustPackages.clippy
                cargo-mommy
                wasm-pack
                just
                miniserve
                vscode-css-languageserver
                geckodriver
              ];
              RUST_SRC_PATH = rustPlatform.rustLibSrc;
            };
        };
    };
}
