// Slice 4D Shooter - the first multiplayer shooter set in 4D space
// Copyright (C) 2023-2026  Timofei Molokov

use std::{
    panic::{catch_unwind, AssertUnwindSafe},
    sync::{atomic::{AtomicBool, Ordering}, mpsc, Arc, OnceLock},
    thread::{self, JoinHandle},
};

use winit::{dpi::PhysicalSize, event_loop::EventLoopProxy, window::Window};

use super::{
    shared_systems::{EngineSharedSystems, global_settings::GlobalSettings, render::{GpuContext, RenderScheduler}},
    world::world_descriptor::{WorldDescriptor, WorldPreparation},
};

/// Starts async map/settings loading and GPU driver discovery in parallel. No window or Engine yet.
/// MainLoop supplies the window surface and consumes the completed load.
pub fn load(initial_world: String, graphic_backend: Option<wgpu::Backend>, headless: bool) -> EngineLoader
{
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()
        .expect("ERROR: can't build the tokio async runtime");

    log::info!("engine loader: loading '{}'", initial_world);
    let settings = runtime.spawn(GlobalSettings::load());
    let instance = (!headless).then(|| runtime.spawn_blocking(move || GpuContext::create_instance(graphic_backend)));

    let (sender, receiver) = mpsc::channel();
    let wake = Arc::new(OnceLock::new());
    let cancelled = Arc::new(AtomicBool::new(false));
    let worker_wake = wake.clone();
    let worker_cancelled = cancelled.clone();

    // An OS thread coordinates blocking initialization and owns the runtime until it is ready.
    // In particular, runtime destruction must not happen inside a Tokio async task.
    let worker = thread::Builder::new().name("engine-loader".into()).spawn(move || {
        let result = catch_unwind(AssertUnwindSafe(|| -> Result<Option<LoadedEngine>, String> {
            let resources = Arc::new(super::shared_systems::resources::ResourceSystem::new(headless));
            let world_resources = resources.clone();
            let initial_key = initial_world.clone();
            let descriptor = runtime.spawn(async move { WorldPreparation::load(&initial_world, world_resources).await });
            let settings = runtime.block_on(settings).map_err(|error| error.to_string())?;
            let mut shared = EngineSharedSystems::new(headless, runtime, settings, resources);

            if let Some(instance) = instance
            {
                let instance = shared.runtime.block_on(instance).map_err(|error| error.to_string())?;
                if worker_cancelled.load(Ordering::Relaxed) { return Ok(None); }

                let (reply, surface) = mpsc::sync_channel(1);
                if !publish(&sender, &worker_wake, LoadEvent::CreateSurface { instance: instance.clone(), reply })
                {
                    return Ok(None);
                }
                let Ok(WindowSurface { window, surface, size }) = surface.recv() else {
                    return Ok(None);
                };
                if worker_cancelled.load(Ordering::Relaxed) { return Ok(None); }

                let (gpu, surface, config) = pollster::block_on(GpuContext::new(instance, surface, size));
                if worker_cancelled.load(Ordering::Relaxed) { return Ok(None); }

                // UI textures/pipelines and the upscale pipeline are expensive too.
                shared.render = Some(RenderScheduler::new(&gpu, surface, config, window, &mut shared.ui));
                shared.gpu = Some(gpu);
            }

            let preparation = shared.runtime.block_on(descriptor).map_err(|error| error.to_string())??;
            if worker_cancelled.load(Ordering::Relaxed) { return Ok(None); }
            let descriptor = Arc::new(preparation.finish(shared.gpu.clone()));
            shared.resources.register_world_loader(shared.gpu.clone(), shared.runtime.handle().clone());
            shared.resources.register_initial_world(&initial_key, descriptor.clone());
            if worker_cancelled.load(Ordering::Relaxed) { return Ok(None); }

            Ok(Some(LoadedEngine { shared, descriptor }))
        })).unwrap_or_else(|panic| {
            let message = panic.downcast_ref::<String>().map(String::as_str)
                .or_else(|| panic.downcast_ref::<&str>().copied())
                .unwrap_or("unknown initialization panic");
            Err(message.to_owned())
        });
        publish(&sender, &worker_wake, LoadEvent::Finished(result));
    }).expect("ERROR: can't spawn the engine loading thread");

    EngineLoader { headless, receiver: Some(receiver), worker: Some(worker), wake, cancelled }
}

/// Owns only the lifetime of initialization, including cancellation and intermediate handoffs.
/// Dropped after Engine::new consumes LoadedEngine; no task handles remain in Engine.
pub struct EngineLoader
{
    pub(super) headless: bool,
    pub(super) receiver: Option<mpsc::Receiver<LoadEvent>>,
    worker: Option<JoinHandle<()>>,
    wake: Arc<OnceLock<EventLoopProxy<()>>>,
    cancelled: Arc<AtomicBool>,
}

impl EngineLoader
{
    pub(super) fn set_waker(&self, proxy: EventLoopProxy<()>)
    {
        self.wake.set(proxy).expect("the loader already has an event loop");
        // Events may have arrived before the proxy was installed.
        let _ = self.wake.get().unwrap().send_event(());
    }

    pub(super) fn cancel(&self)
    {
        self.cancelled.store(true, Ordering::Relaxed);
    }
}

impl Drop for EngineLoader
{
    fn drop(&mut self)
    {
        self.cancel();
        // Drop queued surface replies before joining: the worker may be waiting for one.
        // Disconnecting also makes any later publish fail instead of leaving it waiting forever.
        self.receiver.take();
        if let Some(worker) = self.worker.take()
        {
            let _ = worker.join();
        }
    }
}

/// Only the loader can construct this value: all required loading work has succeeded.
pub struct LoadedEngine
{
    pub(super) shared: EngineSharedSystems,
    pub(super) descriptor: Arc<WorldDescriptor>,
}

pub(super) struct WindowSurface
{
    pub window: Arc<Window>,
    pub surface: wgpu::Surface<'static>,
    pub size: PhysicalSize<u32>,
}

pub(super) enum LoadEvent
{
    CreateSurface { instance: wgpu::Instance, reply: mpsc::SyncSender<WindowSurface> },
    Finished(Result<Option<LoadedEngine>, String>),
}

fn publish(sender: &mpsc::Sender<LoadEvent>, wake: &OnceLock<EventLoopProxy<()>>, event: LoadEvent) -> bool
{
    if sender.send(event).is_err() { return false; }
    // Publish before waking. A wake is not evidence that a thread's JoinHandle has finished.
    if let Some(proxy) = wake.get() { let _ = proxy.send_event(()); }
    true
}
