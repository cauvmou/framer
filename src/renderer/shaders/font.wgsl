// Vertex shader

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) tex_coords: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
}

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.tex_coords = model.tex_coords;
    out.clip_position = vec4<f32>(model.position.xy, 0.0, 1.0);
    return out;
}

@group(0) @binding(0)
var msdf: texture_2d<f32>;
@group(0) @binding(1)
var msdf_sampler: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var msd = textureSample(msdf, msdf_sampler, in.tex_coords);
    var sd = median(msd.x, msd.y, msd.z);
    var screen_px_distance = screenPxRange(in.tex_coords)*(sd - 0.5);
    var opacity = clamp(screen_px_distance + 0.5, 0.0, 1.0);
    return mix(vec4<f32>(0.0, 1.0, 0.4, 0.0), vec4<f32>(0.0, 0.0, 0.0, 1.0), opacity);
    //return vec4<f32>(msd.xyz, 1.0);
    //return vec4<f32>(opacity, opacity, opacity, 1.0);
}

fn median(r: f32, g: f32, b: f32) -> f32 {
    return max(min(r, g), min(max(r, g), b));
}

fn screenPxRange(tex_coord: vec2<f32>) -> f32 {
    var pxRange = 64.0; // REPLACE WITH UNIFORM
    var unitRange = vec2<f32>(pxRange)/vec2<f32>(textureDimensions(msdf, 0));
    var screenTexSize = vec2<f32>(1.0)/fwidth(tex_coord);
    return max(0.5*dot(unitRange, screenTexSize), 1.0);
}