<h1 align="center">
  <a href="https://github.com/fatho/syn-txt"><img src="doc/logo.png" alt="syn.txt" width="256"></a>
</h1>

<p align="center">
  <a href="https://github.com/fatho/syn-txt/actions?query=branch%3Amaster"><img src="https://github.com/fatho/syn-txt/workflows/Build%20and%20test/badge.svg" alt="Build Status"></a>
</p>

The goal is for this to be a declarative, text-based synthesizer/audio workstation.
At the moment, it is very much work in progress. See the [roadmap](/planning/roadmap.md)
for a rough outline of what is planned for the near future.

However, there is already a working prototype that includes
- an interpreter for a very small subset of a scheme-like language that can be used for defining piano rolls (see [`test-data/demo.syn`](test-data/demo.syn) for an example)
- a synthesizer turning notes into waveforms
- the `musicc` (music compiler) to play piano rolls on the built-in test synthesizer

## Usage

The project is written in [rust](https://www.rust-lang.org/) and all tooling is pulled in via [nix](https://nixos.org/nix/).
There is a [`shell.nix`](shell.nix) file declaring all necessary dependencies.
Simply run `nix-shell` in the repository root to drop into a shell where all dependencies are avaiable.
Then, the normal [`cargo`](https://doc.rust-lang.org/cargo/) workflow applies.

```
# Play the included example melody
cargo run --bin musicc test-data/demo.syn
```

At the moment, it depends on spawning a [sox](http://sox.sourceforge.net/) subprocess and piping the audio data to it for actually playing sound.
If everything worked, it should produce something similar to the following audio snippet:

<audio controls="controls">
  <source src="doc/source/_static/demo.ogg" type="audio/ogg">
  Your browser does not support the <code>audio</code> element. 
</audio>