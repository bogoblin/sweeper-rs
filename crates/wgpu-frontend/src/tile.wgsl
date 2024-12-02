struct VertexInput {
    @builtin(vertex_index) idx: u32,
}

struct InstanceInput {
    @builtin(instance_index) idx: u32,
    @location(0) position: vec2<f32>,
    @location(1) tex_coords: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
}

struct CameraUniform {
    @location(0) center: vec2<f32>,
    @location(1) tile_size: vec2<f32>,
}
@group(1) @binding(0)
var<uniform> camera: CameraUniform;

@vertex
fn vs_main(vertex: VertexInput, instance: InstanceInput) -> VertexOutput {
    var offset: vec2<f32>;
    switch (vertex.idx % 6) {
        case 0u: { offset = vec2<f32>(0.0, 0.0); }
        case 1u: { offset = vec2<f32>(0.0, 1.0); }
        case 2u: { offset = vec2<f32>(1.0, 0.0); }
        case 3u: { offset = vec2<f32>(1.0, 0.0); }
        case 4u: { offset = vec2<f32>(0.0, 1.0); }
        case 5u: { offset = vec2<f32>(1.0, 1.0); }
        default: { offset = vec2<f32>(0.0, 0.0); }
    }

    var position: vec2<f32> = vec2(instance.position + offset);

    var out: VertexOutput;
    out.clip_position = vec4<f32>(vec2((position - camera.center)*camera.tile_size), 0.0, 1.0);
    out.tex_coords = instance.tex_coords + vec2(offset.x, 1.0 - offset.y)/16.0;
    return out;
}

@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(t_diffuse, s_diffuse, in.tex_coords);
}