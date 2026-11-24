# Recover a lost presentation surface

- **Status:** Done — accepted by the author
- **Priority:** High
- **Created:** 2026-11-24
- **Closed:** 2026-11-24
- **Resolution:** Done
- **Context:** [wgpu contract and thread ownership](lost-render-surface-context.md)

With a live device, a lost surface can leave the game running without a picture. Recover the current window during the splash and during play. The window thread owns native surface creation and the render thread owns its configuration, drawing and presentation.

[accepted] Allow one replacement on the same window, device and presentation format per loss episode. A frame presented on the replacement resets the retry budget. The winit proxy carries the request during loading and play. The window thread returns the replacement through the render inbox. Close cancels further window work and the render thread can process shutdown while waiting. An incompatible replacement or device loss exits the application with a diagnostic.

The author accepted the implementation after review. Surface-state tests passed in all three client binaries. The sandbox could not open a display for the end-to-end check of a visible frame after loss. The context records the remaining native-window scenarios and how to exercise them.
