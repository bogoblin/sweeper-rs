use crate::camera::Camera;
use crate::shader::HasBindGroup;
use crate::texture::Texture;
use crate::tile_sprites::TileSprites;
use wgpu::{BindGroup, BindGroupLayout, ShaderSource};
use world::{Chunk, Position, Rect, Tile, UpdatedRect};

pub struct TileMapTexture {
    tiles: Texture,
    output: Texture,
    sprites: Texture,
    render_pipeline: wgpu::RenderPipeline,
    // TODO: Update this Rect and load and unload chunks when it updates
    tile_map_area: Rect,
}

impl TileMapTexture {
    pub(crate) const SIZE: usize = 8192;
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue, camera: &Camera) -> Self {
        let tiles = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d {
                width: Self::SIZE as u32,
                height: Self::SIZE as u32,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R8Uint,
            usage: wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let view = tiles.create_view(&wgpu::TextureViewDescriptor::default());

        let bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Uint,
                        },
                        count: None,
                    },
                ],
                label: None,
            });
        let bind_group = device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                layout: &bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&view),
                    },
                ],
                label: None,
            }
        );
        let tiles = Texture {
            texture: tiles,
            view,
            sampler: None,
            bind_group,
            bind_group_layout,
        };

        let output = device.create_texture(&wgpu::TextureDescriptor {
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
        });
        let output = Texture::from_wgpu_texture(output, device).unwrap();

        let sprites = TileSprites::new().create_texture(device, queue);

        let common_shader = include_str!("common.wgsl");
        let mut wgsl_source = String::from(common_shader);
        wgsl_source.push_str(include_str!("zoom_render.wgsl"));
        let shader = device.create_shader_module(
            wgpu::ShaderModuleDescriptor {
                label: None,
                source: ShaderSource::Wgsl(wgsl_source.into()),
            }
        );
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Zoom Render Pipeline Layout"),
                bind_group_layouts: &[
                    camera.bind_group_layout(),
                    sprites.bind_group_layout(),
                    tiles.bind_group_layout(),
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
                    format: output.texture.format(),
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
            tiles,
            output,
            sprites,
            render_pipeline,
            tile_map_area: Rect::from_center_and_size(Position::origin(), Self::SIZE as i32, Self::SIZE as i32),
        }
    }

    pub fn render(&self, camera: &Camera, device: &wgpu::Device, queue: &wgpu::Queue) {
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: None,
        });
        {
            let render_pass_desc = wgpu::RenderPassDescriptor {
                label: Some("Zoom Render Pass"),
                color_attachments: &[
                    Some(wgpu::RenderPassColorAttachment {
                        view: &self.output.view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Load,
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
            render_pass.set_bind_group(0, camera.bind_group(), &[]);
            render_pass.set_bind_group(1, self.sprites.bind_group(), &[]);
            render_pass.set_bind_group(2, self.tiles.bind_group(), &[]);
            render_pass.draw(0..6, 0..1);
        }
        queue.submit(std::iter::once(encoder.finish()));
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
        if !self.tile_map_area.contains(chunk.position.position()) { return; }
        
        let Position(x, y) = chunk.position.position();
        let x = x as usize % Self::SIZE;
        let y = y as usize % Self::SIZE;
        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &self.tiles.texture,
                mip_level: 0,
                origin: wgpu::Origin3d {
                    x: x as u32,
                    y: y as u32,
                    z: 0,
                },
                aspect: wgpu::TextureAspect::All,
            },
            &chunk.tiles.bytes(),
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(16),
                rows_per_image: Some(16),
            },
            wgpu::Extent3d {
                width: 16,
                height: 16,
                depth_or_array_layers: 1,
            }
        );
    }

    pub fn write_tile(&self, queue: &wgpu::Queue, tile: Tile, position: Position) {
        if !self.tile_map_area.contains(position) { return; }
        
        let Position(x, y) = position;
        let x = x as usize % Self::SIZE;
        let y = y as usize % Self::SIZE;
        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &self.tiles.texture,
                mip_level: 0,
                origin: wgpu::Origin3d {
                    x: x as u32,
                    y: y as u32,
                    z: 0,
                },
                aspect: wgpu::TextureAspect::All,
            },
            &[tile.0],
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(1),
                rows_per_image: Some(1),
            },
            wgpu::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            }
        );
    }
}

impl HasBindGroup for TileMapTexture {
    fn bind_group(&self) -> &BindGroup {
        self.output.bind_group()
    }

    fn bind_group_layout(&self) -> &BindGroupLayout {
        self.output.bind_group_layout()
    }
}