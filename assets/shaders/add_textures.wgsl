// combines two textures and returns the output
#import bevy_pbr::utils

@group(0) @binding(0)
var screen_texture: texture_2d<f32>;

@group(0) @binding(1)
var screen_texture_sampler: sampler;

@group(0) @binding(2)
var shadow_mask_texture: texture_2d<f32>;

@group(0) @binding(3)
var shadow_mask_texture_sampler: sampler;

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    return textureSample(screen_texture, screen_texture_sampler, in.uv) + textureSample(shadow_mask_texture, shadow_mask_texture_sampler, in.uv);
}