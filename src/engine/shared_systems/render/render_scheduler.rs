// Slice 4D Shooter - the first multiplayer shooter set in 4D space
// Copyright (C) 2023-2026  Timofei Molokov

use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicU32, AtomicU64, Ordering},
        mpsc, Arc, Weak,
    },
    thread::JoinHandle,
    time::{Duration, Instant},
};

use glam::UVec2;
use wgpu::{PipelineCompilationOptions, PollStatus, ShaderRuntimeChecks};
use winit::{dpi::PhysicalSize, event_loop::EventLoopProxy, window::Window};

use crate::engine::{main_loop::AppEvent, world::WorldRenderConfig};

use super::{
    gpu_context::GpuContext,
    passes::{blit, texture_view, WorldPlan},
    presentation::Presentation,
    ui_renderer::UIRenderer,
};

/// Accepted range of the `target_render_frame_rate` user setting; values outside are clamped.
const MIN_TARGET_FRAME_RATE: u32 = 10;
const MAX_TARGET_FRAME_RATE: u32 = 1000;

/// If a render-loop iteration took less than this, the thread sleeps a bit (as before).
const MIN_ITERATION_DURATION: Duration = Duration::from_millis(3);
const ITERATION_SLEEP: Duration = Duration::from_micros(3122);
// Simulation ticks keep staging uploads even when the window has no drawable.
const SKIPPED_FRAME_UPLOAD_INTERVAL: Duration = Duration::from_millis(100);

/// Identifies a world's output texture. Plain data (valid in headless mode too, it just never resolves).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct WorldTextureId {
    scheduler: u64,
    order: u64,
}

impl WorldTextureId {
    pub(crate) fn renders_before(self, consumer: Self) -> bool {
        self.scheduler == consumer.scheduler && self.order > consumer.order
    }
}

static NEXT_SCHEDULER: AtomicU64 = AtomicU64::new(1);

/// The part of a world's render state shared between the world (main thread) and its slot in the
/// render loop. Atomics only: the main thread stores, the render thread loads each frame.
/// `Relaxed` is enough: each value is standalone, and resolution + scale are packed into one atomic,
/// so the render thread always sees a consistent pair.
pub struct WorldRenderShared {
    /// 0 = paused, 1 = every render frame, N = every Nth render frame.
    render_rate: AtomicU32,
    /// width u16 | height u16 | scale f32 bits.
    output: AtomicU64,
}

impl WorldRenderShared {
    pub fn new(config: &WorldRenderConfig) -> Self {
        WorldRenderShared {
            render_rate: AtomicU32::new(config.render_rate),
            output: AtomicU64::new(pack_output(config.resolution, config.scale)),
        }
    }

    pub fn render_rate(&self) -> u32 {
        self.render_rate.load(Ordering::Relaxed)
    }

    #[allow(dead_code)] // nested-world API (issues/programming/engine-refactoring-world-isolation/), not used by the game yet
    pub fn set_render_rate(&self, render_rate: u32) {
        self.render_rate.store(render_rate, Ordering::Relaxed);
    }

    pub fn packed_output(&self) -> u64 {
        self.output.load(Ordering::Relaxed)
    }

    pub fn output(&self) -> (UVec2, f32) {
        unpack_output(self.packed_output())
    }

    pub fn set_output(&self, resolution: UVec2, scale: f32) {
        self.output
            .store(pack_output(resolution, scale), Ordering::Relaxed);
    }
}

pub fn pack_output(resolution: UVec2, scale: f32) -> u64 {
    let width = resolution.x.clamp(1, u16::MAX as u32) as u64;
    let height = resolution.y.clamp(1, u16::MAX as u32) as u64;

    (width << 48) | (height << 32) | (scale.to_bits() as u64)
}

pub fn unpack_output(packed: u64) -> (UVec2, f32) {
    let width = ((packed >> 48) & 0xFFFF) as u32;
    let height = ((packed >> 32) & 0xFFFF) as u32;
    let scale = f32::from_bits((packed & 0xFFFF_FFFF) as u32);

    (UVec2::new(width, height), scale)
}

/// A registered world in the render loop. The world holds the `Arc`, the slot a `Weak`:
/// when the world is dropped, `upgrade()` fails and the slot is removed (no unregister call).
struct WorldSlot {
    id: WorldTextureId,
    shared: Weak<WorldRenderShared>,
    plan: Option<Arc<WorldPlan>>,
    targets: Option<WorldTargets>,
    ui_bindings: HashMap<usize, (u64, wgpu::BindGroup)>,
    frame_counter: u32,
}

struct WorldTargets {
    packed: u64,
    upscale: bool,
    generation: u64,
    output: wgpu::TextureView,
    sampled: wgpu::BindGroup,
    scene: Option<(wgpu::TextureView, wgpu::BindGroup)>,
}

impl WorldTargets {
    fn new(gpu: &GpuContext, packed: u64, upscale: bool, generation: u64) -> Self {
        let (resolution, scale) = unpack_output(packed);
        let limit = UVec2::splat(gpu.device.limits().max_texture_dimension_2d);
        let resolution = resolution.min(limit);
        let scaled = (resolution.as_vec2() * scale.clamp(0.0, 1.0))
            .as_uvec2()
            .max(UVec2::ONE);
        let output = texture_view(
            gpu,
            if upscale { resolution } else { scaled },
            "finished world",
        );
        let sampled = gpu.create_upscale_bind_group(&output);
        let scene = upscale.then(|| {
            let view = texture_view(gpu, scaled, "scaled scene");
            let binding = gpu.create_upscale_bind_group(&view);
            (view, binding)
        });
        Self {
            packed,
            upscale,
            generation,
            output,
            sampled,
            scene,
        }
    }
}

enum RenderLoopMessage {
    RegisterWorld(WorldSlot),
    Publish(WorldTextureId, Arc<WorldPlan>),
    PresentWorld(Option<WorldTextureId>),
    ResizeSurface(PhysicalSize<u32>),
    ReplaceSurface(wgpu::Surface<'static>, PhysicalSize<u32>),
    Shutdown,
}

/// Window-thread endpoint. It sends resizes and replacements through the same FIFO,
/// including while the loader still owns the scheduler.
pub(in crate::engine) struct WindowPresentation {
    instance: wgpu::Instance,
    to_render_loop: mpsc::Sender<RenderLoopMessage>,
}

impl WindowPresentation {
    pub fn resize(&self, size: PhysicalSize<u32>) {
        let _ = self
            .to_render_loop
            .send(RenderLoopMessage::ResizeSurface(size));
    }

    pub fn replace_surface(&self, window: &Arc<Window>) -> Result<(), wgpu::CreateSurfaceError> {
        let surface = self.instance.create_surface(window.clone())?;
        let _ = self.to_render_loop.send(RenderLoopMessage::ReplaceSurface(
            surface,
            window.inner_size(),
        ));
        Ok(())
    }
}

/// The shared render scheduler (part of the Engine Shared Systems). Owns the render thread.
///
/// The render loop runs on a dedicated OS thread (not a tokio task: it's an endless timing loop).
/// It's stopped explicitly in `Drop` (a `Shutdown` message + `join()`), because dropping a
/// `std::thread::JoinHandle` only detaches the thread.
pub struct RenderScheduler {
    to_render_loop: mpsc::Sender<RenderLoopMessage>,
    render_thread: Option<JoinHandle<()>>,
    scheduler_id: u64,
    next_texture_id: u64,
    pub(crate) ui_renderer: Arc<UIRenderer>,
}

impl RenderScheduler {
    pub(in crate::engine) fn new(
        gpu: &GpuContext,
        instance: wgpu::Instance,
        presentation: Presentation,
        window: Arc<Window>,
        proxy: EventLoopProxy<AppEvent>,
        target_frame_rate: u32,
    ) -> Self {
        let clamped_frame_rate =
            target_frame_rate.clamp(MIN_TARGET_FRAME_RATE, MAX_TARGET_FRAME_RATE);

        if clamped_frame_rate != target_frame_rate {
            log::warn!(
                "render scheduler: target_render_frame_rate {} is outside {}..={}, using {}",
                target_frame_rate,
                MIN_TARGET_FRAME_RATE,
                MAX_TARGET_FRAME_RATE,
                clamped_frame_rate,
            );
        }

        let min_frame_duration = Duration::from_secs_f64(1.0 / clamped_frame_rate as f64);

        let surface_config = &presentation.config;
        let ui_renderer = Arc::new(UIRenderer::new(gpu));
        let upscale_pipeline = create_upscale_pipeline(gpu, gpu.output_format, "fs_main");
        let present_pipeline = create_upscale_pipeline(gpu, surface_config.format, "fs_present");
        let splash_pipeline = create_upscale_pipeline(gpu, surface_config.format, "fs_main");

        let (to_render_loop, inbox) = mpsc::channel();

        let _ = proxy.send_event(AppEvent::RenderStarted(WindowPresentation {
            instance,
            to_render_loop: to_render_loop.clone(),
        }));

        let render_loop = RenderLoop {
            inbox,
            gpu: gpu.clone(),
            window,
            presentation,
            proxy: proxy.clone(),
            upscale_pipeline,
            present_pipeline,
            splash_pipeline,
            ui_renderer: ui_renderer.clone(),
            splash: None,
            slots: Vec::new(),
            presented: None,
            min_frame_duration,
            next_frame_deadline: None,
            next_upload_flush: Instant::now(),
            prev_frame_instant: None,
            total_time: 0.0,
            total_frames_count: 0,
            min_time: f64::MAX,
            max_time: 0.0,
        };

        let render_thread = std::thread::Builder::new()
            .name("render".to_string())
            .spawn(move || {
                // Native wgpu may raise an uncaptured configure/driver error as a panic.
                let failure = match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    render_loop.run()
                })) {
                    Ok(Ok(())) => return,
                    Ok(Err(error)) => error,
                    Err(panic) => panic
                        .downcast_ref::<String>()
                        .map(String::as_str)
                        .or_else(|| panic.downcast_ref::<&str>().copied())
                        .unwrap_or("unknown render thread panic")
                        .to_owned(),
                };
                let _ = proxy.send_event(AppEvent::RenderFailed(format!(
                    "Rendering failed: {failure}"
                )));
            })
            .expect("ERROR: can't spawn the render thread");

        log::info!(
            "render scheduler: render thread started, target {} FPS",
            clamped_frame_rate
        );

        RenderScheduler {
            to_render_loop,
            render_thread: Some(render_thread),
            scheduler_id: NEXT_SCHEDULER.fetch_add(1, Ordering::Relaxed),
            next_texture_id: 1,
            ui_renderer,
        }
    }

    pub(crate) fn publish(&self, id: WorldTextureId, plan: Arc<WorldPlan>) {
        let _ = self
            .to_render_loop
            .send(RenderLoopMessage::Publish(id, plan));
    }

    pub(crate) fn register_world(&mut self, shared: &Arc<WorldRenderShared>) -> WorldTextureId {
        let id = WorldTextureId {
            scheduler: self.scheduler_id,
            order: self.next_texture_id,
        };
        self.next_texture_id += 1;

        let _ = self
            .to_render_loop
            .send(RenderLoopMessage::RegisterWorld(WorldSlot {
                id,
                shared: Arc::downgrade(shared),
                plan: None,
                targets: None,
                ui_bindings: HashMap::new(),
                frame_counter: 0,
            }));

        id
    }

    /// Which world is shown in the window. `None` → the splash screen.
    pub fn present_world(&self, id: Option<WorldTextureId>) {
        let _ = self
            .to_render_loop
            .send(RenderLoopMessage::PresentWorld(id));
    }
}

impl Drop for RenderScheduler {
    fn drop(&mut self) {
        let _ = self.to_render_loop.send(RenderLoopMessage::Shutdown);

        if let Some(render_thread) = self.render_thread.take() {
            if render_thread.join().is_err() {
                log::error!("render scheduler: the render thread panicked");
            }
        }

        log::info!("render scheduler: render thread stopped");
    }
}

/// Owned exclusively by the render thread (no locks).
struct RenderLoop {
    inbox: mpsc::Receiver<RenderLoopMessage>,
    gpu: GpuContext,
    window: Arc<Window>,
    presentation: Presentation,
    proxy: EventLoopProxy<AppEvent>,
    upscale_pipeline: wgpu::RenderPipeline,
    present_pipeline: wgpu::RenderPipeline,
    splash_pipeline: wgpu::RenderPipeline,
    ui_renderer: Arc<UIRenderer>,
    splash: Option<wgpu::BindGroup>,
    /// In registration order. Rendered in reverse: a nested world is always registered after the
    /// world whose actor created it, so children are rendered before their parents.
    slots: Vec<WorldSlot>,
    presented: Option<WorldTextureId>,
    /// The period of the target render frame rate (a user setting), set at creation.
    min_frame_duration: Duration,
    /// The earliest start of the next frame. Advanced by `min_frame_duration` on a fixed timeline,
    /// so the average rate never exceeds the cap and lateness under one period isn't accumulated.
    next_frame_deadline: Option<Instant>,
    next_upload_flush: Instant,

    prev_frame_instant: Option<Instant>,
    total_time: f64,
    total_frames_count: u64,
    min_time: f64,
    max_time: f64,
}

impl RenderLoop {
    fn run(mut self) -> Result<(), String> {
        // The splash (shown until the first world is presented) is decoded here, so the main
        // thread isn't delayed by it.
        let splash_view = self.gpu.create_texture_from_image(
            include_bytes!("../../../../media/slice_4d_shooter_poster.png"),
            "splash texture",
        );
        self.splash = Some(self.gpu.create_upscale_bind_group(&splash_view));

        loop {
            let iteration_start = std::time::Instant::now();

            if !self.process_inbox()? {
                break;
            }

            self.render_frame()?;

            if iteration_start.elapsed() < MIN_ITERATION_DURATION {
                std::thread::sleep(ITERATION_SLEEP);
            }
        }

        if self.total_frames_count > 0 {
            log::info!(
                "render loop: avg dt {}, min dt {}, max dt {}",
                self.total_time / self.total_frames_count as f64,
                self.min_time,
                self.max_time,
            );
        }
        // GPU objects (slots, surface, UI renderer) are dropped here, on the render thread.
        Ok(())
    }

    /// Returns false when the loop must stop.
    fn process_inbox(&mut self) -> Result<bool, String> {
        loop {
            match self.inbox.try_recv() {
                Ok(RenderLoopMessage::RegisterWorld(slot)) => {
                    self.slots.push(slot);
                }
                Ok(RenderLoopMessage::Publish(id, plan)) => {
                    if let Some(slot) = self.slots.iter_mut().find(|slot| slot.id == id) {
                        slot.plan = Some(plan);
                        slot.ui_bindings.clear();
                    }
                }
                Ok(RenderLoopMessage::PresentWorld(id)) => {
                    self.presented = id;
                }
                Ok(RenderLoopMessage::ResizeSurface(size)) => {
                    self.presentation.state.resize(size);
                }
                Ok(RenderLoopMessage::ReplaceSurface(surface, size)) => {
                    self.presentation.replace(surface, size)?;
                }
                Ok(RenderLoopMessage::Shutdown) => {
                    return Ok(false);
                }
                Err(mpsc::TryRecvError::Empty) => {
                    return Ok(true);
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    return Ok(false);
                }
            }
        }
    }

    fn render_frame(&mut self) -> Result<(), String> {
        // No frame queueing: don't submit while the GPU is still busy with the previous frame.
        match self.gpu.device.poll(wgpu::wgt::PollType::Poll) {
            Ok(PollStatus::Poll) => return Ok(()),
            Ok(PollStatus::QueueEmpty) | Ok(PollStatus::WaitSucceeded) => {}
            Err(e) => return Err(format!("GPU poll failed: {e}")),
        }

        let now = Instant::now();

        if let Some(deadline) = self.next_frame_deadline {
            if now < deadline {
                return Ok(());
            }

            // More than one period late (a stall): restart the timeline instead of catching up.
            let next_deadline = deadline + self.min_frame_duration;
            self.next_frame_deadline = Some(if next_deadline <= now {
                now + self.min_frame_duration
            } else {
                next_deadline
            });
        } else {
            self.next_frame_deadline = Some(now + self.min_frame_duration);
        }

        // Remove the slots of dropped worlds even while the window is hidden.
        let count = self.slots.len();
        self.slots.retain(|slot| slot.shared.strong_count() > 0);
        if self.slots.len() != count {
            for slot in &mut self.slots {
                slot.ui_bindings.clear();
            }
        }

        let Some(output) = self.presentation.acquire(&self.gpu.device, &self.proxy)? else {
            if now >= self.next_upload_flush {
                self.gpu.queue.submit([]);
                self.next_upload_flush = now + SKIPPED_FRAME_UPLOAD_INTERVAL;
            }
            return Ok(());
        };

        if let Some(instant) = self.prev_frame_instant {
            let current_frame_time = now.duration_since(instant).as_secs_f64();

            self.total_time += current_frame_time;
            self.total_frames_count += 1;
            self.min_time = self.min_time.min(current_frame_time);
            self.max_time = self.max_time.max(current_frame_time);
        }

        self.prev_frame_instant = Some(now);

        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("render scheduler frame encoder"),
            });

        encode_worlds(
            &mut self.slots,
            &self.gpu,
            &self.ui_renderer,
            &self.upscale_pipeline,
            &mut encoder,
        );

        let presented = self
            .presented
            .and_then(|id| self.slots.iter().find(|slot| slot.id == id))
            .and_then(|slot| slot.targets.as_ref());
        if let Some(targets) = presented {
            blit(
                &self.gpu,
                &mut encoder,
                &self.present_pipeline,
                &targets.sampled,
                &view,
            );
        } else if let Some(splash) = &self.splash {
            blit(
                &self.gpu,
                &mut encoder,
                &self.splash_pipeline,
                splash,
                &view,
            );
        }

        self.gpu.queue.submit(std::iter::once(encoder.finish()));

        self.window.pre_present_notify();

        self.gpu.queue.present(output);
        self.presentation.state.presented();
        Ok(())
    }
}

/// Encodes due plans in dependency order into the caller's single frame submission.
fn encode_worlds(
    slots: &mut [WorldSlot],
    gpu: &GpuContext,
    ui: &UIRenderer,
    upscale: &wgpu::RenderPipeline,
    encoder: &mut wgpu::CommandEncoder,
) {
    for index in (0..slots.len()).rev() {
        let (before, children) = slots.split_at_mut(index + 1);
        let slot = &mut before[index];
        let (Some(shared), Some(plan)) = (slot.shared.upgrade(), slot.plan.as_ref()) else {
            continue;
        };
        let rate = shared.render_rate();
        if rate == 0 {
            continue;
        }
        slot.frame_counter += 1;
        if slot.frame_counter < rate {
            continue;
        }
        slot.frame_counter = 0;
        let packed = shared.packed_output();
        if slot
            .targets
            .as_ref()
            .is_none_or(|t| t.packed != packed || t.upscale != plan.upscale)
        {
            let generation = slot.targets.as_ref().map_or(1, |t| t.generation + 1);
            slot.targets = Some(WorldTargets::new(gpu, packed, plan.upscale, generation));
        }
        let targets = slot.targets.as_ref().unwrap();
        for texture in &plan.textures {
            texture.draw.encode(gpu, encoder, &texture.view);
        }
        if let Some((scene, binding)) = &targets.scene {
            plan.scene.encode(gpu, encoder, scene);
            blit(gpu, encoder, upscale, binding, &targets.output);
        } else {
            plan.scene.encode(gpu, encoder, &targets.output);
        }
        ui.encode(
            &gpu.device,
            encoder,
            &targets.output,
            &plan.ui,
            &mut slot.ui_bindings,
            |id| {
                let targets = children
                    .iter()
                    .find(|child| child.id == id)?
                    .targets
                    .as_ref()?;
                Some((targets.generation, &targets.output))
            },
        );
    }
}

fn create_upscale_pipeline(
    gpu: &GpuContext,
    surface_format: wgpu::TextureFormat,
    fragment: &str,
) -> wgpu::RenderPipeline {
    let upscale_shader = unsafe {
        gpu.device.create_shader_module_trusted(
            wgpu::ShaderModuleDescriptor {
                label: Some("Upscale Shader"),
                source: wgpu::ShaderSource::Wgsl(
                    include_str!("shaders/upscale_shader.wgsl").into(),
                ),
            },
            ShaderRuntimeChecks::unchecked(),
        )
    };

    let upscale_render_pipeline_layout =
        gpu.device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("upscale render pipeline layout"),
                bind_group_layouts: &[Some(&gpu.upscale_bind_group_layout)],
                immediate_size: 0,
            });

    gpu.device
        .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("upscale shader render pipeline"),
            layout: Some(&upscale_render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &upscale_shader,
                entry_point: Some("vs_main"),
                buffers: &[Some(gpu.raymarch_vertex_layout.clone())],
                compilation_options: PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &upscale_shader,
                compilation_options: PipelineCompilationOptions::default(),
                entry_point: Some(fragment),
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
            multiview_mask: None,
            cache: None,
        })
}

#[cfg(test)]
mod tests;
