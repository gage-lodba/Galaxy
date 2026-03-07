use vulkano::buffer::BufferContents;
use vulkano::pipeline::graphics::vertex_input::Vertex;

#[derive(BufferContents, Vertex)]
#[repr(C)]
pub struct StarVertex {
    #[format(R32G32_SFLOAT)]
    pub position: [f32; 2],
    #[format(R32G32B32A32_SFLOAT)]
    pub color: [f32; 4],
    /// 0.0 = normal star, 1.0 = shooting star
    /// For shooting stars: direction encoded in star_data
    #[format(R32G32B32A32_SFLOAT)]
    pub star_data: [f32; 4],
}
