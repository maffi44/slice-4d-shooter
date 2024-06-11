use crate::engine::render::render_data::RenderData;

use winit::window::Window;
use wgpu::{
    rwh::{
        HasDisplayHandle,
        HasWindowHandle
    },
    util::{
        BufferInitDescriptor,
        DeviceExt,
    },
    BindGroup,
    Buffer,
    BufferUsages,
    Color,
    InstanceFlags,
    MaintainResult,
    // PipelineCompilationOptions,
};



#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 3],
}

impl Vertex {
    const ATTRIBS: [wgpu::VertexAttribute; 1] =
        wgpu::vertex_attr_array![0 => Float32x3];

    fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;

        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}


const VERTICES: &[Vertex] = &[
    Vertex { position: [-1.0, 3.0, 0.0]},
    Vertex { position: [3.0, -1.0, 0.0]},
    Vertex { position: [-1.0, -1.0, 0.0]},
];

const INDICES: &[u16] = &[0, 2, 1];


pub struct Renderer {
    surface: wgpu::Surface<'static>,
    
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,

    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    num_indices: u32,
    uniform_bind_group_0: BindGroup,
    uniform_bind_group_1: BindGroup,

    pub dynamic_normal_shapes_buffer: Buffer,
    pub dynamic_stickiness_shapes_buffer: Buffer,
    pub dynamic_negative_shapes_buffer: Buffer,
    pub dynamic_neg_stickiness_shapes_buffer: Buffer,
    pub other_dynamic_data_buffer: Buffer,
    pub spherical_areas_data_buffer: Buffer,
    pub beam_areas_data_buffer: Buffer,
    pub player_forms_data_buffer: Buffer,

    total_time: f64,
    prev_time_instant: Option<web_time::Instant>,
    total_frames_count: u64,
    target_frame_duration: f64,
    // prev_surface_texture: Option<SurfaceTexture>,
    // prev_frame_rendered: Arc<Mutex<bool>>,

}

impl Renderer {
    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        // self.prev_surface_texture = None;
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }
 
    pub async fn new(window: &Window, render_data: &RenderData, target_frame_duration: f64) -> Renderer {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            dx12_shader_compiler: Default::default(),
            flags: InstanceFlags::empty(),
            gles_minor_version: wgpu::Gles3MinorVersion::Automatic,
        });

        log::info!("renderer: wgpu instance init");

        let surface = unsafe { instance.create_surface_unsafe(
            wgpu::SurfaceTargetUnsafe::RawHandle {
                raw_display_handle: window.display_handle().unwrap().as_raw(),
                raw_window_handle: window.window_handle().unwrap().as_raw()
            }
        ).unwrap() };
        log::info!("renderer: wgpu surface init");

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();
        log::info!("renderer: wgpu adapter init");

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    required_features: if cfg!(target_arch = "wasm32") {
                        wgpu::Features::empty()
                    } else {
                        wgpu::Features::default()
                    },
                    // WebGL doesn't support all of wgpu's features, so if
                    // we're building for the web we'll have to disable some.
                    required_limits: if cfg!(target_arch = "wasm32") {
                        wgpu::Limits::downlevel_webgl2_defaults()
                    } else {
                        wgpu::Limits::default()
                    },
                    label: None,
                },
                None, // Trace path
            )
            .await
            .unwrap();
        log::info!("renderer: wgpu device and queue init");

        let surface_caps = surface.get_capabilities(&adapter);
        // Shader code in this tutorial assumes an sRGB surface texture. Using a different
        // one will result all the colors coming out darker. If you want to support non
        // sRGB surfaces, you'll need to account for that when drawing to the frame.
        log::info!("renderer: gpu surface_caps init");

        let surface_format = surface_caps
        .formats
        .iter()
        .copied()
        .find(|f| f.is_srgb())
        .unwrap_or(surface_caps.formats[0]);
        log::info!("renderer: wgpu surface_format init");

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::AutoNoVsync,
            alpha_mode: wgpu::CompositeAlphaMode::default(),
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        log::info!("renderer: wgpu config init");

        surface.configure(&device, &config);
        log::info!("renderer: wgpu surface configurated");

        // for WGSL shaders
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Vertex Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/main_shader.wgsl").into())
        });

        
        // for GLSL shaders
        // let vert_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        //     label: Some("Vertex Shader"),

        //     source: wgpu::ShaderSource::Glsl {
        //         shader: include_str!("shaders/shader_optimized_2.vert").into(),
        //         stage: wgpu::naga::ShaderStage::Vertex,
        //         defines: HashMap::<String,String,BuildHasherDefault<rustc_hash::FxHasher>>::with_hasher(
        //             BuildHasherDefault::default() 
        //         ),

        //     },
        // });
        // let frag_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        //     label: Some("Fragment Shader"),

        //     source: wgpu::ShaderSource::Glsl {
        //         shader: include_str!("shaders/shader_optimized_2.frag").into(),
        //         stage: wgpu::naga::ShaderStage::Fragment,
        //         defines: HashMap::<String,String,BuildHasherDefault<rustc_hash::FxHasher>>::with_hasher(
        //             BuildHasherDefault::default() 
        //         ),
                
        //     },
        // });

        //for Spir-V shaders
        // let vert_shader = unsafe {device.create_shader_module_unchecked(include_spirv!("shaders/vert_2.spv"))};
        // let frag_shader = unsafe {device.create_shader_module_unchecked(include_spirv!("shaders/frag_2.spv"))};

        log::info!("renderer: wgpu shaders init");

        let static_normal_shapes_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("time_buffer"),
            contents: bytemuck::cast_slice(render_data.static_data.static_shapes_data.normal.as_slice()),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let static_stickiness_shapes_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("time_buffer"),
            contents: bytemuck::cast_slice(render_data.static_data.static_shapes_data.stickiness.as_slice()),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let static_negative_shapes_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("time_buffer"),
            contents: bytemuck::cast_slice(render_data.static_data.static_shapes_data.negative.as_slice()),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let static_neg_stickiness_shapes_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("time_buffer"),
            contents: bytemuck::cast_slice(render_data.static_data.static_shapes_data.neg_stickiness.as_slice()),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let other_static_data = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("camera_buffer"),
            contents: bytemuck::cast_slice(&[render_data.static_data.other_static_data]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });


        let dynamic_normal_shapes_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("time_buffer"),
            contents: bytemuck::cast_slice(render_data.dynamic_data.dynamic_shapes_data.normal.as_slice()),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let dynamic_stickiness_shapes_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("time_buffer"),
            contents: bytemuck::cast_slice(render_data.dynamic_data.dynamic_shapes_data.stickiness.as_slice()),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let dynamic_negative_shapes_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("time_buffer"),
            contents: bytemuck::cast_slice(render_data.dynamic_data.dynamic_shapes_data.negative.as_slice()),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let dynamic_neg_stickiness_shapes_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("time_buffer"),
            contents: bytemuck::cast_slice(render_data.dynamic_data.dynamic_shapes_data.neg_stickiness.as_slice()),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let other_dynamic_data_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("camera_buffer"),
            contents: bytemuck::cast_slice(&[render_data.dynamic_data.other_dynamic_data]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let spherical_areas_data_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("camera_buffer"),
            contents: bytemuck::cast_slice(render_data.dynamic_data.spherical_areas_data.as_slice()),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let beam_areas_data_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("camera_buffer"),
            contents: bytemuck::cast_slice(render_data.dynamic_data.beam_areas_data.as_slice()),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let player_forms_data_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("camera_buffer"),
            contents: bytemuck::cast_slice(render_data.dynamic_data.player_forms_data.as_slice()),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });
        log::info!("renderer: wgpu uniform buffers init");

        let uniform_bind_group_layout_0 =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 3,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 4,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 5,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 6,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 7,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 8,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 9,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }
                ],
                label: Some("uniform_bind_group_layout_0"),
            }
        );

        let uniform_bind_group_layout_1 =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }
                ],
                label: Some("uniform_bind_group_layout_1"),
            }
        );
        log::info!("renderer: wgpu uniform_bind_group_layout_0 init");

        let uniform_bind_group_0 = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &uniform_bind_group_layout_0,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: static_normal_shapes_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: static_negative_shapes_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: static_stickiness_shapes_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: static_neg_stickiness_shapes_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: other_static_data.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: dynamic_normal_shapes_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 6,
                    resource: dynamic_negative_shapes_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 7,
                    resource: dynamic_stickiness_shapes_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 8,
                    resource: dynamic_neg_stickiness_shapes_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 9,
                    resource: other_dynamic_data_buffer.as_entire_binding(),
                }
            ],
            
            label: Some("shader_unforms_and_storge_bind_group_0"),
        });

        let uniform_bind_group_1 = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &uniform_bind_group_layout_1,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: spherical_areas_data_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: beam_areas_data_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: player_forms_data_buffer.as_entire_binding(),
                },
            ],
            
            label: Some("shader_unforms_and_storge_bind_group_0"),
        });

        log::info!("renderer: wgpu uniform_bind_group_0 init");

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[&uniform_bind_group_layout_0, &uniform_bind_group_layout_1],
            push_constant_ranges: &[],
        });

        log::info!("renderer: wgpu render_pipeline_layout init");

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main", // 1.
                buffers: &[
                    Vertex::desc(),
                ], // 2.
                // compilation_options: PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState { // 3.
                module: &shader,
                // compilation_options: PipelineCompilationOptions::default(),
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState { // 4.
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
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

        log::info!("renderer: wgpu render_pipeline init");
        
        let vertex_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(VERTICES),
                usage: wgpu::BufferUsages::VERTEX,
            }
        );

        log::info!("renderer: wgpu vertex_buffer init");

        let index_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: bytemuck::cast_slice(INDICES),
                usage: wgpu::BufferUsages::INDEX,
            }
        );

        log::info!("renderer: wgpu index_buffer init");

        let num_indices = INDICES.len() as u32;

        

        Renderer {
            surface,
            device,
            queue,
            config, 
            size,
            render_pipeline,
            num_indices,
            vertex_buffer,
            index_buffer,
            uniform_bind_group_0,
            uniform_bind_group_1,

            dynamic_normal_shapes_buffer,
            dynamic_stickiness_shapes_buffer,
            dynamic_negative_shapes_buffer,
            dynamic_neg_stickiness_shapes_buffer,
            other_dynamic_data_buffer,
            spherical_areas_data_buffer,
            beam_areas_data_buffer,
            player_forms_data_buffer,

            total_frames_count: 0u64,
            total_time: 0.0,
            prev_time_instant: None,
            target_frame_duration,
            // prev_surface_texture: None,
            // prev_frame_rendered: Arc::new(Mutex::new(true)),
        }
    }


    pub fn render(&mut self, /*window: &Window*/) -> Result<(), wgpu::SurfaceError> {

        // let instatnt_full = web_time::Instant::now();

        match self.device.poll(wgpu::MaintainBase::Poll) {
            MaintainResult::Ok => {return Ok(());}
            MaintainResult::SubmissionQueueEmpty => {},
        }

        if let Some(instant) = self.prev_time_instant {
            let current_frame_time = instant.elapsed().as_secs_f64();

            if current_frame_time < self.target_frame_duration - 0.001 {
                return Ok(());
            }

            // self.total_time += current_frame_time;
            // self.total_frames_count += 1;

            // println!(
            //     "AV DT {}, CUR DT: {}",
            //     self.total_time / (self.total_frames_count) as f64,
            //     current_frame_time,
            // );
        }

        self.prev_time_instant = Some(web_time::Instant::now());
        
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(
                            Color {
                                r: 0.0,
                                g: 1.0,
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
            });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.uniform_bind_group_0, &[]);
            render_pass.set_bind_group(1, &self.uniform_bind_group_1, &[]);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(0..self.num_indices, 0, 0..1);
        }

        // let istts = web_time::Instant::now();
        self.queue.submit(std::iter::once(encoder.finish()));
        // println!("submit time: {}",istts.elapsed().as_secs_f64());

        // window.pre_present_notify();
        
        // let istts = web_time::Instant::now();
        output.present();
        // println!("output time: {}",istts.elapsed().as_secs_f64());


        // println!("RENDER TIME in RENDERER {}", instatnt_full.elapsed().as_secs_f64());
        Ok(())
    }

}