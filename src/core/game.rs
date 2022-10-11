/**
 * Copyright (C) 2022 meisme <meisme.mail@pm.me>
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
use std::{ffi::CStr, i8};

use ash::vk;

use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};

extern crate env_logger;

#[cfg(debug_assertions)]
const VALIDATION: &'static str = "VK_LAYER_KHRONOS_validation";

#[cfg(not(debug_assertions))]
const VALIDATION: &'static str = "";

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

        write!(f, "Priority: {}, Device name: {}", self.priority, device_name.unwrap_or("Unknown device"))
    }
}

const APP_NAME: &'static str = "VKCR Game";
const ENGINE_NAME: &'static str = "VKCR Engine";

impl Game {
    pub fn init() {
        #[cfg(all(windows))]
        let extensions: Vec<*const i8> = vec![
            ash::extensions::khr::Surface::name().as_ptr(),
            ash::extensions::khr::Win32Surface::name().as_ptr(),
            ash::extensions::ext::DebugUtils::name().as_ptr(),
        ];

        #[cfg(all(
            unix,
            not(target_os = "android"),
            not(target_os = "macos"),
            not(target_os = "linux")
        ))]
        let instance_extensions: Vec<*const i8> = vec![
            ash::extensions::khr::Surface::name().as_ptr(),
            ash::extensions::khr::XlibSurface::name().as_ptr(),
            ash::extensions::ext::DebugUtils::name().as_ptr(),
        ];

        #[cfg(target_os = "linux")]
        let instance_extensions: Vec<*const i8> = vec![
            ash::extensions::khr::Surface::name().as_ptr(),
            ash::extensions::khr::XlibSurface::name().as_ptr(),
            ash::extensions::khr::WaylandSurface::name().as_ptr(),
            ash::extensions::ext::DebugUtils::name().as_ptr(),
        ];

        #[cfg(target_os = "macos")]
        let instance_extensions: Vec<*const i8> = vec![
            ash::extensions::khr::Surface::name().as_ptr(),
            ash::extensions::khr::MacOSSurface::name().as_ptr(),
            ash::extensions::ext::DebugUtils::name().as_ptr(),
        ];

        #[cfg(target_os = "android")]
        let instance_extensions: Vec<*const i8> = vec![
            ash::extensions::khr::Surface::name().as_ptr(),
            ash::extensions::khr::AndroidSurface::name().as_ptr(),
            ash::extensions::ext::DebugUtils::name().as_ptr(),
        ];

        let validation_layers: Vec<*const i8> = vec![VALIDATION.as_ptr() as *const i8];

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
            .api_version(vk::make_api_version(0, 1, 0, 0));

        let layers = entry
            .enumerate_instance_layer_properties()
            .expect("Failed to enumerate instance layer properties");

        debug!("Available layers: ");

        for layer in layers.iter() {
            debug!(
                "{}",
                std::str::from_utf8(unsafe {
                    &*(layer.layer_name.as_slice() as *const [i8] as *const [u8])
                })
                .expect("Failed to create string from layer name")
            );
        }

        let instance_create_info = vk::InstanceCreateInfo::builder()
            .application_info(&application_info)
            .enabled_extension_names(instance_extensions.as_slice());

        let instance: ash::Instance = unsafe {
            entry
                .create_instance(&instance_create_info, None)
                .expect("Failed to create instance")
        };

        let physical_devices = unsafe {
            instance
                .enumerate_physical_devices()
                .expect("Failed to enumerate physical devices")
        };

        let mut devices: Vec<GraphicsDevice> = Vec::new();

        for physical_device in physical_devices {
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

        let device_create_info = vk::DeviceCreateInfo::builder()
            .enabled_layer_names(validation_layers.as_slice())
            .enabled_extension_names(&device_extensions);

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

        let swapchain_create_info = vk::SwapchainCreateInfoKHR::builder()
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
            .surface(surface)
            .present_mode(vk::PresentModeKHR::FIFO);

        let swapchain = unsafe {
            ash::extensions::khr::Swapchain::new(&instance, &device)
                .create_swapchain(&swapchain_create_info, None)
                .expect("Failed to create swapchain")
        };

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
