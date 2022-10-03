{
  description = "virtual environments";

  inputs.devshell.url = "github:numtide/devshell";
  inputs.flake-utils.url = "github:numtide/flake-utils";

  outputs = { self, flake-utils, devshell, nixpkgs }:
    flake-utils.lib.eachDefaultSystem (system: {
      defaultPackage =
        let pkgs = import nixpkgs { inherit system; };
        in
        pkgs.rustPlatform.buildRustPackage {
          pname = "semantic-release";
          version = "0.3.0";
          src = ./.;
          cargoLock.lockFile = ./Cargo.lock;
          buildInputs = [ pkgs.pkg-config pkgs.openssl_3.dev pkgs.openssl_3 pkgs.gcc ];
          OPENSSL_INCLUDE_DIR = "${pkgs.openssl_3.dev}";
          OPENSSL_LIB_DIR = "${pkgs.openssl_3.out}/lib";
        };
      devShell =
        let
          pkgs = import nixpkgs {
            inherit system;

            overlays = [ devshell.overlay ];
          };
        in
        pkgs.devshell.mkShell {
          imports = [ (pkgs.devshell.importTOML ./devshell.toml) ];
          env = [
            {
              name = "RUST_SRC_PATH";
              value = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
            }
            {
              name = "OPENSSL_INCLUDE_DIR";
              value = "${pkgs.openssl_3.dev}";
            }
            {
              name = "OPENSSL_LIB_DIR";
              value = "${pkgs.openssl_3.out}/lib";
            }
          ];
        };
    });
}
