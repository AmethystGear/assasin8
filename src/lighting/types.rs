use bevy::{
    prelude::*,
    render::{extract_component::ExtractComponent, render_resource::ShaderType},
};

#[derive(Component)]
pub struct ShadowCaster {
    pub verts: Vec<Vec2>,
    pub visibility: f32,
}

#[derive(Component, Default, Clone, ExtractComponent, ShaderType, Debug)]
pub struct OcclusionData {
    pub start: Vec2,
    pub end: Vec2,
    pub visibility: f32,
}

pub fn shadow_caster_to_occlusion_data(
    (transform, shadow_caster): (&Transform, &ShadowCaster),
) -> Vec<OcclusionData> {
    let mut occlusions = vec![];
    for i in 0..shadow_caster.verts.len() / 3 {
        occlusions.push(OcclusionData {
            start: shadow_caster.verts[i * 3],
            end: shadow_caster.verts[i * 3 + 1],
            visibility: shadow_caster.visibility,
        });
        occlusions.push(OcclusionData {
            start: shadow_caster.verts[i * 3 + 1],
            end: shadow_caster.verts[i * 3 + 2],
            visibility: shadow_caster.visibility,
        });
        occlusions.push(OcclusionData {
            start: shadow_caster.verts[i * 3 + 2],
            end: shadow_caster.verts[i * 3],
            visibility: shadow_caster.visibility,
        });
    }
    occlusions
        .into_iter()
        .map(|o| {
            let start = transform.transform_point(Vec3::new(o.start.x, o.start.y, 0.0));
            let end = transform.transform_point(Vec3::new(o.end.x, o.end.y, 0.0));
            let start = Vec2::new(start.x, start.y);
            let end = Vec2::new(end.x, end.y);
            OcclusionData {
                start,
                end,
                visibility: o.visibility,
            }
        })
        .collect()
}

#[derive(Component)]
pub struct LightSource {
    pub intensity: f32,
    pub color: Color,
}

#[derive(Component, Clone, Default, ExtractComponent, ShaderType)]
pub struct LightData {
    pub pos: Vec2,
    pub intensity: f32,
    pub color: Color,
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
