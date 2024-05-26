use std::ffi::{c_char, CStr, CString};

use ash::{
    ext::physical_device_drm,
    vk::{self, ApplicationInfo, DeviceCreateInfo, InstanceCreateInfo},
    Entry,
};

use winit::{
    event::{ElementState, Event, KeyEvent, WindowEvent},
    event_loop::{self, ControlFlow, EventLoop},
    keyboard::{Key, NamedKey},
    platform::run_on_demand::EventLoopExtRunOnDemand,
    raw_window_handle::{HasDisplayHandle, HasWindowHandle},
    window::WindowBuilder,
};

use crate::init::{InstanceBuilder, TKQueue};

mod init;

const APPLICATION_NAME: &'static str = "Vulkan App";
const DEBUG_EXT: &'static str = "VK_EXT_debug_utils";
const VALIDATION_LAYER: &'static str = "VK_LAYER_KHRONOS_validation";

fn main() {
    println!("Hello, world!");
    unsafe {
        let event_loop = EventLoop::new().unwrap();

        let window = WindowBuilder::new()
            .with_title(APPLICATION_NAME)
            .with_inner_size(winit::dpi::LogicalSize::new(
                f64::from(1920.0),
                f64::from(1080.0),
            ))
            .build(&event_loop)
            .unwrap();

        let (mut instance, mut entry, debug_callback, debug_loader) = InstanceBuilder::new()
            .enable_debug()
            .set_required_version(1, 3, 0)
            .set_app_name("Vulkan App")
            .build();

        let surface = ash_window::create_surface(
            &entry,
            &instance,
            window.display_handle().unwrap().as_raw(),
            window.window_handle().unwrap().as_raw(),
            None,
        )
        .expect("surface failed");

        let surface_loader = ash::khr::surface::Instance::new(&entry, &instance);

        // for now, pick the first alternative
        let physical_device = instance
            .enumerate_physical_devices()
            .expect("no gpu that support Vulkan")[0];
    }
}
