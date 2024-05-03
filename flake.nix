{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";

    flake-parts.url = "github:hercules-ci/flake-parts";
    systems.url = "github:nix-systems/default";

    naersk.url = "github:nix-community/naersk";
    flake-compat.url = "https://flakehub.com/f/edolstra/flake-compat/1.tar.gz";

    # Dev tools
    treefmt-nix.url = "github:numtide/treefmt-nix";
    treefmt-nix.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = inputs@{ flake-parts, self, ... }:
    inputs.flake-parts.lib.mkFlake { inherit inputs self; } {
      systems = import inputs.systems;
      flake = {
          nixosModules = {
            default = {
              imports = [ ./module.nix ];
              nixpkgs.overlays = [ self.overlays.default ];
            };
          };
      };
      imports = [
        inputs.treefmt-nix.flakeModule
        inputs.flake-parts.flakeModules.easyOverlay
      ];
      perSystem = { config, self', pkgs, lib, system, ... }:
        let
          cargoToml = builtins.fromTOML (builtins.readFile ./Cargo.toml);
          cargoMigToml = builtins.fromTOML (builtins.readFile ./migration/Cargo.toml);
          nonRustDeps = with pkgs; [
            libiconv
            openssl
          ];
          naersk' = pkgs.callPackage inputs.naersk { };
          rust-toolchain = pkgs.symlinkJoin {
            name = "rust-toolchain";
            paths = [ pkgs.rustc pkgs.cargo pkgs.cargo-watch pkgs.rust-analyzer pkgs.rustPlatform.rustcSrc ];
          };
          checks = {
            x86_64-linux.default = config.packages.default;
          };
        in
        {
          overlayAttrs = {
            inherit (config.packages) lysand-ap-layer ls-ap-migration;
          };
          # Rust package
          packages.default = naersk'.buildPackage {
            inherit (cargoToml.package) name version;
            src = ./.;
            buildInputs = nonRustDeps;
            nativeBuildInputs = with pkgs; [
              rust-toolchain
              pkg-config
            ];
          };
          packages.lysand-ap-layer = naersk'.buildPackage {
            inherit (cargoToml.package) name version;
            src = ./.;
            buildInputs = nonRustDeps;
            nativeBuildInputs = with pkgs; [
              rust-toolchain
              pkg-config
            ];
          };
          packages.ls-ap-migration = naersk'.buildPackage {
            inherit (cargoMigToml.package) name version;
            src = ./migration;
            buildInputs = nonRustDeps;
            nativeBuildInputs = with pkgs; [
              rust-toolchain
              pkg-config
            ];
          };

          # Rust dev environment
          devShells.default = pkgs.mkShell {
            inputsFrom = [
              config.treefmt.build.devShell
            ];
            shellHook = ''
              # For rust-analyzer 'hover' tooltips to work.
              export RUST_SRC_PATH=${pkgs.rustPlatform.rustLibSrc}

              echo
              echo "🍎🍎 Run 'just <recipe>' to get started"
              just
            '';
            buildInputs = nonRustDeps;
            nativeBuildInputs = with pkgs; [
              just
              rust-toolchain
              pkg-config
              sea-orm-cli
            ];
            RUST_BACKTRACE = 1;
          };

          # Add your auto-formatters here.
          # cf. https://numtide.github.io/treefmt/
          treefmt.config = {
            projectRootFile = "flake.nix";
            programs = {
              nixpkgs-fmt.enable = true;
              rustfmt.enable = true;
            };
          };
        };
    };
}
