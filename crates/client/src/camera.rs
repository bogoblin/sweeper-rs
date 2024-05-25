use std::f32::consts::PI;
use cgmath::{InnerSpace, Point2, Point3, Vector2, Vector3, Zero};
use js_sys::Math::tan;
use wgpu::{BindGroup, BindGroupLayout, Buffer, Device, Queue};
use wgpu::util::DeviceExt;
use winit::dpi::{PhysicalPosition, PhysicalSize};
use winit::event::WindowEvent;
use winit::keyboard::{Key, NamedKey, SmolStr};

pub(crate) struct Camera {
    transform: Transform,
    camera_controller: CameraController,
    uniform: CameraUniform,
    buffer: Buffer,
    pub(crate) bind_group_layout: BindGroupLayout,
    pub(crate) bind_group: BindGroup,
}

struct Transform {
    eye: Point3<f32>,
    target: Point3<f32>,
    up: Vector3<f32>,
    aspect: f32,
    fovy: f32,
    znear: f32,
    zfar: f32,
}

impl Transform {
    fn build_view_projection_matrix(&self) -> cgmath::Matrix4<f32> {
        let view = cgmath::Matrix4::look_at_rh(self.eye, self.target, self.up);
        let proj = cgmath::perspective(cgmath::Deg(self.fovy), self.aspect, self.znear, self.zfar);
        return OPENGL_TO_WGPU_MATRIX * proj * view;
    }
}

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.5,
    0.0, 0.0, 0.0, 1.0,
);

impl Camera {
    pub(crate) fn new(device: &Device) -> Self {
        let uniform = CameraUniform::new();
        let buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Camera Buffer"),
                contents: bytemuck::cast_slice(&[uniform]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        );
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }
            ],
            label: Some("camera_bind_group_layout"),
        });
        let mut new_camera = Self {
            transform: Transform {
                eye: (0.0, 0.0, 20.0).into(),
                target: (0.0, 0.0, 0.0).into(),
                up: (0.0, 1.0, 0.0).into(),
                aspect: 1.0,
                fovy: 45.0,
                znear: 0.01,
                zfar: 100.0,
            },
            camera_controller: CameraController::new(0.2),
            uniform,
            bind_group: device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: buffer.as_entire_binding(),
                    }
                ],
                label: Some("camera_bind_group"),
            }),
            buffer,
            bind_group_layout,
        };
        let projection_matrix = new_camera.transform.build_view_projection_matrix();
        new_camera.uniform.view_proj = projection_matrix.into();
        new_camera
    }


    pub(crate) fn input(&mut self, event: &WindowEvent) -> bool {
        self.camera_controller.process_events(event)
    }

    pub(crate) fn update(&mut self, queue: &Queue) {
        self.camera_controller.update_transform(&mut self.transform);
        let projection_matrix = self.transform.build_view_projection_matrix();
        self.uniform.view_proj = projection_matrix.into();
        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[self.uniform]))
    }

    pub fn cursor_to_xy_plane(&self, Vector2 { x, y }: Vector2<f32>) -> Point3<f32> {
        let to_plane = self.transform.target - self.transform.eye;
        let d = to_plane.magnitude();
        let tan_fov_y = tan((self.transform.fovy * PI/180.0 * 0.5) as f64) as f32;
        let tan_fov_x = tan_fov_y * self.transform.aspect;
        let y_calc = y * tan_fov_y * d;
        let x_calc = x * tan_fov_x * d;
        self.transform.target + Vector3::new(x_calc, y_calc, 0.0) * 1.5 // idk why it's 1.5 but it works perfectly
    }

    pub fn set_aspect_ratio(&mut self, aspect_ratio: f32) {
        self.transform.aspect = aspect_ratio;
    }
}

// We need this for Rust to store our data correctly for the shaders
#[repr(C)]
// This is so we can store this in a buffer
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct CameraUniform {
    // We can't use cgmath with bytemuck directly, so we'll have
    // to convert the Matrix4 into a 4x4 f32 array
    view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
    fn new() -> Self {
        use cgmath::SquareMatrix;
        Self {
            view_proj: cgmath::Matrix4::identity().into(),
        }
    }
}

struct CameraController {
    speed: f32,
    is_forward_pressed: bool,
    is_backward_pressed: bool,
    is_left_pressed: bool,
    is_right_pressed: bool,
    is_zoom_in_pressed: bool,
    is_zoom_out_pressed: bool,
}

impl CameraController {
    fn new(speed: f32) -> Self {
        Self {
            speed,
            is_forward_pressed: false,
            is_backward_pressed: false,
            is_left_pressed: false,
            is_right_pressed: false,
            is_zoom_in_pressed: false,
            is_zoom_out_pressed: false,
        }
    }

    fn process_events(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::KeyboardInput {
                event,
                ..
            } => {
                let is_pressed = event.state.is_pressed();
                match &event.logical_key {
                    Key::Named(NamedKey::ArrowLeft) => self.is_left_pressed = is_pressed,
                    Key::Named(NamedKey::ArrowRight) => self.is_right_pressed = is_pressed,
                    Key::Named(NamedKey::ArrowUp) => self.is_forward_pressed = is_pressed,
                    Key::Named(NamedKey::ArrowDown) => self.is_backward_pressed = is_pressed,
                    Key::Named(_) => {},
                    Key::Character(char) => {
                        match char.as_str() {
                            "+" => self.is_zoom_in_pressed = is_pressed,
                            "-" => self.is_zoom_out_pressed = is_pressed,
                            _ => {}
                        }
                    },
                    Key::Unidentified(_) => {},
                    Key::Dead(_) => {},
                }
                true
            }
            _ => false,
        }
    }

    fn update_transform(&self, camera: &mut Transform) {
        use cgmath::InnerSpace;
        let forward = camera.target - camera.eye;

        let vertical = camera.up.normalize()
            * (self.is_forward_pressed as i32 - self.is_backward_pressed as i32) as f32;
        let horizontal: Vector3<f32> = forward.cross(camera.up).normalize()
            * (self.is_right_pressed as i32 - self.is_left_pressed as i32) as f32;

        camera.eye += (vertical + horizontal) * self.speed;
        camera.target += (vertical + horizontal) * self.speed;

        let zoom = (self.is_zoom_in_pressed as i32 - self.is_zoom_out_pressed as i32) as f32;
        camera.fovy += zoom * self.speed;
    }
}
