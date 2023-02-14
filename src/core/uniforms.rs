use std::mem::size_of;

use ash::vk;
use gpu_allocator::vulkan;

use super::{app::MAX_CONCURRENT_FRAMES, buffers::create_buffer, camera::Camera};

fn create_descriptor_set(device: &ash::Device) -> vk::DescriptorSetLayout {
    let ubo_layout_binding = vk::DescriptorSetLayoutBinding::builder()
        .binding(0)
        .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
        .descriptor_count(1)
        .stage_flags(vk::ShaderStageFlags::VERTEX);

    let binding = [*ubo_layout_binding];
    let descriptor_create_info = vk::DescriptorSetLayoutCreateInfo::builder().bindings(&binding);

    let set_layout = unsafe { device.create_descriptor_set_layout(&descriptor_create_info, None) }
        .expect("Failed to create descriptor set layout");

    set_layout
}

