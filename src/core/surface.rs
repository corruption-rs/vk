use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};

use super::structures::SurfaceInfo;

pub fn create_surface(
    window: &winit::window::Window,
    entry: &ash::Entry,
    instance: &ash::Instance,
) -> SurfaceInfo {
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

    let surface_loader = ash::extensions::khr::Surface::new(&entry, &instance);

    SurfaceInfo {
        surface,
        surface_loader,
    }
}
