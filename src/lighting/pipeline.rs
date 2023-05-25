use super::{
    light::{ComputedTargetSizes, ShadowRenderBindGroup},
    types::{
        light_source_to_light_data, shadow_caster_to_occlusion_data, CameraData, LightDataBuf,
        LightSource, OcclusionDataBuf, ShadowCaster,
    },
};
use bevy::{
    prelude::*,
    render::{
        extract_resource::ExtractResource,
        render_asset::RenderAssets,
        render_resource::{
            AddressMode, BindGroupDescriptor, BindGroupEntry, BindGroupLayout,
            BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingResource, BindingType,
            BufferBindingType, CachedComputePipelineId, ComputePipelineDescriptor, Extent3d,
            FilterMode, PipelineCache, SamplerDescriptor, ShaderStages, StorageBuffer,
            StorageTextureAccess, TextureDimension, TextureFormat, TextureUsages,
            TextureViewDimension, UniformBuffer,
        },
        renderer::{RenderDevice, RenderQueue},
        texture::ImageSampler,
        Extract,
    },
    sprite::Mesh2dHandle,
};
use encase::ShaderType;

#[derive(Default, Resource)]
pub struct ShadowPipelineAssets {
    pub camera: UniformBuffer<CameraData>,
    pub lights: StorageBuffer<LightDataBuf>,
    pub occlusions: StorageBuffer<OcclusionDataBuf>,
}

impl ShadowPipelineAssets {
    pub fn write_buffer(&mut self, device: &RenderDevice, queue: &RenderQueue) {
        self.lights.write_buffer(device, queue);
        self.occlusions.write_buffer(device, queue);
        self.camera.write_buffer(device, queue);
    }
}

pub fn prepare_pipeline_assets(
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    mut compute_assets: ResMut<ShadowPipelineAssets>,
) {
    compute_assets.write_buffer(&render_device, &render_queue);
}

pub fn extract_pipeline_assets(
    meshes: Extract<Res<Assets<Mesh>>>,
    target_sizes: Extract<Res<ComputedTargetSizes>>,
    query_lights: Extract<Query<(&Transform, &LightSource)>>,
    query_occluders: Extract<Query<(&Transform, &Mesh2dHandle), With<ShadowCaster>>>,
    query_camera: Extract<Query<(&Camera, &GlobalTransform)>>,

    mut gpu_target_sizes: ResMut<ComputedTargetSizes>,
    mut gpu_pipeline_assets: ResMut<ShadowPipelineAssets>,
) {
    *gpu_target_sizes = **target_sizes;

    let mut lights = gpu_pipeline_assets.lights.get_mut();
    lights.data = query_lights
        .iter()
        .map(light_source_to_light_data)
        .collect();
    lights.count = lights.data.len() as u32;

    let mut occluders = gpu_pipeline_assets.occlusions.get_mut();
    occluders.data = query_occluders
        .iter()
        .flat_map(|(t, mesh_handle)| {
            shadow_caster_to_occlusion_data(t, meshes.get(&mesh_handle.0).unwrap())
        })
        .collect();
    occluders.count = occluders.data.len() as u32;

    let Ok((camera, camera_global_transform)) = query_camera.get_single() else {
        panic!("couldn't get camera");
    };

    let mut camera_params = gpu_pipeline_assets.camera.get_mut();
    let projection = camera.projection_matrix();
    let inverse_projection = projection.inverse();
    let view = camera_global_transform.compute_matrix();
    let inverse_view = view.inverse();

    camera_params.view_proj = projection * inverse_view;
    camera_params.inverse_view_proj = view * inverse_projection;
    camera_params.screen_size = Vec2::new(
        gpu_target_sizes.primary_target_size.x,
        gpu_target_sizes.primary_target_size.y,
    );
    camera_params.screen_size_inv = Vec2::new(
        1.0 / gpu_target_sizes.primary_target_size.x,
        1.0 / gpu_target_sizes.primary_target_size.y,
    );

    let scale = 2.0;
    camera_params.sdf_scale = Vec2::splat(scale);
    camera_params.inv_sdf_scale = Vec2::splat(1. / scale);
}

#[derive(Resource)]
pub struct ShadowPipeline {
    pub layout: BindGroupLayout,
    pub pipeline_id: CachedComputePipelineId,
}

impl FromWorld for ShadowPipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();

        let layout = render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("sdf_bind_group_layout"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: Some(CameraData::min_size()),
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: Some(OcclusionDataBuf::min_size()),
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: Some(LightDataBuf::min_size()),
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 3,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::StorageTexture {
                        access: StorageTextureAccess::ReadWrite,
                        format: TextureFormat::Rgba8Uint,
                        view_dimension: TextureViewDimension::D2,
                    },
                    count: None,
                },
            ],
        });

        let assets_server = world.resource::<AssetServer>();
        let shader = assets_server.load("shaders/shadowcaster.wgsl");

        let pipeline_cache = world.resource_mut::<PipelineCache>();
        let pipeline_id = pipeline_cache.queue_compute_pipeline(ComputePipelineDescriptor {
            label: Some("pipeline".into()),
            layout: vec![layout.clone()],
            shader,
            shader_defs: vec![],
            entry_point: "main".into(),
            push_constant_ranges: vec![],
        });

        ShadowPipeline {
            layout,
            pipeline_id,
        }
    }
}

fn create_texture_2d(size: (u32, u32), format: TextureFormat, filter: FilterMode) -> Image {
    let mut image = Image::new_fill(
        Extent3d {
            width: size.0,
            height: size.1,
            ..Default::default()
        },
        TextureDimension::D2,
        &[
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0,
        ],
        format,
    );

    image.texture_descriptor.usage =
        TextureUsages::COPY_DST | TextureUsages::STORAGE_BINDING | TextureUsages::TEXTURE_BINDING;

    image.sampler_descriptor = ImageSampler::Descriptor(SamplerDescriptor {
        mag_filter: filter,
        min_filter: filter,
        address_mode_u: AddressMode::ClampToBorder,
        address_mode_v: AddressMode::ClampToBorder,
        address_mode_w: AddressMode::ClampToBorder,
        ..Default::default()
    });

    image
}

#[derive(Clone, Resource, ExtractResource, Default)]
pub struct PipelineTarget {
    pub target: Option<Handle<Image>>,
}

pub fn setup_pipeline(
    mut images: ResMut<Assets<Image>>,
    mut target_wrapper: ResMut<PipelineTarget>,
    targets_sizes: ResMut<ComputedTargetSizes>,
) {
    target_wrapper.target = Some(images.add(create_texture_2d(
        targets_sizes.primary_target_usize.into(),
        TextureFormat::Rgba8Uint,
        FilterMode::Linear,
    )));
}

pub fn queue_bind_group(
    mut commands: Commands,
    pipeline: Res<ShadowPipeline>,
    gpu_images: Res<RenderAssets<Image>>,
    target: Res<PipelineTarget>,
    assets: Res<ShadowPipelineAssets>,
    render_device: Res<RenderDevice>,
) {
    if let (Some(lights), Some(occluders), Some(camera_data)) = (
        assets.lights.binding(),
        assets.occlusions.binding(),
        assets.camera.binding(),
    ) {
        let target = target
            .target
            .as_ref()
            .expect("targets should be initialized");
        let view = &gpu_images[&target];
        let bind_group = render_device.create_bind_group(&BindGroupDescriptor {
            label: "shadow_pass".into(),
            layout: &pipeline.layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: camera_data.clone(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: occluders.clone(),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: lights.clone(),
                },
                BindGroupEntry {
                    binding: 3,
                    resource: BindingResource::TextureView(&view.texture_view),
                },
            ],
        });
        commands.insert_resource(ShadowRenderBindGroup { bind_group });
    }
}
