use ash::vk::{self, CompositeAlphaFlagsKHR};

use super::structures::{DeviceInfo, SurfaceInfo, SwapchainInfo};

pub fn create_swapchain(
    device_info: DeviceInfo,
    surface_info: SurfaceInfo,
    instance: &ash::Instance,
) -> SwapchainInfo {
    let capabilities = unsafe {
        surface_info
            .surface_loader
            .get_physical_device_surface_capabilities(
                device_info.logical_devices[0].physical_device,
                surface_info.surface,
            )
    }
    .expect("Failed to get capabilities");

    let formats = unsafe {
        surface_info
            .surface_loader
            .get_physical_device_surface_formats(
                device_info.logical_devices[0].physical_device,
                surface_info.surface,
            )
    }.expect("Failed to get device formats");

    let indices: Vec<u32> = device_info
        .queue_families
        .into_iter()
        .map(|k| k.index)
        .collect();

    let swapchain_create_info = vk::SwapchainCreateInfoKHR::builder()
        .surface(surface_info.surface)
        .pre_transform(capabilities.current_transform)
        .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
        .image_format(formats.clone()[0].format)
        .image_color_space(formats.clone()[0].color_space)
        .image_extent(capabilities.min_image_extent)
        .image_array_layers(1)
        .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
        .composite_alpha(CompositeAlphaFlagsKHR::OPAQUE)
        .min_image_count(
            2.max(capabilities.min_image_count + 1)
                .min(capabilities.max_image_count),
        )
        .clipped(true)
        .queue_family_indices(&indices)
        .present_mode(vk::PresentModeKHR::FIFO);

    let loader = ash::extensions::khr::Swapchain::new(&instance, &device_info.device);

    let swapchain = unsafe {
        loader
            .create_swapchain(&swapchain_create_info, None)
            .expect("Failed to create swapchain")
    };

    let swapchain_images =
        unsafe { loader.get_swapchain_images(swapchain) }.expect("Failed to get swapchain images");

    let mut swapchain_views: Vec<vk::ImageView> = Vec::new();

    for image in swapchain_images {
        let subresource_range = vk::ImageSubresourceRange::builder()
            .aspect_mask(vk::ImageAspectFlags::COLOR)
            .level_count(1)
            .layer_count(1);

        let view_create_info = vk::ImageViewCreateInfo::builder()
            .format(formats.clone()[0].format)
            .view_type(vk::ImageViewType::TYPE_2D)
            .subresource_range(*subresource_range)
            .image(image);

        let view = unsafe {
            device_info
                .device
                .create_image_view(&view_create_info, None)
        }
        .expect("Failed to create image view");

        swapchain_views.push(view);
    }

    SwapchainInfo { swapchain, loader, swapchain_views, extent: capabilities.current_extent, formats }
}
