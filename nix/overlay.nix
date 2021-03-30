{ sources ? import ./sources.nix }:
self: super: {
  sources = if super ? sources then super.sources // sources else sources;

  monaco-editor = self.stdenv.mkDerivation {
    pname = "monaco-editor";
    version = self.sources.monaco-editor.version;
    src = self.sources.monaco-editor;
    dontBuild = true;
    installPhase = ''
      cp -r $src $out
    '';
  };

  rustChannel = self.rustChannelOf { channel = "1.51.0"; };
  rustToolchain = self.rustChannel.rust.override {
    targets = [
      (self.rust.toRustTarget self.stdenv.hostPlatform)
      "wasm32-unknown-unknown"
    ];
    extensions = [
      "rust-src"
    ];
  };

  wasm-bindgen-cli = self.callPackage
    ({ rustPlatform, fetchFromGitHub, lib, openssl, pkg-config, stdenv, runCommand
    }:

    rustPlatform.buildRustPackage rec {
      pname = "wasm-bindgen-cli";
      version = "0.2.73";

      src =
        let
          tarball = fetchFromGitHub {
            owner = "rustwasm";
            repo = "wasm-bindgen";
            rev = version;
            sha256 = "0g6v2k83gwddxbb4hljk9f58spd6ig2c8swwbhcimanrkzsx5dr6";
          };
        in runCommand "source" { } ''
          cp -R ${tarball} $out
          chmod -R +w $out
          cp ${./wasm-bindgen.cargo.lock} $out/Cargo.lock
        '';

      buildInputs = [ openssl ];
      nativeBuildInputs = [ pkg-config ];

      cargoSha256 = "12mm4nsmi42i3q6fa6ghh743varl87wy4kds1hwi9hdbj6nxbnmz";
      cargoBuildFlags = [ "-p" pname ];

      meta = with lib; {
        homepage = "https://rustwasm.github.io/docs/wasm-bindgen/";
        license = licenses.asl20;
        description = "Facilitating high-level interactions between wasm modules and JavaScript";
        maintainers = with maintainers; [ ma27 rizary ];
        platforms = platforms.unix;
      };
    }) {};

  # Note, since our target is wasm32, we need the x86 version of binaryen, *not* x86_64
  binaryen_90_x86 = self.callPackage (
    { stdenv }:
    stdenv.mkDerivation rec {
      pname = "binaryen";
      version = "90";

      # These are all static binaries ready to be used
      src = fetchTarball {
        url = "https://github.com/WebAssembly/binaryen/releases/download/version_90/binaryen-version_90-x86-linux.tar.gz";
        sha256 = "0sar906h372xm1z6phiq5qphkzqdjz1c9jg04vjlpdr9nq9ikf63";
      };

      dontBuild = true;

      installPhase = ''
        mkdir -p $out/bin
        cp $src/* $out/bin
      '';

      meta = with stdenv.lib; {
        homepage = "https://github.com/WebAssembly/binaryen";
        description = "Compiler infrastructure and toolchain library for WebAssembly, in C++";
        platforms = platforms.all;
        maintainers = with maintainers; [ asppsa ];
        license = licenses.asl20;
      };
    }
  ) {};
}
