# Mozilla rust overlay, for better development support
let
  upstream = builtins.fromJSON (builtins.readFile ./nixpkgs-mozilla.json);
in
  import (fetchTarball upstream + "/rust-overlay.nix")
