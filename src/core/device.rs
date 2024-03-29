use ash::vk;

#[derive(Debug, Clone)]
pub struct LogicalDevice {
    pub physical_device: vk::PhysicalDevice,
    pub properties: vk::PhysicalDeviceProperties,
    pub priority: u64,
}

#[derive(Clone)]
pub struct DeviceInfo {
    pub logical_devices: Vec<LogicalDevice>,
    pub device: ash::Device,
    pub queue_families: Vec<QueueFamily>,
    pub queue: vk::Queue,
}

#[derive(Debug, Clone)]
pub struct QueueFamily {
    pub priorities: Box<[f32]>,
    pub index: u32,
}

pub fn create_device(instance: &ash::Instance) -> DeviceInfo {
    let physical_devices = unsafe {
        instance
            .enumerate_physical_devices()
            .expect("Failed to enumerate physical devices")
    };

    let mut logical_devices: Vec<LogicalDevice> = Vec::new();
    let mut index = None;
    for physical_device in physical_devices {
        let families =
            unsafe { instance.get_physical_device_queue_family_properties(physical_device) };
        let properties = unsafe { instance.get_physical_device_properties(physical_device) };
        for (i, family) in families.iter().enumerate() {
            if family.queue_flags.contains(vk::QueueFlags::GRAPHICS) {
                let mut priority = match properties.device_type {
                    vk::PhysicalDeviceType::DISCRETE_GPU => 4,
                    vk::PhysicalDeviceType::INTEGRATED_GPU => 3,
                    vk::PhysicalDeviceType::VIRTUAL_GPU => 2,
                    _ => 1,
                };
                priority *= properties.limits.max_image_dimension1_d;
                logical_devices.push(LogicalDevice {
                    physical_device,
                    priority: priority.into(),
                    properties,
                });
                index = Some(i);
            }
        }
    }

    if logical_devices.is_empty() {
        panic!("No devices that support Vulkan were found");
    }

    for graphics_device in &logical_devices {
        debug!("{}", graphics_device);
    }

    let device_extensions: Vec<*const i8> = vec![ash::extensions::khr::Swapchain::name().as_ptr()];

    let queue_families = vec![QueueFamily {
        priorities: Box::new([1.0]),
        index: index.expect("No devices that support Vulkan were found") as u32
    }];

    let queue_create_info = vk::DeviceQueueCreateInfo::builder()
        .flags(vk::DeviceQueueCreateFlags::empty())
        .queue_family_index(queue_families[0].index)
        .queue_priorities(&queue_families[0].priorities);

    let queue_create_infos = [*queue_create_info];

    let device_create_info = vk::DeviceCreateInfo::builder()
        .enabled_extension_names(&device_extensions)
        .queue_create_infos(&queue_create_infos);

    logical_devices.sort_by_key(|v| std::cmp::Reverse(v.priority));

    let device = unsafe {
        instance.create_device(
            logical_devices[0].physical_device,
            &device_create_info,
            None,
        )
    }
    .expect("Failed to create device");

    let queue = unsafe { device.get_device_queue(queue_families[0].index, 0) };

    DeviceInfo {
        logical_devices,
        device,
        queue_families,
        queue,
    }
}

impl std::fmt::Display for LogicalDevice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let device_name = std::str::from_utf8(unsafe {
            &*(self.properties.device_name.as_slice() as *const [i8] as *const [u8])
        });

        write!(
            f,
            "Priority: {}; Device name: {}",
            self.priority,
            device_name.unwrap_or("Unknown device")
        )
    }
}