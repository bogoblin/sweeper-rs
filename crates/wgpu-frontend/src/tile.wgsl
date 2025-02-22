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
    var color: vec4<f32>;
    let dims = vec2<f32>(textureDimensions(zoom_render));
    let uv = world_to_uv(in.world_coords.xy, dims, camera.tile_map_size.x);
    if camera.tile_map_size.x >= 16 {
        let relative_uv = uv - floor(uv);
        let dimensions = vec2<f32>(textureDimensions(zoom_render));
        let texture_coords = relative_uv * vec2<f32>(textureDimensions(zoom_render));
        // Pixel perfect scaling from: https://colececil.dev/blog/2017/scaling-pixel-art-without-destroying-it/
        let location_within_texel = fract(texture_coords);
        let texels_per_pixel = vec2(16.0, 16.0)/camera.tile_size.xy;
        let interpolation_amount = clamp(location_within_texel / texels_per_pixel, vec2(0.0, 0.0), vec2(0.5, 0.5))
            + clamp((location_within_texel - vec2(1.0, 1.0)) / texels_per_pixel + vec2(0.5, 0.5), vec2(0.0, 0.0), vec2(0.5, 0.5));

        let final_texture_coords = floor(texture_coords) + interpolation_amount;

        color = textureSample(zoom_render, zoom_sampler, final_texture_coords/dimensions);
    } else {
        color = textureSample(zoom_render, zoom_sampler, uv);
    }

    if (
        in.world_coords.x < camera.tile_map_rect.x
        || in.world_coords.y < camera.tile_map_rect.y
        || in.world_coords.x > camera.tile_map_rect.z
        || in.world_coords.y > camera.tile_map_rect.w
    ) {
        color += vec4(0.1, 0.0, 0.0, 0.2);
    }

    return color / color.a;
}