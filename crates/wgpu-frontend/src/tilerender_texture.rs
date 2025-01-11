use image::imageops::FilterType;
use world::{Chunk, Position, Tile};

pub struct TilerenderTexture {
    pub bind_group: wgpu::BindGroup,
    pub bind_group_layout: wgpu::BindGroupLayout,
    
    // Render Scale: 0 - 8
    // 0 = 1X, 1 = 2X, 2 = 4X, 3 = 8X, ...
    // n = 2^nX
    pub textures: Vec<wgpu::Texture>,
    pub views: Vec<wgpu::TextureView>,
    tile_sprites: TileSprites
}
struct TileSprites {
    sprite_bytes: Vec<Vec<Vec<u8>>>,
}

impl TileSprites {
    pub(crate) fn get_bytes(&self, tile: Tile, scale: usize) -> &[u8] {
        let scaled_bytes = &self.sprite_bytes[scale];
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
    pub fn new() -> Self {
        let sprite_sheet = image::load_from_memory(include_bytes!("./tiles.png")).unwrap();
        let mut sprite_images = vec![];
        let mut sprite_bytes = vec![];
        for i in 0..12 {
            let sprite = sprite_sheet.crop_imm(16 * i, 0, 16, 16);
            sprite_images.push(sprite);
        }
        let mut scale = 0;
        while 16 >> scale >= 1 {
            let mut scale_vec = vec![];
            for sprite in &sprite_images {
                let scaled = sprite.resize(16 >> scale, 16 >> scale, FilterType::CatmullRom);
                scale_vec.push(scaled.as_bytes().to_vec());
            }
            sprite_bytes.push(scale_vec);
            scale += 1;
        }

        Self {
            sprite_bytes
        }
    }
}

impl TilerenderTexture {
    const SIZE: usize = 8192;
    const ZOOM_LEVELS: usize = 5;

    pub fn new(device: &wgpu::Device) -> Self {
        let textures: Vec<_> = (0..Self::ZOOM_LEVELS).map(|_render| {
            device.create_texture(&wgpu::TextureDescriptor {
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
            })
        }).collect();
        
        let views: Vec<_> = textures.iter().map(|texture| {
            texture.create_view(&wgpu::TextureViewDescriptor::default())
        }).collect();
        
        let bind_group_entries: Vec<_> = (0..Self::ZOOM_LEVELS).map(|i| {
            wgpu::BindGroupLayoutEntry {
                binding: i as u32,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    multisampled: false,
                    view_dimension: wgpu::TextureViewDimension::D2,
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                },
                count: None,
            }
        }).collect();

        let bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &bind_group_entries,
                label: None,
            });
        
        let mut binding = 0;
        let bind_group_entries: Vec<_> = views.iter().map(|view| {
            let result = wgpu::BindGroupEntry {
                binding,
                resource: wgpu::BindingResource::TextureView(view),
            };
            binding += 1;
            result
        }).collect();

        let bind_group = device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                layout: &bind_group_layout,
                entries: &bind_group_entries,
                label: None,
            }
        );
        Self {
            textures,
            views,
            bind_group,
            bind_group_layout,
            tile_sprites: TileSprites::new(),
        }
    }

    pub fn write_tile(&self, queue: &wgpu::Queue, tile: Tile, position: Position) {
        for i in 0..Self::ZOOM_LEVELS {
            let tile_size = 16 >> i;
            let Position(x, y) = position * tile_size;
            let x = x as usize % Self::SIZE;
            let y = y as usize % Self::SIZE;
            queue.write_texture(
                wgpu::ImageCopyTexture {
                    texture: &self.textures[i],
                    mip_level: 0,
                    origin: wgpu::Origin3d {
                        x: x as u32,
                        y: y as u32,
                        z: 0,
                    },
                    aspect: wgpu::TextureAspect::All,
                },
                self.tile_sprites.get_bytes(tile, i),
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
    
    pub fn write_chunk(&self, queue: &wgpu::Queue, chunk: &Chunk) {
        // This works for scales 0-4 (16x)
        // For higher scales, we need to copy and scale the renders somehow
        for position in chunk.position.position_iter() {
            self.write_tile(queue, chunk.get_tile(position), position);
        }
    }

}