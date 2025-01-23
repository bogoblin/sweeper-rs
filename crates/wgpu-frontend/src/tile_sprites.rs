use image::imageops::FilterType;
use wgpu::{BindGroup, BindGroupLayout};
use world::Tile;
use crate::shader::HasBindGroup;
use crate::texture::Texture;

pub struct TileSprites {
    pub light: Vec<Vec<Vec<u8>>>,
    pub dark: Vec<Vec<Vec<u8>>>,
    pub texture: Texture,
    pub dark_mode: DarkMode
}

#[derive(Clone, Debug)]
pub enum DarkMode {
    Light,
    Dark,
}

impl TileSprites {
    pub fn toggle_dark_mode(&mut self, queue: &wgpu::Queue) -> DarkMode {
        match self.dark_mode {
            DarkMode::Light => {
                self.dark_mode = DarkMode::Dark;
                self.write_texture(queue, true);
            }
            DarkMode::Dark => {
                self.dark_mode = DarkMode::Light;
                self.write_texture(queue, false);
            }
        }
        self.dark_mode.clone()
    }
    
    pub fn write_texture(&self, queue: &wgpu::Queue, dark_mode: bool) {
        for tile_index in 0..=255 {
            for mip_level in 0..5 {
                let x = tile_index & 0b1111;
                let y = tile_index >> 4;
                let tile = Tile(tile_index);
                let tile_size = 16 >> mip_level;

                queue.write_texture(
                    wgpu::ImageCopyTexture {
                        texture: &self.texture.texture,
                        mip_level,
                        origin: wgpu::Origin3d {
                            x: (x * tile_size) as u32,
                            y: (y * tile_size) as u32,
                            z: 0,
                        },
                        aspect: wgpu::TextureAspect::All,
                    },
                    self.get_bytes(tile, mip_level as usize, dark_mode),
                    wgpu::ImageDataLayout {
                        offset: 0,
                        bytes_per_row: Some((tile_size * 4) as u32),
                        rows_per_image: Some(tile_size as u32),
                    },
                    wgpu::Extent3d {
                        width: tile_size as u32,
                        height: tile_size as u32,
                        depth_or_array_layers: 1,
                    }
                );
            }
        }
    }
    
    pub fn get_bytes(&self, tile: Tile, scale: usize, dark_mode: bool) -> &[u8] {
        let scaled_bytes = if dark_mode { &self.dark[scale] } else { &self.light[scale] };
        if tile.is_revealed() {
            if tile.is_mine() {
                return &scaled_bytes[11];
            }
            return &scaled_bytes[tile.adjacent() as usize];
        }
        if tile.is_flag() {
            return &scaled_bytes[10];
        }
        &scaled_bytes[9]
    }
}

impl TileSprites {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue, dark_mode: bool) -> Self {
        let light = create_sprite_mipmaps(include_bytes!("./tiles.png"));
        let dark = create_sprite_mipmaps(include_bytes!("./tilesdark.png"));

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Tile Sprite Atlas"),
            size: wgpu::Extent3d {
                width: 256,
                height: 256,
                depth_or_array_layers: 1,
            },
            mip_level_count: 5,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let sampler = device.create_sampler(
            &wgpu::SamplerDescriptor {
                address_mode_u: wgpu::AddressMode::Repeat,
                address_mode_v: wgpu::AddressMode::Repeat,
                address_mode_w: wgpu::AddressMode::Repeat,
                mag_filter: wgpu::FilterMode::Nearest,
                min_filter: wgpu::FilterMode::Nearest,
                mipmap_filter: wgpu::FilterMode::Linear,
                ..Default::default()
            }
        );
        let texture = Texture::from_wgpu_texture_and_sampler(texture, sampler, device).expect("Couldn't create sprite atlas texture");

        let result = Self {
            light, dark, texture, dark_mode: DarkMode::Dark
        };
        result.write_texture(queue, dark_mode);
        
        result
    }
}

impl HasBindGroup for TileSprites {
    fn bind_group(&self) -> &BindGroup {
        &self.texture.bind_group
    }

    fn bind_group_layout(&self) -> &BindGroupLayout {
        &self.texture.bind_group_layout
    }
}

fn create_sprite_mipmaps(image_bytes: &[u8]) -> Vec<Vec<Vec<u8>>> {
    let sprite_sheet = image::load_from_memory(image_bytes).unwrap();
    let mut sprite_images = vec![];
    let mut sprite_bytes = vec![];
    for i in 0..12 {
        let sprite = sprite_sheet.crop_imm(16 * i, 0, 16, 16);
        sprite_images.push(sprite);
    }
    let mut scale = 0;
    while 16 >> scale >= 1 {
        let mut scale_vec = vec![];
        for image_index in 0..sprite_images.len() {
            let sprite = &sprite_images[image_index];
            scale_vec.push(sprite.as_bytes().to_vec());
            if sprite.width() > 1 {
                let scaled = sprite.resize(sprite.width() / 2, sprite.height() / 2, FilterType::Triangle);
                sprite_images[image_index] = scaled;
            }
        }
        sprite_bytes.push(scale_vec);
        scale += 1;
    }
    
    sprite_bytes
}
