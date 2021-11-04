pub mod model;


use std::rc::Rc;

pub struct Entity {
    pub mx_world: cgmath::Matrix4<f32>,
    pub rotation_speed: f32,
    pub color: wgpu::Color,
    pub vertex_buf: Rc<wgpu::Buffer>,
    pub index_buf: Rc<wgpu::Buffer>,
    pub index_format: wgpu::IndexFormat,
    pub index_count: usize,
    pub uniform_offset: wgpu::DynamicOffset,
}
