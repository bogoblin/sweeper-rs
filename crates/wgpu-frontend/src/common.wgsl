struct CameraUniform {
    @location(0) world_rect: vec4<f32>,
    @location(1) tile_size: vec4<f32>,
    @location(2) tile_map_rect: vec4<f32>,
    @location(3) tile_map_size: vec4<f32>,
    @location(4) full_tile_map_rect: vec4<i32>,
    @location(5) texture_size: vec2<i32>,
    @location(6) texture_size_f32: vec2<f32>,
    @location(7) time: i32,
}
@group(0) @binding(0)
var<uniform> camera: CameraUniform;

fn world_to_clip(world_coords: vec2<f32>) -> vec2<f32> {
    var coords = 2.0 * ((world_coords - camera.world_rect.xy) / (camera.world_rect.zw - camera.world_rect.xy)) - vec2(1.0, 1.0);
    coords.y *= -1.0;
    return coords;
}

struct ViewBorderVertex {
    @location(0) position: vec4<f32>,
    @location(1) world_coords: vec4<f32>,
}
fn six_vertex_square(idx: u32) -> ViewBorderVertex {
    var out: ViewBorderVertex;
    switch (idx % 6) {
        case 0u: {
            out.position = vec4(-1.0, 1.0, 0.0, 1.0);
            out.world_coords = vec4(camera.world_rect.xy, 0.0, 0.0);
        }
        case 1u: {
            out.position = vec4(-1.0, -1.0, 0.0, 1.0);
            out.world_coords = vec4(camera.world_rect.xw, 0.0, 0.0);
        }
        case 2u: {
            out.position = vec4(1.0, 1.0, 0.0, 1.0);
            out.world_coords = vec4(camera.world_rect.zy, 0.0, 0.0);
        }
        case 3u: {
            out.position = vec4(1.0, 1.0, 0.0, 1.0);
            out.world_coords = vec4(camera.world_rect.zy, 0.0, 0.0);
        }
        case 4u: {
            out.position = vec4(-1.0, -1.0, 0.0, 1.0);
            out.world_coords = vec4(camera.world_rect.xw, 0.0, 0.0);
        }
        case 5u: {
            out.position = vec4(1.0, -1.0, 0.0, 1.0);
            out.world_coords = vec4(camera.world_rect.zw, 0.0, 0.0);
        }
        default: {}
    }
    return out;
}