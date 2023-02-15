use std::{mem::{size_of, size_of_val}, ptr};

use ash::vk;

use super::{vertex::Vertex, camera::Camera};
use gpu_allocator::vulkan;

pub fn create_buffer(
    device: &ash::Device,
    allocator: &mut vulkan::Allocator,
    size: u64,
    name: &str,
    sharing_mode: vk::SharingMode,
    usage: vk::BufferUsageFlags,
    location: gpu_allocator::MemoryLocation,
) -> (vk::Buffer, vulkan::Allocation) {
    let buffer_info = vk::BufferCreateInfo::builder()
        .size(size)
        .usage(usage)
        .sharing_mode(sharing_mode);

    let buffer = unsafe { device.create_buffer(&buffer_info, None) }
        .expect(&format!("Failed to create {}", name));

    let memory_requirements = unsafe { device.get_buffer_memory_requirements(buffer) };

    let allocation = allocator
        .allocate(&vulkan::AllocationCreateDesc {
            name: &format!("{} allocation", name),
            requirements: memory_requirements,
            location,
            linear: true,
        })
        .expect("Failed to allocate");

    unsafe { device.bind_buffer_memory(buffer, allocation.memory(), allocation.offset()) }
        .expect("Failed to bind memory");

    (buffer, allocation)
}

pub fn create_vertex_buffer(
    vertices: Vec<Vertex>,
    allocator: &mut gpu_allocator::vulkan::Allocator,
    device: &ash::Device,
    command_pool: vk::CommandPool,
    queue: vk::Queue,
) -> (vk::Buffer, gpu_allocator::vulkan::Allocation) {
    let (staging_buffer, staging_allocation) = create_buffer(
        device,
        allocator,
        size_of::<Vertex>() as u64 * vertices.as_slice().to_vec().len() as u64,
        "Staging Buffer",
        vk::SharingMode::EXCLUSIVE,
        vk::BufferUsageFlags::VERTEX_BUFFER | vk::BufferUsageFlags::TRANSFER_SRC,
        gpu_allocator::MemoryLocation::CpuToGpu,
    );
    unsafe {
        ptr::copy_nonoverlapping(
            bytemuck::cast_slice(&vertices).as_ptr() as *const u8,
            staging_allocation
                .mapped_ptr()
                .expect("Failed to get pointer")
                .as_ptr() as *mut u8,
            size_of::<Vertex>() * vertices.len(),
        )
    };

    let (vertex_buffer, allocation) = create_buffer(
        device,
        allocator,
        size_of::<Vertex>() as u64 * vertices.as_slice().to_vec().len() as u64,
        "Vertex Buffer",
        vk::SharingMode::EXCLUSIVE,
        vk::BufferUsageFlags::VERTEX_BUFFER | vk::BufferUsageFlags::TRANSFER_DST,
        gpu_allocator::MemoryLocation::GpuOnly,
    );

    copy_buffer(
        &device,
        vertex_buffer,
        staging_buffer,
        size_of::<Vertex>() as u64 * vertices.as_slice().to_vec().len() as u64,
        command_pool,
        queue,
    );

    unsafe {
        device.destroy_buffer(staging_buffer, None)
    };
    

    allocator.free(staging_allocation).expect("Failed to free staging allocation");
    // unsafe {
    //     device.free_memory(staging_allocation.memory(), None)
    // };

    (vertex_buffer, allocation)
}

pub fn create_index_buffer(
    indices: Vec<u16>,
    allocator: &mut gpu_allocator::vulkan::Allocator,
    device: &ash::Device,
    command_pool: vk::CommandPool,
    queue: vk::Queue,
) -> (vk::Buffer, gpu_allocator::vulkan::Allocation) {
    let (staging_buffer, staging_allocation) = create_buffer(
        device,
        allocator,
        size_of::<u16>() as u64 * indices.as_slice().to_vec().len() as u64,
        "Staging Buffer",
        vk::SharingMode::EXCLUSIVE,
        vk::BufferUsageFlags::INDEX_BUFFER | vk::BufferUsageFlags::TRANSFER_SRC,
        gpu_allocator::MemoryLocation::CpuToGpu,
    );
    unsafe {
        ptr::copy_nonoverlapping(
            bytemuck::cast_slice(&indices).as_ptr() as *const u8,
            staging_allocation
                .mapped_ptr()
                .expect("Failed to get pointer")
                .as_ptr() as *mut u8,
            size_of::<u16>() * indices.len(),
        )
    };

    let (index_buffer, allocation) = create_buffer(
        device,
        allocator,
        size_of::<u16>() as u64 * indices.as_slice().to_vec().len() as u64,
        "Vertex Buffer",
        vk::SharingMode::EXCLUSIVE,
        vk::BufferUsageFlags::INDEX_BUFFER | vk::BufferUsageFlags::TRANSFER_DST,
        gpu_allocator::MemoryLocation::GpuOnly,
    );

    copy_buffer(
        &device,
        index_buffer,
        staging_buffer,
        size_of::<u16>() as u64 * indices.as_slice().to_vec().len() as u64,
        command_pool,
        queue,
    );

    unsafe {
        device.destroy_buffer(staging_buffer, None)
    };
    

    allocator.free(staging_allocation).expect("Failed to free staging allocation");

    (index_buffer, allocation)
}

fn copy_buffer(
    device: &ash::Device,
    src: vk::Buffer,
    dst: vk::Buffer,
    size: u64,
    command_pool: vk::CommandPool,
    queue: vk::Queue,
) {
    let allocate_info = vk::CommandBufferAllocateInfo::builder()
        .level(vk::CommandBufferLevel::PRIMARY)
        .command_pool(command_pool)
        .command_buffer_count(1);

    let command_buffer = unsafe { device.allocate_command_buffers(&allocate_info) }
        .expect("Failed to allocate command buffer");

    let begin_info =
        vk::CommandBufferBeginInfo::builder().flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

    unsafe {
        device.begin_command_buffer(
            *command_buffer
                .first()
                .expect("Failed to get command buffer"),
            &begin_info,
        )
    }
    .expect("Failed to begin command buffer");

    let copy_region = vk::BufferCopy::builder().size(size).build();

    unsafe {
        device.cmd_copy_buffer(
            *command_buffer
                .first()
                .expect("Failed to get command buffer"),
            dst,
            src,
            &[copy_region],
        )
    }

    unsafe {
        device.end_command_buffer(
            *command_buffer
                .first()
                .expect("Failed to get command buffer"),
        )
    }
    .expect("Failed to begin command buffer");

    let submit_info = vk::SubmitInfo::builder().command_buffers(&command_buffer);
    unsafe { device.queue_submit(queue, &[*submit_info], vk::Fence::null()) }
        .expect("Failed to begin command buffer");

    unsafe { device.queue_wait_idle(queue) }.expect("Failed to wait for queue idle");

    unsafe { device.free_command_buffers(command_pool, &command_buffer) };
}



pub fn create_uniform_buffer<T: bytemuck::Pod>(
    uniform_structure: T,
    allocator: &mut gpu_allocator::vulkan::Allocator,
    device: &ash::Device,
    command_pool: vk::CommandPool,
    queue: vk::Queue,
) -> (vk::Buffer, gpu_allocator::vulkan::Allocation) {

    let (staging_buffer, staging_allocation) = create_buffer(
        device,
        allocator,
        size_of_val(&uniform_structure) as u64,
        "Staging Buffer",
        vk::SharingMode::EXCLUSIVE,
        vk::BufferUsageFlags::UNIFORM_BUFFER | vk::BufferUsageFlags::TRANSFER_SRC,
        gpu_allocator::MemoryLocation::CpuToGpu,
    );

    unsafe {
        ptr::copy_nonoverlapping(
            bytemuck::cast_slice(&[uniform_structure]).as_ptr() as *const u8,
            staging_allocation
                .mapped_ptr()
                .expect("Failed to get pointer")
                .as_ptr() as *mut u8,
                size_of_val(&uniform_structure),
        )
    };

    let (uniform_buffer, allocation) = create_buffer(
        device,
        allocator,
        size_of::<Camera>() as u64,
        "Uniform Buffer",
        vk::SharingMode::EXCLUSIVE,
        vk::BufferUsageFlags::UNIFORM_BUFFER | vk::BufferUsageFlags::TRANSFER_DST,
        gpu_allocator::MemoryLocation::GpuOnly,
    );

    copy_buffer(
        &device,
        uniform_buffer,
        staging_buffer,
        size_of_val(&uniform_structure) as u64,
        command_pool,
        queue,
    );

    unsafe {
        device.destroy_buffer(staging_buffer, None)
    };
    

    allocator.free(staging_allocation).expect("Failed to free staging allocation");

    (uniform_buffer, allocation)
}

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

