use std::ffi::{c_char, CStr, CString};

use ash::vk::{self, Extent2D, PresentModeKHR};
use builder::{DeviceBuilder, SwapchainBuilder, VulkanContext};
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

        let (instance, entry, debug_callback, debug_loader) = InstanceBuilder::new()
            .enable_debug()
            .set_required_version(1, 3, 0)
            .set_app_name("Vulkan App")
            .set_xlib_ext()
            .build();

        let (device, physical, graphic_queue, transfer_queue) =
            DeviceBuilder::new(instance.clone())
                .ext_dynamic_rendering()
                .ext_image_cube_array()
                .ext_sampler_anisotropy()
                .select_physical_device()
                .build();

        let mut vulkan_context = VulkanContext::new(entry, instance, device, physical, window);
        vulkan_context.physical = physical;

        vulkan_context.debug_loader = Some(debug_loader);
        vulkan_context.debug_messenger = debug_callback;

        vulkan_context.graphic_queue = graphic_queue;
        vulkan_context.transfer = transfer_queue;

        SwapchainBuilder::new(&mut vulkan_context)
            .select_image_format(vk::Format::B8G8R8A8_SRGB)
            .select_sharing_mode(vk::SharingMode::EXCLUSIVE)
            .select_presentation_mode(PresentModeKHR::MAILBOX)
            .build();




    }
}
