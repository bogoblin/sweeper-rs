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
    if (
        in.world_coords.x < camera.tile_map_rect.x
        || in.world_coords.y < camera.tile_map_rect.y
        || in.world_coords.x > camera.tile_map_rect.z
        || in.world_coords.y > camera.tile_map_rect.w
    ) {
        return vec4(0.0, 0.0, 0.0, 1.0);
    }
    let dims = vec2<f32>(textureDimensions(zoom_render));
    let pixel_in_world_space = 0.25*vec2(1.0, 1.0) / camera.tile_size.xy;
    var color = vec4<f32>(0.0, 0.0, 0.0, 0.0);

    color += textureSample(zoom_render, zoom_sampler,
        world_to_uv(vec2(1.0, 0.0)*pixel_in_world_space + in.world_coords.xy, dims, camera.tile_map_size.x));
    color += textureSample(zoom_render, zoom_sampler,
        world_to_uv(vec2(-1.0, 0.0)*pixel_in_world_space + in.world_coords.xy, dims, camera.tile_map_size.x));
    color += textureSample(zoom_render, zoom_sampler,
        world_to_uv(vec2(0.0, 1.0)*pixel_in_world_space + in.world_coords.xy, dims, camera.tile_map_size.x));
    color += textureSample(zoom_render, zoom_sampler,
        world_to_uv(vec2(0.0, -1.0)*pixel_in_world_space + in.world_coords.xy, dims, camera.tile_map_size.x));

    color += 0.5*textureSample(zoom_render, zoom_sampler,
        world_to_uv(vec2(1.0, 1.0)*pixel_in_world_space + in.world_coords.xy, dims, camera.tile_map_size.x));
    color += 0.5*textureSample(zoom_render, zoom_sampler,
        world_to_uv(vec2(-1.0, 1.0)*pixel_in_world_space + in.world_coords.xy, dims, camera.tile_map_size.x));
    color += 0.5*textureSample(zoom_render, zoom_sampler,
        world_to_uv(vec2(-1.0, -1.0)*pixel_in_world_space + in.world_coords.xy, dims, camera.tile_map_size.x));
    color += 0.5*textureSample(zoom_render, zoom_sampler,
        world_to_uv(vec2(1.0, -1.0)*pixel_in_world_space + in.world_coords.xy, dims, camera.tile_map_size.x));

    color += 4 * textureSample(zoom_render, zoom_sampler,
        world_to_uv(in.world_coords.xy, dims, camera.tile_map_size.x));

    return color / color.a;
}