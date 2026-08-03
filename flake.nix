{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";
    systems.url = "github:nix-systems/default";

    naersk = {
      url = "github:nix-community/naersk";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    gitignore = {
      url = "github:hercules-ci/gitignore.nix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = inputs @ {self, ...}: let
    forAllSystems = f:
      inputs.nixpkgs.lib.genAttrs
      (import inputs.systems)
      (system: let
        pkgs = import inputs.nixpkgs {
          inherit system;
          overlays = [inputs.rust-overlay.overlays.default];
        };
      in
        f system pkgs);

    meta = (inputs.nixpkgs.lib.importTOML ./neothesia/Cargo.toml).package;
  in {
    packages = forAllSystems (_system: pkgs: let
      rustToolchain =
        pkgs.rust-bin.nightly.latest.default.override
        {extensions = ["rust-src" "rust-analyzer"];};

      naersk = pkgs.callPackage inputs.naersk {
        rustc = rustToolchain;
        cargo = rustToolchain;
        clippy = rustToolchain;
      };
    in {
      default = naersk.buildPackage {
        meta.mainProgram = "neothesia";
        src = pkgs.lib.cleanSourceWith {
          name = "${meta.name}-${meta.version}-src";
          src = ./.;
          filter = inputs.gitignore.lib.gitignoreFilterWith {
            basePath = ./.;
            extraRules =
              # gitignore
              ''
                flake.*
                LICENSE.md
                README.md
                .github
                makefile
                *.sf2
                *.mid
                docs
              '';
          };
        };
      };
    });

    devShells = forAllSystems (system: pkgs: {
      default =
        pkgs.mkShell
        {inputsFrom = [self.packages.${system}.default];};
    });
  };
}
