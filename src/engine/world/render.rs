// Slice 4D Shooter - the first multiplayer shooter set in 4D space
// Copyright (C) 2023-2026  Timofei Molokov

mod buffers;
mod dynamic_texture;
use buffers::WorldGpuBuffers;
use dynamic_texture::EffectInstance;
pub use dynamic_texture::{
    DynamicTexture, ImageSource, LocalImageKey, ScannerPlane, TextureEffect,
};

use super::{actors::Actors, ui::UISystem, world_descriptor::WorldDescriptor, WorldRenderConfig};
use crate::engine::shared_systems::{
    render::{
        passes::{texture_view, Draw, TextureDraw, WorldPlan},
        render_data::RenderData,
        render_scheduler::WorldRenderShared,
        uniform::Uniform,
        GpuContext, RenderQualityData, WorldTextureId,
    },
    resources::{
        graphics::{BuiltinPipeline, CubemapTexture, RenderPipelineResource},
        ResourceLoader,
    },
    EngineSharedSystems,
};
use fyrox_resource::Resource;
use glam::UVec2;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use tokio::runtime::Handle;
use winit::dpi::PhysicalSize;

pub struct WorldRenderSystem {
    shared: Arc<WorldRenderShared>,
    texture_id: WorldTextureId,
    gpu: GpuContext,
    runtime: Handle,
    buffers: WorldGpuBuffers,
    render_data: RenderData,
    stickiness_radius: f32,
    scene: Draw,
    level_key: String,
    _level: Resource<crate::engine::shared_systems::resources::graphics::LevelResource>,
    main: Resource<RenderPipelineResource>,
    _sky: Resource<CubemapTexture>,
    pending_scene: Option<Resource<RenderPipelineResource>>,
    upscale: bool,
    effects: Vec<Arc<EffectInstance>>,
    next_effect: u64,
    dirty: Arc<AtomicBool>,
}

impl WorldRenderSystem {
    pub fn new(
        descriptor: &WorldDescriptor,
        config: WorldRenderConfig,
        shared: &mut EngineSharedSystems,
    ) -> Option<Self> {
        let gpu = shared.gpu.as_ref()?;
        let scheduler = shared.render.as_mut()?;
        let descriptor_gpu = descriptor.gpu.as_ref()?;
        let render_data = RenderData::new(descriptor.static_render_data().clone());
        let buffers = WorldGpuBuffers::new(gpu, &render_data);
        let scene_groups = buffers.bind_scene(
            gpu,
            descriptor_gpu.sky.data_ref().value(),
            &descriptor.skybox.filter.sampler(&gpu.device),
        );
        let slot = Arc::new(WorldRenderShared::new(&config));
        let texture_id = scheduler.register_world(&slot);
        Some(Self {
            texture_id,
            shared: slot,
            gpu: gpu.clone(),
            runtime: shared.runtime.handle().clone(),
            scene: Draw {
                pipeline: descriptor_gpu.main.data_ref().value().clone(),
                groups: scene_groups,
            },
            buffers,
            render_data,
            stickiness_radius: descriptor.all_shapes_stickiness_radius,
            level_key: descriptor.level_key.clone(),
            _level: descriptor.source.clone(),
            main: descriptor_gpu.main.clone(),
            _sky: descriptor_gpu.sky.clone(),
            pending_scene: None,
            upscale: config.with_upscale.unwrap_or(descriptor.with_upscale),
            effects: Vec::new(),
            next_effect: 0,
            dirty: Arc::new(AtomicBool::new(true)),
        })
    }

    /// All scene variants consume the same level specialization and instance buffers.
    #[allow(dead_code)] // Runtime scene changes are available to gameplay.
    pub fn set_scene(
        &mut self,
        loader: &mut ResourceLoader,
        shader: BuiltinPipeline,
    ) -> Result<(), &'static str> {
        if matches!(
            shader,
            BuiltinPipeline::Scanner | BuiltinPipeline::XwzMinimap
        ) {
            return Err("Expected a scene pipeline");
        }
        self.pending_scene = Some(RenderPipelineResource::request(
            loader,
            shader,
            Some(&self.level_key),
        ));
        Ok(())
    }

    pub fn update(&mut self, actors: &Actors, ui: &mut UISystem, shared: &EngineSharedSystems) {
        let (resolution, _) = self.shared.output();

        let output_size = PhysicalSize::new(resolution.x.max(1), resolution.y.max(1));

        let render_quality_data = RenderQualityData {
            shadows_enabled: shared.global_settings.user.shadows_enabled,
        };

        let static_bounding_box = self.render_data.static_data.static_bounding_box.clone();

        self.render_data.update_dynamic_render_data(
            actors,
            &shared.time,
            output_size,
            &static_bounding_box,
            &render_quality_data,
            self.stickiness_radius,
        );

        self.buffers
            .write(&self.gpu.queue, &self.render_data.dynamic_data);

        if let Some(resource) = &self.pending_scene {
            use crate::engine::shared_systems::resources::ResourceRead;
            if let std::task::Poll::Ready(result) = resource.try_read(|asset| asset.value().clone())
            {
                match result {
                    Ok(pipeline) => {
                        self.scene.pipeline = pipeline;
                        self.main = resource.clone();
                        self.dirty.store(true, Ordering::Release);
                    }
                    Err(error) => log::error!("Replacing scene pipeline failed: {error:?}"),
                }
                self.pending_scene = None;
            }
        }

        let aspect = resolution.x as f32 / resolution.y as f32;
        if self.dirty.swap(false, Ordering::AcqRel) || ui.bindings_changed {
            self.effects
                .retain(|entry| entry.owned.load(Ordering::Acquire));
            let ready: Vec<_> = self
                .effects
                .iter()
                .filter_map(|entry| {
                    entry
                        .state
                        .borrow()
                        .as_ref()?
                        .as_ref()
                        .ok()
                        .map(|draw| (entry.key, draw.clone()))
                })
                .collect();
            let scheduler = shared.render.as_ref().unwrap();
            let draws = scheduler
                .ui_renderer
                .prepare(&self.gpu.device, ui, aspect, |key| {
                    ready
                        .iter()
                        .find(|(id, _)| *id == key)
                        .map(|(_, draw)| draw.view.clone())
                });
            ui.write_buffers_ui(&self.gpu.queue, aspect);
            scheduler.publish(
                self.texture_id,
                Arc::new(WorldPlan {
                    scene: self.scene.clone(),
                    upscale: self.upscale,
                    textures: ready.into_iter().map(|(_, draw)| draw).collect(),
                    ui: draws,
                }),
            );
            ui.bindings_changed = false;
        } else {
            ui.write_buffers_ui(&self.gpu.queue, aspect);
        }
    }

    pub fn texture_id(&self) -> WorldTextureId {
        self.texture_id
    }

    #[allow(dead_code)] // nested-world API (issues/programming/engine-refactoring-world-isolation/), not used by the game yet
    pub fn set_render_rate(&self, render_rate: u32) {
        self.shared.set_render_rate(render_rate);
    }

    pub fn set_output_resolution(&self, resolution: UVec2) {
        let (_, scale) = self.shared.output();
        self.shared.set_output(resolution, scale);
    }

    pub fn set_output_scale(&self, scale: f32) {
        let (resolution, _) = self.shared.output();
        self.shared.set_output(resolution, scale);
    }

    #[allow(dead_code)] // nested-world API (issues/programming/engine-refactoring-world-isolation/), not used by the game yet
    pub fn set_output_resolution_and_scale(&self, resolution: UVec2, scale: f32) {
        self.shared.set_output(resolution, scale);
    }

    pub fn output_scale(&self) -> f32 {
        self.shared.output().1
    }
}
