// Slice 4D Shooter - the first multiplayer shooter set in 4D space
// Copyright (C) 2023-2026  Timofei Molokov

use super::*;
use crate::engine::{
    shared_systems::{
        resources::{
            graphics::{
                BuiltinPipelineType, CubemapTexture, LevelResource, RenderPipelineResource,
            },
            ResourceSystem, Retention, WorldResource,
        },
        EngineSharedSystems,
    },
    world::{
        render::{DynamicTextureType, ScannerPlane, WorldRenderSystem},
        ui::UIElementType,
        world_descriptor::{WorldDescriptor, WorldDescriptorGpu},
        WorldConfig,
    },
};

/// Use the real resource loaders, actor activation, plan publication and frame encoder without a window.
#[test]
#[ignore = "requires a native graphics adapter, including software Vulkan"]
fn offscreen_worlds_and_dynamic_images() {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()
        .unwrap();
    let instance = GpuContext::create_instance(None);
    let adapter = runtime
        .block_on(instance.request_adapter(&Default::default()))
        .unwrap();
    eprintln!("GPU: {}", adapter.get_info().name);
    let (device, queue) = runtime
        .block_on(adapter.request_device(&Default::default()))
        .unwrap();
    let gpu = GpuContext::from_device(device, queue);
    let (_sender, device_rx) = tokio::sync::watch::channel(Some(gpu.clone()));
    let system = ResourceSystem::new(runtime.handle().clone(), Some(device_rx));
    let mut loader = system.loader();
    let level_key = "capture_the_flag_map";
    let world = loader.load::<WorldResource>(level_key, Retention::Session);
    let ready = runtime.block_on(world).unwrap();
    let descriptor = ready.data_ref().0.as_ref().unwrap().clone();
    let mut shared = EngineSharedSystems::new(true, runtime, Default::default(), loader);
    shared.gpu = Some(gpu.clone());
    let ui = Arc::new(UIRenderer::new(&gpu));
    let (sender, inbox) = mpsc::channel();
    shared.render = Some(RenderScheduler {
        to_render_loop: sender,
        render_thread: None,
        scheduler_id: NEXT_SCHEDULER.fetch_add(1, Ordering::Relaxed),
        next_texture_id: 1,
        ui_renderer: ui.clone(),
    });
    let upscale = create_upscale_pipeline(&gpu, gpu.output_format, "fs_main");
    let config = WorldRenderConfig {
        resolution: UVec2::new(32, 24),
        scale: 0.5,
        render_rate: 1,
        with_upscale: Some(true),
    };
    let mut parent = descriptor.give_me_world(
        WorldConfig {
            render: Some(config),
        },
        &mut shared,
    );
    // A different sky uses the original BSP specialization but binds a different cube.
    let fixture = std::env::temp_dir().join(fyrox_core::uuid::Uuid::new_v4().to_string());
    std::fs::create_dir(&fixture).unwrap();
    image::RgbaImage::from_pixel(1, 1, image::Rgba([255, 128, 0, 255]))
        .save(fixture.join("face.png"))
        .unwrap();
    let path = fixture.join("test.cubemap");
    let faces = [
        "positive_x",
        "negative_x",
        "positive_y",
        "negative_y",
        "positive_z",
        "negative_z",
    ];
    let description: std::collections::BTreeMap<_, _> = faces.map(|face| (face, "face.png")).into();
    std::fs::write(&path, serde_json::to_vec(&description).unwrap()).unwrap();
    let sky = shared
        .resources
        .load::<CubemapTexture>(path.to_str().unwrap(), Retention::WhileUsed);
    let level = shared
        .resources
        .load::<LevelResource>(level_key, Retention::WhileUsed);
    let main = RenderPipelineResource::request(
        &mut shared.resources,
        BuiltinPipelineType::Slice,
        Some(level_key),
    );
    assert_eq!(main.key(), descriptor.gpu.as_ref().unwrap().main.key());
    let variant = shared
        .runtime
        .block_on(WorldDescriptor::prepare(
            descriptor.level_key.clone(),
            level,
            Some(WorldDescriptorGpu { main, sky }),
            system.clone(),
        ))
        .unwrap();
    assert!(Arc::ptr_eq(&variant.level, &descriptor.level));
    assert_ne!(
        variant.gpu.as_ref().unwrap().sky.key(),
        descriptor.gpu.as_ref().unwrap().sky.key()
    );
    std::fs::remove_dir_all(fixture).unwrap();

    let child = variant.give_me_world(
        WorldConfig {
            render: Some(WorldRenderConfig {
                render_rate: 2,
                with_upscale: Some(false),
                ..config
            }),
        },
        &mut shared,
    );
    let child_id = child.output_texture_id().unwrap();
    parent
        .ui
        .set_image(UIElementType::XwzMinimap, child_id)
        .unwrap();
    assert!(parent
        .ui
        .set_image(
            UIElementType::Crosshair,
            parent.output_texture_id().unwrap()
        )
        .is_err());

    let p1 =
        RenderPipelineResource::request(&mut shared.resources, BuiltinPipelineType::Scanner, None);
    let p2 =
        RenderPipelineResource::request(&mut shared.resources, BuiltinPipelineType::Scanner, None);
    assert_eq!(p1.key(), p2.key());
    shared.runtime.block_on(p1).unwrap();
    let mut texture_world = WorldRenderSystem::new(&descriptor, config, &mut shared).unwrap();
    let minimap = texture_world.create_dynamic_texture(
        &mut shared.resources,
        DynamicTextureType::XwzMinimap,
        UVec2::splat(8),
    );
    shared.runtime.block_on(minimap.ready()).unwrap();
    assert_eq!(minimap.try_ready(), Some(Ok(())));
    assert!(parent
        .ui
        .set_image(UIElementType::Crosshair, minimap.image())
        .is_err());
    let (device_sender, pending_device) = tokio::sync::watch::channel(None);
    let waiting_system = ResourceSystem::new(shared.runtime.handle().clone(), Some(pending_device));
    let mut pending_loader = waiting_system.loader();
    // Invalid instance inputs must finish validation even while the pipeline waits for a device.
    let bad_size = texture_world.create_dynamic_texture(
        &mut pending_loader,
        DynamicTextureType::Scanner(ScannerPlane::Zx),
        UVec2::ZERO,
    );
    let size_error = shared
        .runtime
        .block_on(async {
            tokio::time::timeout(Duration::from_secs(5), bad_size.ready())
                .await
                .unwrap()
        })
        .unwrap_err();
    assert!(size_error.contains("Invalid texture size"));
    let cancelled = texture_world.create_dynamic_texture(
        &mut pending_loader,
        DynamicTextureType::Scanner(ScannerPlane::Zw),
        UVec2::ONE,
    );
    assert!(cancelled.try_ready().is_none());
    drop(texture_world);
    drop(device_sender);
    assert!(shared.runtime.block_on(cancelled.ready()).is_err());
    let mut slots: Vec<WorldSlot> = Vec::new();
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        parent.simulate(&mut shared);
        for message in inbox.try_iter() {
            match message {
                RenderLoopMessage::RegisterWorld(slot) => slots.push(slot),
                RenderLoopMessage::Publish(id, plan) => {
                    slots.iter_mut().find(|s| s.id == id).unwrap().plan = Some(plan);
                }
                _ => unreachable!(),
            }
        }
        if slots[0]
            .plan
            .as_ref()
            .is_some_and(|plan| plan.textures.len() == 2)
        {
            break;
        }
        assert!(
            Instant::now() < deadline,
            "scanner producers were not published"
        );
        std::thread::sleep(Duration::from_millis(1));
    }
    slots.retain(|slot| slot.shared.strong_count() > 0);
    assert_eq!(slots.len(), 2);
    assert_eq!(slots[0].plan.as_ref().unwrap().textures.len(), 2);
    // Solid scene inputs make the producer's first visible pixel and stale-image removal observable.
    let present = create_upscale_pipeline(&gpu, gpu.output_format, "fs_present");
    let mut setup = gpu.device.create_command_encoder(&Default::default());
    for (slot, color) in slots
        .iter_mut()
        .zip([wgpu::Color::BLUE, wgpu::Color::GREEN])
    {
        let marker = texture_view(&gpu, UVec2::ONE, "test scene marker");
        drop(super::super::passes::begin_pass(
            &mut setup,
            &marker,
            wgpu::LoadOp::Clear(color),
            "marker color",
        ));
        let plan = Arc::get_mut(slot.plan.as_mut().unwrap()).unwrap();
        plan.scene = super::super::passes::Draw {
            pipeline: present.clone(),
            groups: vec![gpu.create_upscale_bind_group(&marker)],
        };
    }
    gpu.queue.submit([setup.finish()]);
    let frame = |slots: &mut Vec<WorldSlot>| {
        let mut encoder = gpu.device.create_command_encoder(&Default::default());
        encode_worlds(slots, &gpu, &ui, &upscale, &mut encoder);
        gpu.queue.submit([encoder.finish()]);
        gpu.device
            .poll(wgpu::PollType::wait_indefinitely())
            .unwrap();
    };
    frame(&mut slots);
    assert!(slots[1].targets.is_none());
    assert_eq!(widget_pixel(&gpu, &present, &slots[0]), [0, 0, 255, 255]);
    assert!(slots[0].ui_bindings.is_empty());
    frame(&mut slots);
    assert_eq!(
        slots[0].targets.as_ref().unwrap().output.texture().width(),
        32
    );
    assert_eq!(
        slots[1].targets.as_ref().unwrap().output.texture().width(),
        16
    );
    assert_eq!(slots[0].ui_bindings.values().next().unwrap().0, 1);
    assert_eq!(widget_pixel(&gpu, &present, &slots[0]), [0, 255, 0, 255]);
    child.set_render_rate(0);
    child.set_output_resolution(UVec2::new(48, 32));
    frame(&mut slots);
    assert_eq!(
        slots[1].targets.as_ref().unwrap().output.texture().width(),
        16
    );
    child.set_render_rate(1);
    frame(&mut slots);
    assert_eq!(
        slots[1].targets.as_ref().unwrap().output.texture().width(),
        24
    );
    assert_eq!(slots[0].ui_bindings.values().next().unwrap().0, 2);
    drop(child);
    slots.retain(|slot| slot.shared.strong_count() > 0);
    frame(&mut slots);
    assert!(slots[0].ui_bindings.is_empty());
    assert_eq!(widget_pixel(&gpu, &present, &slots[0]), [0, 0, 255, 255]);
}

fn widget_pixel(gpu: &GpuContext, present: &wgpu::RenderPipeline, slot: &WorldSlot) -> [u8; 4] {
    let target = slot.targets.as_ref().unwrap();
    let texture = gpu.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("offscreen presentation"),
        size: target.output.texture().size(),
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: gpu.output_format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let pixel = gpu.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("widget readback"),
        size: 4,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut encoder = gpu.device.create_command_encoder(&Default::default());
    blit(
        gpu,
        &mut encoder,
        present,
        &target.sampled,
        &texture.create_view(&Default::default()),
    );
    encoder.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            origin: wgpu::Origin3d { x: 24, y: 18, z: 0 },
            ..texture.as_image_copy()
        },
        wgpu::TexelCopyBufferInfo {
            buffer: &pixel,
            layout: Default::default(),
        },
        wgpu::Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 1,
        },
    );
    gpu.queue.submit([encoder.finish()]);
    pixel
        .slice(..)
        .map_async(wgpu::MapMode::Read, |result| result.unwrap());
    gpu.device
        .poll(wgpu::PollType::wait_indefinitely())
        .unwrap();
    let mapped = pixel.slice(..).get_mapped_range().unwrap();
    mapped[..].try_into().unwrap()
}
