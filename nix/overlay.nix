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
}
