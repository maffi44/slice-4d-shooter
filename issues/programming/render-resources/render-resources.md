# Render resources for worlds and dynamic UI textures

- **Status:** Closed
- **Priority:** High
- **Created:** 2026-11-20
- **Closed:** 2026-11-28
- **Resolution:** Done
- **Context:** [Scanner ownership and data flow](render-resources-context.md)

[accepted] Implemented together with [per-world scheduling](../world-render-passes/). The current HUD is the first consumer. The next stage is [Fyrox UI](../fyrox-ui/).

## Caller and ownership

- The CPU level resource contains geometry and actor descriptions. Generated scene and minimap pipelines share its static render data and BSP. Headless preparation only requests simulation dependencies.
- A graphical descriptor awaits its `RenderPipelineResource` and `CubemapTexture`. Each live world creates and updates its own typed uniform buffers, chooses the sky sampler and binds its instances. A scene replacement keeps the previous pipeline active until preparation succeeds.
- At activation the scanner calls `create_dynamic_texture` twice. The actor retains both producers. Each plane's local key feeds two team-specific widgets. The obstacle-course player can independently request an XWZ minimap. The level loader compiles only the main scene.
- The producer exposes `image()`, `try_ready()` and `ready().await`. A failed preparation returns the error. A retired instance releases its GPU resources when the last in-flight plan releases them. See the [architecture reference](../../../docs/engine-architecture.md#rendering-srcengineshared_systemsrender) for the caller example and the [context](render-resources-context.md) for decisions and verification.

## Review

The author accepted the changes after the second review. The dedicated offscreen Vulkan test exercises shared pipeline requests, per-world skies, a generated minimap, local-key validation, preparation failure and teardown. See the [review record](render-resources-context.md) for verification limits.
