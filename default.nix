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
                !syntxt-lang/
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

    cargoSha256 = "05n6rhjc54w716xp0v1rp41xxfqbvhimhc3rgz6wa87dalcfb0m8";
  };

  syn-txt-doc = nixpkgs.callPackage ./doc/default.nix {};
}
