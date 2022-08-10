{
  description = "virtual environments";

  inputs.devshell.url = "github:numtide/devshell";
  inputs.flake-utils.url = "github:numtide/flake-utils";

  outputs = { self, flake-utils, devshell, nixpkgs }:
    flake-utils.lib.eachDefaultSystem (system: {
      defaultPackage = let pkgs = import nixpkgs { inherit system; };
      in pkgs.rustPlatform.buildRustPackage {
        pname = "semantic-release";
        version = "0.2.0";
        src = ./.;
        cargoLock.lockFile = ./Cargo.lock;
      };
      devShell = let
        pkgs = import nixpkgs {
          inherit system;

          overlays = [ devshell.overlay ];
        };
      in pkgs.devshell.mkShell {
        imports = [ (pkgs.devshell.importTOML ./devshell.toml) ];
      };
    });
}
