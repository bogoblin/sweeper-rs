use image::imageops::FilterType;
use crate::texture::Texture;

pub struct TilerenderTexture {
    pub texture: Texture,
    pub bind_group: wgpu::BindGroup,
    pub bind_group_layout: wgpu::BindGroupLayout,
}

impl TilerenderTexture {
    const WIDTH: u32 = 2048;
    const HEIGHT: u32 = 2048;
    
    pub fn new(device: &wgpu::Device) -> Self {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d {
                width: Self::WIDTH,
                height: Self::HEIGHT,
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
    
    pub fn draw_image(&mut self, image: image::DynamicImage, queue: &wgpu::Queue) {
        // This actually draws the image with the wrong gamma. 
        // It doesn't really matter if I fix it or not because I'm going to use the GPU to draw to this texture later
        let resized = image.resize_exact(Self::WIDTH, Self::HEIGHT, FilterType::Nearest);
        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &self.texture.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &resized.into_rgba8(),
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(Self::WIDTH*4),
                rows_per_image: Some(Self::HEIGHT),
            },
            wgpu::Extent3d {
                width: Self::WIDTH,
                height: Self::HEIGHT,
                depth_or_array_layers: 1,
            }
        )
    }
}