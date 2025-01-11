struct CameraUniform {
    @location(0) zoom_render_blend_0: vec4<f32>,
    @location(1) zoom_render_blend_1: vec4<f32>,
    @location(2) world_rect: vec4<f32>,
    @location(3) tile_size: vec4<f32>,
}
@group(1) @binding(0)
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

// Sprites
@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;

// Rendered Tiles
@group(2) @binding(0)
var rendered_0: texture_2d<f32>;
@group(2) @binding(1)
var rendered_1: texture_2d<f32>;
@group(2) @binding(2)
var rendered_2: texture_2d<f32>;
@group(2) @binding(3)
var rendered_3: texture_2d<f32>;
@group(2) @binding(4)
var rendered_4: texture_2d<f32>;

@fragment
fn fs_main(in: VertexOut) -> @location(0) vec4<f32> {
    let tile_uv = world_to_uv(in.world_coords.xy, vec2<f32>(textureDimensions(rendered_0)), 16.0);
    var tile_color = vec4(0.0, 0.0, 0.0, 0.0);
    tile_color += textureSample(rendered_4, s_diffuse, tile_uv/16.0) * camera.zoom_render_blend_1.x;
    tile_color += textureSample(rendered_3, s_diffuse, tile_uv/8.0) * camera.zoom_render_blend_0.w;
    tile_color += textureSample(rendered_2, s_diffuse, tile_uv/4.0) * camera.zoom_render_blend_0.z;
    tile_color += textureSample(rendered_1, s_diffuse, tile_uv/2.0) * camera.zoom_render_blend_0.y;
    tile_color += textureSample(rendered_0, s_diffuse, tile_uv) * camera.zoom_render_blend_0.x;

    let blend = tile_color.w;
    return tile_color * blend + textureSample(t_diffuse, s_diffuse, in.world_coords.xy) * (1.0 - blend);
}