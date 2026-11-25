# Compatibility roadmap

The [GTA VI PC Race standings](https://vuyp.github.io/gta6-pc-race/#standings)
are a starting point for reading public project reports. XEMSS records its
own results with the exact revision, adapter, driver, and workload.

| Milestone | Acceptance criterion |
| --- | --- |
| PE and shader tools | Inspect an AMD64 PE, translate a synthetic compute shader, validate the emitted DXIL with a Windows SDK, and reject parts inside the DXBC header or offset table. |
| Loader | Map and relocate a generated AMD64 image, resolve synthetic imports, and call its entry point with a defined result and clear failures for unsupported facilities. |
| D3D12 runtime | Create descriptor heaps and root bindings, submit a synthetic draw and compute dispatch, and check pixel and buffer readbacks against known values. Verify [descriptor slots are reclaimed when binding fails](https://github.com/Marvisak/VIctory/issues/911). |
| Shader compatibility | Add minimal regression cases for [helper-lane control flow](https://github.com/Marvisak/VIctory/issues/908) and [typed UAV component masks](https://github.com/Marvisak/VIctory/issues/920). Compare hardware and WARP where the feature is supported. |
| Rendering diagnosis | Capture equivalent frame stages with authorized inputs and identify the first differing pass or resource. Publish the diagnosis and a synthetic reproducer. |

The shader tests cover ordering of signed zero in two- and three-operand
RDNA2 floating-point min/max. [XCL #1282](https://github.com/ivall/xcl/issues/1282)
describes a related discrepancy on other hosts. XEMSS checks both f32
and f16 output bits through its own DXIL pipeline.
