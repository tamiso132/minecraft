use std::ffi::{c_char, CStr, CString};

use ash::vk;
use builder::DeviceBuilder;
use winit::{
    event_loop::{self, ControlFlow, EventLoop},
    raw_window_handle::{HasDisplayHandle, HasWindowHandle},
    window::WindowBuilder,
};

use crate::builder::{InstanceBuilder, TKQueue};

mod builder;
mod init;
mod util;

const APPLICATION_NAME: &'static str = "Vulkan App";
const DEBUG_EXT: &'static str = "VK_EXT_debug_utils";
const VALIDATION_LAYER: &'static str = "VK_LAYER_KHRONOS_validation";

extern crate vk_mem;

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

        let (mut device, mut physical) = DeviceBuilder::new(instance.clone())
            .ext_dynamic_rendering()
            .ext_image_cube_array()
            .ext_sampler_anisotropy()
            .select_physical_device()
            .build();

        let allocator_info = vk_mem::AllocatorCreateInfo::new(&instance, &device, physical);
        let mut allocator = vk_mem::Allocator::new(allocator_info);
    }
}
