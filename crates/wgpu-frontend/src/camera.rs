use crate::shader::HasBindGroup;
use crate::tilerender_texture::TileMapTexture;
use crate::{as_world_position, Fingers};
use cgmath::{Matrix, Matrix3, MetricSpace, Vector2, Vector4, Zero};
#[cfg(target_arch = "wasm32")]
use web_sys::Performance;
use wgpu::util::DeviceExt;
use winit::dpi::{PhysicalPosition, PhysicalSize};
use world::{Position, Rect};

#[derive(Debug)]
pub struct Camera {
    pub center: Vector2<f32>,
    pub zoom_level: f32,
    size: Vector2<f32>,
    drag: Option<Drag>,

    buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    bind_group_layout: wgpu::BindGroupLayout,
    uniform: CameraUniform,
    scale_factor: f64,
    #[cfg(target_arch = "wasm32")]
    performance: Performance,
    mouse_position: PhysicalPosition<f64>
}

impl Camera {
    pub(crate) fn visible_world_rect(&self) -> Rect {
        let world_rect = self.rect();
        Rect {
            left: world_rect.x.floor() as i32,
            top: world_rect.y.floor() as i32,
            right: world_rect.z.ceil() as i32,
            bottom: world_rect.w.ceil() as i32,
        }
    }
}

#[derive(Debug)]
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
            uniform,
            scale_factor: 1.0,
            #[cfg(target_arch = "wasm32")]
            performance: web_sys::window().unwrap().performance().unwrap(),
            mouse_position: Default::default(),
        }
    }

    pub fn resize(&mut self, new_size: &PhysicalSize<u32>, scale_factor: f64) {
        self.size = Vector2::new(new_size.width as f32, new_size.height as f32);
        let scale_factor_change_ratio = scale_factor / self.scale_factor;
        self.zoom_level *= scale_factor_change_ratio as f32;
        self.scale_factor = scale_factor;
    }
    
    pub fn scale(&self) -> i32 {
        (16.0/self.tile_size()).log2().floor() as i32
    }
    
    pub fn tile_map_size(&self) -> u32 {
        let scale = self.scale();
        let mut tile_map_size = if scale > 0 {
            16 >> scale
        } else {
            16
        };
        if tile_map_size < 1 { tile_map_size = 1; }
        tile_map_size
    }
    
    pub fn write_to_queue(&mut self, queue: &wgpu::Queue, offset: wgpu::BufferAddress) {
        self.uniform.world_rect = self.rect().into();
        self.uniform.tile_size = [self.tile_size(), self.tile_size(), 0.0, 0.0];
        let tile_map_size = self.tile_map_size() as f32;
        self.uniform.tile_map_size = [tile_map_size, tile_map_size, 0.0, 0.0];
        let tiles_in_texture = (TileMapTexture::SIZE/tile_map_size as usize) as i32;
        let tile_map_area = Rect::from_center_and_size(Position(
            (self.world_center().0 >> (4))<<(4),
            (self.world_center().1 >> (4))<<(4),
        ), tiles_in_texture, tiles_in_texture);
        self.uniform.tile_map_rect = [tile_map_area.left as f32, tile_map_area.top as f32, tile_map_area.right as f32, tile_map_area.bottom as f32];
        self.uniform.full_tile_map_rect = Rect::from_center_and_size(
            self.world_center().chunk_position().position(),
            TileMapTexture::SIZE as i32,
            TileMapTexture::SIZE as i32,
        );
        #[cfg(target_arch = "wasm32")] {
            self.uniform.time = self.performance.now() as i32;
        }
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
    
    pub fn zoom_around_mouse_position(&mut self, zoom_delta: f32) {
        self.zoom_around(zoom_delta, &self.mouse_position.clone());
    }
    
    pub fn start_drag(&mut self, drag_start: &PhysicalPosition<f64>) {
        if self.drag.is_none() {
            self.drag = Some(Drag {
                center: self.center.clone(),
                screen_start: position_to_vector(drag_start.clone()),
            });
        }
    }
    pub fn end_drag(&mut self, end_position: &PhysicalPosition<f64>) -> f32 {
        let distance = if let Some(drag) = &self.drag {
            println!("Drag ended");
            let end_position = Vector2::new(end_position.x as f32, end_position.y as f32);
            end_position.distance(drag.screen_start)
        } else { 0.0 };
        self.drag = None;
        distance
    }

    pub fn update_mouse_position(&mut self, mouse_position: &PhysicalPosition<f64>) {
        self.mouse_position = mouse_position.clone();
        if let Some(drag) = &self.drag {
            let drag_vector = position_to_vector(mouse_position.clone()) - drag.screen_start;
            let drag_vector_in_world_space = drag_vector/self.tile_size();
            self.center = drag.center - drag_vector_in_world_space;
        }
    }
    
    pub fn view_matrix(&self) -> Matrix3<f64> {
        let scale = (self.tile_size()/16.0) as f64;
        let center = -self.center;
        Matrix3::new(
            scale, 0.0, center.x as f64,
            0.0, scale, center.y as f64,
            0.0, 0.0, 1.0
        ).transpose() // Using transpose here so that the matrix looks correct in the code
    }
    
    /// This doesn't support arbitrary view matrices, only matrices in the form:
    /// ( s 0 -u )
    /// ( 0 s -v )
    /// ( 0 0 1 )
    pub fn set_view_matrix(&mut self, view_matrix: Matrix3<f64>) {
        let scale = view_matrix.x.x as f32;
        self.center = -1.0 * view_matrix.z.truncate().map(|c| c as f32);
        self.zoom_level = 8.0 * scale.log2();
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
    world_rect: [f32; 4],
    tile_size: [f32; 4],
    tile_map_rect: [f32; 4],
    tile_map_size: [f32; 4],
    full_tile_map_rect: Rect,
    time: i32,
    _pad: [f32; 3]
}

fn position_to_vector(position: PhysicalPosition<f64>) -> Vector2<f32> {
    Vector2::new(position.x as f32, position.y as f32)
}