{ nixpkgs ? import ./nix/nixpkgs-pinned.nix {
    overlays = [
      (import ./nix/rust-analyzer.nix)
    ];
  }
}:
{
  syn-txt = with nixpkgs; rustPlatform.buildRustPackage rec {
    pname = "syn-txt";
    version = "0.0.1";

    src =
      let
        prefixWhitelist = builtins.map builtins.toString [
          ./Cargo.toml
          ./Cargo.lock
          ./LICENSE
          # Blanket-include for subdirectories
          ./examples
          ./src
        ];
        # Compute source based on whitelist
        whitelistedSrc = lib.cleanSourceWith {
          src = lib.cleanSource ./.;
          filter = path: _type: lib.any (prefix: lib.hasPrefix prefix path) prefixWhitelist;
        };
        # Blacklist some additional files hiding in subdirectories
        blacklistedSrc = lib.cleanSourceWith {
          src = whitelistedSrc;
          filter = path: type:
            ! (  lib.hasInfix "/__pycache__/" path
              || lib.hasSuffix "/__pycache__" path
              || lib.hasSuffix ".md" path
              );
        };
      in
        blacklistedSrc;

    cargoSha256 = "0342vns1krnkkbgzzbsfqhixcfk0qgc79xsajb6g2cbrlhxa4bhz";
  };

  syn-txt-doc = nixpkgs.callPackage ./doc/default.nix {};
}
