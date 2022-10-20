use ash::vk;

#[derive(Debug, Clone)]
pub struct LogicalDevice {
    pub physical_device: vk::PhysicalDevice,
    pub properties: vk::PhysicalDeviceProperties,
    pub priority: u8,
}

#[derive(Debug, Clone)]
pub struct QueueFamily<'a> {
    pub priorities: &'a [f32],
    pub index: u32,
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

impl std::fmt::Display for QueueFamily<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut priority_string = String::new();
        for priority in self.priorities {
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
