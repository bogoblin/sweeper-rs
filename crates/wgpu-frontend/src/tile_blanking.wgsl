@group(1) @binding(0)
var<uniform> blanking_rect: vec4<i32>;

struct VertexOut {
    @builtin(position) position: vec4<f32>,
}

@vertex
fn vs_main(@builtin(vertex_index) idx: u32) -> VertexOut {
    var out: VertexOut;
    switch (idx % 6) {
        case 0u: {
            out.position = vec4(-1.0, 1.0, 0.0, 1.0);
        }
        case 1u: {
            out.position = vec4(-1.0, -1.0, 0.0, 1.0);
        }
        case 2u: {
            out.position = vec4(1.0, 1.0, 0.0, 1.0);
        }
        case 3u: {
            out.position = vec4(1.0, 1.0, 0.0, 1.0);
        }
        case 4u: {
            out.position = vec4(-1.0, -1.0, 0.0, 1.0);
        }
        case 5u: {
            out.position = vec4(1.0, -1.0, 0.0, 1.0);
        }
        default: {}
    }
    return out;
}

@fragment
fn fs_main(in: VertexOut) -> @location(0) vec4<u32> {
    let texture_size = camera.full_tile_map_rect.z - camera.full_tile_map_rect.x;
    let texture_size_f32 = vec2<f32>(f32(texture_size), f32(texture_size));

    let tiles_in_texture = texture_size_f32.x;
    let origin = ceil(vec2<f32>(camera.full_tile_map_rect.xy) / tiles_in_texture) * tiles_in_texture;
    var world_position = vec2<i32>(origin + in.position.xy);
    if (world_position.x >= camera.full_tile_map_rect.z) {
        world_position.x -= (camera.full_tile_map_rect.z - camera.full_tile_map_rect.x);
    }
    if (world_position.y >= camera.full_tile_map_rect.w) {
        world_position.y -= (camera.full_tile_map_rect.w - camera.full_tile_map_rect.y);
    }
    
    if (
        world_position.x >= blanking_rect.x &&
        world_position.y >= blanking_rect.y &&
        world_position.x < blanking_rect.z &&
        world_position.y < blanking_rect.w
    ) {
        return vec4<u32>(0, 0, 0, 255);
    }
    discard;
}
