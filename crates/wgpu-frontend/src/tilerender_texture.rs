use crate::texture::Texture;

pub struct TilerenderTexture {
    pub texture: Texture,
    pub bind_group: wgpu::BindGroup,
    pub bind_group_layout: wgpu::BindGroupLayout,
}

impl TilerenderTexture {
    pub fn new(device: &wgpu::Device) -> Self {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d {
                width: 8192,
                height: 8192,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        
        let texture = Texture::from_wgpu_texture(texture, device).unwrap();

        let (bind_group, bind_group_layout) = texture.bind_group_and_layout(device);
        Self {
            texture,
            bind_group,
            bind_group_layout
        }
    }
}