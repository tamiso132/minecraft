#![feature(inherent_associated_types)]

use std::ffi::{c_char, CStr, CString};

use ash::vk::{self, Extent2D, PresentModeKHR};
use util::{builder, init};
use vulkan::VulkanContext;
use winit::{
    event_loop::{self, ControlFlow, EventLoop},
    raw_window_handle::{HasDisplayHandle, HasWindowHandle},
    window::WindowBuilder,
};

mod util;
mod vulkan;

const APPLICATION_NAME: &'static str = "Vulkan App";
const DEBUG_EXT: &'static str = "VK_EXT_debug_utils";
const VALIDATION_LAYER: &'static str = "VK_LAYER_KHRONOS_validation";

extern crate vk_mem;

const IMAGE_SAMPLED_BINDING: u32 = 0;
const STORAGE_BUFFER_BINDING: u32 = 1;
const UNIFORM_BINDING: u32 = 2;
const COMBINED_SAMPLER_BINDING: u32 = 3;

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

        let (instance, entry, debug_callback, debug_loader) = builder::InstanceBuilder::new()
            .enable_debug()
            .set_required_version(1, 3, 0)
            .set_app_name("Vulkan App")
            .set_xlib_ext()
            .build();

        let (device, physical, graphic_queue, transfer_queue) =
            builder::DeviceBuilder::new(instance.clone())
                .ext_dynamic_rendering()
                .ext_image_cube_array()
                .ext_sampler_anisotropy()
                .ext_bindless_descriptors()
                .ext_shader_object()
                .select_physical_device()
                .build();

        let mut vulkan_context = VulkanContext::new(entry, instance, device, physical, window);
        vulkan_context.physical = physical;

        vulkan_context.debug_loader = Some(debug_loader);
        vulkan_context.debug_messenger = debug_callback;

        vulkan_context.graphic_queue = graphic_queue;
        vulkan_context.transfer = transfer_queue;

        builder::SwapchainBuilder::new(&mut vulkan_context)
            .select_image_format(vk::Format::B8G8R8A8_SRGB)
            .select_sharing_mode(vk::SharingMode::EXCLUSIVE)
            .select_presentation_mode(PresentModeKHR::MAILBOX)
            .build();

        vulkan_context.main_pool =
            util::create_pool(&vulkan_context, vulkan_context.graphic_queue.get_family());

        vulkan_context.main_cmd = util::create_cmd(&vulkan_context, vulkan_context.main_pool);

       

        

        let gui_vert_shader = util::create_shader(
            &vulkan_context,
            "shaders/spiv/gui.vert.spv".to_owned(),
            vk::ShaderStageFlags::VERTEX,
            set_layout,
        );

        let gui_frag_shader = util::create_shader(
            &vulkan_context,
            "shaders/spiv/gui.vert.spv".to_owned(),
            vk::ShaderStageFlags::FRAGMENT,
            set_layout,
        );

        vulkan_context
            .device
            .begin_command_buffer(
                vulkan_context.main_cmd,
                &vk::CommandBufferBeginInfo::default()
                    .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT),
            )
            .expect("failed to begin cmd");
    }
}
