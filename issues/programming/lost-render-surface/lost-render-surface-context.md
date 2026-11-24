# Lost surface recovery context

[Current task](lost-render-surface.md)

## 2026-11-24 — Dependency update review

[accepted] In wgpu 30.0.2, `CurrentSurfaceTexture::Outdated` calls for reconfiguring the existing surface. `Lost` calls for creating a new surface and configuring it. The native Vulkan backend can produce `Lost` for `ERROR_SURFACE_LOST_KHR`. Configuring the lost surface can fail again. This finding follows from the API and backend source. The review environment could not inject a real window-system loss.

Today `EngineLoader` asks the winit thread to create a surface for the instance before device selection. After startup the render loop owns the surface, configuration and pipelines. The startup request channel ends with the loader. The event-loop thread requirement for Metal is documented in [nonblocking startup](../nonblocking-engine-startup/).

[proposed] With the device still usable, allow one outstanding replacement request per affected window. Recheck format and size before the first presentation. A resize or close can arrive while the reply is in flight. Preserve simulation, slot ownership and the render thread's shutdown path. Decide how to report device loss separately, since replacing the surface alone cannot restore device resources.
