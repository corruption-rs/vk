use ash::vk;

use super::structures::{PipelineInfo, SwapchainInfo};

pub fn create_framebuffers(
    swapchain_info: SwapchainInfo,
    pipeline_info: PipelineInfo,
    device: &ash::Device,
) -> Vec<vk::Framebuffer> {
    let mut framebuffers = Vec::<vk::Framebuffer>::new();
    framebuffers.reserve(swapchain_info.swapchain_views.len());
    for view in &swapchain_info.swapchain_views {
        let attachments = [*view];
        let framebuffer_info = vk::FramebufferCreateInfo::builder()
            .render_pass(pipeline_info.render_pass)
            .attachments(&attachments)
            .width(swapchain_info.extent.width)
            .height(swapchain_info.extent.height)
            .layers(1);

        let framebuffer = unsafe { device.create_framebuffer(&framebuffer_info, None) }
            .expect("Failed to create framebuffer");

        framebuffers.push(framebuffer);
    }
    framebuffers
}
