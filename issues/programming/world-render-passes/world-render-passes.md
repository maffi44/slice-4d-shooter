# Schedule scene, texture and UI passes per world

- **Status:** Closed
- **Priority:** High
- **Created:** 2026-11-24
- **Closed:** 2026-11-28
- **Resolution:** Done
- **Context:** [Initial design and implementation order](world-render-passes-context.md)

[accepted] Implemented together with [render resources](../render-resources/) using the current HUD. [Fyrox UI](../fyrox-ui/) follows as a separate stage.

## Execution and lifetime

A rendered world reserves a slot before actor callbacks and publishes its complete plan after activation. Structural edits replace the plan at a tick boundary. Ordinary parameter writes reuse the existing passes. Each due slot encodes its local producers, scene, optional upscale/bloom and HUD into the scheduler's frame encoder. All due slots and presentation share a submission.

A child is registered later and drawn before its parent. The parent displays the latest completed child output. Before the first draw and after teardown the corresponding widget is skipped. Pausing and resizing together preserve the old texture until the next due draw. A local texture can feed several widgets in its owning world. World-output image bindings validate registration order and follow the source texture generation.

The descriptor selects `with_upscale` and each view may override it. Upscaled worlds publish nominal-resolution output with the HUD drawn after bloom. Direct worlds publish scaled output with the HUD. The window presents the finished image. The XWZ minimap draws as a regular HUD widget.

## Review

The [offscreen regression](../../../src/engine/shared_systems/render/render_scheduler/tests.rs) reads pixels from a parent showing a child at a different cadence, checks pause/resize/resume and verifies that the stale image disappears on retirement. See the [verification log](world-render-passes-context.md) for environment limits and the [architecture reference](../../../docs/engine-architecture.md) for caller contracts. The author accepted the revised implementation on 2026-11-28.
