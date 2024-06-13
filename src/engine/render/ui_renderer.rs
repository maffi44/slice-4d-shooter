use std::{collections::HashMap, sync::{Arc, Mutex}};

use crate::engine::{render::render_data::RenderData, ui::{TextureType, UIElement, UISystem}};

use glam::Vec2;
use winit::window::Window;
use wgpu::{
    hal::empty::Encoder, rwh::{
        HasDisplayHandle,
        HasWindowHandle
    }, util::{
        BufferInitDescriptor,
        DeviceExt,
    }, BindGroup, Buffer, BufferUsages, Color, CommandEncoder, Device, InstanceFlags, MaintainResult, Queue, ShaderStages, Surface, SurfaceConfiguration, TextureView
    // PipelineCompilationOptions,
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
    Vertex { position: [-1.0, -1.0, 0.0], uv: [0.0, 1.0]},
    Vertex { position: [1.0, 1.0, 0.0], uv: [1.0, 0.0]},
    Vertex { position: [-1.0, 1.0, 0.0], uv: [0.0, 0.0]},
    Vertex { position: [1.0, -1.0, 0.0], uv: [1.0, 1.0]},
];

const INDICES: &[u16] = &[0, 1, 2, 0, 3, 1];


pub struct UIRenderer {

    image_render_pipeline: wgpu::RenderPipeline,
    progress_bar_render_pipeline: wgpu::RenderPipeline,

    images_bind_groups: Vec<(BindGroup, Arc<Mutex<bool>>)>,
    progress_bars_bind_groups: Vec<(BindGroup, Arc<Mutex<bool>>)>,

    rect_vertex_buffer: wgpu::Buffer,
    rect_index_buffer: wgpu::Buffer,
    rect_num_indices: u32,
}

impl UIRenderer {

    pub fn new(
        ui_system: &mut UISystem,
        device: &Device,
        config: &SurfaceConfiguration,
        queue: &Queue,
        screen_aspect: f32,
    ) -> UIRenderer {

        let rect_vertex_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Rect Vertex Buffer"),
                contents: bytemuck::cast_slice(RECT_VERTICES),
                usage: wgpu::BufferUsages::VERTEX,
            }
        );

        let rect_index_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Rect Index Buffer"),
                contents: bytemuck::cast_slice(INDICES),
                usage: wgpu::BufferUsages::INDEX,
            }
        );

        let rect_num_indices = INDICES.len() as u32;

        let progress_bar_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("progerss_bar_shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/progress_bar_shader.wgsl").into())
        });

        let progress_bar_bind_group_layout = device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                label: Some("progress_bar_bind_group_layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::VERTEX,
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
                            sample_type: wgpu::TextureSampleType::Float {
                                filterable: true
                            },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float {
                                filterable: true
                            },
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
                ]
            }
        );

        let progress_bar_pipeline_layout = device.create_pipeline_layout(
            &wgpu::PipelineLayoutDescriptor {
                label: Some("progress bar pipeline layout"),
                bind_group_layouts: &[&progress_bar_bind_group_layout],
                push_constant_ranges: &[],
            }
        );

        let progress_bar_render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("progress bar pipeline"),
            layout: Some(&progress_bar_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &progress_bar_shader,
                entry_point: "vs_main", // 1.
                buffers: &[
                    Vertex::desc(),
                ], // 2.
                // compilation_options: PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState { // 3.
                module: &progress_bar_shader,
                // compilation_options: PipelineCompilationOptions::default(),
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState { // 4.
                    format: config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList, // 1.
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw, // 2.
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None, // 1.
            multisample: wgpu::MultisampleState {
                count: 1, // 2.
                mask: !0, // 3.
                alpha_to_coverage_enabled: false, // 4.
            },
            multiview: None, // 5.
        });

        let image_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("image_shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/image_shader.wgsl").into())
        });

        let image_bind_group_layout = device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                label: Some("image_bind_group_layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::VERTEX,
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
                            sample_type: wgpu::TextureSampleType::Float {
                                filterable: true
                            },
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
                ]
            }
        );

        let image_render_pipeline_layout = device.create_pipeline_layout(
            &wgpu::PipelineLayoutDescriptor {
                label: Some("image pipeline layout"),
                bind_group_layouts: &[&image_bind_group_layout],
                push_constant_ranges: &[],
            }
        );

        let image_render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("image pipeline"),
            layout: Some(&image_render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &image_shader,
                entry_point: "vs_main", // 1.
                buffers: &[
                    Vertex::desc(),
                ], // 2.
                // compilation_options: PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState { // 3.
                module: &image_shader,
                // compilation_options: PipelineCompilationOptions::default(),
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState { // 4.
                    format: config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList, // 1.
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw, // 2.
                cull_mode: Some(wgpu::Face::Back),
                // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                polygon_mode: wgpu::PolygonMode::Fill,
                // Requires Features::DEPTH_CLIP_CONTROL
                unclipped_depth: false,
                // Requires Features::CONSERVATIVE_RASTERIZATION
                conservative: false,
            },
            depth_stencil: None, // 1.
            multisample: wgpu::MultisampleState {
                count: 1, // 2.
                mask: !0, // 3.
                alpha_to_coverage_enabled: false, // 4.
            },
            multiview: None, // 5.
        });

        let ui_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let mut images_bind_groups = Vec::new();
        let mut progress_bars_bind_groups = Vec::new();

        let mut textures_views: HashMap<TextureType, (TextureView, (u32,u32))> = HashMap::new();

        for (_, ui_elem) in &mut ui_system.ui_elements {
            match ui_elem {
                UIElement::Image(ui_image) => {
                    
                    make_texture_view(
                        &mut textures_views,
                        ui_image.get_texture_type(),
                        ui_system.texture_sources.get(ui_image.get_texture_type()).unwrap(),
                        device,
                        queue
                    );

                    let (texture_view, (tex_width, tex_height)) = textures_views
                        .get(ui_image.get_texture_type())
                        .unwrap();

                    let texture_aspect = *tex_width as f32 / *tex_height as f32;

                    let texture_size = Vec2::new(*tex_width as f32, *tex_height as f32);
                    
                    let rect_transform_buffer = device.create_buffer_init(
                        &BufferInitDescriptor {
                            label: Some("rect transform buffer"),
                            contents: bytemuck::cast_slice(&[
                                ui_image
                                    .ui_data
                                    .rect
                                    .get_rect_transform_uniform(
                                        texture_aspect,
                                        screen_aspect
                                    )
                            ]),
                            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
                        }
                    );

                    let image_bind_group = device.create_bind_group(
                        &wgpu::BindGroupDescriptor {
                            layout: &image_bind_group_layout ,
                            entries: &[
                                wgpu::BindGroupEntry {
                                    binding: 0,
                                    resource: rect_transform_buffer.as_entire_binding(),
                                },
                                wgpu::BindGroupEntry {
                                    binding: 1,
                                    resource: wgpu::BindingResource::TextureView(&texture_view),
                                },
                                wgpu::BindGroupEntry {
                                    binding: 2,
                                    resource: wgpu::BindingResource::Sampler(&ui_sampler),
                                }
                            ],
                            label: Some("image_bind_group"),
                        }
                    );

                    ui_image.initialize(
                        texture_size,
                        texture_aspect,
                        rect_transform_buffer
                    );

                    images_bind_groups.push(
                        (image_bind_group, ui_image.get_is_visible_cloned_arc())
                    );
                },
                UIElement::ProgressBar(ui_progress_bar) => {

                    make_texture_view(
                        &mut textures_views,
                        ui_progress_bar.get_texture_type(),
                        ui_system.texture_sources.get(ui_progress_bar.get_texture_type()).unwrap(),
                        device,
                        queue
                    );

                    make_texture_view(
                        &mut textures_views,
                        ui_progress_bar.get_mask_texture_type(),
                        ui_system.texture_sources.get(ui_progress_bar.get_mask_texture_type()).unwrap(),
                        device,
                        queue
                    );

                    let (texture_view, (tex_width, tex_height)) = textures_views
                        .get(ui_progress_bar.get_texture_type())
                        .unwrap();

                    let (mask_texture_view, (mask_width, mask_height)) = textures_views
                        .get(ui_progress_bar.get_mask_texture_type())
                        .unwrap();

                    let texture_aspect = *tex_width as f32 / *tex_height as f32;

                    let texture_size = Vec2::new(*tex_width as f32, *tex_height as f32);


                    let mask_texture_aspect = *mask_width as f32 / *mask_height as f32;

                    let mask_texture_size = Vec2::new(*mask_width as f32, *mask_height as f32);
                    
                    let rect_transform_buffer = device.create_buffer_init(
                        &BufferInitDescriptor {
                            label: Some("rect transform buffer"),
                            contents: bytemuck::cast_slice(&[
                                ui_progress_bar
                                    .ui_data
                                    .rect
                                    .get_rect_transform_uniform(
                                        texture_aspect,
                                        screen_aspect,
                                    )
                                ]),
                            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
                        }
                    );

                    let progress_bar_value_buffer = device.create_buffer_init(
                        &BufferInitDescriptor {
                            label: Some("progress_bar_value_buffer"),
                            contents: bytemuck::cast_slice(&[ui_progress_bar.get_progress_bar_uniform()]),
                            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
                        }
                    );

                    let progress_bar_bind_group = device.create_bind_group(
                        &wgpu::BindGroupDescriptor {
                            layout: &progress_bar_bind_group_layout ,
                            entries: &[
                                wgpu::BindGroupEntry {
                                    binding: 0,
                                    resource: rect_transform_buffer.as_entire_binding(),
                                },
                                wgpu::BindGroupEntry {
                                    binding: 1,
                                    resource: wgpu::BindingResource::TextureView(&texture_view),
                                },
                                wgpu::BindGroupEntry {
                                    binding: 2,
                                    resource: wgpu::BindingResource::TextureView(&mask_texture_view),
                                },
                                wgpu::BindGroupEntry {
                                    binding: 3,
                                    resource: wgpu::BindingResource::Sampler(&ui_sampler),
                                },
                                wgpu::BindGroupEntry {
                                    binding: 4,
                                    resource: progress_bar_value_buffer.as_entire_binding(),
                                },
                            ],
                            label: Some("progress_bar_bind_group"),
                        }
                    );

                    ui_progress_bar.initialize(
                        texture_size,
                        texture_aspect,
                        mask_texture_size,
                        mask_texture_aspect,
                        rect_transform_buffer,
                        progress_bar_value_buffer,
                    );

                    progress_bars_bind_groups.push(
                        (progress_bar_bind_group, ui_progress_bar.get_is_visible_cloned_arc())
                    );
                }
            }
        }

        UIRenderer {
            image_render_pipeline,
            progress_bar_render_pipeline,

            images_bind_groups,
            progress_bars_bind_groups,

            rect_num_indices,
            rect_vertex_buffer,
            rect_index_buffer,
        }
    }


    

    pub fn render_ui(
        &mut self,
        encoder: &mut CommandEncoder,
        view: &TextureView,

    ) {

        for (bindgroup, is_visible) in &self.images_bind_groups {

            if *is_visible.lock().unwrap() {

                let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Render Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Load,
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                });
        
                render_pass.set_pipeline(&self.image_render_pipeline);
                render_pass.set_bind_group(0, bindgroup, &[]);
                render_pass.set_vertex_buffer(0, self.rect_vertex_buffer.slice(..));
                render_pass.set_index_buffer(self.rect_index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                render_pass.draw_indexed(0..self.rect_num_indices, 0, 0..1);
    
            }
        }

        for (bindgroup, is_visible) in &self.progress_bars_bind_groups {

            if *is_visible.lock().unwrap() {

                let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Render Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Load,
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                });
        
                render_pass.set_pipeline(&self.progress_bar_render_pipeline);
                render_pass.set_bind_group(0, bindgroup, &[]);
                render_pass.set_vertex_buffer(0, self.rect_vertex_buffer.slice(..));
                render_pass.set_index_buffer(self.rect_index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                render_pass.draw_indexed(0..self.rect_num_indices, 0, 0..1);
    
            }
        }
    }

}

pub fn make_texture_view(
    textures_views: &mut HashMap<TextureType, (TextureView, (u32,u32))>,
    texture_type: &TextureType,
    texture_source: &[u8],
    device: &Device,
    queue: &Queue
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

    let diffuse_texture = device.create_texture(
        &wgpu::TextureDescriptor {
            label: Some("image texture"),
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],

        }
    );

    queue.write_texture(
        wgpu::ImageCopyTexture {
            texture: &diffuse_texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        &diffuse_rgba,
        wgpu::ImageDataLayout {
            offset: 0,
            bytes_per_row: Some(4 * dimensions.0),
            rows_per_image: Some(dimensions.1),
        },
        texture_size,
    );

    let texture_view = diffuse_texture.create_view(
        &wgpu::TextureViewDescriptor::default()
    );

    textures_views.insert(texture_type.clone(), (texture_view, dimensions));
}