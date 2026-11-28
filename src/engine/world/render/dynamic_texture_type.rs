// Slice 4D Shooter - the first multiplayer shooter set in 4D space
// Copyright (C) 2023-2026  Timofei Molokov

use super::{
    BuiltinPipelineType, GpuContext, RenderPipelineResource, ResourceLoader, Uniform,
    WorldRenderSystem,
};
use crate::engine::shared_systems::render::render_data::dynamic_render_data::OtherDynamicData;
use fyrox_resource::Resource;

#[repr(u32)]
pub enum ScannerPlane {
    Zx,
    Zw,
}
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct ScannerParams {
    padding: [u32; 3],
    orientation: u32,
}

pub enum DynamicTextureType {
    Scanner(ScannerPlane),
    XwzMinimap,
}

pub(super) enum DynamicTextureInputs {
    Scanner {
        plane: ScannerPlane,
        world_data: Uniform<OtherDynamicData>,
    },
    Scene(Vec<wgpu::BindGroup>),
}

impl DynamicTextureType {
    pub(super) fn request_pipeline_and_inputs(
        self,
        world: &WorldRenderSystem,
        loader: &mut ResourceLoader,
    ) -> (Resource<RenderPipelineResource>, DynamicTextureInputs) {
        let (shader, level, inputs) = match self {
            DynamicTextureType::Scanner(plane) => (
                BuiltinPipelineType::Scanner,
                None,
                DynamicTextureInputs::Scanner {
                    plane,
                    world_data: world.buffers.other_dynamic_data_buffer.clone(),
                },
            ),
            DynamicTextureType::XwzMinimap => (
                BuiltinPipelineType::XwzMinimap,
                Some(world.level_key.as_str()),
                DynamicTextureInputs::Scene(world.scene.groups.clone()),
            ),
        };
        (
            RenderPipelineResource::request(loader, shader, level),
            inputs,
        )
    }
}

impl DynamicTextureInputs {
    pub(super) fn bind(self, gpu: &GpuContext) -> Vec<wgpu::BindGroup> {
        match self {
            Self::Scanner { plane, world_data } => {
                let params = Uniform::new(
                    &gpu.device,
                    "scanner plane",
                    &ScannerParams {
                        padding: [0; 3],
                        orientation: plane as u32,
                    },
                );
                vec![gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("scanner"),
                    layout: &gpu.scanner_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: world_data.binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: params.binding(),
                        },
                    ],
                })]
            }
            Self::Scene(groups) => groups,
        }
    }
}
