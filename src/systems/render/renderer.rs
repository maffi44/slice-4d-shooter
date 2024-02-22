use crate::systems::world::World;

use super::render_data::{
    CameraUniform,
    TimeUniform,
    AllShapesArraysMetadata, StaticShapesArraysUniformData,
};
use winit::window::Window;
use wgpu::{
    rwh::{HasDisplayHandle, HasRawDisplayHandle, HasRawWindowHandle, HasWindowHandle}, util::{
        BufferInitDescriptor, DeviceExt, RenderEncoder
    }, BindGroup, Buffer, BufferUsages, Color, InstanceFlags
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
    pub camera_buffer: Buffer,
    pub time_buffer: Buffer,
    uniform_bind_group_0: BindGroup,
    uniform_bind_group_1: BindGroup,
    // time: std::time::SystemTime,
    // already_rendered: Arc<Mutex<bool>>,
}

impl Renderer {
    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    pub fn render(&mut self, window: &Window) -> Result<(), wgpu::SurfaceError> {
        // self.device.poll(wgpu::MaintainBase::Poll);
        // if *(self.already_rendered.lock().unwrap()) == true {
        //     *(self.already_rendered.lock().unwrap()) = false
        // } else {
        //     return Ok(());
        // }
        
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
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16); // 1.
            render_pass.draw_indexed(0..self.num_indices, 0, 0..1); // 2.
        }
        // submit will accept anything that implements IntoIter
        self.queue.submit(std::iter::once(encoder.finish()));

        // let instant = web_time::Instant::now();
        // self.queue.on_submitted_work_done(move || {
        //     log::error!("RENDER DONE with {}", instant.elapsed().as_secs_f32())
        // });

        window.pre_present_notify();

        output.present();

        Ok(())
    }


    pub async fn new(window: &Window, world: &World) -> Renderer {
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
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        log::info!("renderer: wgpu adapter init");

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    required_features: wgpu::Features::empty(),
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
            desired_maximum_frame_latency: 3,
        };

        log::info!("renderer: wgpu config init");

        surface.configure(&device, &config);

        log::info!("renderer: wgpu surface configurated");
        
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),

            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/shader.wgsl").into()),
        });

        log::info!("renderer: wgpu shader init");

        let init_camera_uniform = CameraUniform {
            cam_pos: [0.0, 0.0, 0.0, 0.0],
            cam_rot: [
                1.0, 0.0, 0.0, 0.0,
                0.0, 1.0, 0.0, 0.0,
                0.0, 0.0, 1.0, 0.0,
                0.0, 0.0, 0.0, 1.0,
            ],
            aspect: [1.0, 0.0, 0.0, 0.0],
        };

        log::info!("renderer: init_camera_uniform init");
        
        let init_time = TimeUniform::new_zero();

        log::info!("renderer: init_time init");

        let shapes_array_data = StaticShapesArraysUniformData::new(world);

        log::info!("renderer: shapes_array_data init");

        let camera_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("camera_buffer"),
            contents: bytemuck::cast_slice(&[init_camera_uniform]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let time_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("time_buffer"),
            contents: bytemuck::cast_slice(&[init_time.time]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let shapes_array_metadata_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("time_buffer"),
            contents: bytemuck::cast_slice(&[shapes_array_data.metadata]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

//         @group(0) @binding(3) var<uniform> cubes: array<Shape, 512>;
// @group(0) @binding(4) var<uniform> s_cubes: array<StickinessShape, 512>;
// @group(0) @binding(5) var<uniform> neg_cubes: array<NegShape, 512>;
// @group(0) @binding(6) var<uniform> s_neg_cubes: array<StickinessNegShape, 512>;

// @group(0) @binding(7) var<uniform> spheres: array<Shape, 512>;
// @group(0) @binding(8) var<uniform> s_spheres: array<StickinessShape, 512>;
// @group(0) @binding(9) var<uniform> neg_spheres: array<NegShape, 512>;
// @group(0) @binding(10) var<uniform> s_neg_spheres: array<StickinessNegShape, 512>;

// @group(0) @binding(11) var<uniform> inf_cubes: array<Shape, 512>;
// @group(1) @binding(0) var<uniform> s_inf_cubes: array<StickinessShape, 512>;
// @group(1) @binding(1) var<uniform> neg_inf_cubes: array<NegShape, 512>;
// @group(1) @binding(2) var<uniform> s_neg_inf_cubes: array<StickinessNegShape, 512>;

// @group(1) @binding(3) var<uniform> sph_cubes: array<Shape, 512>;
// @group(1) @binding(4) var<uniform> s_sph_cubes: array<StickinessShape, 512>;
// @group(1) @binding(5) var<uniform> neg_sph_cubes: array<NegShape, 512>;
// @group(1) @binding(6) var<uniform> s_neg_sph_cubes: array<StickinessNegShape, 512>;

        log::info!("Pre normal_cubes_buffer");
        
        let normal_cubes_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("time_buffer"),
            contents: bytemuck::cast_slice(&[shapes_array_data.cubes.normal]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let stickiness_cubes_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("time_buffer"),
            contents: bytemuck::cast_slice(&[shapes_array_data.cubes.stickiness]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let negative_cubes_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("time_buffer"),
            contents: bytemuck::cast_slice(&[shapes_array_data.cubes.negative]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let neg_stickiness_cubes_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("time_buffer"),
            contents: bytemuck::cast_slice(&[shapes_array_data.cubes.neg_stickiness]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        
        let normal_spheres_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("time_buffer"),
            contents: bytemuck::cast_slice(&[shapes_array_data.spheres.normal]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let stickiness_spheres_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("time_buffer"),
            contents: bytemuck::cast_slice(&[shapes_array_data.spheres.stickiness]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let negative_spheres_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("time_buffer"),
            contents: bytemuck::cast_slice(&[shapes_array_data.spheres.negative]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let neg_stickiness_spheres_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("time_buffer"),
            contents: bytemuck::cast_slice(&[shapes_array_data.spheres.neg_stickiness]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });


        let normal_inf_w_cubes_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("time_buffer"),
            contents: bytemuck::cast_slice(&[shapes_array_data.inf_w_cubes.normal]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let stickiness_inf_w_cubes_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("time_buffer"),
            contents: bytemuck::cast_slice(&[shapes_array_data.inf_w_cubes.stickiness]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let negative_inf_w_cubes_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("time_buffer"),
            contents: bytemuck::cast_slice(&[shapes_array_data.inf_w_cubes.negative]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let neg_stickiness_inf_w_cubes_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("time_buffer"),
            contents: bytemuck::cast_slice(&[shapes_array_data.inf_w_cubes.neg_stickiness]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });


        let normal_sph_cubes_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("time_buffer"),
            contents: bytemuck::cast_slice(&[shapes_array_data.sph_cubes.normal]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let stickiness_sph_cubes_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("time_buffer"),
            contents: bytemuck::cast_slice(&[shapes_array_data.sph_cubes.stickiness]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let negative_sph_cubes_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("time_buffer"),
            contents: bytemuck::cast_slice(&[shapes_array_data.sph_cubes.negative]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let neg_stickiness_sph_cubes_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("time_buffer"),
            contents: bytemuck::cast_slice(&[shapes_array_data.sph_cubes.neg_stickiness]),
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
                    },

                    wgpu::BindGroupLayoutEntry {
                        binding: 10,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
                label: Some("uniform_bind_group_layout_0"),
            }
        );

        log::info!("renderer: wgpu uniform_bind_group_layout_0 init");


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

                ],
                label: Some("uniform_bind_group_layout_1"),
            }
        );

        log::info!("renderer: wgpu uniform_bind_group_layout_1 init");
        
        let uniform_bind_group_0 = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &uniform_bind_group_layout_0,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: time_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: shapes_array_metadata_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: normal_cubes_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: stickiness_cubes_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: negative_cubes_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 6,
                    resource: neg_stickiness_cubes_buffer.as_entire_binding(),
                },

                wgpu::BindGroupEntry {
                    binding: 7,
                    resource: normal_spheres_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 8,
                    resource: stickiness_spheres_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 9,
                    resource: negative_spheres_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 10,
                    resource: neg_stickiness_spheres_buffer.as_entire_binding(),
                },
                ],
            
            label: Some("shader_unforms_and_storge_bind_group_0"),
        });

        log::info!("renderer: wgpu uniform_bind_group_0 init");

        let uniform_bind_group_1 = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &uniform_bind_group_layout_1,
            entries: &[

                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: normal_inf_w_cubes_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: stickiness_inf_w_cubes_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: negative_inf_w_cubes_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: neg_stickiness_inf_w_cubes_buffer.as_entire_binding(),
                },

                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: normal_sph_cubes_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: stickiness_sph_cubes_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 6,
                    resource: negative_sph_cubes_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 7,
                    resource: neg_stickiness_sph_cubes_buffer.as_entire_binding(),
                },
                ],
            
            label: Some("shader_unforms_and_storge_bind_group_1"),
        });

        log::info!("renderer: wgpu uniform_bind_group_1 init");

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
            },
            fragment: Some(wgpu::FragmentState { // 3.
                module: &shader,
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
            camera_buffer,
            time_buffer,
            uniform_bind_group_0,
            uniform_bind_group_1,
            // time: std::time::SystemTime::now(),
            // already_rendered: Arc::new(Mutex::new(true)),
        }
    }
}