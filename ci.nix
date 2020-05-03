{ nixpkgs ? import ./nix/nixpkgs-pinned.nix {}
}:
nixpkgs.buildEnv {
  name = "syntxt-ci";
  paths = with nixpkgs; [
    rustc
    cargo
  ];
}
