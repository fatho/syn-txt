<h1 align="center">
  <a href="https://github.com/fatho/syn-txt"><img src="doc/logo.png" alt="syn.txt" width="256"></a>
</h1>

The goal is for this to be a declarative, text-based synthesizer/audio workstation.
At the moment, it is very much work in progress. See the [roadmap](/planning/roadmap.md)
for a rough outline of what is planned for the near future.

However, there is already a working prototype that includes
- an interpreter for a very small subset of a scheme-like language that can be used for defining piano rolls (see [`test-data/music.syn`](test-data/music.syn) for an example)
- a synthesizer turning notes into waveforms
- the `musicc` (music compiler) to play piano rolls on the built-in test synthesizer

## Usage

The project is written in [rust](https://www.rust-lang.org/) and all tooling is pulled in via [nix](https://nixos.org/nix/).
There is a [`shell.nix`](shell.nix) file declaring all necessary dependencies.
Simply run `nix-shell` in the repository root to drop into a shell where all dependencies are avaiable.
Then, the normal [`cargo`](https://doc.rust-lang.org/cargo/) workflow applies.

```
# Play the included example melody
cargo run --bin musicc test-data/music.syn
```


At the moment, sound is played by spawning `sox` in a subprocess and piping the audio data to it.