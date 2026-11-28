// Slice 4D Shooter - the first multiplayer shooter set in 4D space
// Copyright (C) 2023-2026  Timofei Molokov

use std::collections::HashMap;

use crate::engine::world::ui::{TextureType, UIElement, UISystem};

use super::uniform::Uniform;
use crate::engine::world::ui::RectTransformUniform;
use glam::Vec2;
use wgpu::{
    util::DeviceExt, BindGroup, Buffer, CommandEncoder, Device, PipelineCompilationOptions, Queue,
    ShaderStages, TexelCopyBufferLayout, TexelCopyTextureInfoBase, TextureView,
};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 3],
    uv: [f32; 2],
}

impl Vertex {
    const ATTRIBS: [wgpu::VertexAttribute; 2] =
        wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x2];

    fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;

        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

const RECT_VERTICES: &[Vertex] = &[
    Vertex {
        position: [-1.0, -1.0, 0.0],
        uv: [0.0, 1.0],
    },
    Vertex {
        position: [1.0, 1.0, 0.0],
        uv: [1.0, 0.0],
    },
    Vertex {
        position: [-1.0, 1.0, 0.0],
        uv: [0.0, 0.0],
    },
    Vertex {
        position: [1.0, -1.0, 0.0],
        uv: [1.0, 1.0],
    },
];

const INDICES: &[u16] = &[0, 1, 2, 0, 3, 1];

use super::{passes::begin_pass, GpuContext, WorldTextureId};
use crate::engine::world::render::{ImageSource, LocalImageKey};

pub(crate) enum UiBinding {
    Ready(BindGroup),
    World(WorldTextureId, Uniform<RectTransformUniform>),
}
pub(crate) struct UiDraw {
    pipeline: wgpu::RenderPipeline,
    binding: UiBinding,
}
pub(crate) struct UIRenderer {
    rect_vertex_buffer: Buffer,
    rect_index_buffer: Buffer,
    image_render_pipeline: wgpu::RenderPipeline,
    world_render_pipeline: wgpu::RenderPipeline,
    progress_bar_render_pipeline: wgpu::RenderPipeline,
    image_bind_group_layout: wgpu::BindGroupLayout,
    progress_bar_bind_group_layout: wgpu::BindGroupLayout,
    ui_sampler: wgpu::Sampler,
    textures: HashMap<TextureType, (TextureView, (u32, u32))>,
}
impl UIRenderer {
    pub fn new(gpu: &GpuContext) -> Self {
        let device = &gpu.device;
        let rect_vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Rect Vertex Buffer"),
            contents: bytemuck::cast_slice(RECT_VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let rect_index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Rect Index Buffer"),
            contents: bytemuck::cast_slice(INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });

        // ------------------------------------------------------------
        // create UIProgressBar render pipeline and other render stuff
        // ------------------------------------------------------------

        let progress_bar_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("progress_bar_shader"),
            source: wgpu::ShaderSource::Wgsl(
                include_str!("shaders/progress_bar_shader.wgsl").into(),
            ),
        });

        let progress_bar_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("progress_bar_bind_group_layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::VERTEX_FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 3,
                        visibility: ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 4,
                        visibility: ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            });

        let progress_bar_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("progress bar pipeline layout"),
                bind_group_layouts: &[Some(&progress_bar_bind_group_layout)],
                immediate_size: 0,
            });

        let progress_bar_render_pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("progress bar pipeline"),
                layout: Some(&progress_bar_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &progress_bar_shader,
                    entry_point: Some("vs_main"),     // 1.
                    buffers: &[Some(Vertex::desc())], // 2.
                    compilation_options: PipelineCompilationOptions::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    // 3.
                    module: &progress_bar_shader,
                    compilation_options: PipelineCompilationOptions::default(),
                    entry_point: Some("fs_main"),
                    targets: &[Some(wgpu::ColorTargetState {
                        // 4.
                        format: gpu.output_format,
                        blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList, // 1.
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw, // 2.
                    cull_mode: None,
                    polygon_mode: wgpu::PolygonMode::Fill,
                    unclipped_depth: false,
                    conservative: false,
                },
                depth_stencil: None, // 1.
                multisample: wgpu::MultisampleState {
                    count: 1,                         // 2.
                    mask: !0,                         // 3.
                    alpha_to_coverage_enabled: false, // 4.
                },
                multiview_mask: None, // 5.
                cache: None,
            });

        // ------------------------------------------------------------
        // create UIImage render pipeline and other render stuff
        // ------------------------------------------------------------

        let image_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("image_shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/image_shader.wgsl").into()),
        });

        let image_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("image_bind_group_layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::VERTEX_FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            });

        let image_render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("image pipeline layout"),
                bind_group_layouts: &[Some(&image_bind_group_layout)],
                immediate_size: 0,
            });

        let image_pipeline = |entry| {
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("image pipeline"),
                layout: Some(&image_render_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &image_shader,
                    entry_point: Some("vs_main"),     // 1.
                    buffers: &[Some(Vertex::desc())], // 2.
                    compilation_options: PipelineCompilationOptions::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    // 3.
                    module: &image_shader,
                    compilation_options: PipelineCompilationOptions::default(),
                    entry_point: Some(entry),
                    targets: &[Some(wgpu::ColorTargetState {
                        // 4.
                        format: gpu.output_format,
                        blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList, // 1.
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw, // 2.
                    cull_mode: None,
                    // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                    polygon_mode: wgpu::PolygonMode::Fill,
                    // Requires Features::DEPTH_CLIP_CONTROL
                    unclipped_depth: false,
                    // Requires Features::CONSERVATIVE_RASTERIZATION
                    conservative: false,
                },
                depth_stencil: None, // 1.
                multisample: wgpu::MultisampleState {
                    count: 1,                         // 2.
                    mask: !0,                         // 3.
                    alpha_to_coverage_enabled: false, // 4.
                },
                multiview_mask: None, // 5.
                cache: None,
            })
        };

        // --------------------------------------------------------------
        let ui_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });

        let image_render_pipeline = image_pipeline("fs_main");
        let world_render_pipeline = image_pipeline("fs_world");
        let mut textures = HashMap::new();
        for (kind, bytes) in UISystem::texture_sources() {
            make_texture_view(&mut textures, &kind, bytes, device, &gpu.queue);
        }
        Self {
            rect_vertex_buffer,
            rect_index_buffer,
            image_render_pipeline,
            world_render_pipeline,
            progress_bar_render_pipeline,
            image_bind_group_layout,
            progress_bar_bind_group_layout,
            ui_sampler,
            textures,
        }
    }

    fn image_group(
        &self,
        device: &Device,
        rect: &Uniform<RectTransformUniform>,
        view: &TextureView,
    ) -> BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("UI image"),
            layout: &self.image_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: rect.binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&self.ui_sampler),
                },
            ],
        })
    }

    pub fn prepare(
        &self,
        device: &Device,
        ui: &mut UISystem,
        screen_aspect: f32,
        local_view: impl Fn(LocalImageKey) -> Option<TextureView>,
    ) -> Vec<UiDraw> {
        let mut draws = Vec::new();
        for (id, element) in &mut ui.ui_elements {
            let order = element.get_ui_data().rect.drawing_order;
            match element {
                UIElement::Image(image) => {
                    let static_image = self.textures.get(image.get_texture_type());
                    let size = match image.source {
                        Some(ImageSource::Local(key)) => key.size.max(glam::UVec2::ONE).as_vec2(),
                        _ => static_image
                            .map_or(Vec2::ONE, |(_, (w, h))| Vec2::new(*w as f32, *h as f32)),
                    };
                    let aspect = size.x / size.y;
                    let rect = Uniform::new(
                        device,
                        "image transform",
                        &image.ui_data.rect.get_rect_transform_uniform(
                            aspect,
                            screen_aspect,
                            None,
                            0.0,
                        ),
                    );
                    image.initialize(size, aspect, rect.clone());
                    let (pipeline, binding) = match image.source {
                        Some(ImageSource::Local(key)) => {
                            let Some(view) = local_view(key) else {
                                continue;
                            };
                            (
                                &self.image_render_pipeline,
                                UiBinding::Ready(self.image_group(device, &rect, &view)),
                            )
                        }
                        Some(ImageSource::World(id)) => {
                            (&self.world_render_pipeline, UiBinding::World(id, rect))
                        }
                        None => {
                            let Some((view, _)) = static_image else {
                                continue;
                            };
                            (
                                &self.image_render_pipeline,
                                UiBinding::Ready(self.image_group(device, &rect, view)),
                            )
                        }
                    };
                    draws.push((
                        (order, false, *id),
                        UiDraw {
                            pipeline: pipeline.clone(),
                            binding,
                        },
                    ));
                }
                UIElement::ProgressBar(ui_progress_bar) => {
                    let (texture_view, (tex_width, tex_height)) = self
                        .textures
                        .get(ui_progress_bar.get_texture_type())
                        .unwrap();

                    let (mask_texture_view, (mask_width, mask_height)) = self
                        .textures
                        .get(ui_progress_bar.get_mask_texture_type())
                        .unwrap();

                    let texture_aspect = *tex_width as f32 / *tex_height as f32;

                    let texture_size = Vec2::new(*tex_width as f32, *tex_height as f32);

                    let mask_texture_aspect = *mask_width as f32 / *mask_height as f32;

                    let mask_texture_size = Vec2::new(*mask_width as f32, *mask_height as f32);

                    let rect_transform_buffer = Uniform::new(
                        device,
                        "bar transform",
                        &ui_progress_bar.ui_data.rect.get_rect_transform_uniform(
                            texture_aspect,
                            screen_aspect,
                            None,
                            0.0,
                        ),
                    );
                    let progress_bar_value_buffer = Uniform::new(
                        device,
                        "bar value",
                        &ui_progress_bar.get_progress_bar_uniform(),
                    );

                    let progress_bar_bind_group =
                        device.create_bind_group(&wgpu::BindGroupDescriptor {
                            layout: &self.progress_bar_bind_group_layout,
                            entries: &[
                                wgpu::BindGroupEntry {
                                    binding: 0,
                                    resource: rect_transform_buffer.binding(),
                                },
                                wgpu::BindGroupEntry {
                                    binding: 1,
                                    resource: wgpu::BindingResource::TextureView(&texture_view),
                                },
                                wgpu::BindGroupEntry {
                                    binding: 2,
                                    resource: wgpu::BindingResource::TextureView(
                                        &mask_texture_view,
                                    ),
                                },
                                wgpu::BindGroupEntry {
                                    binding: 3,
                                    resource: wgpu::BindingResource::Sampler(&self.ui_sampler),
                                },
                                wgpu::BindGroupEntry {
                                    binding: 4,
                                    resource: progress_bar_value_buffer.binding(),
                                },
                            ],
                            label: Some("progress_bar_bind_group"),
                        });

                    ui_progress_bar.initialize(
                        texture_size,
                        texture_aspect,
                        mask_texture_size,
                        mask_texture_aspect,
                        rect_transform_buffer,
                        progress_bar_value_buffer,
                    );

                    draws.push((
                        (order, true, *id),
                        UiDraw {
                            pipeline: self.progress_bar_render_pipeline.clone(),
                            binding: UiBinding::Ready(progress_bar_bind_group),
                        },
                    ));
                }
            }
        }
        draws.sort_unstable_by_key(|(key, _)| *key);
        draws.into_iter().map(|(_, draw)| draw).collect()
    }

    pub fn encode<'a>(
        &self,
        device: &Device,
        encoder: &mut CommandEncoder,
        output: &TextureView,
        draws: &[UiDraw],
        cache: &mut HashMap<usize, (u64, BindGroup)>,
        world: impl Fn(WorldTextureId) -> Option<(u64, &'a TextureView)>,
    ) {
        if draws.is_empty() {
            return;
        }
        let mut pass = begin_pass(encoder, output, wgpu::LoadOp::Load, "world HUD");
        pass.set_vertex_buffer(0, self.rect_vertex_buffer.slice(..));
        pass.set_index_buffer(self.rect_index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        for (index, draw) in draws.iter().enumerate() {
            let group = match &draw.binding {
                UiBinding::Ready(group) => group,
                UiBinding::World(id, rect) => {
                    let Some((generation, view)) = world(*id) else {
                        cache.remove(&index);
                        continue;
                    };
                    let entry = cache
                        .entry(index)
                        .or_insert_with(|| (generation, self.image_group(device, rect, view)));
                    if entry.0 != generation {
                        *entry = (generation, self.image_group(device, rect, view));
                    }
                    &entry.1
                }
            };
            pass.set_pipeline(&draw.pipeline);
            pass.set_bind_group(0, group, &[]);
            pass.draw_indexed(0..INDICES.len() as u32, 0, 0..1);
        }
    }
}

pub fn make_texture_view(
    textures_views: &mut HashMap<TextureType, (TextureView, (u32, u32))>,
    texture_type: &TextureType,
    texture_source: &[u8],
    device: &Device,
    queue: &Queue,
) {
    if textures_views.contains_key(texture_type) {
        return;
    }

    let diffuse_image = image::load_from_memory(texture_source).unwrap();
    let diffuse_rgba = diffuse_image.to_rgba8();

    use image::GenericImageView;
    let dimensions = diffuse_image.dimensions();

    let texture_size = wgpu::Extent3d {
        width: dimensions.0,
        height: dimensions.1,
        depth_or_array_layers: 1,
    };

    let diffuse_texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("image texture"),
        size: texture_size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });

    queue.write_texture(
        TexelCopyTextureInfoBase {
            texture: &diffuse_texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        &diffuse_rgba,
        TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(4 * dimensions.0),
            rows_per_image: Some(dimensions.1),
        },
        texture_size,
    );

    let texture_view = diffuse_texture.create_view(&wgpu::TextureViewDescriptor::default());

    textures_views.insert(texture_type.clone(), (texture_view, dimensions));
}
