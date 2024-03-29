use ash::vk;


#[derive(Clone)]
pub struct DebugInfo {
    pub loader: ash::extensions::ext::DebugUtils,
    pub messenger: vk::DebugUtilsMessengerEXT,
}

unsafe extern "system" fn debug_callback(
    _message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    _message_type: vk::DebugUtilsMessageTypeFlagsEXT,
    _p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _p_user_data: *mut std::ffi::c_void,
) -> vk::Bool32 {
    {
        let _message = std::ffi::CStr::from_ptr((*_p_callback_data).p_message);

        println!(
            "[{:?}] [{:?}] {}",
            _message_severity,
            _message_type,
            _message
                .to_str()
                .expect("Failed to convert message from CStr to str")
        );
    }
    vk::FALSE
}

pub fn create_debug(entry: &ash::Entry, instance: &ash::Instance) -> DebugInfo {
    let debug_utils = ash::extensions::ext::DebugUtils::new(entry, instance);
    let debug_create_info = vk::DebugUtilsMessengerCreateInfoEXT::builder()
        .pfn_user_callback(Some(debug_callback))
        .message_severity(
            vk::DebugUtilsMessageSeverityFlagsEXT::ERROR
                | vk::DebugUtilsMessageSeverityFlagsEXT::INFO
                | vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE
                | vk::DebugUtilsMessageSeverityFlagsEXT::WARNING,
        )
        .message_type(
            vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
                | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE
                | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION,
        );

    let messenger = unsafe { debug_utils.create_debug_utils_messenger(&debug_create_info, None) }
        .expect("Failed to create debug utils messenger");

    DebugInfo {
        loader: debug_utils,
        messenger,
    }
}
