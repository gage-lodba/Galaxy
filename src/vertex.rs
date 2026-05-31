use vulkano::buffer::BufferContents;
use vulkano::pipeline::graphics::vertex_input::Vertex;

#[derive(BufferContents, Vertex)]
#[repr(C)]
pub struct StarVertex {
    #[format(R32G32_SFLOAT)]
    pub position: [f32; 2],
    #[format(R32G32B32A32_SFLOAT)]
    pub color: [f32; 4],
    /// Per-object animation data: `[kind, seed, a, b]`.
    ///
    /// `kind` selects the celestial object type (see [`crate::galaxy::kind`]);
    /// it must match the `KIND_*` constants in the vertex shader. `seed`
    /// decorrelates per-object animation. The meaning of `a`/`b` is
    /// kind-specific (e.g. a direction vector, an orbital radius/angle, or an
    /// animation period).
    #[format(R32G32B32A32_SFLOAT)]
    pub star_data: [f32; 4],
    /// Extra kind-specific parameters `[c, d, e, f]` (e.g. trail offset,
    /// rotation speed, sprite size, or a sub-role flag).
    #[format(R32G32B32A32_SFLOAT)]
    pub extra: [f32; 4],
}
