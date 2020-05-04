{ nixpkgs ? import ./nix/nixpkgs-pinned.nix {
    overlays = [
      (import ./nix/rust-analyzer.nix)
    ];
  }
}:
nixpkgs.mkShell {
  name = "syntxt-dev";
  nativeBuildInputs = with nixpkgs; [
    rustc
    cargo
    rust-analyzer
    clippy
    rustfmt
    # For running the examples
    sox
    # Allows running the update script right from this shell
    python3
    git
    nix
  ];

  # Always enable rust backtraces in development shell
  RUST_BACKTRACE = "1";

  # Provide sources for rust-analyzer, because nixpkgs rustc doesn't include them in the sysroot
  RUST_SRC_PATH = "${nixpkgs.rustPlatform.rustcSrc}";
}
