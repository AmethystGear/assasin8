use bevy::{
    prelude::*,
    render::{extract_component::ExtractComponent, render_resource::ShaderType},
};

#[derive(Default, Clone, ShaderType)]
pub struct CameraData {
    pub screen_size: Vec2,
    pub screen_size_inv: Vec2,
    pub view_proj: Mat4,
    pub inverse_view_proj: Mat4,
    pub sdf_scale: Vec2,
    pub inv_sdf_scale: Vec2,
}

#[derive(Component)]
pub struct ShadowCaster;

#[derive(Component, Default, Clone, ExtractComponent, ShaderType)]
pub struct OcclusionData {
    start: Vec2,
    end: Vec2,
}

pub fn shadow_caster_to_occlusion_data(transform: &Transform, mesh: &Mesh) -> Vec<OcclusionData> {
    let ind: Vec<_> = mesh.indices().unwrap().iter().collect();
    let vertices: Vec<_> = mesh
        .attribute(Mesh::ATTRIBUTE_POSITION)
        .unwrap()
        .as_float3()
        .unwrap()
        .iter()
        .map(|[x, y, _]| (*x, *y).into())
        .collect();
    let mut occlusions = vec![];
    for i in 0..ind.len() / 3 {
        occlusions.push(OcclusionData {
            start: vertices[ind[i * 3]],
            end: vertices[ind[i * 3 + 1]],
        });
        occlusions.push(OcclusionData {
            start: vertices[ind[i * 3 + 1]],
            end: vertices[ind[i * 3 + 2]],
        });
        occlusions.push(OcclusionData {
            start: vertices[ind[i * 3 + 2]],
            end: vertices[ind[i * 3]],
        });
    }
    occlusions
        .into_iter()
        .map(|o| {
            let start = transform.transform_point(Vec3::new(o.start.x, o.start.y, 0.0));
            let end = transform.transform_point(Vec3::new(o.start.x, o.start.y, 0.0));
            let start = Vec2::new(start.x, start.y);
            let end = Vec2::new(end.x, end.y);
            OcclusionData { start, end }
        })
        .collect()
}

#[derive(Component)]
pub struct LightSource {
    intensity: f32,
    color: Color,
}

#[derive(Component, Clone, Default, ExtractComponent, ShaderType)]
pub struct LightData {
    pos: Vec2,
    intensity: f32,
    color: Color,
}

pub fn light_source_to_light_data(
    (transform, light_source): (&Transform, &LightSource),
) -> LightData {
    LightData {
        pos: Vec2::new(transform.translation.x, transform.translation.y),
        intensity: light_source.intensity,
        color: light_source.color,
    }
}

#[rustfmt::skip]
#[derive(Default, Clone, ShaderType)]
pub struct LightDataBuf {
    pub count: u32,
    #[size(runtime)]
    pub data:  Vec<LightData>,
}

#[rustfmt::skip]
#[derive(Default, Clone, ShaderType)]
pub struct OcclusionDataBuf {
    pub count: u32,
    #[size(runtime)]
    pub data:  Vec<OcclusionData>,
}
