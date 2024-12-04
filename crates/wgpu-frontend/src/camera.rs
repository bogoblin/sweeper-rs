use wgpu::util::DeviceExt;
use winit::dpi::{PhysicalSize};

pub type WorldSpaceVec = [f32; 2];

pub struct Camera {
    pub center: WorldSpaceVec,
    pub zoom_level: f32,
    size: PhysicalSize<u32>,

    pub buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
    pub bind_group_layout: wgpu::BindGroupLayout,
    uniform: CameraUniform,
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
            center: [0.0, 0.0],
            zoom_level: 0.0,
            size: size.clone(),
            buffer,
            bind_group,
            bind_group_layout,
            uniform
        }
    }

    pub fn resize(&mut self, new_size: &PhysicalSize<u32>) {
        self.size = new_size.clone();
    }
    pub fn write_to_queue(&mut self, queue: &wgpu::Queue, offset: wgpu::BufferAddress) {
        self.uniform.top_left = self.top_left();
        self.uniform.tile_size = [self.tile_size(), self.tile_size()];
        queue.write_buffer(&self.buffer, offset, bytemuck::cast_slice(&[self.uniform]));
    }

    pub fn tile_size(&self) -> f32 {
        16.0 * 2.0_f32.powf(self.zoom_level / 8.0)
    }
    pub fn top_left(&self) -> WorldSpaceVec {
        [
            self.center[0] - (self.size.width/2) as f32 / self.tile_size(),
            self.center[1] - (self.size.height/2) as f32 / self.tile_size(),
        ]
    }

    pub fn pan_pixels(&mut self, right: f32, down: f32) {
        self.center[0] += right / self.tile_size();
        self.center[1] += down / self.tile_size();
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable, Default)]
struct CameraUniform {
    top_left: [f32; 2],
    tile_size: [f32; 2],
}
