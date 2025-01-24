struct CameraUniform {
    @location(0) world_rect: vec4<f32>,
    @location(1) tile_size: vec4<f32>,
    @location(2) tile_map_rect: vec4<f32>,
    @location(3) tile_map_size: vec4<f32>,
    @location(4) full_tile_map_rect: vec4<i32>,
}
@group(0) @binding(0)
var<uniform> camera: CameraUniform;

fn world_to_clip(world_coords: vec2<f32>) -> vec2<f32> {
    var coords = 2.0 * ((world_coords - camera.world_rect.xy) / (camera.world_rect.zw - camera.world_rect.xy)) - vec2(1.0, 1.0);
    coords.y *= -1.0;
    return coords;
}