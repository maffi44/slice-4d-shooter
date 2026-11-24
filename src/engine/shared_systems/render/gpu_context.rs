// Slice 4D Shooter - the first multiplayer shooter set in 4D space
// Copyright (C) 2023-2026  Timofei Molokov



use image::{GenericImageView, ImageBuffer, Rgba};
use winit::dpi::PhysicalSize;
use wgpu::{
    Backend, BackendOptions, ExperimentalFeatures, Extent3d, InstanceFlags, MemoryBudgetThresholds,
    TexelCopyBufferLayout, TexelCopyTextureInfoBase,
    util::{
        BufferInitDescriptor,
        DeviceExt,
    }
};

use super::{presentation::Presentation, Vertex, INDICES, INDICES_MINIMAP, VERTICES, VERTICES_MINIMAP};


/// The format of every world's output texture (and of the world pipelines' color target).
/// Fixed, so a descriptor's pipelines work for any world (root or nested), independent of the
/// window surface's format. sRGB, so colours stay identical: the upscale pass decodes it when
/// sampling and the surface encodes again, exactly like the former surface-format target texture.
pub const WORLD_OUTPUT_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8UnormSrgb;


/// The shared wgpu objects needed to create GPU resources: device, queue, the layouts and
/// resources shared by all worlds. Part of the Engine Shared Systems (`shared.gpu`, `None` when headless).
/// Cheap to clone (wgpu handles), so it can be sent to a loading thread.
#[derive(Clone)]
pub struct GpuContext
{
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub output_format: wgpu::TextureFormat,

    pub raymarch_bind_group_layout_0: wgpu::BindGroupLayout,
    pub raymarch_bind_group_layout_1: wgpu::BindGroupLayout,
    pub raymarch_pipeline_layout: wgpu::PipelineLayout,
    pub raymarch_vertex_layout: wgpu::VertexBufferLayout<'static>,

    pub upscale_bind_group_layout: wgpu::BindGroupLayout,
    pub upscale_sampler: wgpu::Sampler,

    pub sky_box_view: wgpu::TextureView,
    pub sky_box_sampler: wgpu::Sampler,

    pub fullscreen_vertex_buffer: wgpu::Buffer,
    pub fullscreen_index_buffer: wgpu::Buffer,
    pub fullscreen_num_indices: u32,
    pub minimap_vertex_buffer: wgpu::Buffer,
    pub minimap_index_buffer: wgpu::Buffer,
    pub minimap_num_indices: u32,
}


impl GpuContext
{
    /// Driver discovery can block, so the loader calls this on a worker.
    pub fn create_instance(specific_backend: Option<Backend>) -> wgpu::Instance
    {
        // Keep background pipeline preparation: GL's shared-context timeout is incompatible
        // with long concurrent driver calls. See issues/programming/opengl-context-contention/.
        let allowed_backends = wgpu::Backends::VULKAN
            | wgpu::Backends::DX12
            | wgpu::Backends::METAL;
        let backends = match specific_backend {
            Some(backend) => wgpu::Backends::from(backend),
            None => allowed_backends,
        };
        assert!(allowed_backends.contains(backends), "Unsupported graphics backend: {:?}", specific_backend);

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends,
            flags: InstanceFlags::empty(),
            backend_options: BackendOptions::from_env_or_default(),
            memory_budget_thresholds: MemoryBudgetThresholds::default(),
            // only the GL backend needs a display handle; it is disabled
            display: None,
        });

        log::info!("gpu context: wgpu instance init");

        instance
    }

    /// The surface must already have been created on the event-loop thread (Metal requires it).
    /// Run even this async function on a blocking worker: native driver calls can block while polled.
    pub(in crate::engine) async fn new(
        instance: &wgpu::Instance,
        surface: wgpu::Surface<'static>,
        size: PhysicalSize<u32>,
        on_device_lost: impl Fn(wgpu::DeviceLostReason, String) + Send + 'static,
    ) -> (GpuContext, Presentation)
    {
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
                apply_limit_buckets: false,
            })
            .await
            .unwrap();

        let adapter_info = adapter.get_info();
        log::info!("gpu context: wgpu adapter init: {} ({:?})", adapter_info.name, adapter_info.backend);

        let mut res = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    required_features: wgpu::Features::default(),
                    required_limits: wgpu::Limits::default(),
                    memory_hints: wgpu::MemoryHints::Performance,
                    trace: wgpu::Trace::Off,
                    experimental_features: ExperimentalFeatures::disabled(),
                }
            )
            .await;

        if res.is_err()
        {
            res = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::downlevel_defaults(),
                    memory_hints: wgpu::MemoryHints::Performance,
                    trace: wgpu::Trace::Off,
                    experimental_features: ExperimentalFeatures::disabled(),
                },
            )
            .await;
        }

        let (device, queue) = res.unwrap();
        device.set_device_lost_callback(on_device_lost);

        log::info!("gpu context: wgpu device and queue init");

        let surface_caps = surface.get_capabilities(&adapter);

        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: wgpu::PresentMode::AutoNoVsync,
            alpha_mode: wgpu::CompositeAlphaMode::default(),
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
            color_space: wgpu::SurfaceColorSpace::Auto,
        };

        let uniform_buffer_entry = |binding: u32| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        };

        let raymarch_bind_group_layout_0 =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    uniform_buffer_entry(0),
                    uniform_buffer_entry(1),
                    uniform_buffer_entry(2),
                    uniform_buffer_entry(3),
                    uniform_buffer_entry(4),
                    uniform_buffer_entry(5),
                    uniform_buffer_entry(6),
                    uniform_buffer_entry(7),
                ],
                label: Some("uniform_bind_group_layout_0"),
            }
        );

        let raymarch_bind_group_layout_1 =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    uniform_buffer_entry(0),
                    uniform_buffer_entry(1),
                    uniform_buffer_entry(2),
                    wgpu::BindGroupLayoutEntry {
                        binding: 3,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 4,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::Cube,
                            multisampled: false,
                        },
                        count: None,
                    }
                ],
                label: Some("uniform_bind_group_layout_1"),
            }
        );

        let raymarch_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("main shader render pipeline layout"),
                bind_group_layouts: &[Some(&raymarch_bind_group_layout_0), Some(&raymarch_bind_group_layout_1)],
                immediate_size: 0,
        });

        let upscale_bind_group_layout = device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                label: Some("upscale_bind_group_layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
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
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                        count: None,
                    },
                ]
            }
        );

        let upscale_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });

        let sky_box_view = load_cube_texture(&device, &queue);

        let sky_box_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });

        let fullscreen_vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("main shader vertex buffer"),
            contents: bytemuck::cast_slice(VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let fullscreen_index_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("main shader index buffer"),
            contents: bytemuck::cast_slice(INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });

        let minimap_vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("minimap vertex buffer"),
            contents: bytemuck::cast_slice(VERTICES_MINIMAP),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let minimap_index_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("minimap index buffer"),
            contents: bytemuck::cast_slice(INDICES_MINIMAP),
            usage: wgpu::BufferUsages::INDEX,
        });

        log::info!("gpu context: shared layouts and resources init");

        let gpu = GpuContext {
            device,
            queue,
            output_format: WORLD_OUTPUT_FORMAT,

            raymarch_bind_group_layout_0,
            raymarch_bind_group_layout_1,
            raymarch_pipeline_layout,
            raymarch_vertex_layout: Vertex::desc(),

            upscale_bind_group_layout,
            upscale_sampler,

            sky_box_view,
            sky_box_sampler,

            fullscreen_vertex_buffer,
            fullscreen_index_buffer,
            fullscreen_num_indices: INDICES.len() as u32,
            minimap_vertex_buffer,
            minimap_index_buffer,
            minimap_num_indices: INDICES_MINIMAP.len() as u32,
        };

        (gpu, Presentation::new(surface, surface_config, adapter, size))
    }


    /// A bind group for the upscale pass that samples `view`.
    pub fn create_upscale_bind_group(&self, view: &wgpu::TextureView) -> wgpu::BindGroup
    {
        self.device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                label: Some("upscale render bind group"),
                layout: &self.upscale_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(view)
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&self.upscale_sampler)
                    }
                ]
            }
        )
    }


    /// Uploads an RGBA8 sRGB image as a 2D texture.
    pub fn create_texture_from_image(&self, image_bytes: &[u8], label: &str) -> wgpu::TextureView
    {
        let (rgba, (width, height)) = decode_image(image_bytes);

        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some(label),
            size: Extent3d { width, height, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        self.queue.write_texture(
            TexelCopyTextureInfoBase {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &rgba,
            TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * width),
                rows_per_image: Some(height),
            },
            Extent3d { width, height, depth_or_array_layers: 1 },
        );

        texture.create_view(&wgpu::TextureViewDescriptor::default())
    }
}


fn decode_image(texture_buffer: &[u8]) -> (ImageBuffer<Rgba<u8>, Vec<u8>>, (u32,u32)) {
    let img = image::load_from_memory(texture_buffer).unwrap();
    let rgba = img.to_rgba8();
    let (width, height) = img.dimensions();

    (rgba, (width, height))
}


fn load_cube_texture(device: &wgpu::Device, queue: &wgpu::Queue) -> wgpu::TextureView {
    let faces = [
        include_bytes!("../../../assets/sky_boxes/star_sky/star_sky_right1.png").as_slice(),
        include_bytes!("../../../assets/sky_boxes/star_sky/star_sky_left2.png").as_slice(),
        include_bytes!("../../../assets/sky_boxes/star_sky/star_sky_top3.png").as_slice(),
        include_bytes!("../../../assets/sky_boxes/star_sky/star_sky_bottom4.png").as_slice(),
        include_bytes!("../../../assets/sky_boxes/star_sky/star_sky_front5.png").as_slice(),
        include_bytes!("../../../assets/sky_boxes/star_sky/star_sky_back6.png").as_slice(),
    ];

    let mut textures_data = Vec::new();

    let mut dimensions = (0_u32, 0_u32);
    for face in &faces {
        let (texture_rgba, dims) = decode_image(face);
        textures_data.push(texture_rgba);
        dimensions = dims;
    }

    let cube_texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Cube Texture"),
        view_formats: &[],
        size: wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 6,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
    });

    let texture_size = Extent3d {
        width: dimensions.0,
        height: dimensions.1,
        depth_or_array_layers: 1,
    };

    for (i, data) in textures_data.iter().enumerate() {

        queue.write_texture(
            TexelCopyTextureInfoBase {
                texture: &cube_texture,
                mip_level: 0,
                origin: wgpu::Origin3d {
                    x: 0,
                    y: 0,
                    z: i as u32,
                },
                aspect: wgpu::TextureAspect::All,
            },
            &data,
            TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * dimensions.0),
                rows_per_image: Some(dimensions.1),
            },
            texture_size,
        );
    }

    cube_texture.create_view(&wgpu::TextureViewDescriptor {
        dimension: Some(wgpu::TextureViewDimension::Cube),
        ..Default::default()
    })
}
