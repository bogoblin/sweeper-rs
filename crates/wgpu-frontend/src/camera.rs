use cgmath::{Vector2, Vector4, Zero};
use wgpu::util::DeviceExt;
use winit::dpi::{PhysicalPosition, PhysicalSize};
use world::Position;
use crate::{as_world_position, MouseState};
use crate::shader::HasBindGroup;

pub struct Camera {
    pub center: Vector2<f32>,
    pub zoom_level: f32,
    size: Vector2<f32>,
    drag: Option<Drag>,

    buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    bind_group_layout: wgpu::BindGroupLayout,
    uniform: CameraUniform,
}

struct Drag {
    center: Vector2<f32>,
    screen_start: Vector2<f32>
}

impl Camera {
    pub fn new(device: &wgpu::Device, size: &PhysicalSize<u32>) -> Self {
        let uniform = CameraUniform::default();
        let buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Camera Buffer"),
                contents: bytemuck::cast_slice(&[uniform]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        );
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("camera_bind_group_layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }
            ],
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("camera_bind_group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buffer.as_entire_binding(),
                }
            ]
        });

        Self {
            center: Vector2::zero(),
            zoom_level: 0.0,
            size: Vector2::new(size.width as f32, size.height as f32),
            drag: None,
            buffer,
            bind_group,
            bind_group_layout,
            uniform
        }
    }

    pub fn resize(&mut self, new_size: &PhysicalSize<u32>) {
        self.size = Vector2::new(new_size.width as f32, new_size.height as f32);
    }
    
    pub fn write_to_queue(&mut self, queue: &wgpu::Queue, offset: wgpu::BufferAddress) {
        self.uniform.world_rect = self.rect().into();
        self.uniform.tile_size = [self.tile_size(), self.tile_size(), 0.0, 0.0];
        self.uniform.zoom_render_blend = self.zoom_blend();
        queue.write_buffer(&self.buffer, offset, bytemuck::cast_slice(&[self.uniform]));
    }

    pub fn tile_size(&self) -> f32 {
        16.0 * 2.0_f32.powf(self.zoom_level / 8.0)
    }
    pub fn rect(&self) -> Vector4<f32> {
        let top_left = self.top_left();
        let bottom_right = self.bottom_right();
        Vector4::new(
            top_left.x,
            top_left.y,
            bottom_right.x,
            bottom_right.y,
        )
    }
    
    fn zoom_blend(&self) -> [f32; 8] {
        let scale = (16.0/self.tile_size()).log2();
        if scale < 0.0 {
            let mut result: [f32; 8] = Default::default();
            result[0] = 1.0;
            return result;
        }
        for lower_scale in 0..7 {
            let upper_scale = lower_scale + 1;
            if scale < upper_scale as f32 {
                let mut result: [f32; 8] = Default::default();
                result[lower_scale] = (scale - upper_scale as f32).abs();
                result[upper_scale] = 1.0 - result[lower_scale];
                return result;
            }
        }
        Default::default()
    }

    pub fn top_left(&self) -> Vector2<f32> {
        self.center - (self.size/2.0)/self.tile_size()
    }

    pub fn bottom_right(&self) -> Vector2<f32> {
        self.center + (self.size/2.0)/self.tile_size()
    }

    pub fn pan_pixels(&mut self, right: f32, down: f32) {
        self.center += Vector2::new(right, down)/self.tile_size();
    }
    pub fn zoom_around(&mut self, zoom_delta: f32, position: &PhysicalPosition<f64>) {
        let mouse_position_before_zoom = self.screen_to_world(position);
        self.zoom_level += zoom_delta;
        let mouse_position_after_zoom = self.screen_to_world(position);
        let difference = mouse_position_before_zoom - mouse_position_after_zoom;
        self.center += difference;
    }
    
    pub fn start_drag(&mut self, drag_start: &PhysicalPosition<f64>) {
        if self.drag.is_none() {
            self.drag = Some(Drag {
                center: self.center.clone(),
                screen_start: position_to_vector(drag_start.clone()),
            });
        }
    }
    pub fn end_drag(&mut self) {
        if self.drag.is_some() {
            println!("Drag ended");
        }
        self.drag = None;
    }

    pub fn update_drag(&mut self, mouse: &MouseState) {
        if let Some(drag) = &self.drag {
            let drag_vector = position_to_vector(mouse.position.clone()) - drag.screen_start;
            let drag_vector_in_world_space = drag_vector/self.tile_size();
            self.center = drag.center - drag_vector_in_world_space;
        }
    }

    pub fn screen_to_world(&self, position: &PhysicalPosition<f64>) -> Vector2<f32> {
        let position = position_to_vector(*position);
        let screen_to_center = self.size/2.0 - position;
        let distance_from_view_center_in_world_space = screen_to_center/self.tile_size();
        self.center - distance_from_view_center_in_world_space
    }
    
    pub fn world_center(&self) -> Position {
        as_world_position(self.center)
    }
}

impl HasBindGroup for Camera {
    fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }

    fn bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.bind_group_layout
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable, Default)]
struct CameraUniform {
    zoom_render_blend: [f32; 8],
    world_rect: [f32; 4],
    tile_size: [f32; 4],
}

fn position_to_vector(position: PhysicalPosition<f64>) -> Vector2<f32> {
    Vector2::new(position.x as f32, position.y as f32)
}