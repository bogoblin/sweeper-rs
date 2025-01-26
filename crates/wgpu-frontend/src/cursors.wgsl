struct VertexOut {
    @builtin(position) position: vec4<f32>,
    @location(0) texture_coords: vec2<f32>,
    @location(1) show: u32,
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
    let animation_t = saturate(f32(camera.time - instance.time_moved)/200.0);
    let animated_position = animation_t * instance.position + (1.0 - animation_t) * instance.prev_position;
    out.position = vec4<f32>(world_to_clip(animated_position + 2.0*texture_coords), 0.0, 1.0);
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

    return out;
}

@group(1) @binding(0)
var sprite: texture_2d<f32>;
@group(1) @binding(1)
var sprite_sampler: sampler;


@fragment
fn fs_main(in: VertexOut) -> @location(0) vec4<f32> {
    if (bool(in.show)) {
        return textureSample(sprite, sprite_sampler, in.texture_coords);
    } else {
        return vec4(0.0, 0.0, 0.0, 0.0);
    }
//return vec4(in.texture_coords.xy, 0.0, 1.0);
}