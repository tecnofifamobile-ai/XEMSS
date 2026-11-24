# Source provenance

XEMSS is distributed under GPL-3.0-or-later. See [LICENSE](../LICENSE).

- `crates/victory-pe` and `crates/victory-shader` come from [VIctory, revision ffe310e](https://github.com/Marvisak/VIctory/tree/ffe310eb2098e52169f4c44db5ffdabf43a72622).
  The import preserves upstream author notices, including the Mesa/Valve MIT
  notice in `crates/victory-shader/src/gfx10/opcodes.rs`.
- The XEMSS CLI, its integration fixture, and the DXBC header-boundary fix are
  maintained here. The imported source starts at the pinned revision.
- [XCL, revision 930e8b4](https://github.com/ivall/xcl/tree/930e8b49c1c2ec683d22c724a398b39111cd0e56)
  is a GPL-2.0-only project used as a public behavior reference. XEMSS
  incorporates no XCL source.

The repository contains source, build configuration, and synthetic test
generators. Keep game executables and assets, firmware, signing keys,
proprietary SDK files, and shader captures containing commercial data outside
the repository. The upstream BCPack table blob is excluded from the port
because [VIctory's asset policy](https://github.com/Marvisak/VIctory/blob/ffe310eb2098e52169f4c44db5ffdabf43a72622/docs/asset-policy.md)
identifies it as outside its open-source license.
