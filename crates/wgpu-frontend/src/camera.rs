use std::mem::size_of;
use crate::shader::HasBindGroup;
use crate::as_world_position;
use cgmath::{Matrix, Matrix3, MetricSpace, Vector2, Vector4, Zero};
use cgmath::num_traits::ToPrimitive;
use log::trace;
#[cfg(target_arch = "wasm32")]
use web_sys::Performance;
use wgpu::BufferAddress;
use winit::dpi::{PhysicalPosition, PhysicalSize};
use world::Position;
use world::Rect;
#[cfg(target_arch = "wasm32")]
use crate::url::url::UrlInfo;

#[derive(Debug)]
pub struct Camera {
    pub center: Vector2<f64>,
    pub zoom_level: f64,
    size: Vector2<f64>,
    drag: Option<Drag>,

    buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    bind_group_layout: wgpu::BindGroupLayout,
    scale_factor: f64,
    #[cfg(target_arch = "wasm32")]
    performance: Performance,
    mouse_position: PhysicalPosition<f64>,
    #[cfg(target_arch = "wasm32")]
    url_info: UrlInfo,
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
    center: Vector2<f64>,
    screen_start: Vector2<f64>
}

impl Camera {
    pub fn new(device: &wgpu::Device, size: &PhysicalSize<u32>) -> Self {
        let mut center: Vector2<f64> = Vector2::zero();
        let zoom_level = 0.0;
        #[cfg(target_arch = "wasm32")]
        {
            let url = UrlInfo::new();
            center.x = url.get_f64("x").unwrap_or_default().clamp(i32::MIN as f64 / 2.0, i32::MAX as f64 / 2.0);
            center.y = url.get_f64("y").unwrap_or_default().clamp(i32::MIN as f64 / 2.0, i32::MAX as f64 / 2.0);
            // zoom_level = url.get_f64("zoom").unwrap_or_default().clamp(-48.0, 48.0);
        }

        let buffer = device.create_buffer(
            &wgpu::BufferDescriptor {
                label: Some("Camera Buffer"),
                size: size_of::<CameraUniform>() as BufferAddress,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
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
            center,
            zoom_level,
            size: Vector2::new(size.width as f64, size.height as f64),
            drag: None,
            buffer,
            bind_group,
            bind_group_layout,
            scale_factor: 1.0,
            #[cfg(target_arch = "wasm32")]
            performance: web_sys::window().unwrap().performance().unwrap(),
            mouse_position: Default::default(),
            #[cfg(target_arch = "wasm32")]
            url_info: UrlInfo::new(),
        }
    }

    pub fn resize(&mut self, new_size: &PhysicalSize<u32>, scale_factor: f64) {
        self.size = Vector2::new(new_size.width as f64, new_size.height as f64);
        let scale_factor_change_ratio = scale_factor / self.scale_factor;
        let new_tile_size = self.tile_size() * scale_factor_change_ratio;
        self.set_tile_size(new_tile_size);
        self.scale_factor = scale_factor;
        trace!("zoom level is {}", self.zoom_level);
        trace!("set scale factor to {}, changed by {}%", scale_factor, scale_factor_change_ratio*100.0);
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
    
    pub fn min_zoom_level(&self) -> f64 {
        Self::tile_size_to_zoom_level(0.25)
    }

    pub fn write_to_queue(&mut self, queue: &wgpu::Queue, offset: BufferAddress, texture_size: u32) {
        let tile_map_size = self.tile_map_size() as f32;
        let tiles_in_texture = (texture_size as usize/tile_map_size as usize) as i32;
        let tile_map_area = Rect::from_center_and_size(Position(
            (self.world_center().0 >> (4))<<(4),
            (self.world_center().1 >> (4))<<(4),
        ), tiles_in_texture, tiles_in_texture);
        
        // Move the top left corner of the tile map area to within [-texture_size, texture_size] to maintain precision:
        let modified_tile_map_area = tile_map_area.modulo(texture_size as i32);
        let world_offset = modified_tile_map_area.top_left() - tile_map_area.top_left();
        let world_offset_f64 = Vector2::new(
            world_offset.0.to_f64().unwrap_or_default(),
            world_offset.1.to_f64().unwrap_or_default()
        );
        
        let modified_world_rect = self.rect() + world_offset_f64
            .extend(world_offset_f64.x).extend(world_offset_f64.y);

        #[cfg(target_arch = "wasm32")]
        {
            self.url_info.set_f64("x", self.center.x);
            self.url_info.set_f64("y", self.center.y);
            // self.url_info.set_f64("zoom", self.zoom_level);
            self.url_info.update_url();
        }

        let mut uniform = CameraUniform {
            world_rect: modified_world_rect.map(|c| c as f32).into(),
            tile_size: [self.tile_size() as f32, self.tile_size() as f32, 0.0, 0.0],
            tile_map_size: [tile_map_size, tile_map_size, 0.0, 0.0],
            tile_map_rect: [modified_tile_map_area.left as f32, modified_tile_map_area.top as f32, modified_tile_map_area.right as f32, modified_tile_map_area.bottom as f32],
            full_tile_map_rect: Rect::from_center_and_size(
                self.world_center().chunk_position().position(),
                texture_size as i32,
                texture_size as i32,
            ),
            texture_size: [texture_size as i32, texture_size as i32],
            texture_size_f32: [texture_size as f32, texture_size as f32],
            ..Default::default()
        };

        #[cfg(target_arch = "wasm32")] {
            uniform.time = self.performance.now() as i32;
        }
        
        queue.write_buffer(&self.buffer, offset, bytemuck::cast_slice(&[uniform]));
    }

    pub fn tile_size(&self) -> f64 {
        Self::zoom_level_to_tile_size(self.zoom_level)
    }
    
    pub fn set_tile_size(&mut self, tile_size: f64) {
        self.set_zoom_level(Self::tile_size_to_zoom_level(tile_size));
    }
    
    fn set_zoom_level(&mut self, zoom_level: f64) {
        if zoom_level < self.min_zoom_level() {
            self.zoom_level = self.min_zoom_level();
        } else {
            self.zoom_level = zoom_level;
        }
    }
    
    fn zoom_level_to_tile_size(zoom_level: f64) -> f64 {
        16.0 * 2.0f64.powf(zoom_level / 8.0)
    }
    
    fn tile_size_to_zoom_level(tile_size: f64) -> f64 {
        8.0 * (tile_size / 16.0).log2()
    }
    
    pub fn rect(&self) -> Vector4<f64> {
        let top_left = self.center - (self.size/2.0)/self.tile_size();
        let bottom_right = self.center + (self.size/2.0)/self.tile_size();
        Vector4::new(
            top_left.x,
            top_left.y,
            bottom_right.x,
            bottom_right.y,
        )
    }
    
    pub fn zoom_around(&mut self, zoom_delta: f64, position: &PhysicalPosition<f64>) {
        let mouse_position_before_zoom = self.screen_to_world(position);
        self.set_zoom_level(self.zoom_level + zoom_delta);
        let mouse_position_after_zoom = self.screen_to_world(position);
        let difference = mouse_position_before_zoom - mouse_position_after_zoom;
        self.center += difference;
    }
    
    pub fn zoom_around_mouse_position(&mut self, zoom_delta: f64) {
        self.zoom_around(zoom_delta, &self.mouse_position.clone());
    }
    
    pub fn start_drag(&mut self, drag_start: &PhysicalPosition<f64>) {
        if self.drag.is_none() {
            self.drag = Some(Drag {
                center: self.center,
                screen_start: position_to_vector(*drag_start),
            });
        }
    }
    pub fn end_drag(&mut self, end_position: &PhysicalPosition<f64>) -> f64 {
        let distance = if let Some(drag) = &self.drag {
            println!("Drag ended");
            let end_position = Vector2::new(end_position.x, end_position.y);
            end_position.distance(drag.screen_start)
        } else { 0.0 };
        self.drag = None;
        distance
    }

    pub fn update_mouse_position(&mut self, mouse_position: &PhysicalPosition<f64>) {
        self.mouse_position = *mouse_position;
        if let Some(drag) = &self.drag {
            let drag_vector = position_to_vector(mouse_position.clone()) - drag.screen_start;
            let drag_vector_in_world_space = drag_vector/self.tile_size();
            self.center = drag.center - drag_vector_in_world_space;
        }
    }
    
    pub fn view_matrix(&self) -> Matrix3<f64> {
        CameraMatrix::new(self.tile_size(), self.center.map(|c| c)).view_matrix
    }
    
    /// This doesn't support arbitrary view matrices, only matrices in the form:
    /// ( s 0 -u )
    /// ( 0 s -v )
    /// ( 0 0 1 )
    pub fn set_view_matrix(&mut self, view_matrix: Matrix3<f64>) {
        let m = CameraMatrix::from(view_matrix);
        self.center = m.center();
        self.set_tile_size(m.tile_size());
    }
    
    pub fn screen_to_world(&self, position: &PhysicalPosition<f64>) -> Vector2<f64> {
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
    texture_size: [i32; 2],
    texture_size_f32: [f32; 2],
    time: i32,
    _pad: [f32; 3]
}

fn position_to_vector(position: PhysicalPosition<f64>) -> Vector2<f64> {
    Vector2::new(position.x, position.y)
}

pub struct CameraMatrix {
    /// Maps world space (x,y,1) to screen space, where the center of the screen is (0,0,1)
    view_matrix: Matrix3<f64>
}

/// # Example
/// 
/// ```rust
/// use cgmath::Vector2;
/// use wgpu_frontend::camera::CameraMatrix;
/// let center = Vector2::new(4.0, 4.0);
/// let m = CameraMatrix::new(16.0, center.clone());
/// let t = m.tile_size();
/// let c = m.center();
/// assert_eq!(16.0, t);
/// assert_eq!(center.clone(), c);
/// ```
impl CameraMatrix {
    pub fn from(view_matrix: Matrix3<f64>) -> Self {
        Self { view_matrix }
    }
    pub fn new(tile_size: f64, center: Vector2<f64>) -> Self {
        Self {
            view_matrix: Matrix3::new(
                tile_size, 0.0, -tile_size * center.x,
                0.0, tile_size, -tile_size * center.y,
                0.0, 0.0, 1.0
            ).transpose() // Using transpose here so that the matrix looks correct in the code
        }
    }
    pub fn tile_size(&self) -> f64 {
        self.view_matrix.x.x
    }
    pub fn center(&self) -> Vector2<f64> {
        self.view_matrix.z.truncate() / -self.tile_size()
    }
}