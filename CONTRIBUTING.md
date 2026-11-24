# Contributing to XEMSS

Use the [quick start](README.md#quick-start) to set up Windows x64. The
[architecture guide](docs/architecture.md) identifies the crate and tests
for each part of the project.

## Report a bug

Search the [issue tracker](https://github.com/tecnofifamobile-ai/XEMSS/issues) for the same failure. Include the
XEMSS commit or version, Windows version, exact command, expected result,
and actual output. For a GPU result, include the adapter, driver version,
and whether the run used hardware or WARP.

Attach a small synthetic input or the code that generates it when possible.
Keep commercial or confidential inputs local. Follow the
[source and asset policy](docs/provenance.md) for material included in a report.

## Verify a change

Run the checks used by [Windows CI](.github/workflows/ci.yml).

```powershell
cargo fmt -p xemss -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
```

The CLI tests generate their own shader. To exercise the optional Windows
SDK validation path, set `XEMSS_TEST_DXIL` to the absolute path to the SDK's
`dxil.dll` and run the integration tests.

```powershell
$env:XEMSS_TEST_DXIL = 'C:\Program Files (x86)\Windows Kits\10\bin\<version>\x64\dxil.dll'
cargo test -p xemss --locked --test cli
```

Replace `<version>` with the installed SDK version. For shader changes, record
which GPU backends ran and whether any SDK-dependent comparisons were skipped.

## Open a pull request

Describe the user-visible change, how to reproduce it, and the checks you ran.
Include a regression test when fixing behavior the existing tests miss. Link
the relevant issue or upstream report when one exists. Preserve upstream
notices when changing code imported from VIctory.
