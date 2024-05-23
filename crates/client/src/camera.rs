use cgmath::{Point3, Vector3};
use wgpu::{BindGroup, BindGroupLayout, Buffer, Device, Queue};
use wgpu::util::DeviceExt;
use winit::event::WindowEvent;
use winit::keyboard::{Key, NamedKey};

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
}

impl CameraController {
    fn new(speed: f32) -> Self {
        Self {
            speed,
            is_forward_pressed: false,
            is_backward_pressed: false,
            is_left_pressed: false,
            is_right_pressed: false,
        }
    }

    fn process_events(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::KeyboardInput {
                event,
                ..
            } => {
                let is_pressed = event.state.is_pressed();
                match event.logical_key {
                    Key::Named(NamedKey::ArrowLeft) => self.is_left_pressed = is_pressed,
                    Key::Named(NamedKey::ArrowRight) => self.is_right_pressed = is_pressed,
                    Key::Named(NamedKey::ArrowUp) => self.is_forward_pressed = is_pressed,
                    Key::Named(NamedKey::ArrowDown) => self.is_backward_pressed = is_pressed,
                    Key::Named(_) => {},
                    Key::Character(_) => {},
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
        let forward_norm = forward.normalize();
        let forward_mag = forward.magnitude();

        // Prevents glitching when the camera gets too close to the
        // center of the scene.
        if self.is_forward_pressed && forward_mag > self.speed {
            camera.eye += forward_norm * self.speed;
        }
        if self.is_backward_pressed {
            camera.eye -= forward_norm * self.speed;
        }

        let right = forward_norm.cross(camera.up);

        // Redo radius calc in case the forward/backward is pressed.
        let forward = camera.target - camera.eye;
        let forward_mag = forward.magnitude();

        if self.is_right_pressed {
            // Rescale the distance between the target and the eye so
            // that it doesn't change. The eye, therefore, still
            // lies on the circle made by the target and eye.
            camera.eye = camera.target - (forward + right * self.speed).normalize() * forward_mag;
        }
        if self.is_left_pressed {
            camera.eye = camera.target - (forward - right * self.speed).normalize() * forward_mag;
        }
    }
}
