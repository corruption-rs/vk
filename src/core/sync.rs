use ash::vk;

use super::{app::MAX_CONCURRENT_FRAMES, structures::SyncInfo};

pub fn create_sync(device: &ash::Device) -> SyncInfo {
    let semaphore_info = vk::SemaphoreCreateInfo::builder();
    let fence_info = vk::FenceCreateInfo::builder().flags(vk::FenceCreateFlags::SIGNALED);
    let mut render_semaphores = vec![vk::Semaphore::null(); MAX_CONCURRENT_FRAMES.into()];
    let mut image_semaphores = vec![vk::Semaphore::null(); MAX_CONCURRENT_FRAMES.into()];
    let mut frame_fences = vec![vk::Fence::null(); MAX_CONCURRENT_FRAMES.into()];

    for i in 0..MAX_CONCURRENT_FRAMES {
        render_semaphores[i as usize] = unsafe { device.create_semaphore(&semaphore_info, None) }
            .expect("Failed to create semaphore");
        image_semaphores[i as usize] = unsafe { device.create_semaphore(&semaphore_info, None) }
            .expect("Failed to create semaphore");
        frame_fences[i as usize] =
            unsafe { device.create_fence(&fence_info, None) }.expect("Failed to create fence");
    }

    SyncInfo {
        render_semaphores,
        image_semaphores,
        frame_fences,
    }
}
