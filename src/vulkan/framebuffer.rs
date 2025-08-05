use std::sync::Arc;

use vulkano::{
    image::{Image, view::ImageView},
    render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass},
};

pub fn get_framebuffers(
    images: &[Arc<Image>],
    render_pass: Arc<RenderPass>,
) -> Vec<Arc<Framebuffer>> {
    images
        .iter()
        .map(|image| {
            let view = ImageView::new_default(image.clone()).unwrap();
            Framebuffer::new(
                render_pass.clone(),
                FramebufferCreateInfo {
                    attachments: vec![view],
                    ..Default::default()
                },
            )
            .unwrap()
        })
        .collect::<Vec<_>>()
}
