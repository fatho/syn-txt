let
  upstream = builtins.fromJSON (builtins.readFile ./nixpkgs.json);
in
  import (fetchTarball upstream)
