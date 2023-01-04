use ash::vk;

use super::structures::SyncInfo;

pub fn create_sync(device: &ash::Device) -> SyncInfo {
    let semaphore_info = vk::SemaphoreCreateInfo::builder().build();
    let fence_info = vk::FenceCreateInfo::builder()
        .flags(vk::FenceCreateFlags::SIGNALED)
        .build();
    let image_semaphore = unsafe { device.create_semaphore(&semaphore_info, None) }
        .expect("Failed to create semaphore");
    let render_semaphore = unsafe { device.create_semaphore(&semaphore_info, None) }
        .expect("Failed to create semaphore");
    let frame_fence =
        unsafe { device.create_fence(&fence_info, None) }.expect("Failed to create fence");

    SyncInfo {
        semaphores: vec![image_semaphore, render_semaphore],
        frame_fence,
    }
}
