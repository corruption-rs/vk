use ash::vk;

#[derive(Clone)]
pub struct DebugInfo {
    pub loader: ash::extensions::ext::DebugUtils,
    pub messenger: vk::DebugUtilsMessengerEXT,
}

#[derive(Clone)]
pub struct DeviceInfo {
    pub logical_devices: Vec<LogicalDevice>,
    pub device: ash::Device,
    pub queue_families: Vec<QueueFamily>,
}

#[derive(Debug, Clone)]
pub struct LogicalDevice {
    pub physical_device: vk::PhysicalDevice,
    pub properties: vk::PhysicalDeviceProperties,
    pub priority: u8,
}

#[derive(Debug, Clone)]
pub struct QueueFamily {
    pub priorities: Box<[f32]>,
    pub index: u32,
}

#[derive(Clone)]
pub struct SurfaceInfo {
    pub surface: vk::SurfaceKHR,
    pub surface_loader: ash::extensions::khr::Surface,
}

#[derive(Clone)]
pub struct SwapchainInfo {
    pub loader: ash::extensions::khr::Swapchain,
    pub swapchain: vk::SwapchainKHR,
    pub swapchain_views: Vec<vk::ImageView>
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

impl std::fmt::Display for QueueFamily {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut priority_string = String::new();
        for priority in self.priorities.iter() {
            priority_string.push_str(priority.to_string().as_str());
            priority_string.push_str(", ");
        }

        write!(
            f,
            "Priorities: {}; Index: {}",
            priority_string,
            self.index.to_string()
        )
    }
}
