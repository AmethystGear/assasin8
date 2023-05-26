// draws in a light. We use the red channel to store the shadow mask (see shadow_mask.wgsl)
// so we use g,b,a to store the true r,g,b values, then clear r to 1.0 to be ready for the next
// shadow mask pass. On the last pass, we switch back the g,b,a values into r,g,b and set alpha to 1.0. 
@group(0) @binding(0)
var texture: texture_2d<f32>;

@group(0) @binding(1)
var texture_sampler: sampler;

struct LightData {
    data: vec4<f32>,
    last: vec4<f32>,
};

@group(0) @binding(2)
var<uniform> lightdata: LightData;

struct VertexInput {
    @location(0) pos: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) pos: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
}

@vertex
fn vertex(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.tex_coords = model.tex_coords;
    out.pos = vec4<f32>(model.pos, 1.0);
    return out;
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    var color: vec4<f32> = textureSample(texture, texture_sampler, in.tex_coords);
    var c: vec3<f32> = lightdata.data.rgb * lightdata.data.a * color.r;

    if (lightdata.last.x > 0.0) {
        color.r = color.g;
        color.g = color.b;
        color.b = color.a;
        color.r += c.r;
        color.g += c.g;
        color.b += c.b;
        color.a = 1.0;
    } else {
        color.g += c.r;
        color.b += c.g;
        color.a += c.b;
        color.r = 1.0;
    }
    return color;
}