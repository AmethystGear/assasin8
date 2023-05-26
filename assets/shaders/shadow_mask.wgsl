// creates a shadow mask in the red channel
// when using this shader, make sure it only has permissions to write to the red channel,
// otherwise, it'll override the other channels with zeroes. 
// we use the red channel and not the alpha channel because when the triangles overlap
// while setting the alpha channel, the alpha adds. This isn't wha we want, we want
// to set the channel to a fixed value regardless of how many overlaps we have.
struct VertexInput {
    @location(0) pos: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(1) alpha: f32
}

@vertex
fn vertex(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.position = vec4<f32>(in.pos.x, in.pos.y, 0.0, in.pos.z);
    out.alpha = in.tex_coords.x;
    return out;
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(in.alpha, 0.0, 0.0, 0.0);
}