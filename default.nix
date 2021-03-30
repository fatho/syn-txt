{ nixpkgs ? import ./nix/nixpkgs.nix {}
}:
let
  buildRust151Package = nixpkgs.rustPlatform.buildRustPackage.override { rustc = nixpkgs.rustToolchain; };
  version = "0.0.1";
in
rec {
  syntxt = with nixpkgs; buildRust151Package rec {
    pname = "syntxt";
    inherit version;

    src =
      let
        # Remove all things we don't need such as python caches that otherwise mess up the build
        # by causing unnecessary rebuilds due to supposedly changed inputs.
        blacklistedSrc = lib.cleanSourceWith {
          src = ./.;
          filter =
            let
              gitignore = ''
                *
                !syntxt-audio/
                !syntxt-core/
                !syntxt-lang/
                !syntxt-web-wasm/
                !Cargo.toml
                !Cargo.lock
                !LICENSE
                ${builtins.readFile ./.gitignore}
              '';
              extraFilter = path: type: true;
            in
              nix-gitignore.gitignoreFilterPure extraFilter gitignore ./.
            ;
        };
      in
        blacklistedSrc;

    nativeBuildInputs = [wasm-pack wasm-bindgen-cli binaryen_90_x86];
    buildInputs = [sox];

    postBuild = ''
      wasm-bindgen --version
      # BIG HACK: wasm-pack insists on downloading wasm-opt from GitHub itself rather than
      # being content with whatever is on path. Fortunately though, it caches the downloads
      # locally. This happens to be the hash under which the current version of wasm-pack
      # caches wasm-opt, so we just put it there.
      mkdir -p /tmp/wasm-pack-cache/wasm-opt-4d7a65327e9363b7/
      export WASM_PACK_CACHE=/tmp/wasm-pack-cache
      cp ${binaryen_90_x86}/bin/wasm-opt /tmp/wasm-pack-cache/wasm-opt-4d7a65327e9363b7/wasm-opt
      wasm-pack build --target web --release syntxt-web-wasm
    '';

    # buildPhase = ''
    #   echo Hello from the build, whereever that is...
    # '';

    postInstall = ''
      # Additionally include the examples in the output
      mkdir -p $out/examples
      ls -lAh target
      cp target/x86_64-unknown-linux-gnu/release/examples/demo $out/examples

      # And the WASM stuff
      mkdir -p $out/lib/wasm/pkg
      cp syntxt-web-wasm/index.html $out/lib/wasm
      cp syntxt-web-wasm/pkg/*.{wasm,js} $out/lib/wasm/pkg
      cp -r syntxt-web-wasm/static $out/lib/wasm/static
    '';

    NIX_SOX_BIN = "${sox}/bin";

    cargoSha256 = "1qxn8aggwz7jzlp06c51im005x78zl3qq3y5bhg0h7h0xdr3ykk3";
  };

  syntxt-doc = nixpkgs.callPackage ./doc/default.nix {};

  syntxt-wasm-release = nixpkgs.runCommand "syntxt-wasm-release-${version}"
    { inherit syntxt;
      monacoEditor = nixpkgs.monaco-editor;
    }
    ''
    mkdir -p $out/monaco-editor
    cp -r $syntxt/lib/wasm/* $out
    cp ${./doc/favicon.ico} $out/favicon.ico
    cp ${./doc/logo.png} $out/logo.png
    cp -r $monacoEditor/min $out/monaco-editor/min
    '';
}
