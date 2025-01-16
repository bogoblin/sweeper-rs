pub trait HasBindGroup {
    fn bind_group(&self) -> &wgpu::BindGroup;
    fn bind_group_layout(&self) -> &wgpu::BindGroupLayout;
}