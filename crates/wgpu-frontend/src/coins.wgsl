struct VertexOut {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
}

struct CoinInstance {
    @location(0) rect: vec4<f32>,
    @location(1) color: vec4<f32>,
}

@vertex
fn vs_main(@builtin(vertex_index) idx: u32, instance: CursorInstance) -> VertexOut {
    var position: vec2<f32>;
    switch (idx % 6) {
        case 0u: {
            position = instance.rect.xy;
        }
        case 1u: {
            position = instance.rect.zw;
        }
        case 2u: {
            position = instance.rect.zy;
        }
        case 3u: {
            position = instance.rect.xy;
        }
        case 4u: {
            position = instance.rect.xw;
        }
        case 5u: {
            position = instance.rect.zw;
        }
        default: {}
    }
    out.position = vec4<f32>(position, 0.0, 1.0);
    out.color = instance.color;
    return out;
}

@fragment
fn fs_main(in: VertexOut) -> @location(0) vec4<f32> {
    return in.color;
}