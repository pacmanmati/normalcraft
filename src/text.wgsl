@group(0) @binding(0)
var<uniform> camera: mat4x4<f32>;
@group(1) @binding(0)
var texture: texture_2d<f32>;
@group(1) @binding(1)
var samp: sampler;


struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) tex: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) tex: vec2<f32>,
}

@vertex
fn vertex(vertex: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.position = camera * vec4<f32>(vertex.position, 0.0, 1.0);
    out.tex = vertex.tex;
    return out;
}

struct FragmentInput {
    @location(0) tex: vec2<f32>,
}

@fragment
fn fragment(in: FragmentInput) -> @location(0) vec4<f32> {
    return textureSample(texture, samp, in.tex);
}