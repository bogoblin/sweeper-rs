struct CameraUniform {
    @location(0) top_left: vec2<f32>,
    @location(1) tile_size: vec2<f32>,
}
@group(1) @binding(0)
var<uniform> camera: CameraUniform;

struct VertexOut {
    @builtin(position) position: vec4<f32>,
}

@vertex
fn vs_main(@builtin(vertex_index) idx: u32) -> VertexOut {
    // Because everything is based off of UV coordinates, we can use this clip triangle trick to make a triangle that
    // covers the entire screen:
    var out: VertexOut;
    switch (idx % 3) {
        case 0u: { out.position = vec4<f32>(-1.0, -1.0, 0.0, 1.0); }
        case 1u: { out.position = vec4<f32>(3.0, -1.0, 0.0, 1.0); }
        case 2u: { out.position = vec4<f32>(-1.0, 3.0, 0.0, 1.0); }
        default: { out.position = vec4<f32>(0.0, 0.0, 0.0, 1.0); }
    }
    return out;
}

fn screen_to_world(screen: vec2<f32>) -> vec2<f32> {
    return camera.top_left + screen / camera.tile_size;
}

// Sprites
@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;

// Tilemap
@group(2) @binding(0)
var<storage, read_write> tilemap: array<u32>;

@fragment
fn fs_main(in: VertexOut) -> @location(0) vec4<f32> {
    var world_coords: vec2<f32> = screen_to_world(in.position.xy);
    var int_world_coords: vec2<i32> = vec2<i32>(floor(world_coords));
    var tile_data_index: u32 = u32(dot(int_world_coords, vec2(1, 256))) % 256;
    var tile_data: u32 = tile_data_index;
//    var tile_data: u32 = tilemap[tile_data_index];
    var tile_uv: vec2<f32> = (vec2(f32(tile_data % 16), f32(tile_data / 16)) + (world_coords - floor(world_coords)))/16.0;
    return textureSample(t_diffuse, s_diffuse, tile_uv);
}