self: super:
{
  rust-analyzer = self.stdenv.mkDerivation rec {
    pname = "rust-analyzer";
    version = "2020-05-04";
    # We use the precompiled binary, because the latest rust-analyzer only
    # builds with the latest cargo version, which nixpkgs-unstable does not
    # provide yet.
    src = self.fetchurl {
      url = "https://github.com/rust-analyzer/rust-analyzer/releases/download/${version}/rust-analyzer-linux";
      sha256 = "1bdmfcqgcddkbmpnv5v632zdvxp7q3nn81nr84nf1a225y4fzn8f";
    };
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
