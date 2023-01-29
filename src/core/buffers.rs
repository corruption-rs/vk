use std::{
    mem::size_of,
    ptr,
};

use ash::vk;

use super::vertex::Vertex;
use gpu_allocator::vulkan;

pub fn create_vertex_buffer(
    vertices: Vec<Vertex>,
    device: &ash::Device,
    allocator: &mut vulkan::Allocator,
) -> (vk::Buffer, vulkan::Allocation) {
    let buffer_info = vk::BufferCreateInfo::builder()
        .size(size_of::<Vertex>() as u64 * vertices.len() as u64)
        .usage(vk::BufferUsageFlags::VERTEX_BUFFER)
        .sharing_mode(vk::SharingMode::EXCLUSIVE);

    let buffer = unsafe { device.create_buffer(&buffer_info, None) }
        .expect("Failed to create vertex buffer");

    let memory_requirements = unsafe { device.get_buffer_memory_requirements(buffer) };

    let allocation = allocator
        .allocate(&vulkan::AllocationCreateDesc {
            name: "Vertex Buffer Allocation",
            requirements: memory_requirements,
            location: gpu_allocator::MemoryLocation::CpuToGpu,
            linear: true,
        })
        .expect("Failed to allocate");

    unsafe { device.bind_buffer_memory(buffer, allocation.memory(), allocation.offset()) }
        .expect("Failed to bind memory");

    unsafe {
        ptr::copy_nonoverlapping(
            bytemuck::cast_slice(&vertices).as_ptr() as *const u8,
            allocation
                .mapped_ptr()
                .expect("Failed to get pointer")
                .as_ptr() as *mut u8,
            size_of::<Vertex>() * vertices.len(),
        )
    };

    (buffer, allocation)
}
