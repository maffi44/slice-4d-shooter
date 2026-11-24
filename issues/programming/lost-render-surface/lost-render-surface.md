# Recover a lost presentation surface

- **Status:** Open — design pending
- **Priority:** High
- **Complexity:** Complex
- **Created:** 2026-11-24
- **Context:** [wgpu contract and thread ownership](lost-render-surface-context.md)

With a live device, a lost surface can leave the game running without a picture. Recover the current window during the splash and during play. The window thread owns native surface creation and the render thread owns its configuration, drawing and presentation.

[open] Agree how the render thread requests a replacement after startup, how shutdown cancels the request and how an unrecoverable failure reaches the application.

Completion requires an induced loss followed by a visible frame from the same running world. Cover resize or close while recovery is pending, repeated loss and an unavailable device. Verify that surface creation runs on the window thread and that GPU uploads remain bounded throughout recovery.
