// Mipmapped sprites for drawing to the texture
@group(1) @binding(0)
var tile_sprites: texture_2d<f32>;
@group(1) @binding(1)
var sprite_sampler: sampler;

// Representation of the tiles in the world, as a texture in the GPU
@group(2) @binding(0)
var world_tiles: texture_2d<u32>;

struct VertexOut {
    @builtin(position) position: vec4<f32>,
}

@vertex
fn vs_main(@builtin(vertex_index) idx: u32) -> VertexOut {
    var out: VertexOut;
    out.position = six_vertex_square(idx).position;
    return out;
}

// The fragment shader is called when rendering the tiles to the internal texture,
// which will then be scaled and drawn to the screen.
@fragment
fn fs_main(in: VertexOut) -> @location(0) vec4<f32> {
    let world_position = render_texture_world_position(in.position.xy);

    let tiles_origin = floor(world_position / camera.texture_size_f32) * camera.texture_size_f32;

    let position_in_tile = world_position - floor(world_position);

    let tile = textureLoad(world_tiles, vec2<i32>(floor(world_position) - tiles_origin), 0).r;
    let y = f32(tile / 16);
    let x = f32(tile) - 16*y;
    let sprite_uv = (vec2(x, y) + position_in_tile) / 16.0;
    let mip_level = log2(16.0/camera.tile_size.x);
    return textureSampleLevel(tile_sprites, sprite_sampler, sprite_uv, mip_level)
//    + vec4<f32>(tile)/256.0
//    + vec4<f32>(sprite_uv, 0.0, 1.0)
    ;
//    return vec4<f32>(textureLoad(world_tiles, vec2(world_x, world_y), 0));
//    return vec4(vec2(f32(thingy.x), f32(thingy.y))/texture_size_f32, 0.0, 1.0);
//    return vec4<f32>(world_position, -world_position.x, 1.0);
}
