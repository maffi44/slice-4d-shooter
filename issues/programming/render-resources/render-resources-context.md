# Scanner ownership and data flow

[Current task](render-resources.md)

## 2026-11-24 — First caller and boundaries

The existing scanner and minimap take different paths. The HUD scanner draws directly into the window. Its bind group combines a rectangle transform, scanner orientation and six buffers borrowed from the presented world. Separately, two obstacle-course shader branches in `world_descriptor.rs` build an XWZ minimap pipeline, sometimes with generated map geometry. `WorldGpu::encode` draws this minimap into the scene output. Moving the minimap requires preserving its map-specific inputs and appearance. The [map compatibility task](../maps-out-of-sync-with-parser/) records why an existing obstacle-course map may be unsuitable as a test fixture.

[proposed] At activation the player requests a scanner for its own rendered world, keeps the instance and passes its output to an image widget. The effect registers its WGSL, binding layout and pipeline recipe. Prepare expensive pipelines off the tick and render hot paths and share compatible compiled pipelines. Each scanner owns its changing inputs and target. Bind the world's shared scene inputs once per applicable generation. A tick updates orientation and other scanner-specific values. The render thread consumes the latest complete update for that instance when its world is due. Resource creation, rebinding and removal have ordered delivery. A new scanner input must not allocate an unbounded queue of obsolete frames. Existing world uploads have a separate [coherence limitation](../gpu-data-grouping/).

[proposed] Keep the existing resource loader for reusable file-backed inputs. Track runtime targets by live instance and generation. Consumers can retain an image reference while their binding tracks target recreation. Dropping the producer stops scheduling it even if a widget still holds that reference. Agree on placeholder/last-image behavior and stale-handle protection. Prepared spare actors must not register visible passes. Headless activation skips graphics allocation and leaves gameplay data available.

[open] The first shared texture lookup is now needed by the scanner widget. Prove it here for dynamic targets before deciding how much of it a world output and the later [4D screen](../in-world-4d-screens/) can reuse. Verify that a second effect can supply different bindings and inputs without adding a scanner-specific branch to the scheduler or UI backend.
