{ pkgs ? import <nixpkgs> { } }:

with pkgs;

mkShell {
  buildInputs = [
    cargo
    clippy
    rust-analyzer
    rustc
    rustfmt
    maturin
    python311
    python311Packages.python-lsp-server
    libz
  ];

  LD_LIBRARY_PATH = "${pkgs.stdenv.cc.cc.lib}/lib:${pkgs.libz}/lib";
}
