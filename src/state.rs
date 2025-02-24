use bytemuck::Zeroable;
use glam::Mat4;
use wgpu::util::DeviceExt;

use crate::types::View;

pub struct State {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub target: wgpu::Texture,
    pub output_staging_buffer: wgpu::Buffer,
    pub scale_factor: f32,

    pub render_pipeline: wgpu::RenderPipeline,
    pub view: View,
    pub view_buffer: wgpu::Buffer,
    pub view_bind_group: wgpu::BindGroup,
    pub sky_texture: wgpu::Texture,
    pub sky_sampler: wgpu::Sampler,
    pub sky_bind_group: wgpu::BindGroup,
}

impl State {
    pub async fn new(instance: &wgpu::Instance, texture_data: &Vec<u8>, width: u32, height: u32) -> Self {
        let power_pref = wgpu::PowerPreference::default();
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: power_pref,
                force_fallback_adapter: false,
                compatible_surface: None,
            })
            .await
            .expect("failed to find an appropriate adapter.");

        let features = wgpu::Features::empty();
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    required_features: features,
                    required_limits: Default::default(),
                    memory_hints: Default::default(),
                },
                None,
            )
            .await
            .expect("failed to create device.");

        let target = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("render_target"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[wgpu::TextureFormat::Rgba8UnormSrgb],
        });

        let output_staging_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("output_staging_buffer"),
            size: texture_data.capacity() as u64,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        let scale_factor: f32 = 1.0;

        let pathtrace_shader = device.create_shader_module(wgpu::include_wgsl!("pathtrace.wgsl"));

        let view = View {
            resolution: [width, height],
            camera: Mat4::IDENTITY.to_cols_array(),
            focal_length: 1.5,
            ..View::zeroed()
        };

        let view_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("view_buffer"),
            contents: bytemuck::cast_slice(&[view]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let view_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("view_layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: std::num::NonZero::new(std::mem::size_of::<View>() as u64),
                },
                count: None,
            }],
        });

        let view_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("view_bind_group"),
            layout: &view_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: view_buffer.as_entire_binding(),
            }],
        });

        let (sky_image_data, sky_im_width, sky_im_height) = {
            let right = image::load_from_memory(include_bytes!("../images/sky1/right.png"))
                .unwrap()
                .to_rgba8();
            let left = image::load_from_memory(include_bytes!("../images/sky1/left.png"))
                .unwrap()
                .to_rgba8();
            let top = image::load_from_memory(include_bytes!("../images/sky1/top.png"))
                .unwrap()
                .to_rgba8();
            let bottom = image::load_from_memory(include_bytes!("../images/sky1/bottom.png"))
                .unwrap()
                .to_rgba8();
            let front = image::load_from_memory(include_bytes!("../images/sky1/front.png"))
                .unwrap()
                .to_rgba8();
            let back = image::load_from_memory(include_bytes!("../images/sky1/back.png"))
                .unwrap()
                .to_rgba8();

            let (sky_im_width, sky_im_height) = right.dimensions();
            let mut data: Vec<u8> = Vec::new();

            data.extend(right.as_raw());
            data.extend(left.as_raw());
            data.extend(top.as_raw());
            data.extend(bottom.as_raw());
            data.extend(front.as_raw());
            data.extend(back.as_raw());

            (data, sky_im_width, sky_im_height)
        };

        let sky_image_size = wgpu::Extent3d {
            width: sky_im_width,
            height: sky_im_height,
            depth_or_array_layers: 6,
        };

        let sky_texture = device.create_texture_with_data(
            &queue,
            &wgpu::TextureDescriptor {
                label: Some("sky_texture"),
                size: sky_image_size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8Unorm,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            },
            wgpu::util::TextureDataOrder::LayerMajor,
            &sky_image_data,
        );

        let sky_texture_view = sky_texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some("sky_texture_view"),
            dimension: Some(wgpu::TextureViewDimension::Cube),
            ..Default::default()
        });

        let sky_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("sky_texture_sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        let sky_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("sky_bind_group_layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::Cube,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let sky_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("sky_bind_group"),
            layout: &sky_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&sky_texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sky_sampler),
                },
            ],
        });

        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("render_pipeline_layout"),
            bind_group_layouts: &[&view_bind_group_layout, &sky_bind_group_layout],
            push_constant_ranges: &[],
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("render_pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &pathtrace_shader,
                entry_point: Some("vertex"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &pathtrace_shader,
                entry_point: Some("fragment"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: target.format(),
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                conservative: false,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });

        Self {
            device,
            queue,
            target,
            output_staging_buffer,
            scale_factor,
            view,
            view_buffer,
            view_bind_group,
            render_pipeline,
            sky_texture,
            sky_sampler,
            sky_bind_group,
        }
    }
}
