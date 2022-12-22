@group(0) @binding(0)
var<uniform> camera: mat4x4<f32>;
@group(1) @binding(0)
var texture: texture_2d<f32>;
@group(1) @binding(1)
var samp: sampler;


struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex: vec2<f32>,
}

struct InstanceInput {
    @location(2) model_matrix_0: vec4<f32>,
    @location(3) model_matrix_1: vec4<f32>,
    @location(4) model_matrix_2: vec4<f32>,
    @location(5) model_matrix_3: vec4<f32>,
    @location(6) uv_offset: vec2<f32>,
    @location(7) uv_size: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) tex: vec2<f32>,
    @location(1) uv_offset: vec2<f32>,
    @location(2) uv_size: vec2<f32>,
}

@vertex
fn vertex(vertex: VertexInput, instance: InstanceInput) -> VertexOutput {
    let model_matrix = mat4x4<f32>(
        instance.model_matrix_0,
        instance.model_matrix_1,
        instance.model_matrix_2,
        instance.model_matrix_3,
    );

    var out: VertexOutput;
    out.position = camera * model_matrix * vec4<f32>(vertex.position, 1.0);
    out.tex = vertex.tex;
    out.uv_offset = instance.uv_offset;
    out.uv_size = instance.uv_size;
    return out;
}

struct FragmentInput {
    @location(0) tex: vec2<f32>,
    @location(1) uv_offset: vec2<f32>,
    @location(2) uv_size: vec2<f32>,
}

@fragment
fn fragment(in: FragmentInput) -> @location(0) vec4<f32> {
    // map uv onto the mega texture
    // e.g. if we wanted to draw our entire dirt texture on a face
    // we need to take our coords (0,0) -> (1,1)
    // scale them down appropriately
    // what fraction of the image does this form?
    var dimensions: vec2<i32> = textureDimensions(texture);
    var adjustedTex: vec2<f32> = vec2(in.uv_offset.x / f32(dimensions.x) + in.tex.x  * in.uv_size.x / f32(dimensions.x), in.uv_offset.y / f32(dimensions.y) + in.tex.y * in.uv_size.y / f32(dimensions.y));
    return textureSample(texture, samp, adjustedTex);
}