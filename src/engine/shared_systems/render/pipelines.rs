// Slice 4D Shooter - the first multiplayer shooter set in 4D space
// Copyright (C) 2023-2026  Timofei Molokov

use super::{raymarch_shader_generator::generate_raymarch_shader_with_static_bsp_tree, GpuContext};
use crate::engine::world::world_descriptor::LevelData;
use std::borrow::Cow;

/// Built-in pipelines that draw a scene or a world-local image.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BuiltinPipeline {
    Slice,
    Obstacle,
    DitheredObstacle,
    Scanner,
    XwzMinimap,
}

impl BuiltinPipeline {
    pub fn name(self) -> &'static str {
        match self {
            Self::Slice => "raymarch_shader_slice_4d",
            Self::Obstacle => "raymarch_shader_obstacle_course",
            Self::DitheredObstacle => "raymarch_shader_obstacle_course_dithered",
            Self::Scanner => "scanner",
            Self::XwzMinimap => "xwz_minimap",
        }
    }
    pub fn parse(key: &str) -> Option<Self> {
        [
            Self::Slice,
            Self::Obstacle,
            Self::DitheredObstacle,
            Self::Scanner,
            Self::XwzMinimap,
        ]
        .into_iter()
        .find(|variant| variant.name() == key)
    }
}

pub fn compile(
    gpu: &GpuContext,
    kind: BuiltinPipeline,
    level: Option<&LevelData>,
) -> wgpu::RenderPipeline {
    let template = match kind {
        BuiltinPipeline::Slice => include_str!("shaders/raymarch_shader.wgsl"),
        BuiltinPipeline::Obstacle => include_str!("shaders/raymarch_shader_obstacle_course.wgsl"),
        BuiltinPipeline::DitheredObstacle => {
            include_str!("shaders/raymarch_shader_obstacle_course_dithered.wgsl")
        }
        BuiltinPipeline::Scanner => include_str!("shaders/scanner_shader.wgsl"),
        BuiltinPipeline::XwzMinimap => include_str!("shaders/xwz_minimap_shader.wgsl"),
    };
    let source = match level {
        Some(level) => Cow::Owned(generate_raymarch_shader_with_static_bsp_tree(
            template,
            level.bsp(),
            level.static_render_data(),
            false,
            &level.allowed_dyn_objects,
        )),
        None => Cow::Borrowed(template),
    };
    // The shipped raymarch templates index fixed world buffers under the same bounds contract as the collector.
    let desc = wgpu::ShaderModuleDescriptor {
        label: Some(kind.name()),
        source: wgpu::ShaderSource::Wgsl(source),
    };
    let shader = if kind == BuiltinPipeline::Scanner {
        gpu.device.create_shader_module(desc)
    } else {
        unsafe {
            gpu.device
                .create_shader_module_trusted(desc, wgpu::ShaderRuntimeChecks::unchecked())
        }
    };
    let scanner_layout;
    let layout = if kind == BuiltinPipeline::Scanner {
        scanner_layout = gpu
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("scanner pipeline layout"),
                bind_group_layouts: &[Some(&gpu.scanner_layout)],
                immediate_size: 0,
            });
        &scanner_layout
    } else {
        &gpu.raymarch_pipeline_layout
    };
    gpu.device
        .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some(kind.name()),
            layout: Some(layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[Some(gpu.raymarch_vertex_layout.clone())],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: gpu.output_format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: Default::default(),
            depth_stencil: None,
            multisample: Default::default(),
            multiview_mask: None,
            cache: None,
        })
}
