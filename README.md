# XEMSS

**XBOX Emulator for Series S** is an open-source compatibility project for
Windows x64. Its first milestone ports VIctory's PE and shader components into
a standalone diagnostic tool under GPL-3.0-or-later.

## Current capabilities

- Inspect AMD64 PE32+ headers, imports, delay-load imports, exports, and TLS
  metadata without executing the image.
- Inspect DXBC containers and the stages of console shaders they contain.
- Translate supported RDNA2 vertex, pixel, compute, and mesh shader programs to
  a new DXIL file. On Windows, an installed Windows SDK can validate the output.

Game boot and rendering require a loader and a graphics runtime. Those are
future milestones. Follow the [roadmap](docs/roadmap.md) and
[source provenance](docs/provenance.md) for scope and upstream attribution.

## Build and use

On Windows x64, install Rust via rustup and the MSVC C++ build tools with a
Windows SDK. The Rust version is pinned in `rust-toolchain.toml`.

```powershell
cargo build --release --locked
cargo run --locked -p xemss -- inspect-pe path/to/program.exe --imports
cargo run --locked -p xemss -- inspect-shader path/to/shader.dxbc
cargo run --locked -p xemss -- translate-shader path/to/shader.dxbc -o translated.dxil --validate
```

Supply one DXBC shader container to the shader commands. Translation creates the
output only if that path is free. With `--validate`, the tool tries to find
`dxil.dll` in the Windows SDK. Set `VICTORY_DXIL` to its absolute path to
select a specific version.

## Contribute

```powershell
cargo fmt -p xemss -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
```

Report the command, commit, host OS, expected behavior, and actual behavior.
For graphics issues, include the adapter and driver version and a small
synthetic reproducer. Keep commercial or confidential inputs local. The CLI
integration tests generate their own shader and can also exercise
`--validate` when `XEMSS_TEST_DXIL` points to a Windows SDK validator.
