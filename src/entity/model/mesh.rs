pub struct Mesh {
    vertex_buffer: wgpu::Buffer,
    indices_buffer: wgpu::Buffer,
}
impl Mesh {
    pub fn new(vertex_buffer: wgpu::Buffer, indices_buffer: wgpu::Buffer) -> Mesh {
        Mesh {
            vertex_buffer,
            indices_buffer,
        }
    }
}
