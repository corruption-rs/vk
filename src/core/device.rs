use ash::vk;

use super::structures::{LogicalDevice, QueueFamily};

pub fn create_device<'a>(
    instance: ash::Instance,
) -> (Vec<LogicalDevice>, ash::Device, Vec<QueueFamily<'a>>) {
    let physical_devices = unsafe {
        instance
            .enumerate_physical_devices()
            .expect("Failed to enumerate physical devices")
    };

    let mut graphics_devices: Vec<LogicalDevice> = Vec::new();
    let mut index = None;
    for physical_device in physical_devices.clone() {
        let families =
            unsafe { instance.get_physical_device_queue_family_properties(physical_device) };
        let properties = unsafe { instance.get_physical_device_properties(physical_device) };
        let mut i = 0;
        for family in families.iter() {
            if family.queue_flags.contains(vk::QueueFlags::GRAPHICS) {
                let priority = match properties.device_type {
                    vk::PhysicalDeviceType::DISCRETE_GPU => 3,
                    vk::PhysicalDeviceType::INTEGRATED_GPU => 2,
                    vk::PhysicalDeviceType::VIRTUAL_GPU => 1,
                    _ => 0,
                };
                graphics_devices.push(LogicalDevice {
                    physical_device,
                    priority,
                    properties,
                });
                index = Some(i);
            }
            i += 1;
        }
    }

    if graphics_devices.len() == 0 {
        panic!("No devices that support Vulkan were found (are your graphics drivers up to date?)");
    }

    for graphics_device in &graphics_devices {
        debug!("{}", graphics_device);
    }

    let device_extensions: Vec<*const i8> = vec![ash::extensions::khr::Swapchain::name().as_ptr()];

    let queue_family = vec![QueueFamily {
        priorities: &[1.0],
        index: index.expect("How did you get here? There are no Vulkan capable devices on your system, but you somehow got this far")
    }];

    let queue_create_info = vk::DeviceQueueCreateInfo::builder()
        .flags(vk::DeviceQueueCreateFlags::empty())
        .queue_family_index(queue_family[0].index)
        .queue_priorities(queue_family[0].priorities)
        .build();

    let queue_create_infos = vec![queue_create_info];

    let device_create_info = vk::DeviceCreateInfo::builder()
        .enabled_extension_names(&device_extensions)
        .queue_create_infos(&queue_create_infos);

    graphics_devices.sort_by_key(|v| std::cmp::Reverse(v.priority));

    let device = unsafe {
        instance.create_device(
            graphics_devices[0].physical_device,
            &device_create_info,
            None,
        )
    }
    .expect("Failed to create device");

    (graphics_devices, device, queue_family)
}
