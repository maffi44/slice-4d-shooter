// Slice 4D Shooter - the first multiplayer shooter set in 4D space
// Copyright (C) 2023-2026  Timofei Molokov

use winit::{dpi::PhysicalSize, event_loop::EventLoopProxy};
use crate::engine::main_loop::AppEvent;

/// Device and pipelines stay alive across a window surface replacement.
pub(in crate::engine) struct Presentation
{
    pub(super) state: SurfaceState<wgpu::Surface<'static>>,
    pub(super) config: wgpu::SurfaceConfiguration,
    adapter: wgpu::Adapter,
}

impl Presentation
{
    pub(super) fn new(
        surface: wgpu::Surface<'static>, config: wgpu::SurfaceConfiguration,
        adapter: wgpu::Adapter, size: PhysicalSize<u32>,
    ) -> Self
    {
        Self { state: SurfaceState::new(surface, size), config, adapter }
    }

    pub(super) fn replace(&mut self, surface: wgpu::Surface<'static>, size: PhysicalSize<u32>) -> Result<(), String>
    {
        let caps = surface.get_capabilities(&self.adapter);
        if !caps.formats.contains(&self.config.format) {
            return Err(format!("Replacement surface cannot present {:?} on the current adapter (available: {:?})",
                self.config.format, caps.formats));
        }
        self.state.replace(surface, size);
        Ok(())
    }

    pub(super) fn acquire(
        &mut self, device: &wgpu::Device, proxy: &EventLoopProxy<AppEvent>,
    ) -> Result<Option<wgpu::SurfaceTexture>, String>
    {
        use wgpu::CurrentSurfaceTexture;

        let Some((surface, configure)) = self.state.drawable() else { return Ok(None); };
        if let Some(size) = configure {
            self.config.width = size.width;
            self.config.height = size.height;
            surface.configure(device, &self.config);
        }
        match surface.get_current_texture()
        {
            CurrentSurfaceTexture::Success(texture) |
            CurrentSurfaceTexture::Suboptimal(texture) => Ok(Some(texture)),
            CurrentSurfaceTexture::Occluded | CurrentSurfaceTexture::Timeout => Ok(None),
            CurrentSurfaceTexture::Outdated => {
                self.state.outdated();
                Ok(None)
            }
            CurrentSurfaceTexture::Lost => {
                self.state.lost()?;
                log::warn!("render scheduler: window surface lost, requesting a replacement");
                let _ = proxy.send_event(AppEvent::ReplaceSurface);
                Ok(None)
            }
            CurrentSurfaceTexture::Validation if self.state.recovering => {
                Err("Replacement surface failed validation".into())
            }
            CurrentSurfaceTexture::Validation => {
                log::error!("Surface texture acquisition failed validation");
                Ok(None)
            }
        }
    }
}


/// The render thread owns the surface. A replacement gets one chance to resume presentation.
/// Window messages are drained in sending order before configuring the latest drawable size.
pub(super) struct SurfaceState<T>
{
    surface: Option<T>,
    size: PhysicalSize<u32>,
    needs_configure: bool,
    recovering: bool,
}

impl<T> SurfaceState<T>
{
    fn new(surface: T, size: PhysicalSize<u32>) -> Self
    {
        Self { surface: Some(surface), size, needs_configure: true, recovering: false }
    }

    pub fn resize(&mut self, size: PhysicalSize<u32>)
    {
        if self.size != size {
            self.size = size;
            self.needs_configure = true;
        }
    }

    fn outdated(&mut self)
    {
        self.needs_configure = true;
    }

    fn lost(&mut self) -> Result<(), String>
    {
        self.surface = None;
        if self.recovering {
            return Err("Replacement surface was lost before its first frame was presented".into());
        }
        self.recovering = true;
        Ok(())
    }

    fn replace(&mut self, surface: T, size: PhysicalSize<u32>)
    {
        self.surface = Some(surface);
        self.size = size;
        self.needs_configure = true;
    }

    /// A zero-sized window and an outstanding request both use the normal upload-only path.
    fn drawable(&mut self) -> Option<(&T, Option<PhysicalSize<u32>>)>
    {
        if self.size.width == 0 || self.size.height == 0 { return None; }
        let surface = self.surface.as_ref()?;
        let configure = std::mem::take(&mut self.needs_configure).then_some(self.size);
        Some((surface, configure))
    }

    pub fn presented(&mut self)
    {
        if self.recovering {
            self.recovering = false;
            log::info!("render scheduler: presented the first frame on the replacement surface");
        }
    }
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn loss_during_resize_uses_latest_size_and_preserves_retry_until_present()
    {
        let size = PhysicalSize::new(800, 600);
        let restored = PhysicalSize::new(1024, 768);
        let mut presentation = SurfaceState::new(1, size);
        presentation.lost().unwrap();
        presentation.resize(restored);
        assert_eq!(presentation.drawable(), None);
        presentation.replace(2, restored);
        presentation.resize(PhysicalSize::new(0, 0));
        assert_eq!(presentation.drawable(), None);
        presentation.resize(size);
        assert_eq!(presentation.drawable(), Some((&2, Some(size))));
        assert_eq!(presentation.drawable(), Some((&2, None)));
        presentation.outdated();
        assert_eq!(presentation.drawable(), Some((&2, Some(size))));
        assert!(presentation.lost().is_err());
    }

    #[test]
    fn minimize_and_close_while_waiting_release_the_old_surface()
    {
        let surface = std::sync::Arc::new(());
        let weak_surface = std::sync::Arc::downgrade(&surface);
        let mut presentation = SurfaceState::new(surface, PhysicalSize::new(0, 0));
        assert!(presentation.drawable().is_none());
        let restored = PhysicalSize::new(800, 600);
        presentation.resize(restored);
        assert_eq!(presentation.drawable().unwrap().1, Some(restored));
        presentation.lost().unwrap();
        assert!(weak_surface.upgrade().is_none());
        presentation.resize(PhysicalSize::new(0, 0));
        assert!(presentation.drawable().is_none());
        drop(presentation);
    }

    #[test]
    fn later_loss_after_presentation_can_recover_again()
    {
        let size = PhysicalSize::new(800, 600);
        let mut presentation = SurfaceState::new(1, size);
        for surface in [2, 3] {
            presentation.lost().unwrap();
            presentation.replace(surface, size);
            assert_eq!(presentation.drawable(), Some((&surface, Some(size))));
            presentation.presented();
        }
    }
}
