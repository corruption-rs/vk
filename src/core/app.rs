use std::{ffi::CStr, i8};

use ash::vk;

use raw_window_handle::HasRawDisplayHandle;

#[cfg(debug_assertions)]
use crate::core::debug::create_debug;

use crate::core::{
    commands::create_command_pool, device::create_device, framebuffer::create_framebuffers,
    pipeline::create_pipeline, surface::create_surface, swapchain::create_swapchain,
    sync::create_sync,
};

#[cfg(debug_assertions)]
use super::structures::DebugInfo;

use super::{
    commands::record_buffer,
    structures::{CommandInfo, DeviceInfo, PipelineInfo, SurfaceInfo, SwapchainInfo, SyncInfo},
};

extern crate env_logger;

const APP_NAME: &'static str = "VKCR\0";
const ENGINE_NAME: &'static str = "VKCR Renderer\0";

const API_DUMP: &'static str = "VK_LAYER_LUNARG_api_dump\0";
const RENDERDOC_CAPTURE: &'static str = "VK_LAYER_RENDERDOC_Capture\0";
const VALIDATION: &'static str = "VK_LAYER_KHRONOS_validation\0";

pub struct App {
    window: winit::window::Window,
    instance: ash::Instance,
    device_info: DeviceInfo,
    surface_info: SurfaceInfo,
    swapchain_info: SwapchainInfo,
    pipeline_info: PipelineInfo,
    framebuffers: Vec<vk::Framebuffer>,
    command_info: CommandInfo,
    sync_info: SyncInfo,

    #[cfg(debug_assertions)]
    debug_info: DebugInfo,
}

impl App {
    pub fn init() {
        let mut instance_extensions: Vec<*const i8> =
            vec![ash::extensions::ext::DebugUtils::name().as_ptr()];

        let enable_api_dump = std::env::var("ENABLE_API_DUMP").unwrap_or("0".to_string());
        let enable_renderdoc_capture =
            std::env::var("ENABLE_RENDERDOC_CAPTURE").unwrap_or("0".to_string());
        let enable_validation = std::env::var("ENABLE_VALIDATION").unwrap_or("0".to_string());

        std::env::set_var("WINIT_UNIX_BACKEND", "x11");

        env_logger::init();

        let event_loop = winit::event_loop::EventLoop::new();

        let window = winit::window::WindowBuilder::new()
            .with_title("VKCR")
            .build(&event_loop)
            .expect("Failed to create window");

        let application_info = vk::ApplicationInfo::builder()
            .application_name(unsafe { &CStr::from_ptr(APP_NAME.as_ptr() as *const i8) })
            .application_version(vk::make_api_version(0, 0, 1, 0))
            .engine_name(unsafe { &CStr::from_ptr(ENGINE_NAME.as_ptr() as *const i8) })
            .engine_version(vk::make_api_version(0, 0, 1, 0))
            .api_version(vk::make_api_version(0, 1, 3, 0));

        let entry = ash::Entry::linked();

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

        let mut instance_layers: Vec<*const i8> = Vec::new();

        if enable_api_dump == "1" {
            instance_layers.push(API_DUMP.as_ptr() as *const i8);
        }

        if enable_renderdoc_capture == "1" {
            instance_layers.push(RENDERDOC_CAPTURE.as_ptr() as *const i8);
        }

        if enable_validation == "1" {
            instance_layers.push(VALIDATION.as_ptr() as *const i8);
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

        #[cfg(debug_assertions)]
        let debug_info = create_debug(&entry, &instance);

        let device_info = create_device(&instance);

        let surface_info = create_surface(&window, &entry, &instance);

        let swapchain_info = create_swapchain(device_info.clone(), surface_info.clone(), &instance);

        let pipeline_info = create_pipeline(
            &device_info.device,
            "assets/shaders/default",
            &swapchain_info.extent,
            &swapchain_info.formats,
        );

        let framebuffers = create_framebuffers(
            swapchain_info.clone(),
            pipeline_info.clone(),
            &device_info.device,
        );

        let command_info = create_command_pool(
            device_info
                .queue_families
                .first()
                .expect("Failed to get queue family"),
            &device_info.device,
        );

        let sync_info = create_sync(&device_info.device);

        let game = App {
            window,
            instance,
            device_info,
            surface_info,
            swapchain_info,
            pipeline_info,
            framebuffers,
            command_info,
            sync_info,

            #[cfg(debug_assertions)]
            debug_info,
        };

        game.run(event_loop);
    }

    fn run(mut self, event_loop: winit::event_loop::EventLoop<()>) {
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

                winit::event::Event::WindowEvent {
                    window_id,
                    event: winit::event::WindowEvent::Resized(_),
                } if window_id == self.window.id() => {
                    self.render()
                }

                winit::event::Event::RedrawRequested(_) => self.render(),

                _ => self.render(),
            }
        });
    }

    fn cleanup(&self) {
        unsafe {
            self.device_info.device.device_wait_idle()
        }.expect("Failed to wait for device idle");

        for semaphore in &self.sync_info.semaphores {
            unsafe { self.device_info.device.destroy_semaphore(*semaphore, None) }
        }

        unsafe {
            self.device_info
                .device
                .destroy_fence(self.sync_info.frame_fence, None)
        }

        unsafe {
            self.device_info
                .device
                .destroy_command_pool(self.command_info.command_pool, None)
        }

        for framebuffer in &self.framebuffers {
            unsafe {
                self.device_info
                    .device
                    .destroy_framebuffer(*framebuffer, None)
            }
        }

        for view in &self.swapchain_info.swapchain_views {
            unsafe { self.device_info.device.destroy_image_view(*view, None) }
        }

        unsafe {
            self.device_info
                .device
                .destroy_render_pass(self.pipeline_info.render_pass, None)
        }

        unsafe {
            self.device_info
                .device
                .destroy_pipeline_layout(self.pipeline_info.pipeline_layout, None)
        }

        unsafe {
            for shader_module in self.pipeline_info.shader_modules {
                self.device_info
                    .device
                    .destroy_shader_module(shader_module, None)
            }
        }

        unsafe {
            self.device_info.device.destroy_pipeline(
                *self
                    .pipeline_info
                    .pipeline
                    .first()
                    .expect("Failed to find first pipeline"),
                None,
            )
        }

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

        #[cfg(debug_assertions)]
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

    fn render(&mut self) {
        unsafe {
            self.device_info.device.wait_for_fences(
                &[self.sync_info.frame_fence],
                true,
                10000000000,
            )
        }
        .expect("Failed to wait for fences");

        unsafe {
            self.device_info
                .device
                .reset_fences(&[self.sync_info.frame_fence])
        }
        .expect("Failed reset fences");

        let index = unsafe {
            self.swapchain_info.loader.acquire_next_image(
                self.swapchain_info.swapchain,
                10000000000,
                *self
                    .sync_info
                    .semaphores
                    .first()
                    .expect("Failed to get semaphore"),
                vk::Fence::null(),
            )
        }
        .expect("Failed to acquire next image");

        unsafe {
            self.device_info.device.reset_command_buffer(
                self.command_info.command_buffer,
                vk::CommandBufferResetFlags::empty(),
            )
        }
        .expect("Failed to reset command buffer");

        record_buffer(
            index
                .0
                .try_into()
                .expect("Failed to convert index to usize from u32"),
            self.pipeline_info.clone(),
            self.swapchain_info.clone(),
            self.framebuffers.clone(),
            &self.device_info.device,
            self.command_info.clone(),
        );

        let wait_semaphore = self
            .sync_info
            .semaphores
            .first()
            .expect("Failed to get semaphore");

        let signal_semaphore = self
            .sync_info
            .semaphores
            .last()
            .expect("Failed to get semaphore");

        assert_ne!(signal_semaphore, wait_semaphore);

        let submit_info = vk::SubmitInfo::builder()
            .wait_semaphores(&[*wait_semaphore])
            .wait_dst_stage_mask(&[vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT])
            .command_buffers(&[self.command_info.command_buffer])
            .signal_semaphores(&[*signal_semaphore])
            .build();

        unsafe {
            self.device_info.device.queue_submit(
                self.device_info.queue,
                &[submit_info],
                self.sync_info.frame_fence,
            )
        }
        .expect("Failed to submit queue");

        let present_info = vk::PresentInfoKHR::builder()
            .wait_semaphores(&[*signal_semaphore])
            .swapchains(&[self.swapchain_info.swapchain])
            .image_indices(&[index.0])
            .build();

        unsafe {
            self.swapchain_info
                .loader
                .queue_present(self.device_info.queue, &present_info)
        }
        .expect("Failed to present queue");
    }
}
