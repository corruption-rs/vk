use std::{ffi::CStr, i8};

use ash::vk;

use raw_window_handle::HasRawDisplayHandle;

use crate::core::{debug::create_debug, device::create_device, swapchain::create_swapchain};

extern crate env_logger;

const API_DUMP: &'static str = "VK_LAYER_LUNARG_api_dump\0";
const VALIDATION: &'static str = "VK_LAYER_KHRONOS_validation\0";

pub struct Game {
    window: winit::window::Window,
    _instance: ash::Instance,
    _device: ash::Device,
    // pipeline: vk::Pipeline,
}

const APP_NAME: &'static str = "VKCR\0";
const ENGINE_NAME: &'static str = "VKCR Renderer\0";

impl Game {
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

        create_debug(&entry, &instance);

        let (devices, device, queue_family) = create_device(instance.clone());

        let _swapchain =
            create_swapchain(devices, queue_family, &entry, &instance, &window, &device);

        let game = Game {
            window,
            _instance: instance,
            _device: device,
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

                winit::event::Event::WindowEvent {
                    window_id,
                    event: winit::event::WindowEvent::CloseRequested,
                } if window_id == self.window.id() => {
                    Game::cleanup();
                    *control_flow = winit::event_loop::ControlFlow::Exit;
                }
                _ => (),
            }
        });
    }

    fn cleanup() {}

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
