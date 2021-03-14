{ nixpkgs ? import (import ./nix/sources.nix).nixpkgs {}
}:
{
  syn-txt = with nixpkgs; rustPlatform.buildRustPackage rec {
    pname = "syn-txt";
    version = "0.0.1";

    src =
      let
        # Remove all things we don't need such as python caches that otherwise mess up the build
        # by causing unnecessary rebuilds due to supposedly changed inputs.
        blacklistedSrc = lib.cleanSourceWith {
          src = ./.;
          filter =
            let
              gitignore = ''
                ${builtins.readFile ./.gitignore}
                *
                !syntxt-audio/
                !Cargo.toml
                !Cargo.lock
                !LICENSE
              '';
              extraFilter = path: type: true;
            in
              nix-gitignore.gitignoreFilterPure extraFilter gitignore ./.
            ;
        };
      in
        blacklistedSrc;

    buildInputs = [sox];

    # Additionally include the examples in the output
    postInstall = ''
      mkdir -p $out/examples
      ls -lAh target
      ${tree}/bin/tree target
      cp target/x86_64-unknown-linux-gnu/release/examples/demo $out/examples
    '';

    NIX_SOX_BIN = "${sox}/bin";

    cargoSha256 = "0nmcv3xr2n7b2ppipw3dy2g3yhb582s2fscrss1nnk0nyny1d2a4";
  };

  syn-txt-doc = nixpkgs.callPackage ./doc/default.nix {};
}
