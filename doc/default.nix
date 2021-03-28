# Generate the documentation using sphinx
{lib, stdenv, python37}:
stdenv.mkDerivation {
  name = "syn-txt-doc";

  src =
    let
      whitelist = builtins.map builtins.toString [
        ./source

        ./source/_static
        ./source/_static/custom.css
        # TODO: generate these file on the fly as part of the build step
        ./source/_static/chords.ogg
        ./source/_static/demo.ogg

        ./source/conf.py
        ./source/index.rst

        ./favicon.ico
        ./logo.png
        ./Makefile
      ];
    in
      lib.cleanSourceWith {
        src = lib.cleanSource ./.;
        filter = path: _type: lib.elem path whitelist;
      };

    phases = ["unpackPhase" "buildPhase" "installPhase"];
    nativeBuildInputs = [
      (python37.withPackages (ps: with ps; [
        sphinx
      ]))
    ];
    buildPhase = ''
      # Sphinx wants this directory, so lets create it
      mkdir ./source/_templates

      # Put the logo where sphinx can find it
      mv ./logo.png ./source/_static
      mv ./favicon.ico ./source

      # We need to override this date variable to get a 2020 copyright statement,
      # instead of a 1970 one.
      SOURCE_DATE_EPOCH=1588793900 make html
    '';
    installPhase = ''
      mkdir $out
      cp -r _build/html $out
    '';
}
