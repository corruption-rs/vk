use ash::vk::{self, CompositeAlphaFlagsKHR};

use super::{device::DeviceInfo, surface::SurfaceInfo};

#[derive(Clone)]
pub struct SwapchainInfo {
    pub loader: ash::extensions::khr::Swapchain,
    pub swapchains: Vec<vk::SwapchainKHR>,
    pub swapchain_views: Vec<vk::ImageView>,
    pub formats: Vec<vk::SurfaceFormatKHR>,
    pub extent: vk::Extent2D,
    pub current_format: vk::Format,
}

pub fn create_swapchain(
    device_info: DeviceInfo,
    surface_info: SurfaceInfo,
    instance: &ash::Instance,
    window: &winit::window::Window,
    swapchains: Option<Vec<vk::SwapchainKHR>>,
) -> SwapchainInfo {
    unsafe { device_info.device.device_wait_idle() }.expect("Failed to wait for device idle");

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
    }
    .expect("Failed to get device formats");

    let indices: Vec<u32> = device_info
        .queue_families
        .into_iter()
        .map(|k| k.index)
        .collect();

    let mut new_swapchains = swapchains.clone().unwrap_or_else(|| vec![vk::SwapchainKHR::null()]);
    let last_swapchain = *swapchains
        .unwrap_or_else(|| new_swapchains.clone())
        .last()
        .unwrap_or(&vk::SwapchainKHR::null());

    let size = window.inner_size();
    let extent = vk::Extent2D::builder()
        .width(size.width)
        .height(size.height);

    let format = if formats
        .iter()
        .filter(|format| format.format == vk::Format::B8G8R8A8_SRGB)
        .map(|format| format.format)
        .next()
        .is_some()
    {
        vk::Format::B8G8R8A8_SRGB
    } else {
        formats[0].format
    };

    let swapchain_create_info = vk::SwapchainCreateInfoKHR::builder()
        .surface(surface_info.surface)
        .pre_transform(capabilities.current_transform)
        .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
        .image_format(format)
        .image_color_space(formats[0].color_space)
        .image_extent(*extent)
        .image_array_layers(1)
        .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
        .composite_alpha(CompositeAlphaFlagsKHR::OPAQUE)
        .min_image_count(capabilities.min_image_count)
        .clipped(true)
        .queue_family_indices(&indices)
        .old_swapchain(last_swapchain)
        .present_mode(vk::PresentModeKHR::IMMEDIATE);

    let loader = ash::extensions::khr::Swapchain::new(instance, &device_info.device);

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
            .base_mip_level(0)
            .layer_count(1)
            .base_array_layer(0);

        let view_create_info = vk::ImageViewCreateInfo::builder()
            .format(format)
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

    new_swapchains.push(swapchain);

    SwapchainInfo {
        swapchains: new_swapchains.to_vec(),
        loader,
        swapchain_views,
        extent: *extent,
        formats,
        current_format: format
    }
}
