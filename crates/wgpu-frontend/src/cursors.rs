use crate::camera::Camera;
use crate::shader::HasBindGroup;
use crate::texture::Texture;
use log::info;
#[cfg(target_arch = "wasm32")]
use web_sys::Performance;
use wgpu::VertexFormat::{Float32x2, Sint32, Uint32};
use wgpu::{BufferAddress, RenderPipeline, ShaderLocation, ShaderSource, VertexBufferLayout};
use world::player::Player;

#[derive(Debug)]
pub struct Cursors {
    cursor_texture: Texture,
    colors_texture: Texture,
    cursors: [CursorInstance; Cursors::N_CURSORS],
    render_pipeline: RenderPipeline,
    instance_buffer: wgpu::Buffer,
    your_player_id: Option<String>,
    #[cfg(target_arch = "wasm32")]
    performance: Performance,
}

impl Cursors {
}

impl Cursors {
    const N_CURSORS: usize = 1024;
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue, format: wgpu::TextureFormat, camera: &Camera) -> Self {
        let cursor_texture = Texture::from_bytes(device, queue, include_bytes!("cursor.png"), "Cursor Texture").unwrap();
        let common_shader = include_str!("common.wgsl");
        let mut wgsl_source = String::from(common_shader);
        wgsl_source.push_str(include_str!("cursors.wgsl"));
        let shader = device.create_shader_module(
            wgpu::ShaderModuleDescriptor {
                label: None,
                source: ShaderSource::Wgsl(wgsl_source.into()),
            }
        );
        
        let colors_texture = Texture::from_bytes(device, queue, include_bytes!("hue-saturation.png"), "Colors Texture").unwrap();
        
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Cursors Render Pipeline Layout"),
                bind_group_layouts: &[
                    camera.bind_group_layout(),
                    cursor_texture.bind_group_layout(),
                    colors_texture.bind_group_layout(),
                ],
                push_constant_ranges: &[],
            });
        let render_pipeline = device.create_render_pipeline(
            &wgpu::RenderPipelineDescriptor {
                label: Some("Cursor Render Pipeline"),
                layout: Some(&render_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("vs_main"),
                    buffers: &[CursorInstance::desc()],
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
        let cursors = [CursorInstance::default(); Self::N_CURSORS];
        let instance_buffer = device.create_buffer(
            &wgpu::BufferDescriptor {
                label: None,
                size: (Self::N_CURSORS * size_of::<CursorInstance>()) as BufferAddress,
                usage: wgpu::BufferUsages::VERTEX |
                    wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }
        );
        Self {
            cursor_texture,
            colors_texture,
            cursors,
            render_pipeline,
            instance_buffer,
            your_player_id: None,
            #[cfg(target_arch = "wasm32")]
            performance: web_sys::window().unwrap().performance().unwrap(),
        }
    }
    
    pub fn render(&self, device: &wgpu::Device, queue: &wgpu::Queue, view: &wgpu::TextureView, camera: &Camera) {
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Cursor Render Encoder"),
        });
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Cursor Render Pass"),
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
            render_pass.set_bind_group(1, self.cursor_texture.bind_group(), &[]);
            render_pass.set_bind_group(2, self.colors_texture.bind_group(), &[]);
            render_pass.set_vertex_buffer(0, self.instance_buffer.slice(..));
            render_pass.draw(0..6, 0..self.cursors.len() as _);
        }
        queue.submit(std::iter::once(encoder.finish()));
    }
    
    pub fn set_you(&mut self, your_player_id: String, player: &Player, queue: &wgpu::Queue) {
        self.your_player_id = Some(your_player_id);
        self.update_player(player, queue);
    }
    
    pub fn update_player(&mut self, player: &Player, queue: &wgpu::Queue) {
        let index = Player::numeric_hash(&player.player_id, Self::N_CURSORS);
        let cursor_instance = &mut self.cursors[index];
        cursor_instance.prev_position = cursor_instance.position;
        cursor_instance.position = [player.position.0 as f32, player.position.1 as f32];
        if cursor_instance.prev_position.eq(&[0.0, 0.0]) {
            cursor_instance.prev_position = cursor_instance.position;
        }
        #[cfg(target_arch = "wasm32")] {
            cursor_instance.time_moved = self.performance.now() as i32;
        }
        let offset = (size_of::<CursorInstance>() * index) as BufferAddress;
        let is_you = match &self.your_player_id {
            None => false,
            Some(player_id) => {
                player.player_id.eq(player_id)
            }
        };
        if is_you {
            info!("Updating your cursor");
        }
        
        cursor_instance.properties = CursorProperties::new(is_you, &player.player_id);
        queue.write_buffer(&self.instance_buffer, offset, bytemuck::cast_slice(&self.cursors[index..index+1]));
    }
    
    pub fn delete_player(&mut self, player_id: &str, queue: &wgpu::Queue) {
        let index = Player::numeric_hash(player_id, Self::N_CURSORS);
        let offset = (size_of::<CursorInstance>() * index) as BufferAddress;
        self.cursors[index] = CursorInstance::deleted();
        queue.write_buffer(&self.instance_buffer, offset, bytemuck::cast_slice(&self.cursors[index..index+1]));
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, Default)]
#[derive(bytemuck::Pod, bytemuck::Zeroable)]
struct CursorInstance {
    position: [f32; 2],
    prev_position: [f32; 2],
    time_moved: i32,
    properties: CursorProperties,
    _pad: [f32; 2],
}

#[repr(C)]
#[derive(Debug, Copy, Clone, Default)]
#[derive(bytemuck::Pod, bytemuck::Zeroable)]
struct CursorProperties(u32);

impl CursorProperties {
    const IS_YOU: u32 = 1;
    const CONNECTED: u32 = 2;
    fn new(is_you: bool, player_id: &str) -> Self {
        let mut props = 0;
        if is_you { props |= Self::IS_YOU }
        props |= Self::CONNECTED;
        let hue = Player::numeric_hash(player_id, 256) as u32;
        props |= hue << 8;
        
        Self(props)
    }
    
    fn deleted() -> Self {
        Self(0)
    }
}

impl CursorInstance {
    const SHADER_LOCATION_OFFSET: ShaderLocation = 0;
    
    fn deleted() -> Self {
        Self {
            properties: CursorProperties::deleted(),
            ..Default::default()
        }
    }

    fn desc() -> VertexBufferLayout<'static> {
        VertexBufferLayout {
            array_stride: size_of::<CursorInstance>() as BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: Self::SHADER_LOCATION_OFFSET,
                    format: Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: size_of::<[f32; 2]>() as BufferAddress,
                    shader_location: Self::SHADER_LOCATION_OFFSET + 1,
                    format: Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: size_of::<[f32; 4]>() as BufferAddress,
                    shader_location: Self::SHADER_LOCATION_OFFSET + 2,
                    format: Sint32,
                },
                wgpu::VertexAttribute {
                    offset: size_of::<[f32; 5]>() as BufferAddress,
                    shader_location: Self::SHADER_LOCATION_OFFSET + 3,
                    format: Uint32,
                },
            ],
        }
    }
}