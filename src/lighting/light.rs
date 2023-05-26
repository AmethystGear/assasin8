use std::{mem, num::NonZeroU32};

use bevy::{
    core::{Pod, Zeroable},
    prelude::*,
    window::PrimaryWindow,
};
use futures::executor::block_on;
use wgpu::{util::DeviceExt, ColorWrites, FrontFace};

use super::types::{LightData, OcclusionData};

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct Vertex {
    position: [f32; 3],
    tex_coords: [f32; 2],
}

impl Vertex {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2, // NEW!
                },
            ],
        }
    }
}

#[derive(Resource)]
pub struct WGPUState {
    queue: wgpu::Queue,
    device: wgpu::Device,
    shadow_mask_pipeline: wgpu::RenderPipeline,
    add_light_pipeline: wgpu::RenderPipeline,
    light_bind_group_layout: wgpu::BindGroupLayout,
}

const TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8UnormSrgb;

fn make_pipeline(
    name: &str,
    device: &wgpu::Device,
    shader: &wgpu::ShaderModule,
    pipeline_layout: &wgpu::PipelineLayout,
    writes: wgpu::ColorWrites,
) -> wgpu::RenderPipeline {
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some(name),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: "vertex",
            buffers: &[Vertex::desc()],
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: "fragment",
            targets: &[Some(wgpu::ColorTargetState {
                format: TEXTURE_FORMAT,
                blend: Some(wgpu::BlendState {
                    color: wgpu::BlendComponent::REPLACE,
                    alpha: wgpu::BlendComponent::REPLACE,
                }),
                write_mask: writes,
            })],
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: FrontFace::Cw,
            ..Default::default()
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
    })
}

impl Default for WGPUState {
    fn default() -> Self {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::default());
        let adapter = block_on(instance.request_adapter(&wgpu::RequestAdapterOptionsBase {
            power_preference: wgpu::PowerPreference::default(),
            force_fallback_adapter: false,
            compatible_surface: None,
        }))
        .expect("couldn't get adapter");

        let (device, queue) = block_on(adapter.request_device(&Default::default(), None))
            .expect("couldn't get device and queue");

        let shadow_mask = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("shadow_mask_shader"),
            source: wgpu::ShaderSource::Wgsl(
                include_str!("../../assets/shaders/shadow_mask.wgsl").into(),
            ),
        });

        let add_light = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("add_light_shader"),
            source: wgpu::ShaderSource::Wgsl(
                include_str!("../../assets/shaders/add_light.wgsl").into(),
            ),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("shadow_mask_pipeline_layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });

        let shadow_mask_pipeline = make_pipeline(
            "shadow_mask_pipeline",
            &device,
            &shadow_mask,
            &pipeline_layout,
            ColorWrites::RED,
        );

        let light_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
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
                ],
                label: Some("light_bind_group_layout"),
            });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("light_pipeline_layout"),
            bind_group_layouts: &[&light_bind_group_layout],
            push_constant_ranges: &[],
        });

        let add_light_pipeline = make_pipeline(
            "add_light_pipeline",
            &device,
            &add_light,
            &pipeline_layout,
            ColorWrites::ALL,
        );

        Self {
            queue,
            device,
            shadow_mask_pipeline,
            add_light_pipeline,
            light_bind_group_layout,
        }
    }
}

fn get_texture_desc(width: u32, height: u32) -> wgpu::TextureDescriptor<'static> {
    wgpu::TextureDescriptor {
        size: wgpu::Extent3d {
            width: width,
            height: height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: TEXTURE_FORMAT,
        usage: wgpu::TextureUsages::COPY_SRC
            | wgpu::TextureUsages::RENDER_ATTACHMENT
            | wgpu::TextureUsages::TEXTURE_BINDING,
        label: None,
        view_formats: &[],
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct LightUniform {
    data: [f32; 4],
    last: [f32; 4],
}

fn max(a: f32, b: f32) -> f32 {
    if a.is_nan() {
        return b;
    }
    if b.is_nan() {
        return a;
    }
    if a > b {
        a
    } else {
        b
    }
}

fn min(a: f32, b: f32) -> f32 {
    if a.is_nan() {
        return b;
    }
    if b.is_nan() {
        return a;
    }
    if a < b {
        a
    } else {
        b
    }
}

fn max_vec2(a: Vec2, b: Vec2) -> Vec2 {
    Vec2::new(max(a.x, b.x), max(a.y, b.y))
}

fn min_vec2(a: Vec2, b: Vec2) -> Vec2 {
    Vec2::new(min(a.x, b.x), min(a.y, b.y))
}

fn intersect_aabb(ray_origin: Vec2, ray_dir: Vec2, box_min: Vec2, box_max: Vec2) -> bool {
    let t_min = (box_min - ray_origin) / ray_dir;
    let t_max = (box_max - ray_origin) / ray_dir;
    let t1 = min_vec2(t_min, t_max);
    let t2 = max_vec2(t_min, t_max);
    let t_near = max(t1.x, t1.y);
    let t_far = min(t2.x, t2.y);
    t_near < t_far
}

pub fn get_lightmap(
    window: Query<&Window, With<PrimaryWindow>>,
    lights: &Vec<LightData>,
    occlusions: &Vec<OcclusionData>,
    camera_transform: &Transform,
    wgpu_state: Res<WGPUState>,
) {
    let window = window.get_single().expect("No primary window");
    let width = window.width() as u32;
    let height = window.height() as u32;
    println!("{}, {}", width, height);

    let window_extents = Vec3::new(window.width(), window.height(), 0.0);

    let bottom_left = *camera_transform * (Vec3::ZERO - window_extents * 0.5);
    let top_right = *camera_transform * (Vec3::ZERO + window_extents * 0.5);
    let world_window_size = top_right - bottom_left;
    let world_window_size = Vec2::new(world_window_size.x, world_window_size.y);
    let bottom_left = Vec2::new(bottom_left.x, bottom_left.y);
    let top_right = Vec2::new(top_right.x, top_right.y);
    let camera_pos = Vec2::new(
        camera_transform.translation.x,
        camera_transform.translation.y,
    );

    let texture_desc = get_texture_desc(width, height);
    let mut texture = wgpu_state.device.create_texture(&texture_desc);
    let texture_sampler = wgpu_state.device.create_sampler(&wgpu::SamplerDescriptor {
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Nearest,
        mipmap_filter: wgpu::FilterMode::Nearest,
        ..Default::default()
    });
    let mut encoder = wgpu_state
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

    let u32_size = std::mem::size_of::<u32>() as u32;

    let output_buffer_size = (u32_size * width * height) as wgpu::BufferAddress;
    let output_buffer_desc = wgpu::BufferDescriptor {
        size: output_buffer_size,
        usage: wgpu::BufferUsages::COPY_DST
                // this tells wpgu that we want to read this buffer from the cpu
                | wgpu::BufferUsages::MAP_READ,
        label: None,
        mapped_at_creation: false,
    };
    let output_buffer = wgpu_state.device.create_buffer(&output_buffer_desc);

    let mut op = wgpu::LoadOp::Clear(wgpu::Color {
        r: 1.0,
        g: 0.0,
        b: 0.0,
        a: 0.0,
    });
    for i in 0..lights.len() {
        let light = &lights[i];
        let mut verts = vec![];
        for occlusion in occlusions {
            let d1 = occlusion.start - light.pos;
            let d2 = occlusion.end - light.pos;
            if intersect_aabb(occlusion.start, d1, bottom_left, top_right)
                || intersect_aabb(occlusion.end, d2, bottom_left, top_right)
            {
                let occlusion_start = (occlusion.start - camera_pos) / (world_window_size * 0.5);
                let occlusion_end = (occlusion.end - camera_pos) / (world_window_size * 0.5);
                let light_pos = (light.pos - camera_pos) / (world_window_size * 0.5);
                let d1 = occlusion_start - light_pos;
                let d2 = occlusion_end - light_pos;

                let coords = [
                    Vertex {
                        position: [occlusion_start.x, occlusion_start.y, 1.0],
                        tex_coords: [1.0 - occlusion.visibility, 0.0],
                    },
                    Vertex {
                        position: [d1.x, d1.y, 0.0],
                        tex_coords: [1.0 - occlusion.visibility, 0.0],
                    },
                    Vertex {
                        position: [occlusion_end.x, occlusion_end.y, 1.0],
                        tex_coords: [1.0 - occlusion.visibility, 0.0],
                    },
                    Vertex {
                        position: [occlusion_end.x, occlusion_end.y, 1.0],
                        tex_coords: [1.0 - occlusion.visibility, 0.0],
                    },
                    Vertex {
                        position: [d1.x, d1.y, 0.0],
                        tex_coords: [1.0 - occlusion.visibility, 0.0],
                    },
                    Vertex {
                        position: [d2.x, d2.y, 0.0],
                        tex_coords: [1.0 - occlusion.visibility, 0.0],
                    },
                ];

                for coord in coords {
                    verts.push(coord);
                }
            }
        }

        let vertex_buffer =
            wgpu_state
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Vertex Buffer"),
                    contents: bytemuck::cast_slice(&verts),
                    usage: wgpu::BufferUsages::VERTEX,
                });

        let texture_view = texture.create_view(&Default::default());
        {
            let render_pass_desc = wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &texture_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: op,
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            };
            op = wgpu::LoadOp::Load;
            let mut render_pass = encoder.begin_render_pass(&render_pass_desc);
            render_pass.set_pipeline(&wgpu_state.shadow_mask_pipeline);
            render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            render_pass.draw(0..verts.len() as u32, 0..1);
        }

        let light_uniform = LightUniform {
            data: [
                light.color.r(),
                light.color.g(),
                light.color.b(),
                light.intensity,
            ],
            last: [if i == lights.len() - 1 { 1.0 } else { 0.0 }, 0.0, 0.0, 0.0],
        };

        let light_buffer =
            wgpu_state
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("light_buffer"),
                    contents: bytemuck::cast_slice(&[light_uniform]),
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                });

        let light_bind_group = wgpu_state
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &wgpu_state.light_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&texture_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&texture_sampler),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: light_buffer.as_entire_binding(),
                    },
                ],
                label: Some("light_bind_group"),
            });

        let verts: Vec<Vertex> = vec![
            Vertex {
                position: [-1.0, -1.0, 0.0],
                tex_coords: [0.0, 1.0],
            },
            Vertex {
                position: [1.0, -1.0, 0.0],
                tex_coords: [1.0, 1.0],
            },
            Vertex {
                position: [1.0, 1.0, 0.0],
                tex_coords: [1.0, 0.0],
            },
            Vertex {
                position: [-1.0, -1.0, 0.0],
                tex_coords: [0.0, 1.0],
            },
            Vertex {
                position: [1.0, 1.0, 0.0],
                tex_coords: [1.0, 0.0],
            },
            Vertex {
                position: [-1.0, 1.0, 0.0],
                tex_coords: [0.0, 0.0],
            },
        ];

        let vertex_buffer =
            wgpu_state
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Vertex Buffer"),
                    contents: bytemuck::cast_slice(&verts),
                    usage: wgpu::BufferUsages::VERTEX,
                });

        let out_tex = wgpu_state.device.create_texture(&texture_desc);
        let texture_view = out_tex.create_view(&Default::default());
        {
            let render_pass_desc = wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &texture_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            };
            let mut render_pass = encoder.begin_render_pass(&render_pass_desc);
            render_pass.set_pipeline(&wgpu_state.add_light_pipeline);
            render_pass.set_bind_group(0, &light_bind_group, &[]);
            render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            render_pass.draw(0..verts.len() as u32, 0..1);
        }
        texture = out_tex;
    }

    encoder.copy_texture_to_buffer(
        wgpu::ImageCopyTexture {
            aspect: wgpu::TextureAspect::All,
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
        },
        wgpu::ImageCopyBuffer {
            buffer: &output_buffer,
            layout: wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: NonZeroU32::new(u32_size * width),
                rows_per_image: NonZeroU32::new(height),
            },
        },
        texture_desc.size,
    );

    wgpu_state.queue.submit(Some(encoder.finish()));

    {
        let buffer_slice = output_buffer.slice(..);

        // NOTE: We have to create the mapping THEN device.poll() before await
        // the future. Otherwise the application will freeze.
        let (tx, rx) = futures_intrusive::channel::shared::oneshot_channel();
        buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
            tx.send(result).unwrap();
        });
        wgpu_state.device.poll(wgpu::Maintain::Wait);
        block_on(rx.receive()).unwrap().unwrap();

        let data = buffer_slice.get_mapped_range();

        use image::{ImageBuffer, Rgba};
        let buffer = ImageBuffer::<Rgba<u8>, _>::from_raw(width, height, data).unwrap();
        buffer.save("image.png").unwrap();
    }
    output_buffer.unmap();
}
