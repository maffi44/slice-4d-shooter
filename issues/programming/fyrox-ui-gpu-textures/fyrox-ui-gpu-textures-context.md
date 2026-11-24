# Fyrox UI and GPU textures: evidence

[Current task](fyrox-ui-gpu-textures.md)

## 2026-11-24 — Task opened

The author asked to confirm, without changing project code, that Fyrox UI can show dynamic wgpu textures straight from the GPU, with no data copied to disk or to the CPU. The research was to use source analysis and, if needed, an isolated experiment. Both were done. Versions: `fyrox-ui`, `fyrox-texture`, `fyrox-resource`, `fyrox-impl` 1.0.1 (the version the project moved to on 2026-11-24, see [dependency cleanup](../dependency-and-build-cleanup/)); wgpu 30.0.2.

## Source analysis (fyrox 1.0.1)

### What fyrox-ui produces

- `UserInterface::update(screen_size, dt, switches)` runs layout and then calls the private `draw()`. The result stays in the public field `UserInterface::drawing_context` (`lib.rs`, `fn draw`, field `drawing_context`). This differs from 0.36.2, described in the [Fyrox UI context](../fyrox-ui/), where `draw()` returned a borrowed context. A render-thread handoff still needs owned draw data, because the context is overwritten on the next `update`.
- `DrawingContext::get_vertices / get_triangles / get_commands` expose plain data. `Command` contains `clip_bounds`, `bounds`, `brush`, `texture: CommandTexture`, a `triangles` range, `material`, `opacity`, optional `clipping_geometry` and a `transform` (`Matrix3`; vertices stay in local space and the backend applies the transform, as the built-in `widget.shader` does).
- `CommandTexture` is `None | Texture(TextureResource) | Font { font, height, page_index }` (`draw.rs`). A texture command carries only a reference counted by `Arc`.
- `Image::draw` emits `CommandTexture::Texture(texture.clone())`. `Image::measure_override` reads only `data_ref().kind()` for `sync_with_texture_size`, and `NinePatch::draw` only reads `kind()`. Of the editor widgets, only the inspector's `texture_slice` editor reads texture data, and gameplay HUDs do not need it. No runtime widget reads pixel bytes.

### How Fyrox's own renderer resolves textures

- `fyrox-impl/src/renderer/ui_renderer.rs` matches `CommandTexture::Texture` and calls `TextureCache::get`. The cache entry is keyed by the internal `Texture::cache_index` (`#[doc(hidden)] pub`) and holds the GPU texture.
- `TextureCache::get` creates the GPU texture once. It re-uploads (`set_data`) only when `Texture::modifications_count()` differs from the cached counter (`renderer/cache/texture.rs`). GPU-side writes to a render target never change the counter, so Fyrox itself displays render targets without CPU copies.
- `TextureResourceExtension::new_render_target(_with_format)` builds a `Texture` with empty `bytes`, `is_render_target: true`, and `Resource::new_ok(Default::default(), ...)`. That is the nil UUID for every render target (`fyrox-texture/src/lib.rs`).
- `fyrox-impl` depends on `fyrox-graphics-gl`. No wgpu backend exists in 1.0.1, so Slice cannot use Fyrox's renderer with its own wgpu device. The mapping above has to be reimplemented in a Slice wgpu backend.
- Font glyphs are rasterized on the CPU (fontdue) into R8 atlas pages (`font::Page::pixels`, `modified`). The backend uploads a page when it is created or changed. This is static text data, not a dynamic texture copy.

### Identity of a texture reference

- [rejected] `resource_uuid()`, because all render targets created through `new_render_target` share the nil UUID.
- [rejected] `Texture::cache_index`. It would work, but it is a hidden internal field of Fyrox's renderer, not a public contract.
- [proposed] `Resource::key()`, a hash of the `Arc` pointer (`UntypedResource: Hash`). It is stable while the resource is alive. After the last clone is dropped, the address can be reused, so the registry that maps key to view must own a clone (or remove the entry when the producer retires it). This ties into [registry retention](../resource-registry-retention/) and to the open question in [render resources](../render-resources/) about a widget outliving its producer.

## Isolated experiment

Location: the session scratchpad, outside the repository (`fyrox-gpu-texture/` with `src/main.rs`). Project files were not changed. The probe used the project's `Cargo.lock` to match versions: `fyrox-ui` 1.0.1, wgpu 30.0.2 Vulkan. It ran on an NVIDIA GeForce RTX 3070 Laptop GPU, Linux, on 2026-11-24.

Setup:

1. A wgpu render pass (fullscreen triangle) draws a 64x32 RGBA8 target: the top half is the "generation color" and the bottom half is blue. No CPU pixel data exists.
2. `TextureResource::new_render_target(64, 32)` serves as its Fyrox identity. A `HashMap<u64, wgpu::TextureView>` registry maps `resource.key()` to the view.
3. UI: a 100x40 `Canvas` at (16,16) containing a 128x64 `Image` with that texture, so the image is clipped by its parent. Below it is a `Text` "SLICE 4D", 24 px, built-in font.
4. A minimal backend turns `drawing_context` into a vertex buffer and an index buffer. Each command gets a uniform (transform, solid brush, opacity, font flag), a bind group with the resolved view (registry, glyph page or white 1x1) and a scissor from `clip_bounds`. A WGSL shader ports the relevant part of `widget.shader`.
5. The final output was read back only for the checks.

Results (all passed):

| Frame | Action | Check | Result |
|---|---|---|---|
| A | generation 1 (red) | image top red, bottom blue (orientation without flip), pixels outside the 100x40 parent clipped, text drawn (486 lit pixels) | OK |
| B | GPU producer redraws in green; Fyrox resource untouched | image shows green; resource still has 0 bytes, `modifications_count` 0 | OK |
| C | new 32x16 target behind a new `TextureResource`, `ImageMessage::Texture` | image shows magenta | OK |
| D | 128x64 target swapped under the same key | image shows yellow | OK |

Backend counters: 4 registry hits, 0 misses, 1 glyph-page upload. Neither the probe nor fyrox-ui read or wrote dynamic texture pixels on the CPU.

Not covered by the probe (deliberate simplifications; the work belongs to the [integration task](../fyrox-ui/)):

- Gradient brushes (linear and radial), which need the full `widget.shader` port.
- `clipping_geometry` (non-rectangular clipping; Fyrox uses stencil).
- Custom `MaterialResource` per command; the probe assumed the standard widget material.
- sRGB handling. Fyrox renders UI in sRGB; the probe used `Rgba8Unorm` and exact colors.
- Performance. The probe allocates a buffer and a bind group per command per frame. A real backend should reuse buffers and cache bind groups by view and generation.
- Handing draw data from the UI owner to the render thread, and texture lifetime across that handoff.

## Conclusions for the design

- [proposed] Slice's wgpu UI backend owns a registry from texture identity to a live view and its generation. A producer (scanner, world output) registers its target and receives a `TextureResource` to give to widgets. Recreation creates a new resource and sends `ImageMessage::Texture`, so layout sees the correct size and stale keys disappear with the old resource.
- [proposed] Do not set `Image::flip` for wgpu targets. The Fyrox documentation's flip advice applies to OpenGL render targets.
- [open] Placeholder behavior when a widget references a retired texture. The probe falls back to white; [render resources](../render-resources/) leaves this choice open.
