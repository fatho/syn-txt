{ nixpkgs ? import ./nix/nixpkgs.nix {}
}:
nixpkgs.mkShell {
  name = "syntxt-dev";
  nativeBuildInputs = with nixpkgs; [
    rustToolchain
    cargo-audit
    # For running the examples
    sox
    # For documentation stuff
    (python3.withPackages (ps: [
      ps.sphinx
    ]))
    # For web stuff
    cargo-generate
    wasm-pack
    geckodriver
    # Allows running the update script right from this shell
    niv
  ];

  # Always enable rust backtraces in development shell
  RUST_BACKTRACE = "1";

  # Provide the path to the monaco editor for our development server
  MONACO_EDITOR_SRC = "${nixpkgs.monaco-editor}";
}
