# Legato

This is a WIP audio graph framework for Rust.

### Getting Started

At the moment, it's fairly DIY. There are a few examples for setting this up with CPAL. There will also be a number of different scripts to graph data.

```
nix run .#apps.x86_64-linux.spectrogram -- --path ./example.wav --out ./example.png
```


### Planned Features For 0.1.0

- Minimal DSL or macros for graph construction
- SIMD integration for hot paths like FIR, interpolation, etc.
- Semi-tuned NixOS images
- MIDI context and graph
- Fancy docs and examples
- Symponia integration instead of FFMPEG