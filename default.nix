{ pkgs, ... }:

pkgs.rustPlatform.buildRustPackage {
  pname = "semantic-release";
  version = "0.3.0";
  src = ./.;
  cargoLock.lockFile = ./Cargo.lock;
  nativeBuildInputs = with pkgs; [
    gcc
    bintools
    pkg-config
    rust-bin.stable.latest.default
  ];
  # couldn't get this to build with vendored everything...
  buildInputs = with pkgs; [ openssl_3 ];
}
