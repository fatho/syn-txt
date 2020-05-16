self: super:
let
  upstream = builtins.fromJSON (builtins.readFile ./rust-analyzer.json);
in
{
  rust-analyzer = self.stdenv.mkDerivation {
    pname = "rust-analyzer";
    version = upstream.version;
    # We use the precompiled binary, because the latest rust-analyzer only
    # builds with the latest cargo version, which nixpkgs-unstable does not
    # provide yet.
    src = self.fetchurl { inherit (upstream) url sha256; };
    phases = ["buildPhase" "installPhase" "fixupPhase"];
    # However, the precompiled binary is not expecting the path layout of NixOS,
    # therefore we first patch it to use the correct dynamic linker.
    buildPhase = ''
      cp $src ./rust-analyzer
      chmod 777 ./rust-analyzer
      patchelf --set-interpreter ${self.glibc}/lib/ld-linux-x86-64.so.2 ./rust-analyzer
    '';
    installPhase = ''
      mkdir -p $out/bin
      cp ./rust-analyzer $out/bin/rust-analyzer-linux
    '';
  };
}
