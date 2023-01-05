use ash::vk::{self, Offset2D};

use super::{
    app::MAX_CONCURRENT_FRAMES,
    structures::{CommandInfo, PipelineInfo, QueueFamily, SwapchainInfo},
};

pub fn create_command_pool(queue_family: &QueueFamily, device: &ash::Device) -> CommandInfo {
    let command_pool_info = vk::CommandPoolCreateInfo::builder()
        .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
        .queue_family_index(queue_family.index);

    let command_pool = unsafe { device.create_command_pool(&command_pool_info, None) }
        .expect("Failed to create command pool");

    let buffer_info = vk::CommandBufferAllocateInfo::builder()
        .command_pool(command_pool)
        .level(vk::CommandBufferLevel::PRIMARY)
        .command_buffer_count(MAX_CONCURRENT_FRAMES.into());

    let mut command_buffers = unsafe { device.allocate_command_buffers(&buffer_info) }
        .expect("Failed to allocate command buffers");

    command_buffers.reserve((MAX_CONCURRENT_FRAMES - 1).into());

    CommandInfo {
        command_pool,
        command_buffers,
    }
}

pub fn record_buffer(
    index: usize,
    pipeline_info: PipelineInfo,
    swapchain_info: SwapchainInfo,
    framebuffers: Vec<vk::Framebuffer>,
    device: &ash::Device,
    command_buffer: vk::CommandBuffer,
) {
    let buffer_begin_info = vk::CommandBufferBeginInfo::builder();
    unsafe { device.begin_command_buffer(command_buffer, &buffer_begin_info) }
        .expect("Failed to record commands");

    let render_area = vk::Rect2D::builder()
        .extent(swapchain_info.extent)
        .offset(*Offset2D::builder().x(0).y(0));

    let render_pass_info = vk::RenderPassBeginInfo::builder()
        .render_pass(pipeline_info.render_pass)
        .clear_values(&[vk::ClearValue {
            color: vk::ClearColorValue {
                int32: [0, 0, 0, 1],
            },
        }])
        .framebuffer(framebuffers[index])
        .render_area(*render_area);

    let viewport = vk::Viewport::builder()
        .x(0.0)
        .y(0.0)
        .width(swapchain_info.extent.width as f32)
        .height(swapchain_info.extent.height as f32)
        .min_depth(0.0)
        .max_depth(1.0);

    let scissor = vk::Rect2D::builder()
        .offset(*vk::Offset2D::builder().x(0).y(0))
        .extent(swapchain_info.extent);

    unsafe {
        device.cmd_begin_render_pass(
            command_buffer,
            &render_pass_info,
            vk::SubpassContents::INLINE,
        )
    };

    unsafe {
        device.cmd_bind_pipeline(
            command_buffer,
            vk::PipelineBindPoint::GRAPHICS,
            *pipeline_info
                .pipeline
                .first()
                .expect("Failed to get pipeline"),
        )
    };

    unsafe { device.cmd_set_viewport(command_buffer, 0, &[*viewport]) }
    unsafe { device.cmd_set_scissor(command_buffer, 0, &[*scissor]) };
    unsafe { device.cmd_draw(command_buffer, 3, 1, 0, 0) };

    unsafe { device.cmd_end_render_pass(command_buffer) };

    unsafe { device.end_command_buffer(command_buffer) }.expect("Failed to end command buffer");
}
