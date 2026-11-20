// Slice 4D Shooter - the first multiplayer shooter set in 4D space
// Copyright (C) 2023-2026  Timofei Molokov



use std::{
    sync::{
        atomic::{AtomicU32, AtomicU64, Ordering},
        mpsc,
        Arc,
        Weak,
    },
    thread::JoinHandle,
    time::Duration,
};

use glam::UVec2;
use web_time::Instant;
use wgpu::{Color, PipelineCompilationOptions, PollStatus, ShaderRuntimeChecks};
use winit::{dpi::PhysicalSize, window::Window};

use crate::engine::{
    shared_systems::ui::UISystem,
    world::WorldRenderConfig,
};

use super::{
    gpu_context::GpuContext,
    ui_renderer::UIRenderer,
    world_render_system::WorldGpu,
};


/// If a render-loop iteration took less than this, the thread sleeps a bit (as before).
const MIN_ITERATION_DURATION: Duration = Duration::from_millis(3);
const ITERATION_SLEEP: Duration = Duration::from_micros(3122);


/// Identifies a world's output texture. Plain data (valid in headless mode too, it just never resolves).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct WorldTextureId(u64);


/// The part of a world's render state shared between the world (main thread) and its slot in the
/// render loop. Atomics only: the main thread stores, the render thread loads each frame.
/// `Relaxed` is enough: each value is standalone, and resolution + scale are packed into one atomic,
/// so the render thread always sees a consistent pair.
pub struct WorldRenderShared
{
    /// 0 = paused, 1 = every render frame, N = every Nth render frame.
    render_rate: AtomicU32,
    /// width u16 | height u16 | scale f32 bits.
    output: AtomicU64,
}


impl WorldRenderShared
{
    pub fn new(config: &WorldRenderConfig) -> Self
    {
        WorldRenderShared {
            render_rate: AtomicU32::new(config.render_rate),
            output: AtomicU64::new(pack_output(config.resolution, config.scale)),
        }
    }

    pub fn render_rate(&self) -> u32
    {
        self.render_rate.load(Ordering::Relaxed)
    }

    #[allow(dead_code)] // nested-world API (issues/programming/engine-refactoring-world-isolation/), not used by the game yet
    pub fn set_render_rate(&self, render_rate: u32)
    {
        self.render_rate.store(render_rate, Ordering::Relaxed);
    }

    pub fn packed_output(&self) -> u64
    {
        self.output.load(Ordering::Relaxed)
    }

    pub fn output(&self) -> (UVec2, f32)
    {
        unpack_output(self.packed_output())
    }

    pub fn set_output(&self, resolution: UVec2, scale: f32)
    {
        self.output.store(pack_output(resolution, scale), Ordering::Relaxed);
    }
}


pub fn pack_output(resolution: UVec2, scale: f32) -> u64
{
    let width = resolution.x.clamp(1, u16::MAX as u32) as u64;
    let height = resolution.y.clamp(1, u16::MAX as u32) as u64;

    (width << 48) | (height << 32) | (scale.to_bits() as u64)
}


pub fn unpack_output(packed: u64) -> (UVec2, f32)
{
    let width = ((packed >> 48) & 0xFFFF) as u32;
    let height = ((packed >> 32) & 0xFFFF) as u32;
    let scale = f32::from_bits((packed & 0xFFFF_FFFF) as u32);

    (UVec2::new(width, height), scale)
}


/// A registered world in the render loop. The world holds the `Arc`, the slot a `Weak`:
/// when the world is dropped, `upgrade()` fails and the slot is removed (no unregister call).
struct WorldSlot
{
    id: WorldTextureId,
    shared: Weak<WorldRenderShared>,
    gpu: WorldGpu,
    frame_counter: u32,
}


enum RenderLoopMessage
{
    RegisterWorld(WorldSlot),
    PresentWorld(Option<WorldTextureId>),
    ResizeSurface(PhysicalSize<u32>),
    Shutdown,
}


/// The shared render scheduler (part of the Engine Shared Systems). Owns the render thread.
///
/// The render loop runs on a dedicated OS thread (not a tokio task: it's an endless timing loop).
/// It's stopped explicitly in `Drop` (a `Shutdown` message + `join()`), because dropping a
/// `std::thread::JoinHandle` only detaches the thread.
pub struct RenderScheduler
{
    to_render_loop: mpsc::Sender<RenderLoopMessage>,
    render_thread: Option<JoinHandle<()>>,
    next_texture_id: u64,
    surface_size: PhysicalSize<u32>,
}


impl RenderScheduler
{
    pub fn new(
        gpu: &GpuContext,
        surface: wgpu::Surface<'static>,
        surface_config: wgpu::SurfaceConfiguration,
        window: Arc<Window>,
        ui: &mut UISystem,
        min_frame_duration: Duration,
    ) -> Self
    {
        let surface_size = PhysicalSize::new(surface_config.width, surface_config.height);

        // UIRenderer initializes the UI elements' GPU buffers, so it's created here (it needs
        // &mut UISystem) and then moved to the render thread.
        let ui_renderer = UIRenderer::new(
            ui,
            &gpu.device,
            &surface_config,
            &gpu.queue,
            surface_size.width as f32 / surface_size.height as f32,
        );

        let upscale_pipeline = create_upscale_pipeline(gpu, surface_config.format);

        let (to_render_loop, inbox) = mpsc::channel();

        let render_loop = RenderLoop {
            inbox,
            gpu: gpu.clone(),
            window,
            surface,
            surface_config,
            upscale_pipeline,
            ui_renderer,
            splash: None,
            slots: Vec::new(),
            presented: None,
            ui_bound_world: None,
            min_frame_duration,
            next_frame_deadline: None,
            prev_frame_instant: None,
            total_time: 0.0,
            total_frames_count: 0,
            min_time: f64::MAX,
            max_time: 0.0,
        };

        let render_thread = std::thread::Builder::new()
            .name("render".to_string())
            .spawn(move || render_loop.run())
            .expect("ERROR: can't spawn the render thread");

        log::info!("render scheduler: render thread started");

        RenderScheduler {
            to_render_loop,
            render_thread: Some(render_thread),
            next_texture_id: 1,
            surface_size,
        }
    }

    pub fn surface_aspect(&self) -> f32
    {
        self.surface_size.width.max(1) as f32 / self.surface_size.height.max(1) as f32
    }

    /// Moves a world's GPU objects into a new render slot.
    pub fn register_world(&mut self, gpu: WorldGpu, shared: &Arc<WorldRenderShared>) -> WorldTextureId
    {
        let id = WorldTextureId(self.next_texture_id);
        self.next_texture_id += 1;

        let _ = self.to_render_loop.send(RenderLoopMessage::RegisterWorld(WorldSlot {
            id,
            shared: Arc::downgrade(shared),
            gpu,
            frame_counter: 0,
        }));

        id
    }

    /// Which world is shown in the window. `None` → the splash screen.
    pub fn present_world(&self, id: Option<WorldTextureId>)
    {
        let _ = self.to_render_loop.send(RenderLoopMessage::PresentWorld(id));
    }

    pub fn resize_surface(&mut self, size: PhysicalSize<u32>)
    {
        if size.width == 0 || size.height == 0
        {
            return;
        }

        self.surface_size = size;

        let _ = self.to_render_loop.send(RenderLoopMessage::ResizeSurface(size));
    }
}


impl Drop for RenderScheduler
{
    fn drop(&mut self)
    {
        let _ = self.to_render_loop.send(RenderLoopMessage::Shutdown);

        if let Some(render_thread) = self.render_thread.take()
        {
            if render_thread.join().is_err()
            {
                log::error!("render scheduler: the render thread panicked");
            }
        }

        log::info!("render scheduler: render thread stopped");
    }
}


/// Owned exclusively by the render thread (no locks).
struct RenderLoop
{
    inbox: mpsc::Receiver<RenderLoopMessage>,
    gpu: GpuContext,
    window: Arc<Window>,
    surface: wgpu::Surface<'static>,
    surface_config: wgpu::SurfaceConfiguration,
    upscale_pipeline: wgpu::RenderPipeline,
    ui_renderer: UIRenderer,
    splash: Option<wgpu::BindGroup>,
    /// In registration order. Rendered in reverse: a nested world is always registered after the
    /// world whose actor created it, so children are rendered before their parents.
    slots: Vec<WorldSlot>,
    presented: Option<WorldTextureId>,
    /// The world whose buffers the UI renderer (HUD scanner) is bound to, and the texture generation.
    ui_bound_world: Option<WorldTextureId>,

    /// Render data changes once per simulation tick and frames aren't interpolated, so the render
    /// rate is capped at the simulation rate: this is the tick period, set at creation.
    min_frame_duration: Duration,
    /// The earliest start of the next frame. Advanced by `min_frame_duration` on a fixed timeline,
    /// so the average rate never exceeds the cap and lateness under one period isn't accumulated.
    next_frame_deadline: Option<Instant>,

    prev_frame_instant: Option<Instant>,
    total_time: f64,
    total_frames_count: u64,
    min_time: f64,
    max_time: f64,
}


impl RenderLoop
{
    fn run(mut self)
    {
        // The splash (shown until the first world is presented) is decoded here, so the main
        // thread isn't delayed by it.
        let splash_view = self.gpu.create_texture_from_image(
            include_bytes!("../../../../media/slice_4d_shooter_poster.png"),
            "splash texture",
        );
        self.splash = Some(self.gpu.create_upscale_bind_group(&splash_view));

        loop
        {
            let iteration_start = std::time::Instant::now();

            if !self.process_inbox()
            {
                break;
            }

            if let Err(err) = self.render_frame()
            {
                match err
                {
                    wgpu::SurfaceError::Lost |
                    wgpu::SurfaceError::Outdated |
                    wgpu::SurfaceError::Other =>
                    {
                        self.reconfigure_surface(self.window.inner_size());
                    }

                    // The system is out of memory, we should probably quit
                    wgpu::SurfaceError::OutOfMemory => panic!("Out of GPU memory"),

                    _ => log::error!("{:?}", err),
                }
            }

            if iteration_start.elapsed() < MIN_ITERATION_DURATION
            {
                std::thread::sleep(ITERATION_SLEEP);
            }
        }

        if self.total_frames_count > 0
        {
            log::info!(
                "render loop: avg dt {}, min dt {}, max dt {}",
                self.total_time / self.total_frames_count as f64,
                self.min_time,
                self.max_time,
            );
        }
        // GPU objects (slots, surface, UI renderer) are dropped here, on the render thread.
    }


    /// Returns false when the loop must stop.
    fn process_inbox(&mut self) -> bool
    {
        loop
        {
            match self.inbox.try_recv()
            {
                Ok(RenderLoopMessage::RegisterWorld(slot)) =>
                {
                    self.slots.push(slot);
                }
                Ok(RenderLoopMessage::PresentWorld(id)) =>
                {
                    self.presented = id;
                }
                Ok(RenderLoopMessage::ResizeSurface(size)) =>
                {
                    self.reconfigure_surface(size);
                }
                Ok(RenderLoopMessage::Shutdown) =>
                {
                    return false;
                }
                Err(mpsc::TryRecvError::Empty) =>
                {
                    return true;
                }
                Err(mpsc::TryRecvError::Disconnected) =>
                {
                    return false;
                }
            }
        }
    }


    fn reconfigure_surface(&mut self, size: PhysicalSize<u32>)
    {
        if size.width > 0 && size.height > 0
        {
            self.surface_config.width = size.width;
            self.surface_config.height = size.height;
            self.surface.configure(&self.gpu.device, &self.surface_config);
        }
    }


    fn render_frame(&mut self) -> Result<(), wgpu::SurfaceError>
    {
        // No frame queueing: don't submit while the GPU is still busy with the previous frame.
        match self.gpu.device.poll(wgpu::wgt::PollType::Poll)
        {
            Ok(PollStatus::Poll) => return Ok(()),
            Ok(PollStatus::QueueEmpty) | Ok(PollStatus::WaitSucceeded) => {}
            Err(e) => panic!("{}", e),
        }

        let now = Instant::now();

        if let Some(deadline) = self.next_frame_deadline
        {
            if now < deadline
            {
                return Ok(());
            }

            // More than one period late (a stall): restart the timeline instead of catching up.
            let next_deadline = deadline + self.min_frame_duration;
            self.next_frame_deadline = Some(if next_deadline <= now { now + self.min_frame_duration } else { next_deadline });
        }
        else
        {
            self.next_frame_deadline = Some(now + self.min_frame_duration);
        }

        if let Some(instant) = self.prev_frame_instant
        {
            let current_frame_time = now.duration_since(instant).as_secs_f64();

            self.total_time += current_frame_time;
            self.total_frames_count += 1;
            self.min_time = self.min_time.min(current_frame_time);
            self.max_time = self.max_time.max(current_frame_time);
        }

        self.prev_frame_instant = Some(now);

        // remove the slots of dropped worlds
        self.slots.retain(|slot| slot.shared.strong_count() > 0);

        let output = self.surface.get_current_texture()?;

        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self.gpu.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("render scheduler frame encoder"),
        });

        // worlds, children before parents
        for slot in self.slots.iter_mut().rev()
        {
            let Some(shared) = slot.shared.upgrade() else {
                continue;
            };

            slot.gpu.update_output_texture(&self.gpu, shared.packed_output());

            let render_rate = shared.render_rate();

            if render_rate == 0
            {
                continue;
            }

            slot.frame_counter += 1;

            if slot.frame_counter >= render_rate
            {
                slot.frame_counter = 0;

                slot.gpu.encode(&mut encoder, &self.gpu);
            }
        }

        // the presented world (or the splash) → window
        let presented_slot = self.presented
            .and_then(|id| self.slots.iter().find(|slot| slot.id == id));

        let upscale_bind_group = match presented_slot
        {
            Some(slot) => Some(&slot.gpu.output.upscale_bind_group),
            None => self.splash.as_ref(),
        };

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("upscale shader render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(
                            Color {
                                r: 0.0,
                                g: 0.0,
                                b: 0.0,
                                a: 1.0
                            }
                        ),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            if let Some(upscale_bind_group) = upscale_bind_group
            {
                render_pass.set_pipeline(&self.upscale_pipeline);
                render_pass.set_bind_group(0, upscale_bind_group, &[]);
                render_pass.set_vertex_buffer(0, self.gpu.fullscreen_vertex_buffer.slice(..));
                render_pass.set_index_buffer(self.gpu.fullscreen_index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                render_pass.draw_indexed(0..self.gpu.fullscreen_num_indices, 0, 0..1);
            }
        }

        // the HUD scanner reads the presented world's buffers: re-bind when the presented world changes
        let presented_id = presented_slot.map(|slot| slot.id);

        if self.ui_bound_world != presented_id
        {
            self.ui_renderer.bind_world_buffers(
                &self.gpu.device,
                presented_slot.map(|slot| slot.gpu.buffers.scanner_world_buffers()),
            );

            self.ui_bound_world = presented_id;
        }

        // the UI is drawn only on top of a world (not on the splash)
        if presented_id.is_some()
        {
            self.ui_renderer.render_ui(&mut encoder, &view);
        }

        self.gpu.queue.submit(std::iter::once(encoder.finish()));

        self.window.pre_present_notify();

        output.present();

        Ok(())
    }
}


fn create_upscale_pipeline(gpu: &GpuContext, surface_format: wgpu::TextureFormat) -> wgpu::RenderPipeline
{
    let upscale_shader = unsafe {gpu.device.create_shader_module_trusted(
        wgpu::ShaderModuleDescriptor {
            label: Some("Upscale Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/upscale_shader.wgsl").into())
        },
        ShaderRuntimeChecks::unchecked()
    )};

    let upscale_render_pipeline_layout =
        gpu.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("upscale render pipeline layout"),
        bind_group_layouts: &[&gpu.upscale_bind_group_layout],
        push_constant_ranges: &[],
    });

    gpu.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("upscale shader render pipeline"),
        layout: Some(&upscale_render_pipeline_layout),
        vertex: wgpu::VertexState {
            module: &upscale_shader,
            entry_point: Some("vs_main"),
            buffers: &[
                gpu.raymarch_vertex_layout.clone(),
            ],
            compilation_options: PipelineCompilationOptions::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &upscale_shader,
            compilation_options: PipelineCompilationOptions::default(),
            entry_point: Some("fs_main"),
            targets: &[Some(wgpu::ColorTargetState {
                format: surface_format,
                blend: Some(wgpu::BlendState::REPLACE),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: None,
            polygon_mode: wgpu::PolygonMode::Fill,
            unclipped_depth: false,
            conservative: false,
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        multiview: None,
        cache: None,
    })
}
