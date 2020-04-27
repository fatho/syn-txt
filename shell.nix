{ nixpkgs ? import ./nix/nixpkgs-pinned.nix {
    overlays = [
      (import ./nix/rust-overlay.nix)
    ];
  }
}:
let
  rustChannel = nixpkgs.rustChannelOf {
    rustToolchain = ./rust-toolchain;
  };

  rust = rustChannel.rust.override {
    # The source component is needed for rust-analyzer
    extensions = ["rust-src"];
  };
in
nixpkgs.mkShell {
  name = "awesome-rust-app-dev";
  nativeBuildInputs = with nixpkgs; [
    rustc
    cargo
    rust-analyzer
    # For running the examples
    sox
    # Allows running the update script right from this shell
    python3
    git
    nix
  ];
}
