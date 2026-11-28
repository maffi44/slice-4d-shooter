// Slice 4D Shooter - the first multiplayer shooter set in 4D space
// Copyright (C) 2023-2026  Timofei Molokov

use super::{ui_renderer::UiDraw, GpuContext};
use glam::UVec2;
use std::sync::Arc;

#[derive(Clone)]
pub(crate) struct Draw {
    pub pipeline: wgpu::RenderPipeline,
    pub groups: Vec<wgpu::BindGroup>,
}

impl Draw {
    pub fn encode(
        &self,
        gpu: &GpuContext,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
    ) {
        let mut pass = begin_pass(
            encoder,
            target,
            wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
            "world draw",
        );
        pass.set_pipeline(&self.pipeline);
        for (index, group) in self.groups.iter().enumerate() {
            pass.set_bind_group(index as u32, group, &[]);
        }
        fullscreen(gpu, &mut pass);
    }
}

pub(crate) struct TextureDraw {
    pub _pipeline: fyrox_resource::Resource<crate::engine::shared_systems::resources::graphics::RenderPipelineResource>,
    pub draw: Draw,
    pub view: wgpu::TextureView,
}

pub(crate) struct WorldPlan {
    pub scene: Draw,
    pub textures: Vec<Arc<TextureDraw>>,
    pub upscale: bool,
    pub ui: Vec<UiDraw>,
}

pub(crate) fn texture_view(gpu: &GpuContext, size: UVec2, label: &str) -> wgpu::TextureView {
    gpu.device
        .create_texture(&wgpu::TextureDescriptor {
            label: Some(label),
            size: wgpu::Extent3d {
                width: size.x,
                height: size.y,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: gpu.output_format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        })
        .create_view(&Default::default())
}

pub(crate) fn begin_pass<'a>(
    encoder: &'a mut wgpu::CommandEncoder,
    view: &wgpu::TextureView,
    load: wgpu::LoadOp<wgpu::Color>,
    label: &str,
) -> wgpu::RenderPass<'a> {
    encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some(label),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view,
            resolve_target: None,
            depth_slice: None,
            ops: wgpu::Operations {
                load,
                store: wgpu::StoreOp::Store,
            },
        })],
        ..Default::default()
    })
}

fn fullscreen(gpu: &GpuContext, pass: &mut wgpu::RenderPass<'_>) {
    pass.set_vertex_buffer(0, gpu.fullscreen_vertex_buffer.slice(..));
    pass.set_index_buffer(
        gpu.fullscreen_index_buffer.slice(..),
        wgpu::IndexFormat::Uint16,
    );
    pass.draw_indexed(0..gpu.fullscreen_num_indices, 0, 0..1);
}

pub(crate) fn blit(
    gpu: &GpuContext,
    encoder: &mut wgpu::CommandEncoder,
    pipeline: &wgpu::RenderPipeline,
    input: &wgpu::BindGroup,
    target: &wgpu::TextureView,
) {
    let mut pass = begin_pass(
        encoder,
        target,
        wgpu::LoadOp::Clear(wgpu::Color::BLACK),
        "blit",
    );
    pass.set_pipeline(pipeline);
    pass.set_bind_group(0, input, &[]);
    fullscreen(gpu, &mut pass);
}
