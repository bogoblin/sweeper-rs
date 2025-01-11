use crate::texture::Texture;

pub struct TileSheetTexture {
    pub texture: Texture,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
}

impl TileSheetTexture {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
        let bytes = include_bytes!("background.png");
        let texture = Texture::from_bytes(&device, &queue, bytes, "background.png").unwrap();
        let (bind_group, bind_group_layout) = texture.bind_group_and_layout(device);
        
        Self {
            texture,
            bind_group_layout,
            bind_group
        }
    }
}