use std::{ffi::CStr, i8};

use ash::vk;

use gpu_allocator::vulkan;
use raw_window_handle::HasRawDisplayHandle;

use crate::core::buffers::{
    create_descriptor_sets, create_index_buffer, create_uniform_buffers, create_vertex_buffer,
};
use crate::core::camera::Camera;
use crate::core::debug::create_debug;

use crate::core::geometry::{QUAD_INDICES, QUAD_VERTICES};

use crate::core::{
    commands::create_command_pool, device::create_device, framebuffer::create_framebuffers,
    pipeline::create_pipeline, surface::create_surface, swapchain::create_swapchain,
    sync::create_sync,
};

use super::buffers::Buffer;
use super::commands::CommandInfo;

use super::commands::record_buffer;
use super::debug::DebugInfo;
use super::device::DeviceInfo;
use super::pipeline::PipelineInfo;
use super::surface::SurfaceInfo;
use super::swapchain::SwapchainInfo;
use super::sync::SyncInfo;

extern crate env_logger;

const APP_NAME: &str = "VKCR\0";
const ENGINE_NAME: &str = "VKCR Renderer\0";

const API_DUMP: &str = "VK_LAYER_LUNARG_api_dump\0";
const RENDERDOC_CAPTURE: &str = "VK_LAYER_RENDERDOC_Capture\0";

const VALIDATION: &str = "VK_LAYER_KHRONOS_validation\0";

pub const MAX_CONCURRENT_FRAMES: u8 = 9;

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
    is_exiting: bool,
    current_frame: usize,
    debug_info: DebugInfo,
    allocator: Option<vulkan::Allocator>,
    buffers: Option<Vec<Buffer>>,
    descriptor_sets: Vec<vk::DescriptorSet>,
    descriptor_pool: vk::DescriptorPool,
    descriptor_set_layouts: Vec<vk::DescriptorSetLayout>,
    total_delta: f32,
}

impl App {
    pub fn init() {
        let mut instance_extensions: Vec<*const i8> =
            vec![ash::extensions::ext::DebugUtils::name().as_ptr()];

        let enable_api_dump = std::env::var("ENABLE_API_DUMP").unwrap_or_else(|_| "0".to_string());
        let enable_renderdoc_capture =
            std::env::var("ENABLE_RENDERDOC_CAPTURE").unwrap_or_else(|_| "0".to_string());
        let enable_validation =
            std::env::var("ENABLE_VALIDATION").unwrap_or_else(|_| "0".to_string());

        env_logger::init();

        let event_loop = winit::event_loop::EventLoop::new();

        let window = winit::window::WindowBuilder::new()
            .with_title("VKCR")
            .with_min_inner_size(winit::dpi::LogicalSize {
                height: 300,
                width: 300,
            })
            .build(&event_loop)
            .expect("Failed to create window");

        let application_info = vk::ApplicationInfo::builder()
            .application_name(unsafe { CStr::from_ptr(APP_NAME.as_ptr() as *const i8) })
            .application_version(vk::make_api_version(0, 0, 1, 0))
            .engine_name(unsafe { CStr::from_ptr(ENGINE_NAME.as_ptr() as *const i8) })
            .engine_version(vk::make_api_version(0, 0, 1, 0))
            .api_version(vk::make_api_version(0, 1, 3, 239));

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

        let debug_info = create_debug(&entry, &instance);

        let device_info = create_device(&instance);

        let mut allocator = vulkan::Allocator::new(&vulkan::AllocatorCreateDesc {
            instance: instance.clone(),
            device: device_info.clone().device,
            physical_device: device_info
                .logical_devices
                .first()
                .expect("Failed to get first logical device")
                .physical_device,
            debug_settings: Default::default(),
            buffer_device_address: false,
        })
        .expect("Failed to create allocator");

        let surface_info = create_surface(&window, &entry, &instance);

        let swapchain_info = create_swapchain(
            device_info.clone(),
            surface_info.clone(),
            &instance,
            &window,
            None,
        );

        let command_info = create_command_pool(
            device_info
                .queue_families
                .first()
                .expect("Failed to get queue family"),
            &device_info.device,
        );

        let uniform_buffers = create_uniform_buffers(
            Camera {
                model: cgmath::Matrix4::from_scale(1.0),
                view: cgmath::Matrix4::from_scale(1.0),
                proj: cgmath::Matrix4::from_scale(1.0),
            },
            &mut allocator,
            &device_info.device,
            command_info.command_pool,
            device_info.queue,
        );

        let (descriptor_sets, descriptor_pool, descriptor_set_layouts) =
            create_descriptor_sets(&device_info.device, &uniform_buffers, Camera::default());

        let pipeline_info = create_pipeline(
            &device_info.device,
            "assets/shaders/default",
            &swapchain_info.extent,
            swapchain_info.current_format,
            &descriptor_set_layouts,
        );

        let framebuffers = create_framebuffers(
            swapchain_info.clone(),
            pipeline_info.clone(),
            &device_info.device,
        );

        let mut buffers = Vec::new();

        let vertex_buffer = create_vertex_buffer(
            QUAD_VERTICES,
            &mut allocator,
            &device_info.device,
            command_info.command_pool,
            device_info.queue,
        );

        buffers.push(vertex_buffer);

        let index_buffer = create_index_buffer(
            QUAD_INDICES,
            &mut allocator,
            &device_info.device,
            command_info.command_pool,
            device_info.queue,
        );

        buffers.push(index_buffer);

        let sync_info = create_sync(&device_info.device);

        let mut last_modification_time = std::time::Duration::from_millis(0);
        for entry in glob::glob("assets/shaders/*.spv").expect("Failed to get assets/shaders/*.spv")
        {
            match entry {
                Ok(path) => {
                    let metadata = std::fs::metadata(path).expect("Failed to get file metadata");
                    let modification_time = metadata
                        .modified()
                        .expect("Failed to get modification time")
                        .duration_since(std::time::UNIX_EPOCH)
                        .expect("Failed to get time since unix epoch")
                        .as_millis();
                    if modification_time > last_modification_time.as_millis() {
                        last_modification_time =
                            std::time::Duration::from_millis(modification_time.try_into().unwrap());
                    }
                }
                Err(e) => panic!("{}", e),
            }
        }

        for buffer in uniform_buffers {
            buffers.push(buffer);
        }

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
            is_exiting: false,
            current_frame: 0,
            allocator: Some(allocator),
            debug_info,
            descriptor_sets,
            descriptor_pool,
            descriptor_set_layouts,
            buffers: Some(buffers),
            total_delta: 0.1,
        };

        game.run(event_loop);
    }

    fn run(mut self, event_loop: winit::event_loop::EventLoop<()>) {
        event_loop.run(move |event, _, control_flow| {
            *control_flow = winit::event_loop::ControlFlow::Poll;

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

                winit::event::Event::RedrawRequested(_) => self.render(),

                winit::event::Event::MainEventsCleared => {
                    self.window.request_redraw();
                }

                winit::event::Event::RedrawEventsCleared => {
                    self.window.request_redraw();
                }

                winit::event::Event::WindowEvent {
                    window_id,
                    event: winit::event::WindowEvent::Resized(_),
                } if window_id == self.window.id() => self.resize(),

                _ => (),
            }
        });
    }

    fn update(&self, current_image: usize) {
        let camera = Camera {
            model: cgmath::Matrix4::from_axis_angle(
                cgmath::Vector3 {
                    x: 0.0,
                    y: 1.0,
                    z: 0.0,
                },
                cgmath::Deg(self.total_delta / 10000.0),
            ),
            view: cgmath::Matrix4::look_at_rh(
                cgmath::Point3 {
                    x: 0.0,
                    y: 1.0,
                    z: 2.0,
                },
                cgmath::Point3 {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
                cgmath::Vector3 {
                    x: 0.0,
                    y: 1.0,
                    z: 0.0,
                },
            ),
            proj: cgmath::perspective(
                cgmath::Deg(45.0),
                self.swapchain_info.extent.width as f32 / self.swapchain_info.extent.height as f32,
                0.1,
                100.0,
            ),
        };

        let buffers = self.buffers.as_ref().expect("Failed to get buffers");

        let buffers = buffers
            .iter()
            .filter(|buffer| buffer.buffer_type == vk::BufferUsageFlags::UNIFORM_BUFFER)
            .collect::<Vec<&Buffer>>();

        let allocation = &buffers[current_image]
            .allocation
            .as_ref()
            .expect("Failed to get allocation at index");

        unsafe {
            std::ptr::copy_nonoverlapping(
                bytemuck::cast_slice(&[camera]).as_ptr() as *const u8,
                allocation
                    .mapped_ptr()
                    .expect("Memory is not host visible")
                    .as_ptr() as *mut u8,
                std::mem::size_of_val(&camera),
            )
        };
    }

    fn cleanup(&mut self) {
        self.is_exiting = true;

        unsafe { self.device_info.device.device_wait_idle() }
            .expect("Failed to wait for device idle");

        unsafe {
            self.device_info
                .device
                .queue_wait_idle(self.device_info.queue)
        }
        .expect("Failed to wait for queue idle");

        let mut allocator = self.allocator.take().expect("Failed to get allocator");

        for mut buffer in self.buffers.take().unwrap() {
            let allocation = buffer.allocation.take().unwrap();
            unsafe { self.device_info.device.destroy_buffer(buffer.buffer, None) };
            allocator
                .free(allocation)
                .expect("Failed to free allocation");
        }

        drop(allocator);

        for semaphore in &self.sync_info.render_semaphores {
            unsafe { self.device_info.device.destroy_semaphore(*semaphore, None) }
        }

        for semaphore in &self.sync_info.image_semaphores {
            unsafe { self.device_info.device.destroy_semaphore(*semaphore, None) }
        }

        for fence in &self.sync_info.frame_fences {
            unsafe { self.device_info.device.destroy_fence(*fence, None) }
        }

        unsafe {
            self.device_info
                .device
                .destroy_command_pool(self.command_info.command_pool, None)
        }

        unsafe {
            self.device_info.device.destroy_descriptor_set_layout(
                self.descriptor_set_layouts[self.current_frame],
                None,
            )
        }

        unsafe {
            self.device_info
                .device
                .destroy_descriptor_pool(self.descriptor_pool, None)
        };

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

        for swapchain in &self.swapchain_info.swapchains {
            unsafe {
                self.swapchain_info
                    .loader
                    .destroy_swapchain(*swapchain, None)
            };
        }

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
            winit::event::VirtualKeyCode::S => {}
            _ => (),
        }
    }

    fn resize(&mut self) {
        unsafe { self.device_info.device.device_wait_idle() }.expect("Failed to wait for idle");
        for view in &self.swapchain_info.swapchain_views {
            unsafe { self.device_info.device.destroy_image_view(*view, None) }
        }
        self.swapchain_info = create_swapchain(
            self.device_info.clone(),
            self.surface_info.clone(),
            &self.instance,
            &self.window,
            Some(self.swapchain_info.swapchains.clone()),
        );
        for _ in &self.swapchain_info.swapchains.clone() {
            if self.swapchain_info.swapchains.len() > 1 {
                unsafe {
                    self.swapchain_info
                        .loader
                        .destroy_swapchain(self.swapchain_info.swapchains[0], None)
                };

                self.swapchain_info.swapchains.remove(0);
            }
        }

        for framebuffer in &self.framebuffers {
            unsafe {
                self.device_info
                    .device
                    .destroy_framebuffer(*framebuffer, None)
            }
        }

        self.framebuffers = create_framebuffers(
            self.swapchain_info.clone(),
            self.pipeline_info.clone(),
            &self.device_info.device,
        );
    }

    fn render(&mut self) {
        if self.is_exiting {
            return;
        }

        let start = std::time::Instant::now();

        self.update(self.current_frame);

        unsafe {
            self.device_info.device.wait_for_fences(
                &[self.sync_info.frame_fences[self.current_frame]],
                true,
                500000000,
            )
        }
        .expect("Failed to wait for fences");

        // self.check_for_shader_modification();

        let result = unsafe {
            self.swapchain_info.loader.acquire_next_image(
                *self
                    .swapchain_info
                    .swapchains
                    .last()
                    .expect("Failed to get last swapchain"),
                500000000,
                self.sync_info.image_semaphores[self.current_frame],
                vk::Fence::null(),
            )
        };

        if result == Err(vk::Result::ERROR_OUT_OF_DATE_KHR) {
            self.resize();
            return;
        }

        let index = match result {
            Ok(index) => index.0,
            Err(_) => return,
        };

        unsafe {
            self.device_info
                .device
                .reset_fences(&[self.sync_info.frame_fences[self.current_frame]])
        }
        .expect("Failed reset fences");

        unsafe {
            self.device_info.device.reset_command_buffer(
                self.command_info.command_buffers[self.current_frame],
                vk::CommandBufferResetFlags::empty(),
            )
        }
        .expect("Failed to reset command buffer");

        record_buffer(
            index
                .try_into()
                .expect("Failed to convert index to usize from u32"),
            self.pipeline_info.clone(),
            self.swapchain_info.clone(),
            self.framebuffers.clone(),
            &self.device_info.device,
            self.command_info.command_buffers[self.current_frame],
            self.buffers.as_ref().unwrap(),
            QUAD_INDICES
                .len()
                .try_into()
                .expect("Failed to convert to u32"),
            &self.descriptor_sets,
            self.current_frame,
        );

        let signal_semaphores = [self.sync_info.render_semaphores[self.current_frame]];
        let command_buffers = [self.command_info.command_buffers[self.current_frame]];
        let wait_semaphores = [self.sync_info.image_semaphores[self.current_frame]];

        let submit_info = vk::SubmitInfo::builder()
            .wait_semaphores(&wait_semaphores)
            .wait_dst_stage_mask(&[vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT])
            .command_buffers(&command_buffers)
            .signal_semaphores(&signal_semaphores);

        unsafe {
            self.device_info.device.queue_submit(
                self.device_info.queue,
                &[*submit_info],
                self.sync_info.frame_fences[self.current_frame],
            )
        }
        .expect("Failed to submit queue");

        let swapchains = [*self
            .swapchain_info
            .swapchains
            .last()
            .expect("Failed to get last swapchain")];
        let indices = [index];

        let present_info = vk::PresentInfoKHR::builder()
            .wait_semaphores(&signal_semaphores)
            .swapchains(&swapchains)
            .image_indices(&indices);

        let result = unsafe {
            self.swapchain_info
                .loader
                .queue_present(self.device_info.queue, &present_info)
        };

        if result.err() == Some(vk::Result::ERROR_OUT_OF_DATE_KHR) {
            self.resize();
            return;
        }

        self.current_frame = (self.current_frame + 1_usize) % MAX_CONCURRENT_FRAMES as usize;
        let current = std::time::Instant::now();
        let delta = (current - start).as_micros() as f32;
        let delta = if delta == 0.0 { 0.1 } else { delta };
        self.total_delta += delta;
    }
}
