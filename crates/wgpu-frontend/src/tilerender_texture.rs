use crate::shader::HasBindGroup;
use image::imageops::FilterType;
use world::{Chunk, ChunkPosition, Position, Rect, Tile, UpdatedRect, World};
use crate::camera::Camera;

pub struct TilerenderTexture {
    bind_group: wgpu::BindGroup,
    bind_group_layout: wgpu::BindGroupLayout,
    
    // Render Scale: 0 - 8
    // 0 = 1X, 1 = 2X, 2 = 4X, 3 = 8X, ...
    // n = 2^nX
    textures: Vec<wgpu::Texture>,
    views: Vec<wgpu::TextureView>,
    tile_sprites: TileSprites,
    
    draw_areas: Vec<Rect>,
    render_pipeline: wgpu::RenderPipeline,
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
        let sprite_sheet = image::load_from_memory(include_bytes!("./tilesdark.png")).unwrap();
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
                usage: wgpu::TextureUsages::TEXTURE_BINDING
                    | wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::COPY_DST,
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
        let draw_areas = (0..Self::ZOOM_LEVELS).map(|zoom_level| Self::draw_area(Position::origin(), zoom_level))
            .collect();

        let shader = device.create_shader_module(wgpu::include_wgsl!("zoom_render.wgsl"));
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Zoom Render Pipeline Layout"),
                bind_group_layouts: &[
                ],
                push_constant_ranges: &[],
            });
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Zoom Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: textures[0].format(),
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });

        Self {
            textures,
            views,
            bind_group,
            bind_group_layout,
            tile_sprites: TileSprites::new(),
            draw_areas,
            render_pipeline,
        }
    }

    pub fn render_zoom_texture(&self, device: &wgpu::Device, queue: &wgpu::Queue) {
        // WIP: Just trying out writing to a texture :)
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: None,
        });
        {
            let render_pass_desc = wgpu::RenderPassDescriptor {
                label: Some("Zoom Render Pass"),
                color_attachments: &[
                    Some(wgpu::RenderPassColorAttachment {
                        view: &self.views[0],
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::RED),
                            store: wgpu::StoreOp::Store,
                        }
                    })
                ],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            };
            let mut render_pass = encoder.begin_render_pass(&render_pass_desc);

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.draw(0..6, 0..1);
        }
        queue.submit(std::iter::once(encoder.finish()));
    }

    pub fn write_tile(&self, queue: &wgpu::Queue, tile: Tile, position: Position) {
        for i in 0..Self::ZOOM_LEVELS {
            let draw_area = &self.draw_areas[i];
            if !draw_area.contains(position) { continue }
            
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
    
    pub fn write_updated_rect(&self, queue: &wgpu::Queue, updated_rect: &UpdatedRect) {
        for (x, col) in updated_rect.updated.iter().enumerate() {
            for (y, tile) in col.iter().enumerate() {
                if tile.0 == 0 { continue }
                let position = updated_rect.top_left + Position(x as i32, y as i32);
                self.write_tile(queue, tile.clone(), position);
            }
        }
    }
    
    pub fn write_chunk(&self, queue: &wgpu::Queue, chunk: &Chunk) {
        // This works for scales 0-4 (16x)
        // For higher scales, we need to copy and scale the renders somehow
        for position in chunk.position.position_iter() {
            self.write_tile(queue, chunk.get_tile(position), position);
        }
    }
    
    pub fn update_view_from_camera(&mut self, queue: &wgpu::Queue, camera: &Camera, world: &World) {
        let mut chunks_to_draw: Vec<ChunkPosition> = vec![];
        for scale in 0..Self::ZOOM_LEVELS {
            let (rect1, rect2) = self.update_draw_area(scale, camera.world_center());
            chunks_to_draw.append(&mut rect1.chunks_contained());
            chunks_to_draw.append(&mut rect2.chunks_contained());
        }
        
        let n_chunks = chunks_to_draw.len();
        if n_chunks > 0 {
            println!("Updating {n_chunks} chunks");
        }
        for chunk in chunks_to_draw {
            if let Some(chunk) = world.get_chunk(chunk.position()) {
                self.write_chunk(queue, chunk);
            }
        }
    }
    
    pub fn draw_area(position: Position, scale: usize) -> Rect {
        let tiles_in_texture = ((Self::SIZE / 16) << scale) as i32;
        let bits_to_zero_out = scale+4; // Make the positions end with this many zeros
        Rect::from_center_and_size(Position(
            (position.0 >> (bits_to_zero_out))<<(bits_to_zero_out),
            (position.1 >> (bits_to_zero_out))<<(bits_to_zero_out),
        ), tiles_in_texture, tiles_in_texture)
    }

    pub fn update_draw_area(&mut self, scale: usize, new_position: Position) -> (Rect, Rect) {
        let old_draw_area = &self.draw_areas[scale].clone();
        self.draw_areas[scale] = Self::draw_area(new_position, scale);
        let new_draw_area = &self.draw_areas[scale];
        
        if let Some(intersection) = &new_draw_area.intersection(old_draw_area) {
            // Difference is two Rects, one for the vertical and one for the horizontal
            let mut x_rect = new_draw_area.clone();
            if intersection.left == new_draw_area.left {
                x_rect.left = old_draw_area.right;
            }
            else if intersection.right == new_draw_area.right {
                x_rect.right = old_draw_area.left;
            }
            
            let mut y_rect = new_draw_area.clone();
            if intersection.top == new_draw_area.top {
                y_rect.top = old_draw_area.bottom;
            }
            else if intersection.bottom == new_draw_area.bottom {
                y_rect.bottom = old_draw_area.top;
            }
            (x_rect, y_rect)
        } else {
            (new_draw_area.clone(), Rect::default())
        }
    }
}

impl HasBindGroup for TilerenderTexture {
    fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }

    fn bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.bind_group_layout
    }
}