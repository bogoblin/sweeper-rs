struct VertexOut {
    @builtin(position) position: vec4<f32>,
    @location(0) texture_coords: vec2<f32>,
    @location(1) show: u32,
    @location(2) time_moved: i32,
    @location(3) hue: f32,
}

struct CursorInstance {
    @location(0) position: vec2<f32>,
    @location(1) prev_position: vec2<f32>,
    @location(2) time_moved: i32,
    @location(3) properties: u32,
}

@vertex
fn vs_main(@builtin(vertex_index) idx: u32, instance: CursorInstance) -> VertexOut {
    var texture_coords: vec2<f32>;
    switch (idx % 6) {
        case 0u: {
            texture_coords = vec2(0.0, 0.0);
        }
        case 1u: {
            texture_coords = vec2(1.0, 1.0);
        }
        case 2u: {
            texture_coords = vec2(1.0, 0.0);
        }
        case 3u: {
            texture_coords = vec2(0.0, 0.0);
        }
        case 4u: {
            texture_coords = vec2(0.0, 1.0);
        }
        case 5u: {
            texture_coords = vec2(1.0, 1.0);
        }
        default: {}
    }
    var out: VertexOut;
    out.time_moved = instance.time_moved;

    let animation_t = saturate(f32(camera.time - instance.time_moved)/200.0);
    let animated_position = animation_t * instance.position + (1.0 - animation_t) * instance.prev_position;

    out.position = vec4<f32>(world_to_clip(
        vec2(0.5,0.5) // half tile offset so that the cursor is pointing at the center of the tile
        + animated_position
        + 2.0*texture_coords*max(1.0, 32.0/camera.tile_size.x)),
    0.0, 1.0);

    out.texture_coords = texture_coords;
    if (bool(instance.properties & 2)) { // Is this player connected?
        if (bool(instance.properties & 1)) { // Is it you?
            out.show = u32(0);
        } else {
            out.show = u32(1);
        }
    } else {
        out.show = u32(0);
    }
    let hue = (instance.properties >> 8) & 255;
    out.hue = f32(hue)/255.0;

    return out;
}

@group(1) @binding(0)
var sprite: texture_2d<f32>;
@group(1) @binding(1)
var sprite_sampler: sampler;

@group(2) @binding(0)
var colors: texture_2d<f32>;
@group(2) @binding(1)
var colors_sampler: sampler;

@fragment
fn fs_main(in: VertexOut) -> @location(0) vec4<f32> {
    if (bool(in.show)) {
        let texture_color = textureLoad(sprite, vec2<u32>(in.texture_coords * vec2<f32>(textureDimensions(sprite).xy)), 0);
        let color_coord = vec2<f32>(in.hue, texture_color.r);
        let color = textureSample(colors, colors_sampler, color_coord);
        return color * texture_color.a;
    } else {
        return vec4(0.0, 0.0, 0.0, 0.0);
    }
}