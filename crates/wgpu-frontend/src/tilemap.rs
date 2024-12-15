use world::{Chunk, ChunkPosition};


pub struct Tilemap {
    pub buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
    pub bind_group_layout: wgpu::BindGroupLayout,
}

impl Tilemap {
    const WIDTH: u32 = 8096;
    const HEIGHT: u32 = 8096;
    pub fn new(device: &wgpu::Device) -> Self {
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("tilemap_buffer"),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            size: (Self::WIDTH * Self::HEIGHT) as wgpu::BufferAddress,
            mapped_at_creation: false,
        });
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("tilemap_bind_group_layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None
                },
            ],
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("tilemap_bind_group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(buffer.as_entire_buffer_binding()),
                },
            ]
        });

        Self {
            buffer,
            bind_group,
            bind_group_layout
        }
    }
    ///
    ///
    /// # Arguments
    ///
    /// * `position`:
    ///
    /// returns: u64
    ///
    /// # Examples
    ///
    /// ```
    /// use crate::wgpu_frontend::tilemap::Tilemap;
    /// assert_eq!(Tilemap::chunk_address(&world::ChunkPosition::new(0, 0)), 0 as wgpu::BufferAddress);
    /// assert_eq!(Tilemap::chunk_address(&world::ChunkPosition::new(16, 0)), 256 as wgpu::BufferAddress);
    /// assert_eq!(Tilemap::chunk_address(&world::ChunkPosition::new(0, 16)), 8096*16 as wgpu::BufferAddress);
    /// ```
    pub fn chunk_address(position: &ChunkPosition) -> wgpu::BufferAddress {
        let chunkwise_x = (position.0 as u32 % Self::WIDTH)/16;
        let chunkwise_y = (position.1 as u32 % Self::HEIGHT)/16;
        (chunkwise_x * 256 + chunkwise_y * Self::WIDTH * 16) as wgpu::BufferAddress
    }
    // We're storing the tile information on the GPU
    pub fn update_chunk(&self, chunk: &Chunk, queue: &wgpu::Queue) {
        queue.write_buffer(
            &self.buffer,
            Self::chunk_address(&chunk.position),
            // 0 as wgpu::BufferAddress,
            bytemuck::cast_slice(&chunk.tiles.0)
        )
    }
}