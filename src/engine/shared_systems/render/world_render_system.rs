// Slice 4D Shooter - the first multiplayer shooter set in 4D space
// Copyright (C) 2023-2026  Timofei Molokov



use std::sync::Arc;

use glam::UVec2;
use wgpu::{
    BufferUsages,
    Color,
    util::{
        BufferInitDescriptor,
        DeviceExt,
    }
};
use winit::dpi::PhysicalSize;

use crate::engine::{
    shared_systems::EngineSharedSystems,
    world::{
        actors::Actors,
        time::WorldTimeSystem,
        world_descriptor::WorldDescriptor,
        WorldRenderConfig,
    },
};

use super::{
    gpu_context::GpuContext,
    render_data::RenderData,
    render_scheduler::{unpack_output, WorldRenderShared, WorldTextureId},
    ui_renderer::ScannerWorldBuffers,
    RenderQualityData,
};


/// The render part of one World, owned by the World (main thread).
///
/// It computes the world's render data in `simulate()` and writes it into the world's GPU buffers
/// (`queue.write_buffer` is thread-safe). The pipelines, bind groups and output texture were moved
/// into the render thread's slot at registration, so there is no lock between the two threads.
/// Dropping it (with the World) drops the `Arc<WorldRenderShared>`, and the render thread removes the slot.
pub struct WorldRenderSystem
{
    shared: Arc<WorldRenderShared>,
    texture_id: WorldTextureId,
    queue: wgpu::Queue,
    buffers: WorldGpuBuffers,
    render_data: RenderData,
    stickiness_radius: f32,
    is_generated_raymarch_shader: bool,
}


/// The world's GPU buffers. `Clone` copies handles (the same GPU buffers).
#[derive(Clone)]
pub struct WorldGpuBuffers
{
    pub other_dynamic_data_buffer: wgpu::Buffer,
    pub other_static_data_buffer: wgpu::Buffer,
    pub dynamic_negative_shapes_buffer: wgpu::Buffer,
    pub dynamic_normal_shapes_buffer: wgpu::Buffer,
    pub dynamic_stickiness_shapes_buffer: wgpu::Buffer,
    pub dynamic_neg_stickiness_shapes_buffer: wgpu::Buffer,
    pub dynamic_undestroyable_shapes_buffer: wgpu::Buffer,
    pub dynamic_undestroyable_stickiness_shapes_buffer: wgpu::Buffer,
    pub spherical_areas_data_buffer: wgpu::Buffer,
    pub beam_areas_data_buffer: wgpu::Buffer,
    pub player_forms_data_buffer: wgpu::Buffer,
}

impl WorldGpuBuffers
{
    pub fn scanner_world_buffers(&self) -> ScannerWorldBuffers<'_>
    {
        ScannerWorldBuffers {
            other_dynamic_data_buffer: &self.other_dynamic_data_buffer,
            player_forms_data_buffer: &self.player_forms_data_buffer,
            dynamic_normal_shapes_buffer: &self.dynamic_normal_shapes_buffer,
            dynamic_stickiness_shapes_buffer: &self.dynamic_stickiness_shapes_buffer,
            dynamic_negative_shapes_buffer: &self.dynamic_negative_shapes_buffer,
            dynamic_neg_stickiness_shapes_buffer: &self.dynamic_neg_stickiness_shapes_buffer,
        }
    }
}


/// GPU objects of one world, used only by the render thread after registration.
pub struct WorldGpu
{
    main_pipeline: wgpu::RenderPipeline,
    minimap_pipeline: Option<wgpu::RenderPipeline>,
    bind_group_0: wgpu::BindGroup,
    bind_group_1: wgpu::BindGroup,
    pub buffers: WorldGpuBuffers,
    pub output: OutputTexture,
}


/// A world's output texture (the raymarched slice), recreated when the world's
/// output resolution or scale changes.
pub struct OutputTexture
{
    pub view: wgpu::TextureView,
    /// Samples `view` in the upscale pass (when this world is presented in the window).
    pub upscale_bind_group: wgpu::BindGroup,
    /// The packed (resolution, scale) the texture was created for.
    packed_output: u64,
    /// Bumped on every recreation; consumers of the texture rebuild their bind groups.
    pub generation: u32,
}


impl OutputTexture
{
    fn new(gpu: &GpuContext, packed_output: u64, generation: u32) -> Self
    {
        let (resolution, scale) = unpack_output(packed_output);

        let width = ((resolution.x as f32 * scale) as u32).min(resolution.x).max(1);
        let height = ((resolution.y as f32 * scale) as u32).min(resolution.y).max(1);

        let texture = gpu.device.create_texture(
            &wgpu::TextureDescriptor {
                label: Some("world output texture"),
                size: wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: gpu.output_format,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[]
            }
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let upscale_bind_group = gpu.create_upscale_bind_group(&view);

        OutputTexture {
            view,
            upscale_bind_group,
            packed_output,
            generation,
        }
    }
}


impl WorldGpu
{
    /// Recreates the output texture if the world's resolution/scale changed (render thread only).
    /// Returns true if it was recreated.
    pub fn update_output_texture(&mut self, gpu: &GpuContext, packed_output: u64) -> bool
    {
        if self.output.packed_output == packed_output
        {
            return false;
        }

        let generation = self.output.generation.wrapping_add(1);

        self.output = OutputTexture::new(gpu, packed_output, generation);

        true
    }

    /// Records the world's passes (raymarch + optional xwz minimap) into its output texture.
    pub fn encode(&self, encoder: &mut wgpu::CommandEncoder, gpu: &GpuContext)
    {
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("raymarch shader render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.output.view,
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
                multiview_mask: None,
            });

            render_pass.set_pipeline(&self.main_pipeline);
            render_pass.set_bind_group(0, &self.bind_group_0, &[]);
            render_pass.set_bind_group(1, &self.bind_group_1, &[]);
            render_pass.set_vertex_buffer(0, gpu.fullscreen_vertex_buffer.slice(..));
            render_pass.set_index_buffer(gpu.fullscreen_index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..gpu.fullscreen_num_indices, 0, 0..1);
        }

        if let Some(minimap_pipeline) = &self.minimap_pipeline
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("xwz minimap shader render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.output.view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });

            render_pass.set_pipeline(minimap_pipeline);
            render_pass.set_bind_group(0, &self.bind_group_0, &[]);
            render_pass.set_bind_group(1, &self.bind_group_1, &[]);
            render_pass.set_vertex_buffer(0, gpu.minimap_vertex_buffer.slice(..));
            render_pass.set_index_buffer(gpu.minimap_index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..gpu.minimap_num_indices, 0, 0..1);
        }
    }
}


impl WorldRenderSystem
{
    /// Creates the world's GPU objects and registers them with the render scheduler.
    /// `None` in headless mode, or if the descriptor was loaded without a GPU.
    pub fn new(
        descriptor: &WorldDescriptor,
        config: WorldRenderConfig,
        shared: &mut EngineSharedSystems,
    ) -> Option<Self>
    {
        let (Some(gpu), Some(scheduler)) = (&shared.gpu, &mut shared.render) else {
            return None;
        };

        let Some(descriptor_gpu) = &descriptor.gpu else {
            log::warn!("world render system: the world descriptor has no GPU part, the world won't be rendered");
            return None;
        };

        let mut render_data = RenderData::new();

        render_data.static_data.update(descriptor);

        let buffers = create_world_buffers(gpu, &render_data);

        let bind_group_0 = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &gpu.raymarch_bind_group_layout_0,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buffers.other_static_data_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: buffers.dynamic_normal_shapes_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: buffers.dynamic_negative_shapes_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: buffers.dynamic_stickiness_shapes_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: buffers.dynamic_neg_stickiness_shapes_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: buffers.dynamic_undestroyable_shapes_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 6,
                    resource: buffers.dynamic_undestroyable_stickiness_shapes_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 7,
                    resource: buffers.other_dynamic_data_buffer.as_entire_binding(),
                }
            ],
            label: Some("shader_uniforms_and_storage_bind_group_0"),
        });

        let bind_group_1 = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &gpu.raymarch_bind_group_layout_1,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buffers.spherical_areas_data_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: buffers.beam_areas_data_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: buffers.player_forms_data_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Sampler(&gpu.sky_box_sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::TextureView(&gpu.sky_box_view),
                },
            ],
            label: Some("shader_uniforms_and_storage_bind_group_1"),
        });

        let world_render_shared = Arc::new(WorldRenderShared::new(&config));

        let output = OutputTexture::new(gpu, world_render_shared.packed_output(), 0);

        let world_gpu = WorldGpu {
            main_pipeline: descriptor_gpu.main_pipeline.clone(),
            minimap_pipeline: descriptor_gpu.minimap_pipeline.clone(),
            bind_group_0,
            bind_group_1,
            buffers: buffers.clone(),
            output,
        };

        let texture_id = scheduler.register_world(world_gpu, &world_render_shared);

        Some(WorldRenderSystem {
            shared: world_render_shared,
            texture_id,
            queue: gpu.queue.clone(),
            buffers,
            render_data,
            stickiness_radius: descriptor.all_shapes_stickiness_radius,
            is_generated_raymarch_shader: descriptor.is_generated_raymarch_shader,
        })
    }


    /// End of `World::simulate`: computes the dynamic render data from the actors and writes the buffers.
    pub fn update(
        &mut self,
        actors: &Actors,
        _time: &WorldTimeSystem,
        shared: &EngineSharedSystems,
    )
    {
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
            self.is_generated_raymarch_shader,
            &render_quality_data,
            self.stickiness_radius,
        );

        let dynamic_data = &self.render_data.dynamic_data;

        self.queue.write_buffer(
            &self.buffers.other_dynamic_data_buffer,
            0,
            bytemuck::cast_slice(&[dynamic_data.other_dynamic_data]),
        );

        self.queue.write_buffer(
            &self.buffers.dynamic_negative_shapes_buffer,
            0,
            bytemuck::cast_slice(dynamic_data.dynamic_shapes_data.negative.as_slice()),
        );

        self.queue.write_buffer(
            &self.buffers.dynamic_normal_shapes_buffer,
            0,
            bytemuck::cast_slice(dynamic_data.dynamic_shapes_data.normal.as_slice()),
        );

        self.queue.write_buffer(
            &self.buffers.dynamic_stickiness_shapes_buffer,
            0,
            bytemuck::cast_slice(dynamic_data.dynamic_shapes_data.stickiness.as_slice()),
        );

        self.queue.write_buffer(
            &self.buffers.dynamic_neg_stickiness_shapes_buffer,
            0,
            bytemuck::cast_slice(dynamic_data.dynamic_shapes_data.neg_stickiness.as_slice()),
        );

        self.queue.write_buffer(
            &self.buffers.dynamic_undestroyable_shapes_buffer,
            0,
            bytemuck::cast_slice(dynamic_data.dynamic_shapes_data.undestroyable.as_slice()),
        );

        self.queue.write_buffer(
            &self.buffers.dynamic_undestroyable_stickiness_shapes_buffer,
            0,
            bytemuck::cast_slice(dynamic_data.dynamic_shapes_data.undestroyable_stickiness.as_slice()),
        );

        self.queue.write_buffer(
            &self.buffers.spherical_areas_data_buffer,
            0,
            bytemuck::cast_slice(dynamic_data.spherical_areas_data.as_slice()),
        );

        self.queue.write_buffer(
            &self.buffers.beam_areas_data_buffer,
            0,
            bytemuck::cast_slice(dynamic_data.beam_areas_data.as_slice()),
        );

        self.queue.write_buffer(
            &self.buffers.player_forms_data_buffer,
            0,
            bytemuck::cast_slice(dynamic_data.player_forms_data.as_slice()),
        );
    }


    pub fn texture_id(&self) -> WorldTextureId
    {
        self.texture_id
    }

    #[allow(dead_code)] // nested-world API (issues/programming/engine-refactoring-world-isolation/), not used by the game yet
    pub fn set_render_rate(&self, render_rate: u32)
    {
        self.shared.set_render_rate(render_rate);
    }

    pub fn set_output_resolution(&self, resolution: UVec2)
    {
        let (_, scale) = self.shared.output();
        self.shared.set_output(resolution, scale);
    }

    pub fn set_output_scale(&self, scale: f32)
    {
        let (resolution, _) = self.shared.output();
        self.shared.set_output(resolution, scale);
    }

    #[allow(dead_code)] // nested-world API (issues/programming/engine-refactoring-world-isolation/), not used by the game yet
    pub fn set_output_resolution_and_scale(&self, resolution: UVec2, scale: f32)
    {
        self.shared.set_output(resolution, scale);
    }

    pub fn output_scale(&self) -> f32
    {
        self.shared.output().1
    }
}


fn create_world_buffers(gpu: &GpuContext, render_data: &RenderData) -> WorldGpuBuffers
{
    let uniform_buffer = |label: &str, contents: &[u8]| {
        gpu.device.create_buffer_init(&BufferInitDescriptor {
            label: Some(label),
            contents,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        })
    };

    let dynamic_data = &render_data.dynamic_data;

    WorldGpuBuffers {
        other_static_data_buffer: uniform_buffer(
            "other_static_data",
            bytemuck::cast_slice(&[render_data.static_data.other_static_data]),
        ),
        dynamic_normal_shapes_buffer: uniform_buffer(
            "dynamic_normal_shapes_buffer",
            bytemuck::cast_slice(dynamic_data.dynamic_shapes_data.normal.as_slice()),
        ),
        dynamic_stickiness_shapes_buffer: uniform_buffer(
            "dynamic_stickiness_shapes_buffer",
            bytemuck::cast_slice(dynamic_data.dynamic_shapes_data.stickiness.as_slice()),
        ),
        dynamic_negative_shapes_buffer: uniform_buffer(
            "dynamic_negative_shapes_buffer",
            bytemuck::cast_slice(dynamic_data.dynamic_shapes_data.negative.as_slice()),
        ),
        dynamic_neg_stickiness_shapes_buffer: uniform_buffer(
            "dynamic_neg_stickiness_shapes_buffer",
            bytemuck::cast_slice(dynamic_data.dynamic_shapes_data.neg_stickiness.as_slice()),
        ),
        dynamic_undestroyable_shapes_buffer: uniform_buffer(
            "dynamic_undestroyable_shapes_buffer",
            bytemuck::cast_slice(dynamic_data.dynamic_shapes_data.undestroyable.as_slice()),
        ),
        dynamic_undestroyable_stickiness_shapes_buffer: uniform_buffer(
            "dynamic_undestroyable_stickiness_shapes_buffer",
            bytemuck::cast_slice(dynamic_data.dynamic_shapes_data.undestroyable_stickiness.as_slice()),
        ),
        other_dynamic_data_buffer: uniform_buffer(
            "other_dynamic_data_buffer",
            bytemuck::cast_slice(&[dynamic_data.other_dynamic_data]),
        ),
        spherical_areas_data_buffer: uniform_buffer(
            "spherical_areas_data_buffer",
            bytemuck::cast_slice(dynamic_data.spherical_areas_data.as_slice()),
        ),
        beam_areas_data_buffer: uniform_buffer(
            "beam_areas_data_buffer",
            bytemuck::cast_slice(dynamic_data.beam_areas_data.as_slice()),
        ),
        player_forms_data_buffer: uniform_buffer(
            "player_forms_data_buffer",
            bytemuck::cast_slice(dynamic_data.player_forms_data.as_slice()),
        ),
    }
}
