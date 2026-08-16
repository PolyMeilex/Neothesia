{
  nixConfig.bash-prompt-prefix = "(neothesia) ";

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
        f pkgs.stdenv.hostPlatform.system pkgs);

    meta = (inputs.nixpkgs.lib.importTOML ./neothesia/Cargo.toml).package;

    workspace = inputs.nixpkgs.lib.cleanSourceWith {
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
  in {
    packages = forAllSystems (system: pkgs: let
      rustToolchain =
        pkgs.rust-bin.nightly.latest.default.override
        {extensions = ["rust-src" "rust-analyzer"];};

      naersk = pkgs.callPackage inputs.naersk {
        rustc = rustToolchain;
        cargo = rustToolchain;
        clippy = rustToolchain;
      };
    in {
      # `nix run github:PolyMeilex/Neothesia`
      default = inputs.self.outputs.packages.${system}.neothesia;
      neothesia = naersk.buildPackage {
        meta.mainProgram = "neothesia";
        src = workspace;
      };

      # `nix run github:PolyMeilex/Neothesia#cli`
      cli = naersk.buildPackage {
        cargoBuildOptions = prev: prev ++ ["--package" "neothesia-cli"];
        nativeBuildInputs = [
          pkgs.pkg-config
          pkgs.ffmpeg_8.dev
        ];
        meta.mainProgram = "neothesia-cli";
        src = workspace;
      };
    });

    devShells = forAllSystems (system: pkgs: {
      # `nix develop github:PolyMeilex/Neothesia`
      default = pkgs.mkShell {
        # grab all build dependencies of all exposed packages
        inputsFrom = pkgs.lib.attrValues inputs.self.packages.${system};
      };
    });
  };
}
