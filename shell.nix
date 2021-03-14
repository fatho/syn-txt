{ nixpkgs ? import (import ./nix/sources.nix).nixpkgs {}
}:
nixpkgs.mkShell {
  name = "syntxt-dev";
  nativeBuildInputs = with nixpkgs; [
    rustc
    cargo
    clippy
    rustfmt
    cargo-audit
    # For running the examples
    sox
    # For documentation stuff
    (python3.withPackages (ps: [
      ps.sphinx
    ]))
    # Allows running the update script right from this shell
    niv
  ];

  # Always enable rust backtraces in development shell
  RUST_BACKTRACE = "1";

  # Provide sources for rust-analyzer, because nixpkgs rustc doesn't include them in the sysroot
  RUST_SRC_PATH = "${nixpkgs.rustPlatform.rustLibSrc}";
}
