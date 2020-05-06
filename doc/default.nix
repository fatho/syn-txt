# Generate the documentation using sphinx
{lib, stdenv, python37}:
stdenv.mkDerivation {
  name = "syn-txt-doc";

  src =
    let
      whitelist = builtins.map builtins.toString [
        ./source
        ./source/conf.py
        ./source/index.rst

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
      # Sphinx wants these directories, so lets create them
      mkdir ./source/{_static,_templates}

      # Put the logo where sphinx can find it
      mv ./logo.png ./source/_static

      # We need to override this date variable to get a 2020 copyright statement,
      # instead of a 1970 one.
      SOURCE_DATE_EPOCH=1588793900 make html
    '';
    installPhase = ''
      mkdir $out
      cp -r _build/html $out
    '';
}
