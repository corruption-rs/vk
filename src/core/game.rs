/**
 * Copyright (C) 2022 vkcr contributors
 *
 * This file is part of vkcr.
 *
 * vkcr is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * vkcr is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with vkcr.  If not, see <http://www.gnu.org/licenses/>.
 */
use std::{ffi::CStr, i8, ops::Deref};

use ash::vk::{self, CompositeAlphaFlagsKHR};

use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};

extern crate env_logger;

const VALIDATION: &'static str = "VK_LAYER_KHRONOS_validation\0";

pub struct Game {
    window: winit::window::Window,
    // instance: ash::Instance,
    // device: ash::Device,
    // queue: vk::Queue,
    // pipeline: vk::Pipeline,
}

#[derive(Debug)]
struct GraphicsDevice {
    device: vk::PhysicalDevice,
    properties: vk::PhysicalDeviceProperties,
    priority: u8,
}

impl std::fmt::Display for GraphicsDevice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let device_name = std::str::from_utf8(unsafe {
            &*(self.properties.device_name.as_slice() as *const [i8] as *const [u8])
        });

        write!(
            f,
            "Priority: {}, Device name: {}",
            self.priority,
            device_name.unwrap_or("Unknown device")
        )
    }
}

const APP_NAME: &'static str = "VKCR Game\0";
const ENGINE_NAME: &'static str = "VKCR Engine\0";

unsafe extern "system" fn validation_callback(
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

impl Game {
    pub fn init() {
        let mut instance_extensions: Vec<*const i8> = vec![
            ash::extensions::khr::Surface::name().as_ptr(),
            ash::extensions::ext::DebugUtils::name().as_ptr(),
        ];

        let enable_validation = std::env::var("ENABLE_VALIDATION").unwrap_or("0".to_string());

        std::env::set_var("WINIT_UNIX_BACKEND", "x11");

        env_logger::init();

        let event_loop = winit::event_loop::EventLoop::new();

        let window = winit::window::WindowBuilder::new()
            .with_title("VKCR Game Pre-Alpha")
            .build(&event_loop)
            .expect("Failed to create window");

        let entry = unsafe { ash::Entry::load().expect("Failed to load entry") };

        let application_info = vk::ApplicationInfo::builder()
            .application_name(unsafe { &CStr::from_ptr(APP_NAME.as_ptr() as *const i8) })
            .application_version(vk::make_api_version(0, 0, 1, 0))
            .engine_name(unsafe { &CStr::from_ptr(ENGINE_NAME.as_ptr() as *const i8) })
            .engine_version(vk::make_api_version(0, 0, 1, 0))
            .api_version(vk::make_api_version(0, 1, 0, 0))
            .build();

        let layers = entry
            .enumerate_instance_layer_properties()
            .expect("Failed to enumerate instance layer properties");

        debug!("Available layers: ");

        for layer in layers.iter() {
            debug!(
                "   {}",
                std::str::from_utf8(unsafe {
                    &*(layer.layer_name.as_slice() as *const [i8] as *const [u8])
                })
                .expect("Failed to create string from layer name")
            );
        }

        let instance_layers: Vec<*const i8> = if enable_validation == "1" {
            vec![VALIDATION.as_ptr() as *const i8]
        } else {
            vec![]
        };

        for extension in ash_window::enumerate_required_extensions(window.raw_display_handle())
            .expect("Failed to enumerate required extensions")
        {
            instance_extensions.push(*extension);
        }

        let instance_create_info = vk::InstanceCreateInfo::builder()
            .application_info(&application_info)
            .enabled_extension_names(instance_extensions.as_slice())
            .enabled_layer_names(instance_layers.as_slice())
            .build();

        let instance: ash::Instance = unsafe {
            entry
                .create_instance(&instance_create_info, None)
                .expect("Failed to create instance")
        };

        let debug_utils = ash::extensions::ext::DebugUtils::new(&entry, &instance);
        let debug_create_info = vk::DebugUtilsMessengerCreateInfoEXT::builder()
            .pfn_user_callback(Some(validation_callback))
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
            )
            .build();

        unsafe { debug_utils.create_debug_utils_messenger(&debug_create_info, None) }
            .expect("Failed to create debug utils messenger");

        let physical_devices = unsafe {
            instance
                .enumerate_physical_devices()
                .expect("Failed to enumerate physical devices")
        };

        let mut devices: Vec<GraphicsDevice> = Vec::new();

        for physical_device in physical_devices.clone() {
            let families =
                unsafe { instance.get_physical_device_queue_family_properties(physical_device) };
            let properties = unsafe { instance.get_physical_device_properties(physical_device) };
            for family in families.iter() {
                if family.queue_flags.contains(vk::QueueFlags::GRAPHICS) {
                    let priority = match properties.device_type {
                        vk::PhysicalDeviceType::DISCRETE_GPU => 3,
                        vk::PhysicalDeviceType::INTEGRATED_GPU => 2,
                        vk::PhysicalDeviceType::VIRTUAL_GPU => 1,
                        _ => 0,
                    };
                    devices.push(GraphicsDevice {
                        device: physical_device,
                        priority,
                        properties,
                    });
                }
            }
        }

        if devices.len() == 0 {
            panic!("No devices capable of graphics operations found.");
        }

        for device in &devices {
            debug!("{}", device);
        }

        let device_extensions: Vec<*const i8> =
            vec![ash::extensions::khr::Swapchain::name().as_ptr()];

        let queue_create_info = vk::DeviceQueueCreateInfo::builder()
            .flags(vk::DeviceQueueCreateFlags::empty())
            .build();

        let queue_create_infos = vec![queue_create_info];

        let device_create_info = vk::DeviceCreateInfo::builder()
            .enabled_layer_names(instance_layers.as_slice())
            .enabled_extension_names(&device_extensions)
            .queue_create_infos(&queue_create_infos)
            .build();

        devices.sort_by_key(|v| std::cmp::Reverse(v.priority));

        let device =
            unsafe { instance.create_device(devices[0].device, &device_create_info, None) }
                .expect("Failed to create device");

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
            surface_extension.get_physical_device_surface_capabilities(devices[0].device, surface)
        }
        .expect("Failed to get capabilities");

        let formats = unsafe {
            surface_extension.get_physical_device_surface_formats(physical_devices[0], surface)
        };

        let swapchain_create_info = vk::SwapchainCreateInfoKHR::builder()
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
            .surface(surface)
            .image_format(formats.expect("Failed to get supported formats")[0].format)
            .image_color_space(vk::ColorSpaceKHR::SRGB_NONLINEAR)
            .min_image_count(capabilities.min_image_count)
            .image_extent(capabilities.min_image_extent)
            .image_array_layers(0)
            .clipped(true)
            .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
            .composite_alpha(CompositeAlphaFlagsKHR::empty())
            .flags(vk::SwapchainCreateFlagsKHR::empty())
            .present_mode(vk::PresentModeKHR::FIFO)
            .build();

        // let swapchain = unsafe {
        //     ash::extensions::khr::Swapchain::new(&instance, &device)
        //         .create_swapchain(&swapchain_create_info, None) // segfault here
        //         .expect("Failed to create swapchain")
        // };

        let game = Game {
            window,
            // instance,
            // device,
            // queue: todo!(),
            // pipeline: todo!(),
        };

        game.run(event_loop);
    }

    fn run(self, event_loop: winit::event_loop::EventLoop<()>) {
        event_loop.run(move |event, _, control_flow| {
            *control_flow = winit::event_loop::ControlFlow::Wait;

            match event {
                winit::event::Event::WindowEvent {
                    window_id,
                    event: winit::event::WindowEvent::KeyboardInput { input, .. },
                } if window_id == self.window.id() && input.virtual_keycode.is_some() => {
                    self.handle_input(input.virtual_keycode)
                }
                _ => (),
            }
        });
    }

    fn handle_input(&self, event: Option<winit::event::VirtualKeyCode>) {
        if event.is_none() {
            return;
        }
        match event.expect("Failed to read input") {
            winit::event::VirtualKeyCode::A => {}
            _ => (),
        }
    }

    fn render(&self) {
        todo!()
    }
}
