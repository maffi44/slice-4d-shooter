// Slice 4D Shooter - the first multiplayer shooter set in 4D space
// Copyright (C) 2023-2026  Timofei Molokov

use crate::engine::shared_systems::render::{
    render_data::{
        dynamic_render_data, static_render_data, BeamArea, PlayerForm, RenderData, Shape,
        SphericalArea,
    },
    uniform::Uniform,
    GpuContext,
};

pub(super) struct WorldGpuBuffers {
    other_static_data_buffer: Uniform<static_render_data::OtherStaticData>,
    pub(super) other_dynamic_data_buffer: Uniform<dynamic_render_data::OtherDynamicData>,
    dynamic_negative_shapes_buffer: Uniform<[Shape; 256]>,
    dynamic_normal_shapes_buffer: Uniform<[Shape; 256]>,
    dynamic_stickiness_shapes_buffer: Uniform<[Shape; 256]>,
    dynamic_neg_stickiness_shapes_buffer: Uniform<[Shape; 256]>,
    dynamic_undestroyable_shapes_buffer: Uniform<[Shape; 256]>,
    dynamic_undestroyable_stickiness_shapes_buffer: Uniform<[Shape; 256]>,
    spherical_areas_data_buffer: Uniform<[SphericalArea; 256]>,
    beam_areas_data_buffer: Uniform<[BeamArea; 256]>,
    player_forms_data_buffer: Uniform<[PlayerForm; 16]>,
}

impl WorldGpuBuffers {
    pub(super) fn new(gpu: &GpuContext, render_data: &RenderData) -> Self {
        let dynamic_data = &render_data.dynamic_data;
        Self {
            other_static_data_buffer: Uniform::new(
                &gpu.device,
                "other_static_data",
                &render_data.static_data.other_static_data,
            ),
            other_dynamic_data_buffer: Uniform::new(
                &gpu.device,
                "other_dynamic_data",
                &dynamic_data.other_dynamic_data,
            ),
            dynamic_negative_shapes_buffer: Uniform::new(
                &gpu.device,
                "dynamic_negative_shapes",
                &dynamic_data.dynamic_shapes_data.negative,
            ),
            dynamic_normal_shapes_buffer: Uniform::new(
                &gpu.device,
                "dynamic_normal_shapes",
                &dynamic_data.dynamic_shapes_data.normal,
            ),
            dynamic_stickiness_shapes_buffer: Uniform::new(
                &gpu.device,
                "dynamic_stickiness_shapes",
                &dynamic_data.dynamic_shapes_data.stickiness,
            ),
            dynamic_neg_stickiness_shapes_buffer: Uniform::new(
                &gpu.device,
                "dynamic_neg_stickiness_shapes",
                &dynamic_data.dynamic_shapes_data.neg_stickiness,
            ),
            dynamic_undestroyable_shapes_buffer: Uniform::new(
                &gpu.device,
                "dynamic_undestroyable_shapes",
                &dynamic_data.dynamic_shapes_data.undestroyable,
            ),
            dynamic_undestroyable_stickiness_shapes_buffer: Uniform::new(
                &gpu.device,
                "dynamic_undestroyable_stickiness_shapes",
                &dynamic_data.dynamic_shapes_data.undestroyable_stickiness,
            ),
            spherical_areas_data_buffer: Uniform::new(
                &gpu.device,
                "spherical_areas_data",
                &dynamic_data.spherical_areas_data,
            ),
            beam_areas_data_buffer: Uniform::new(
                &gpu.device,
                "beam_areas_data",
                &dynamic_data.beam_areas_data,
            ),
            player_forms_data_buffer: Uniform::new(
                &gpu.device,
                "player_forms_data",
                &dynamic_data.player_forms_data,
            ),
        }
    }

    pub(super) fn bind_scene(
        &self,
        gpu: &GpuContext,
        sky: &wgpu::TextureView,
        sampler: &wgpu::Sampler,
    ) -> Vec<wgpu::BindGroup> {
        let bind_group_0 = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &gpu.raymarch_bind_group_layout_0,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: self.other_static_data_buffer.binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: self.dynamic_normal_shapes_buffer.binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: self.dynamic_negative_shapes_buffer.binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: self.dynamic_stickiness_shapes_buffer.binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: self.dynamic_neg_stickiness_shapes_buffer.binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: self.dynamic_undestroyable_shapes_buffer.binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 6,
                    resource: self
                        .dynamic_undestroyable_stickiness_shapes_buffer
                        .binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 7,
                    resource: self.other_dynamic_data_buffer.binding(),
                },
            ],
            label: Some("shader_uniforms_and_storage_bind_group_0"),
        });

        let bind_group_1 = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &gpu.raymarch_bind_group_layout_1,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: self.spherical_areas_data_buffer.binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: self.beam_areas_data_buffer.binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: self.player_forms_data_buffer.binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Sampler(sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::TextureView(sky),
                },
            ],
            label: Some("shader_uniforms_and_storage_bind_group_1"),
        });

        vec![bind_group_0, bind_group_1]
    }

    pub(super) fn write(
        &self,
        queue: &wgpu::Queue,
        dynamic_data: &dynamic_render_data::DynamicRenderData,
    ) {
        self.other_dynamic_data_buffer
            .write(queue, &dynamic_data.other_dynamic_data);
        self.dynamic_negative_shapes_buffer
            .write(queue, &dynamic_data.dynamic_shapes_data.negative);
        self.dynamic_normal_shapes_buffer
            .write(queue, &dynamic_data.dynamic_shapes_data.normal);
        self.dynamic_stickiness_shapes_buffer
            .write(queue, &dynamic_data.dynamic_shapes_data.stickiness);
        self.dynamic_neg_stickiness_shapes_buffer
            .write(queue, &dynamic_data.dynamic_shapes_data.neg_stickiness);
        self.dynamic_undestroyable_shapes_buffer
            .write(queue, &dynamic_data.dynamic_shapes_data.undestroyable);
        self.dynamic_undestroyable_stickiness_shapes_buffer.write(
            queue,
            &dynamic_data.dynamic_shapes_data.undestroyable_stickiness,
        );
        self.spherical_areas_data_buffer
            .write(queue, &dynamic_data.spherical_areas_data);
        self.beam_areas_data_buffer
            .write(queue, &dynamic_data.beam_areas_data);
        self.player_forms_data_buffer
            .write(queue, &dynamic_data.player_forms_data);
    }
}
