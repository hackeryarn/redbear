{
  pkgs ? import <nixpkgs> { },
}:

with pkgs;

mkShell {
  buildInputs = [
    cargo
    clippy
    rust-analyzer
    rustc
    rustfmt
    maturin
    python314
    python314Packages.python-lsp-server
    libz
  ];

  LD_LIBRARY_PATH = "${pkgs.stdenv.cc.cc.lib}/lib:${pkgs.libz}/lib";
}
