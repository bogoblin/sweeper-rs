struct CameraUniform {
    @location(0) world_rect: vec4<f32>,
    @location(1) tile_size: vec4<f32>,
    @location(2) tile_map_rect: vec4<f32>,
    @location(3) tile_map_size: vec4<f32>,
}
@group(0) @binding(0)
var<uniform> camera: CameraUniform;

