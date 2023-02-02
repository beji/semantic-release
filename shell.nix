{ pkgs ? import <nixpkgs> { } }:

with pkgs;

mkShell {
  buildInputs = [
    # language servers
    rust-analyzer
    # rust
    rust-bin.stable.latest.default
    pkg-config
    openssl_3
  ];
}
