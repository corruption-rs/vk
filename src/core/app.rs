use std::{ffi::CStr, i8};

use ash::vk;

use raw_window_handle::HasRawDisplayHandle;

use crate::core::{
    debug::create_debug, device::create_device, surface::create_surface,
    swapchain::create_swapchain,
};

use super::structures::{DebugInfo, DeviceInfo, SurfaceInfo, SwapchainInfo};

extern crate env_logger;

const API_DUMP: &'static str = "VK_LAYER_LUNARG_api_dump\0";
const VALIDATION: &'static str = "VK_LAYER_KHRONOS_validation\0";

pub struct App {
    window: winit::window::Window,
    instance: ash::Instance,
    debug_info: DebugInfo,
    device_info: DeviceInfo,
    surface_info: SurfaceInfo,
    swapchain_info: SwapchainInfo,
}

const APP_NAME: &'static str = "VKCR\0";
const ENGINE_NAME: &'static str = "VKCR Renderer\0";

impl App {
    pub fn init() {
        let mut instance_extensions: Vec<*const i8> =
            vec![ash::extensions::ext::DebugUtils::name().as_ptr()];

        let enable_validation = std::env::var("ENABLE_VALIDATION").unwrap_or("0".to_string());
        let enable_api_dump = std::env::var("ENABLE_API_DUMP").unwrap_or("0".to_string());

        std::env::set_var("WINIT_UNIX_BACKEND", "x11");

        env_logger::init();

        let event_loop = winit::event_loop::EventLoop::new();

        let window = winit::window::WindowBuilder::new()
            .with_title("VKCR")
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
                "   {}",
                std::str::from_utf8(unsafe {
                    &*(layer.layer_name.as_slice() as *const [i8] as *const [u8])
                })
                .expect("Failed to create string from layer name")
            );
        }

        let mut instance_layers: Vec<*const i8> = if enable_validation == "1" {
            vec![VALIDATION.as_ptr() as *const i8]
        } else {
            vec![]
        };

        if enable_api_dump == "1" {
            instance_layers.push(API_DUMP.as_ptr() as *const i8);
        }

        for extension in ash_window::enumerate_required_extensions(window.raw_display_handle())
            .expect("Failed to enumerate required extensions")
        {
            instance_extensions.push(*extension);
        }

        let instance_create_info = vk::InstanceCreateInfo::builder()
            .application_info(&application_info)
            .enabled_extension_names(instance_extensions.as_slice())
            .enabled_layer_names(instance_layers.as_slice());

        let instance: ash::Instance = unsafe {
            entry
                .create_instance(&instance_create_info, None)
                .expect("Failed to create instance")
        };

        let debug_info = create_debug(&entry, &instance);

        let device_info = create_device(instance.clone());

        let surface_info = create_surface(&window, &entry, &instance);

        let swapchain_info = create_swapchain(device_info.clone(), surface_info.clone(), &instance);

        let game = App {
            window,
            instance,
            debug_info,
            device_info,
            surface_info,
            swapchain_info,
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

                winit::event::Event::WindowEvent {
                    window_id,
                    event: winit::event::WindowEvent::CloseRequested,
                } if window_id == self.window.id() => {
                    self.cleanup();
                    *control_flow = winit::event_loop::ControlFlow::Exit;
                }
                _ => (),
            }
        });
    }

    fn cleanup(&self) {
        unsafe {
            self.swapchain_info
                .loader
                .destroy_swapchain(self.swapchain_info.swapchain, None)
        };
        unsafe {
            self.surface_info
                .surface_loader
                .destroy_surface(self.surface_info.surface, None)
        };
        unsafe { self.device_info.device.destroy_device(None) };
        unsafe {
            self.debug_info
                .loader
                .destroy_debug_utils_messenger(self.debug_info.messenger, None)
        };
        unsafe { self.instance.destroy_instance(None) };
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

    fn _render(&self) {
        todo!()
    }
}
