use log::{error};
use crate::camera::Camera;
use crate::shader::HasBindGroup;
use crate::texture::Texture;
use crate::tile_sprites::TileSprites;
use wgpu::{BindGroup, BindGroupLayout, ShaderSource};
use wgpu::util::DeviceExt;
use world::{Chunk, Position, Rect, Tile, UpdatedRect};

pub struct TileMapTexture {
    tiles: Texture,
    output: Texture,
    pub sprites: TileSprites,
    zoom_render_pipeline: wgpu::RenderPipeline,
    tile_blank_pipeline: wgpu::RenderPipeline,
    blanking_rect: BlankingRect,
    tile_map_area: Rect,
    prev_camera_visible_rect: Rect,
    dirty_rect: Rect,
    texture_size: u32,
}

impl TileMapTexture {
    pub(crate) fn texture_size(&self) -> u32 {
        self.texture_size
    }
}

impl TileMapTexture {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue, camera: &Camera, texture_size: u32) -> Self {
        let tiles = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d {
                width: texture_size,
                height: texture_size,
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
        let view = tiles.create_view(&wgpu::TextureViewDescriptor {
            format: Some(tiles.format()),
            ..Default::default()
        });

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
            bind_group,
            bind_group_layout,
        };

        let output = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d {
                width: texture_size,
                height: texture_size,
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

        let sprites = TileSprites::new(&device, &queue);

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
        let zoom_render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
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

        let blanking_rect = BlankingRect::new(device);
        let common_shader = include_str!("common.wgsl");
        let mut wgsl_source = String::from(common_shader);
        wgsl_source.push_str(include_str!("tile_blanking.wgsl"));
        let shader = device.create_shader_module(
            wgpu::ShaderModuleDescriptor {
                label: None,
                source: ShaderSource::Wgsl(wgsl_source.into()),
            }
        );
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Tile Blanking Pipeline Layout"),
                bind_group_layouts: &[
                    camera.bind_group_layout(),
                    blanking_rect.bind_group_layout(),
                ],
                push_constant_ranges: &[],
            });
        let tile_blank_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Tile Blanking Pipeline"),
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
                    format: tiles.texture.format(),
                    blend: None,
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
            zoom_render_pipeline,
            tile_blank_pipeline,
            blanking_rect: BlankingRect::new(device),
            tile_map_area: Rect::default(),
            prev_camera_visible_rect: Default::default(),
            dirty_rect: Default::default(),
            texture_size,
        }
    }

    pub fn render(&mut self, camera: &Camera, device: &wgpu::Device, queue: &wgpu::Queue) {
        // If we've already rendered this view then we don't need to do it again
        if self.dirty_rect.intersection(&camera.visible_world_rect()).is_none() 
            && camera.visible_world_rect() == self.prev_camera_visible_rect {
            return;
        }
        
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

            render_pass.set_pipeline(&self.zoom_render_pipeline);
            render_pass.set_bind_group(0, camera.bind_group(), &[]);
            render_pass.set_bind_group(1, self.sprites.bind_group(), &[]);
            render_pass.set_bind_group(2, self.tiles.bind_group(), &[]);

            let tile_width = camera.tile_map_size() as i32;
            let tile_map_extent = self.texture_size as i32 / tile_width;

            // We only want to render the stuff that's going to appear on the screen.
            // This will consist of up to four rectangles because the screen area is not continuous.
            let visible_world_rect = camera.visible_world_rect();
            let visible_rect_in_pixel_space = PixelRect{
                left: visible_world_rect.left as i64,
                top: visible_world_rect.top as i64,
                right: visible_world_rect.right as i64,
                bottom: visible_world_rect.bottom as i64,
            } * tile_width as i64;
            
            // If we're looking at the whole tile map, then just render the whole thing:
            if visible_rect_in_pixel_space.width() > self.texture_size as i64 
                || visible_rect_in_pixel_space.height() > self.texture_size as i64 {
                render_pass.set_scissor_rect(0, 0, self.texture_size, self.texture_size);
                render_pass.draw(0..6, 0..1);
            } else {
                // Otherwise, only render the parts that are visible on the screen:
                let rects = self.split_rect_along_boundaries(visible_rect_in_pixel_space.modulo(self.texture_size as i64));
                for rect in rects {
                    if rect.area() == 0 { continue }
                    let mut r = rect;

                    if r.left < 0 { r.shift(self.texture_size as i64, 0) }
                    if r.top < 0 { r.shift(0, self.texture_size as i64) }
                    if r.left >= self.texture_size as i64 { r.shift(-(self.texture_size as i64), 0) }
                    if r.top >= self.texture_size as i64 { r.shift(0, -(self.texture_size as i64)) }

                    let left = r.left as u32;
                    let top = r.top as u32;
                    let right = r.width() as u32;
                    let bottom = r.height() as u32;

                    if right > self.texture_size || bottom > self.texture_size {
                        error!("Scissor Rect was constructed badly: {} {}", right, bottom);
                        error!("Visible world rect: {:?}", camera.visible_world_rect());
                        error!("Visible world rect modulo extent: {:?}", camera.visible_world_rect().modulo(tile_map_extent));
                    } else {
                        render_pass.set_scissor_rect(left, top, right, bottom);
                    }
                    render_pass.draw(0..6, 0..1);
                }
            }
            self.prev_camera_visible_rect = camera.visible_world_rect().clone();
            self.dirty_rect = Rect::default();
        }
        queue.submit(std::iter::once(encoder.finish()));
    }

    fn split_rect_along_boundaries(&self, rect: PixelRect) -> Vec<PixelRect> {
        let rects = rect.split_x(0)
            .iter().map(|r| r.split_y(0))
            .reduce(|mut acc, mut vec| {acc.append(&mut vec); acc})
            .unwrap()
            .iter().map(|r| r.split_x(self.texture_size as i64))
            .reduce(|mut acc, mut vec| {acc.append(&mut vec); acc})
            .unwrap()
            .iter().map(|r| r.split_y(self.texture_size as i64))
            .reduce(|mut acc, mut vec| {acc.append(&mut vec); acc})
            .unwrap();
        rects
    }

    pub fn blank_rect(&self, device: &wgpu::Device, queue: &wgpu::Queue, camera: &Camera, rect: Rect) {
        if rect.area() == 0 { return; }
        println!("Blanking {:?}", rect);
        self.blanking_rect.set_rect(queue, rect);

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: None,
        });
        {
            let render_pass_desc = wgpu::RenderPassDescriptor {
                label: Some("Tile Blanking Pass"),
                color_attachments: &[
                    Some(wgpu::RenderPassColorAttachment {
                        view: &self.tiles.view,
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

            render_pass.set_pipeline(&self.tile_blank_pipeline);
            render_pass.set_bind_group(0, camera.bind_group(), &[]);
            render_pass.set_bind_group(1, self.blanking_rect.bind_group(), &[]);
            render_pass.draw(0..6, 0..1);
        }
        queue.submit(std::iter::once(encoder.finish()));
    }

    pub fn write_chunk(&mut self, queue: &wgpu::Queue, chunk: &Chunk) -> Option<()> {
        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &self.tiles.texture,
                mip_level: 0,
                origin: self.position_in_tilemap(chunk.position.position())?,
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
        self.dirty_rect.expand_to_contain(chunk.rect());
        Some(())
    }

    pub fn write_tile(&mut self, queue: &wgpu::Queue, tile: Tile, position: Position) -> Option<()> {
        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &self.tiles.texture,
                mip_level: 0,
                origin: self.position_in_tilemap(position)?,
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
        self.dirty_rect.expand_to_contain(Rect {
            left: position.0,
            top: position.1,
            right: position.0 + 1,
            bottom: position.1 + 1,
        });
        Some(())
    }
    
    fn position_in_tilemap(&self, position: Position) -> Option<wgpu::Origin3d> {
        match self.tile_map_area.contains(position) {
            true => Some(wgpu::Origin3d {
                x: (position.0 as usize % self.texture_size as usize) as u32,
                y: (position.1 as usize % self.texture_size as usize) as u32,
                z: 0,
            }),
            false => None
        }
    }
    
    pub fn update_draw_area(&mut self, camera: &Camera) -> Vec<Rect> {
        let current_area = &self.tile_map_area.clone();
        self.tile_map_area = Rect::from_center_and_size(camera.world_center().chunk_position().position(), self.texture_size as i32, self.texture_size as i32);
        let new_area = &self.tile_map_area;

        if let Some(intersection) = &new_area.intersection(current_area) {
            // Difference is two Rects, one for the vertical and one for the horizontal
            let mut x_rect = new_area.clone();
            if intersection.left == new_area.left {
                x_rect.left = current_area.right;
            }
            else if intersection.right == new_area.right {
                x_rect.right = current_area.left;
            }

            let mut y_rect = new_area.clone();
            if intersection.top == new_area.top {
                y_rect.top = current_area.bottom;
            }
            else if intersection.bottom == new_area.bottom {
                y_rect.bottom = current_area.top;
            }
            vec!(x_rect, y_rect)
        } else {
            vec!(new_area.clone())
        }
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

struct BlankingRect {
    buffer: wgpu::Buffer,
    bind_group: BindGroup,
    bind_group_layout: BindGroupLayout,
}

impl BlankingRect {
    fn new(device: &wgpu::Device) -> Self {
        let rect = Rect::default();
        let buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Blanking Rect Buffer"),
                contents: bytemuck::cast_slice(&[rect]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        );
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Blanking Rect Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None
                    },
                    count: None,
                }
            ]
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Blanking Rect Bind Group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buffer.as_entire_binding(),
                }
            ],
        });
        Self {
            buffer, bind_group, bind_group_layout
        }
    }
    
    fn set_rect(&self, queue: &wgpu::Queue, rect: Rect) {
        queue.write_buffer(
            &self.buffer,
            0,
            bytemuck::cast_slice(&[rect]),
        );
    }
}

impl HasBindGroup for BlankingRect {
    fn bind_group(&self) -> &BindGroup {
        &self.bind_group
    }

    fn bind_group_layout(&self) -> &BindGroupLayout {
        &self.bind_group_layout
    }
}

use derive_more::{Add, Sub, Mul, Div};

#[derive(Debug, Clone, Add, Sub, Mul, Div)]
struct PixelRect {
    left: i64,
    top: i64,
    right: i64,
    bottom: i64,
}

impl PixelRect {
    pub fn area(&self) -> i64 {
        self.width() * self.height()
    }
    
    pub fn width(&self) -> i64 {
        self.right - self.left
    }
    
    pub fn height(&self) -> i64 {
        self.bottom - self.top
    }
    
    pub fn shift(&mut self, x: i64, y: i64) {
        self.left += x;
        self.right += x;
        self.top += y;
        self.bottom += y;
    }
    
    pub fn modulo(&self, modulo: i64) -> PixelRect {
        let mut result = self.clone();
        result.left %= modulo;
        result.top %= modulo;
        result.right = result.left + self.width();
        result.bottom = result.top + self.height();
        result
    }
    
    pub fn split_x(&self, split: i64) -> Vec<PixelRect> {
        if split > self.left && split < self.right {
            let mut r1 = self.clone();
            let mut r2 = self.clone();
            r1.right = split;
            r2.left = split;
            vec![r1, r2]
        } else {
            vec![self.clone()]
        }
    }

    pub fn split_y(&self, split: i64) -> Vec<PixelRect> {
        if split > self.top && split < self.bottom {
            let mut r1 = self.clone();
            let mut r2 = self.clone();
            r1.bottom = split;
            r2.top = split;
            vec![r1, r2]
        } else {
            vec![self.clone()]
        }
    }
}