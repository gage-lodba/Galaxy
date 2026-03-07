use std::sync::Arc;

use vulkano::{
    buffer::{BufferContents, Subbuffer},
    command_buffer::{
        AutoCommandBufferBuilder, CommandBufferUsage, PrimaryAutoCommandBuffer,
        RenderPassBeginInfo, SubpassBeginInfo, SubpassContents,
        allocator::StandardCommandBufferAllocator,
    },
    device::Queue,
    pipeline::{GraphicsPipeline, Pipeline, graphics::viewport::Viewport},
    render_pass::Framebuffer,
};

use crate::vertex::StarVertex;

#[derive(BufferContents)]
#[repr(C)]
struct PushConstants {
    time: f32,
}

pub fn get_command_buffer(
    command_buffer_allocator: &Arc<StandardCommandBufferAllocator>,
    queue: &Arc<Queue>,
    pipeline: &Arc<GraphicsPipeline>,
    framebuffer: &Arc<Framebuffer>,
    vertex_buffer: &Subbuffer<[StarVertex]>,
    viewport: &Viewport,
    time: f32,
) -> Arc<PrimaryAutoCommandBuffer> {
    let mut builder = AutoCommandBufferBuilder::primary(
        command_buffer_allocator.clone(),
        queue.queue_family_index(),
        CommandBufferUsage::OneTimeSubmit,
    )
    .unwrap();

    builder
        .begin_render_pass(
            RenderPassBeginInfo {
                clear_values: vec![Some([0.0, 0.0, 0.0, 1.0].into())],
                ..RenderPassBeginInfo::framebuffer(framebuffer.clone())
            },
            SubpassBeginInfo {
                contents: SubpassContents::Inline,
                ..Default::default()
            },
        )
        .unwrap()
        .set_viewport(0, [viewport.clone()].into_iter().collect())
        .unwrap()
        .bind_pipeline_graphics(pipeline.clone())
        .unwrap()
        .push_constants(pipeline.layout().clone(), 0, PushConstants { time })
        .unwrap()
        .bind_vertex_buffers(0, vertex_buffer.clone())
        .unwrap();

    unsafe { builder.draw(vertex_buffer.len() as u32, 1, 0, 0) }.unwrap();

    builder.end_render_pass(Default::default()).unwrap();

    builder.build().unwrap()
}
