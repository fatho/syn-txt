{ nixpkgs ? import ./nix/nixpkgs-pinned.nix {
    overlays = [
      (import ./nix/rust-analyzer.nix)
    ];
  }
}:
{
  syn-txt-doc = nixpkgs.callPackage ./doc/default.nix {};
}
