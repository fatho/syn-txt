{}:
let
  sources = import ./sources.nix;
  nixpkgs = import sources.nixpkgs {
    overlays = [
      (import ./overlay.nix { inherit sources; })
    ];
  };
in
  nixpkgs
