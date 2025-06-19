use image::{DynamicImage, GenericImage};
use image::imageops::FilterType;
use log::info;
use wgpu::{BindGroup, BindGroupLayout, Queue};
use world::Tile;
use crate::shader::HasBindGroup;
use crate::texture::Texture;

#[derive(Debug)]
pub struct TileSprites {
    pub texture: Texture,
    pub dark_mode: DarkMode,
    pub filter_type: FilterType,
    sprite_mipmaps: SpriteMipmaps
}

#[derive(Clone, Debug)]
#[derive(PartialEq)]
pub enum DarkMode {
    Light,
    Dark,
}

impl TileSprites {
    pub fn set_dark_mode(&mut self, queue: &Queue, mode: DarkMode) -> DarkMode {
        if self.dark_mode != mode {
            self.dark_mode = mode;
            self.write_texture(queue);
            self.dark_mode.clone()
        } else {
            mode
        }
    }
    
    fn source_image(&self) -> &'static[u8] {
        match self.dark_mode {
            DarkMode::Light => Self::LIGHT,
            DarkMode::Dark => Self::DARK
        }
    }
    
    pub fn change_filter(&mut self, queue: &Queue) {
        self.filter_type = match self.filter_type {
            FilterType::Nearest    => FilterType::Triangle,
            FilterType::Triangle   => FilterType::CatmullRom,
            FilterType::CatmullRom => FilterType::Gaussian,
            FilterType::Gaussian   => FilterType::Lanczos3,
            FilterType::Lanczos3   => FilterType::Nearest,
        };
        self.write_texture(queue);
    }
    
    pub fn write_texture(&mut self, queue: &Queue) {
        info!("Creating texture using {:?}", self.filter_type);
        self.sprite_mipmaps = SpriteMipmaps::new(image::load_from_memory(self.source_image()).unwrap(), self.filter_type);
        let mipmaps = &self.sprite_mipmaps;
        
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
                    mipmaps.get_bytes(mip_level as usize, tile),
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
    
    const LIGHT: &'static[u8] = include_bytes!("./tiles.png");
    const DARK: &'static[u8] = include_bytes!("./tilesdark.png");
    pub fn new(device: &wgpu::Device, queue: &Queue) -> Self {
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

        let mut result = Self {
            texture, dark_mode: DarkMode::Light, filter_type: FilterType::Triangle,
            sprite_mipmaps: SpriteMipmaps { mipmaps: vec![] }
        };
        result.write_texture(queue);
        
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

#[derive(Debug)]
pub struct SpriteMipmaps {
    mipmaps: Vec<Vec<Vec<u8>>>
}

impl SpriteMipmaps {
    pub fn get_bytes(&self, mip_level: usize, tile: Tile) -> &[u8] {
        let scaled_bytes = &self.mipmaps[mip_level];
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
    
    pub fn new(sprite_sheet: DynamicImage, filter_type: FilterType) -> Self {
        let mut sprite_images = vec![];
        let mut sprite_bytes = vec![];
        for i in 0..12 {
            let sprite = sprite_sheet.crop_imm(16 * i, 0, 16, 16);
            sprite_images.push(sprite);
        }
        let mut scale = 0;
        while 16 >> scale >= 1 {
            let mut scale_vec = vec![];
            for sprite in &mut sprite_images {
                scale_vec.push(sprite.as_bytes().to_vec());
                if sprite.width() > 1 {
                    // We make a 3x3 tiling of the sprite, scale it down to 50%,
                    // then take the middle part as the scaled down sprite.
                    // This stops the tiles from being darker at the edges than they should be.
                    let w = sprite.width();
                    let h = sprite.height();
                    let mut surrounded3 = DynamicImage::new(w*3, h*3, sprite.color());
                    for ix in 0..3 {
                        for iy in 0..3 {
                            surrounded3.copy_from(sprite, ix*w, iy*h).unwrap();
                        }
                    }
                    surrounded3 = surrounded3.resize((w*3)/2, (h*3)/2, filter_type);
                    *sprite = surrounded3.crop(w/2, h/2, w/2, h/2);
                }
            }
            sprite_bytes.push(scale_vec);
            scale += 1;
        }

        Self {
            mipmaps: sprite_bytes
        }
    }
}
