@group(1) @binding(0)
var<uniform> blanking_rect: vec4<i32>;

struct VertexOut {
    @builtin(position) position: vec4<f32>,
}

@vertex
fn vs_main(@builtin(vertex_index) idx: u32) -> VertexOut {
    var out: VertexOut;
    out.position = six_vertex_square(idx).position;
    return out;
}

@fragment
fn fs_main(in: VertexOut) -> @location(0) vec4<u32> {
    let world_position = vec2<i32>(render_texture_world_position(in.position.xy));
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
