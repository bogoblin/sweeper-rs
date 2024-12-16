use image::{DynamicImage, ImageReader};
use image::imageops::FilterType;
use crate::texture::Texture;

pub struct TilerenderTexture {
    pub texture: Texture,
    pub bind_group: wgpu::BindGroup,
    pub bind_group_layout: wgpu::BindGroupLayout,
    
    // Render Scale: 0 - 8
    // 0 = 1X, 1 = 2X, 2 = 4X, 3 = 8X, ...
    // n = 2^nX
    renders: Vec<RenderBytes>
}
struct RenderBytes {
    bytes: Vec<u8>
}

impl RenderBytes {
    const LENGTH: usize = TilerenderTexture::SIZE * TilerenderTexture::SIZE * 4;
    fn new() -> Self {
        Self {
            bytes: vec![]
        }
    }

    fn tiled_with(mut self, image: DynamicImage) -> Self {
        self.bytes.resize(RenderBytes::LENGTH, 0);
        let width = image.width() as usize;
        let height = image.height() as usize;
        let img_rgba = image.to_rgba8();
        let img_rgba = img_rgba.as_ref();
        let tiles_x = width / TilerenderTexture::SIZE;
        let tiles_y = height / TilerenderTexture::SIZE;
        for row in 0..height { // Write each row in the source image...
            let img_slice_start = row * width * 4;
            let img_slice_end = img_slice_start + width * 4;
            for y_tile in 0..tiles_y { // to each instance of the tile
                let dest_row = y_tile * height + row;
                if dest_row > TilerenderTexture::SIZE { continue }
                for x_tile in 0..tiles_x {
                    // Note: This doesn't completely work if the image to tile with does not fit in to
                    // the source image an integer number of times. However, I don't care about that
                    let dest_col = x_tile * width;
                    let slice_start = 4 * (dest_row * TilerenderTexture::SIZE + dest_col);
                    if slice_start > Self::LENGTH { continue }
                    let slice_end = slice_start + width * 4;
                    if slice_end > Self::LENGTH { continue }
                    self.bytes[slice_start..slice_end].copy_from_slice(&img_rgba[img_slice_start..img_slice_end]);
                }
            }
        }
        self
    }
}

impl TilerenderTexture {
    const SIZE: usize = 1024;
    
    pub fn new(device: &wgpu::Device) -> Self {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d {
                width: Self::SIZE as u32,
                height: Self::SIZE as u32,
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

        let renders = vec![
            RenderBytes::new().tiled_with(image::load_from_memory(include_bytes!("./test_images/1024grid1.png")).expect("Couldn't load image")),
            RenderBytes::new().tiled_with(image::load_from_memory(include_bytes!("./test_images/1024grid2.png")).unwrap()),
            RenderBytes::new().tiled_with(image::load_from_memory(include_bytes!("./test_images/1024grid3.png")).unwrap()),
        ];
        
        let (bind_group, bind_group_layout) = texture.bind_group_and_layout(device);
        Self {
            texture,
            bind_group,
            bind_group_layout,
            renders
        }
    }
    
    pub fn write_renders(&self, queue: &wgpu::Queue) {
        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &self.texture.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &self.renders[0].bytes,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some((Self::SIZE * 4) as u32),
                rows_per_image: Some(Self::SIZE as u32),
            },
            wgpu::Extent3d {
                width: Self::SIZE as u32,
                height: Self::SIZE as u32,
                depth_or_array_layers: 1,
            }
        )
    }
    
    pub fn draw_image(&mut self, image: DynamicImage, queue: &wgpu::Queue) {
        // This actually draws the image with the wrong gamma. 
        // It doesn't really matter if I fix it or not because I'm going to use the GPU to draw to this texture later
        let resized = image.resize_exact(Self::SIZE as u32, Self::SIZE as u32, FilterType::Nearest);
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
                bytes_per_row: Some((Self::SIZE * 4) as u32),
                rows_per_image: Some(Self::SIZE as u32),
            },
            wgpu::Extent3d {
                width: Self::SIZE as u32,
                height: Self::SIZE as u32,
                depth_or_array_layers: 1,
            }
        )
    }
}