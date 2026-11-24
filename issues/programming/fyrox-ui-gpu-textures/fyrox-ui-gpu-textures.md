# Research: show GPU-rendered wgpu textures in Fyrox UI without CPU copies

- **Status:** Closed
- **Priority:** Normal
- **Created:** 2026-11-24
- **Closed:** 2026-11-24
- **Resolution:** Done — the author accepted the research result.
- **Context:** [Source analysis and isolated experiment](fyrox-ui-gpu-textures-context.md)

## Question

Can a [Fyrox UI](../fyrox-ui/) widget display a [dynamic render texture](../render-resources/) that Slice renders with its own wgpu device? The display path must take the texture directly from the GPU and must not read pixels back, re-upload them from the CPU or store them on disk.

## Answer

[accepted as research result] Yes, provided Slice supplies its own wgpu backend for Fyrox UI draw commands. Fyrox 1.0.1 already uses this model internally.

- `fyrox-ui` 1.0.1 does no GPU work itself. `UserInterface::update` fills the public `drawing_context` with vertices, triangles and commands. An image command carries `CommandTexture::Texture(TextureResource)`, which is only a reference. The UI reads only the texture size (`TextureKind`), for layout of `Image` and `NinePatch`.
- `TextureResource::new_render_target(w, h)` creates a placeholder with empty bytes and `is_render_target = true`. Fyrox's own renderer resolves it through a GPU-side cache. It uploads pixels only when the resource's `modifications_count` changes. GPU writes to the target do not change that counter.
- Fyrox's renderer is OpenGL only (`fyrox-graphics-gl`), and Slice owns its wgpu device. Slice therefore cannot use `fyrox-impl` and needs a wgpu backend that maps the reference to a `wgpu::TextureView`.
- An isolated probe outside the repository confirmed the path on Vulkan (RTX 3070 Laptop). It covered a GPU-produced texture in an `Image`, text, a clipped widget, content updates and target recreation. The only per-frame CPU uploads were vertices, indices and uniforms; one glyph-atlas page was uploaded once. Readback was used only to check the result.

## Constraints for the integration design

- Identify a texture by its live resource, not by UUID: every `new_render_target` gets the nil UUID. `Resource::key()` hashes the `Arc` pointer, so the registry entry must keep a clone or be removed when the texture is retired. Otherwise a new resource can reuse the address. Coordinate this with [registry retention](../resource-registry-retention/).
- Resizing: `Texture` has no public `set_kind`. For a new size, create a new `TextureResource` with a new target and send `ImageMessage::Texture`. Swapping the view under the same key also works, but the size reported to the UI stays stale (it matters only with `sync_with_texture_size`).
- Orientation: wgpu and Fyrox UI both put the origin at the top left. Do not use the `Image::flip` workaround that Fyrox needs for OpenGL render targets.
- The probe did not cover gradient brushes, clipping geometry (stencil), custom widget materials, sRGB conversion or a cross-thread handoff of draw data. Those remain within [Fyrox UI integration](../fyrox-ui/).

## Completion criteria

The author reviews the result and accepts or rejects this approach as the basis for the texture contract in [Fyrox UI integration](../fyrox-ui/) and [render resources](../render-resources/).
