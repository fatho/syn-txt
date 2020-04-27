# Rust-Nix Template

Repository template for a Rust project that pulls in the rust compiler (and cargo etc.) via nix.
The `shell.nix` file provides an environment containing `rustc` and `cargo`, as well as the libstd source code for facilitating the use of [rust-analyzer](https://github.com/rust-analyzer/rust-analyzer).

## Updating Upstream

In order to keep up with upstream [nixpkgs](https://github.com/NixOS/nixpkgs/tree/nixpkgs-unstable) and [nixpkgs-mozilla](https://github.com/mozilla/nixpkgs-mozilla),
there is an update script that adjusts the URLs and hashes in [`nix/nixpkgs.json`](nix/nixpkgs.json) and  [`nix/nixpkgs-mozilla.json`](nix/nixpkgs-mozilla.json) to the latest commits of the linked branches.
Simply run `./nix/update.py` within the `nix-shell` provides by `shell.nix`.