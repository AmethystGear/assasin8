use bevy::{prelude::*, render::{extract_component::{ExtractComponentPlugin, ExtractComponent, UniformComponentPlugin}, render_resource::encase::private::ShaderType}};

#[derive(Component, Clone, ExtractComponent)]
struct Light {
    angle_range: f32,
    intensity: f32,
    color: Color,
}

/// anything that casts a shadow needs to have this component.
/// this tells the shadow renderer what faces to cast shadows from.
/// verts should be a list of triangles. 
#[derive(Component, Clone, ExtractComponent)]
struct ShadowGeometry {
    verts : Vec<Vec3>
}


struct ShadowRenderPass;

impl Plugin for ShadowRenderPass {
    fn build(&self, app: &mut App) {
        app
            .add_plugin(ExtractComponentPlugin::<ShadowGeometry>::default())
            .add_plugin(UniformComponentPlugin::<ShadowGeometry>::default());
    }
}