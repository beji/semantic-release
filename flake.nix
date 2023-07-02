{
  description = "Description for the project";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-22.11";
    # for the development shell, as the name suggests
    devshell.url = "github:numtide/devshell";
    # allows filtering files from the sources, prevents rebuilts if, for example, the flake.nix changes
    nix-filter.url = "github:numtide/nix-filter";
    # sort-of working "nix build" intergration for node and rust
    # dream2nix.url = "github:nix-community/dream2nix";
    # for rust toolchain stuff
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = inputs@{ flake-parts, ... }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      imports = [
        # To import a flake module
        # 1. Add foo to inputs
        # 2. Add foo as a parameter to the outputs function
        # 3. Add here: foo.flakeModule
        inputs.devshell.flakeModule
        # inputs.dream2nix.flakeModuleBeta
      ];
      systems = [ "x86_64-linux" ];

      flake = {
        # The usual flake attributes can be defined here, including system-
        # agnostic ones like nixosModule and system-enumerating ones, although
        # those are more easily expressed in perSystem.

      };

      perSystem = { config, self', inputs', pkgs, system, ... }: {
        # Per-system attributes can be defined here. The self' and inputs'
        # module parameters provide easy access to attributes of the same
        # system.

        # dream2nix configuration for builds
        # dream2nix.inputs."replaceme" = {
        #   source = (inputs.nix-filter.lib {
        #     root = ./.;
        #     exclude = [
        #       (inputs.nix-filter.lib.matchExt "nix")
        #       "flake.lock"
        #       ".envrc"
        #       ".secret.envrc"
        #     ];
        #   });
        #   projects = {
        #     replaceme = {
        #       # relPath = "/replaceme";
        #       subsystem = "rust";
        #       translator = "cargo-lock";
        #     };
        #   };
        # };

        # this exposes things to "nix build .#XXXX"
        # the default package could be defined like this:
        # packages.default = config.dream2nix.outputs.replaceme.packages.default;
        #
        # "basic" rust builds without dream2nix
        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "semantic-release";
          version = "1.1.0";
          src = (inputs.nix-filter.lib {
            root = ./.;
            exclude = [
              (inputs.nix-filter.lib.matchExt "nix")
              "flake.lock"
              ".envrc"
              ".secret.envrc"
            ];
          });
          cargoLock.lockFile = ./Cargo.lock;
          nativeBuildInputs = with pkgs;
            [
              pkg-config
              rust-bin.stable.latest.minimal
            ];
          buildInputs = with pkgs; [ ];
        };
        #
        # packages =
        #   {
        #     inherit (config.dream2nix.outputs.frontend.packages) frontend;
        #   };

        # overlays can be defined here
        # see https://flake.parts/overlays.html
        _module.args.pkgs = import inputs.nixpkgs {
          inherit system;
          overlays = [
            inputs.rust-overlay.overlays.default
            # this pulls in something from another input flake, like nixos-unstable
            # (final: prev: {
            #   unstable = inputs.nixpkgs-unstable.legacyPackages.${system};
            # })
          ];
        };

        # packages for devshell can be defined here
        devshells.default = { packages = with pkgs; [
          # language servers
          rust-analyzer
          # rust
          rust-bin.stable.latest.default
          pkg-config
        ];};

      };

    };
}
