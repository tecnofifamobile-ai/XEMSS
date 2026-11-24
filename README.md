# XEMSS

[![Windows CI](https://github.com/tecnofifamobile-ai/XEMSS/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/tecnofifamobile-ai/XEMSS/actions/workflows/ci.yml)

**XBOX Emulator for Series S** is an open-source compatibility project for
Windows x64. The current milestone delivers a command-line toolkit for inspecting
executables and translating shaders. Running games requires the loader and
graphics milestones below.

## Project status

### Available on `main`

| Capability | What works | Verification |
| --- | --- | --- |
| PE inspection | Reads AMD64 PE32+ headers, imports, delay-load imports, exports, and TLS metadata without executing the image. | Parser tests and CLI inspection of its own Windows executable. |
| DXBC inspection | Lists container sections and native shader stages. Rejects sections that point into the container header or offset table. | Synthetic inputs and a parser regression test. |
| RDNA2 shader translation | Emits host DXIL for supported vertex, pixel, compute, and mesh programs. | A synthetic compute shader exercises the full CLI path. |
| DXIL validation | Uses an installed Windows SDK validator before writing the output when requested. | The CLI fixture can exercise the SDK validator locally. |

### Pending

| Milestone | What will demonstrate completion |
| --- | --- |
| AMD64 loader | Map and relocate a synthetic PE, resolve test imports, and call its entry point. |
| D3D12 runtime | Bind resources, submit a synthetic draw and compute dispatch, and verify their readbacks. |
| Broader shader coverage | Reproduce reported control-flow and resource-write discrepancies with synthetic tests. |
| Game compatibility | Measure boot and rendering with authorized inputs. GTA VI remains unverified. |

See the [roadmap](docs/roadmap.md) for acceptance criteria and upstream reports.
[Open pull requests](https://github.com/tecnofifamobile-ai/XEMSS/pulls) show changes awaiting review.

## Quick start

Use Windows x64 with [rustup](https://rustup.rs/), the MSVC C++ build tools,
and a Windows SDK. Cargo uses the Rust version pinned in
[`rust-toolchain.toml`](rust-toolchain.toml).

```powershell
git clone https://github.com/tecnofifamobile-ai/XEMSS.git
Set-Location XEMSS
cargo build --release --locked
.\target\release\xemss.exe --help
```

Try PE inspection on the executable you just built. For the shader commands,
replace `path\to\shader.dxbc` with a local DXBC container you can use.

```powershell
.\target\release\xemss.exe inspect-pe .\target\release\xemss.exe --imports
.\target\release\xemss.exe inspect-shader .\path\to\shader.dxbc
.\target\release\xemss.exe translate-shader .\path\to\shader.dxbc -o translated.dxil --validate
```

Translation requires a new output path. With `--validate`, XEMSS looks for
`dxil.dll` in the Windows SDK. Set `VICTORY_DXIL` to its absolute path to
select a specific validator.

## Repository layout

| Location | Responsibility |
| --- | --- |
| [`apps/xemss/`](apps/xemss/) | CLI commands, file handling, and end-to-end tests. |
| [`crates/victory-pe/`](crates/victory-pe/) | PE32+ parsing and executable metadata. |
| [`crates/victory-shader/`](crates/victory-shader/) | DXBC parsing, RDNA2 translation, DXIL emission, and GPU tests. |
| [`docs/`](docs/) | [Architecture](docs/architecture.md), [roadmap](docs/roadmap.md), and [source provenance](docs/provenance.md). |
| [`.github/`](.github/) | Windows CI and templates for issues and pull requests. |

See [Contributing](CONTRIBUTING.md) for the test commands and how to report
a reproducible failure. XEMSS is distributed under
[GPL-3.0-or-later](LICENSE).
