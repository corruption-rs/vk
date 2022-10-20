use ash::vk;

use super::structures::DebugInfo;

unsafe extern "system" fn debug_callback(
    message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    message_type: vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _p_user_data: *mut std::ffi::c_void,
) -> vk::Bool32 {
    let message = std::ffi::CStr::from_ptr((*p_callback_data).p_message);
    println!(
        "[{:?}] [{:?}] {}",
        message_severity,
        message_type,
        message
            .to_str()
            .expect("Failed to convert message from CStr to str")
    );
    vk::FALSE
}

pub fn create_debug(entry: &ash::Entry, instance: &ash::Instance) -> DebugInfo {
    let debug_utils = ash::extensions::ext::DebugUtils::new(&entry, &instance);
    let debug_create_info = vk::DebugUtilsMessengerCreateInfoEXT::builder()
        .pfn_user_callback(Some(debug_callback))
        .message_severity(
            vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE
                | vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
                | vk::DebugUtilsMessageSeverityFlagsEXT::INFO
                | vk::DebugUtilsMessageSeverityFlagsEXT::ERROR,
        )
        .message_type(
            vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE
                | vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
                | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION,
        );

    let messenger = unsafe { debug_utils.create_debug_utils_messenger(&debug_create_info, None) }
        .expect("Failed to create debug utils messenger");

    DebugInfo {
        loader: debug_utils,
        messenger,
    }
}
