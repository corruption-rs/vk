use ash::vk::{self, CompositeAlphaFlagsKHR, SwapchainKHR};
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};

use super::structures::{LogicalDevice, QueueFamily};

pub fn create_swapchain(
    devices: Vec<LogicalDevice>,
    queue_families: Vec<QueueFamily>,
    entry: &ash::Entry,
    instance: &ash::Instance,
    window: &winit::window::Window,
    device: &ash::Device,
) -> SwapchainKHR {
    let surface = unsafe {
        ash_window::create_surface(
            &entry,
            &instance,
            window.raw_display_handle(),
            window.raw_window_handle(),
            None,
        )
        .expect("Failed to create surface")
    };

    let surface_extension = ash::extensions::khr::Surface::new(&entry, &instance);

    let capabilities = unsafe {
        surface_extension
            .get_physical_device_surface_capabilities(devices[0].physical_device, surface)
    }
    .expect("Failed to get capabilities");

    let formats = unsafe {
        surface_extension.get_physical_device_surface_formats(devices[0].physical_device, surface)
    };

    let indices: Vec<u32> = queue_families.into_iter().map(|k| k.index).collect();

    let swapchain_create_info = vk::SwapchainCreateInfoKHR::builder()
        .surface(surface)
        .pre_transform(capabilities.current_transform)
        .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
        .image_format(formats.clone().expect("Failed to get supported formats")[0].format)
        .image_color_space(formats.clone().expect("Failed to get supported formats")[0].color_space)
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

    let loader = ash::extensions::khr::Swapchain::new(&instance, &device);

    let swapchain = unsafe {
        loader
            .create_swapchain(&swapchain_create_info, None) // crash here
            .expect("Failed to create swapchain")
    };
    swapchain
}
