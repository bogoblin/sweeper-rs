use crate::camera::Camera;
use crate::shader::HasBindGroup;
use cgmath::{Vector2, Zero};
use chrono::Duration;
use std::mem;
use wgpu::VertexFormat::{Float32x2, Float32x4};
use wgpu::{BufferAddress, ShaderLocation, ShaderSource, VertexBufferLayout};
use world::compression::PublicTile;
use world::{Position, Tile};

impl Coins {
    const N_COINS: usize = 1024;
}

#[derive(Debug)]
pub struct Coins {
    pub instance_buffer: wgpu::Buffer,
    coins: Vec<Coin>,
    pub render_pipeline: wgpu::RenderPipeline,
}

impl Coins {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue, format: wgpu::TextureFormat, camera: &Camera) -> Self {
        let common_shader = include_str!("common.wgsl");
        let mut wgsl_source = String::from(common_shader);
        wgsl_source.push_str(include_str!("coins.wgsl"));
        let shader = device.create_shader_module(
            wgpu::ShaderModuleDescriptor {
                label: None,
                source: ShaderSource::Wgsl(wgsl_source.into()),
            }
        );
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Coins Render Pipeline Layout"),
                bind_group_layouts: &[
                    &camera.bind_group_layout(),
                ],
                push_constant_ranges: &[],
            });
        let render_pipeline = device.create_render_pipeline(
            &wgpu::RenderPipelineDescriptor {
                label: Some("Coins Render Pipeline"),
                layout: Some(&render_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("vs_main"),
                    buffers: &[CoinInstance::desc()],
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some("fs_main"),
                    targets: &[Some(wgpu::ColorTargetState {
                        format,
                        blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Cw,
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
            }
        );
        let instance_buffer = device.create_buffer(
            &wgpu::BufferDescriptor {
                label: None,
                size: (Self::N_COINS * mem::size_of::<CoinInstance>()) as BufferAddress,
                usage: wgpu::BufferUsages::VERTEX |
                    wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }
        );
        Self {
            instance_buffer,
            coins: vec![],
            render_pipeline,
        }
    }
    
    pub fn render(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, view: &wgpu::TextureView, camera: &Camera, time_step: Duration) {
        let mut coin_instances = [CoinInstance::default(); Self::N_COINS];
        for coin_index in 0..Self::N_COINS {
            match self.coins.get_mut(coin_index) {
                None => break,
                Some(coin) => {
                    coin.update(Vector2::unit_x(), time_step); // TODO: accelerate towards something
                    coin_instances[coin_index] = coin.instance();
                }
            }
        }
        // TODO: Optimize when there are fewer than the max coins (i.e. all the time lol)
        queue.write_buffer(
            &self.instance_buffer,
            0,
            bytemuck::cast_slice(&coin_instances),
        );
        
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Coin Render Encoder"),
        });
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Coin Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    }
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, camera.bind_group(), &[]);
            render_pass.set_vertex_buffer(0, self.instance_buffer.slice(..));
            render_pass.draw(0..6, 0..coin_instances.len() as _);
        }
        queue.submit(std::iter::once(encoder.finish()));
    }
    
    pub fn add_coin(&mut self, position: &Position, tile: &Tile) {
        self.coins.push(Coin::new(position, tile))
    }
    
}

#[derive(Debug, Copy, Clone)]
struct Coin {
    position: Vector2<f32>,
    velocity: Vector2<f32>,
    tile: Tile,
}

impl Coin {
    pub fn new(position: &Position, tile: &Tile) -> Self {
        Self {
            position: Vector2::new(position.0 as f32, position.1 as f32),
            velocity: Vector2::zero(),
            tile: tile.clone()
        }
    }
    
    fn update(&mut self, acceleration: Vector2<f32>, time_step: Duration) {
        self.velocity += acceleration * time_step.num_milliseconds() as f32 / 1000.0;
        self.position += self.velocity * time_step.num_milliseconds() as f32 / 1000.0;
    }
    
    fn instance(&self) -> CoinInstance {
        let color = match PublicTile::from(&self.tile) {
            PublicTile::Adjacent1 => Some([0.0, 0.0, 1.0, 1.0]),
            PublicTile::Adjacent2 => Some([0.0, 1.0, 0.0, 1.0]),
            PublicTile::Adjacent3 => Some([1.0, 0.0, 0.0, 1.0]),
            PublicTile::Adjacent4 => Some([1.0, 0.0, 1.0, 1.0]),
            PublicTile::Adjacent5 => Some([0.5, 0.0, 0.0, 1.0]),
            PublicTile::Adjacent6 => Some([0.0, 0.0, 1.0, 1.0]),
            PublicTile::Adjacent7 => Some([0.0, 0.0, 1.0, 1.0]),
            PublicTile::Adjacent8 => Some([0.0, 0.0, 1.0, 1.0]),
            _ => None,
        };

        match color {
            None => {
                CoinInstance::invisible()
            }
            Some(color) => {
                CoinInstance {
                    position: self.position.into(),
                    color,
                }
            }
        }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, Default)]
#[derive(bytemuck::Pod, bytemuck::Zeroable)]
struct CoinInstance {
    rect: [f32; 4],
    color: [f32; 4],
}

impl CoinInstance {
    const SHADER_LOCATION_OFFSET: ShaderLocation = 0;
    
    fn invisible() -> Self {
        Self::default()
    }
    
    fn desc() -> VertexBufferLayout<'static> {
        VertexBufferLayout {
            array_stride: mem::size_of::<CoinInstance>() as BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: Self::SHADER_LOCATION_OFFSET + 0,
                    format: Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 4]>() as BufferAddress,
                    shader_location: Self::SHADER_LOCATION_OFFSET + 1,
                    format: Float32x4,
                },
            ],
        }
    }
    
}