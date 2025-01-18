struct VertexOut {
    @builtin(position) position: vec4<f32>,
};

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

//@group(0) @binding(0)
//var tile_sprites: texture_2d<f32>;
//@group(0) @binding(1)
//var sprite_sampler: sampler;
//
//@group(1) @binding(0)
//var world_tiles: texture_2d<u32>;
//@group(1) @binding(1)
//var nearest_sampler: sampler;

@fragment
fn fs_main(in: VertexOut) -> @location(0) vec4<f32> {
//    return textureSample(t_diffuse, s_diffuse, in.tex_coords);
    let render_coords = in.position / 8192.0;
    return vec4(render_coords.xy, 0.0, 1.0);
}