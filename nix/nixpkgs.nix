{}:
let
  sources = import ./sources.nix;

  nixpkgs = import sources.nixpkgs {
    overlays = [
      (import ./overlay.nix { inherit sources; })
      (import (sources.nixpkgs-mozilla + "/rust-overlay.nix"))
    ];
  };
in
  nixpkgs
