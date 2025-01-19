struct CameraUniform {
    @location(0) world_rect: vec4<f32>,
    @location(1) tile_size: vec4<f32>,
    @location(2) tile_map_rect: vec4<f32>,
    @location(3) tile_map_size: vec4<f32>,
}
@group(0) @binding(0)
var<uniform> camera: CameraUniform;

struct VertexOut {
    @builtin(position) position: vec4<f32>,
    @location(0) world_coords: vec4<f32>,
}

@vertex
fn vs_main(@builtin(vertex_index) idx: u32) -> VertexOut {
    var out: VertexOut;
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

fn world_to_uv(world_coords: vec2<f32>, dims: vec2<f32>, tile_size_at_zoom: f32) -> vec2<f32> {
    let world_size_of_full_tiles = dims/tile_size_at_zoom;
    let wc_flipped = vec2(world_coords.x, -1*world_coords.y);
    let top_left_of_uv = floor(world_coords/world_size_of_full_tiles);
    let remaining = (world_coords - world_size_of_full_tiles * top_left_of_uv)/world_size_of_full_tiles;
    return top_left_of_uv + remaining;
}

// Rendered Tiles
@group(1) @binding(0)
var zoom_render: texture_2d<f32>;
@group(1) @binding(1)
var zoom_sampler: sampler;

@fragment
fn fs_main(in: VertexOut) -> @location(0) vec4<f32> {
    let tile_uv = world_to_uv(in.world_coords.xy, vec2<f32>(textureDimensions(zoom_render)), camera.tile_map_size.x);
    return textureSample(zoom_render, zoom_sampler, tile_uv);
}