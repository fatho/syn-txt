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
}
