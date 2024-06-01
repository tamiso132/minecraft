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

        SwapchainBuilder::new(&mut vulkan_context)
            .select_image_format(vk::Format::B8G8R8A8_SRGB)
            .select_sharing_mode(vk::SharingMode::EXCLUSIVE)
            .select_presentation_mode(PresentModeKHR::MAILBOX)
            .build();

        vulkan_context.main_pool =
            util::create_pool(&vulkan_context, vulkan_context.graphic_queue.get_family());

        vulkan_context.main_cmd = util::create_cmd(
            &vulkan_context,
            vulkan_context.graphic_queue.get_family(),
            vulkan_context.main_pool,
        );

        let limits = vulkan_context
            .instance
            .get_physical_device_properties(vulkan_context.physical)
            .limits;

        let pool_sizes = vec![
            init::descriptor_pool_size(
                vk::DescriptorType::SAMPLED_IMAGE,
                limits.max_descriptor_set_sampled_images,
            ),
            init::descriptor_pool_size(
                vk::DescriptorType::STORAGE_BUFFER,
                limits.max_descriptor_set_storage_buffers,
            ),
            init::descriptor_pool_size(
                vk::DescriptorType::UNIFORM_BUFFER,
                limits.max_descriptor_set_uniform_buffers,
            ),
            init::descriptor_pool_size(vk::DescriptorType::COMBINED_IMAGE_SAMPLER, 200),
        ];

        let descriptor_pool_info = vk::DescriptorPoolCreateInfo::default()
            .pool_sizes(&pool_sizes)
            .max_sets(1)
            .flags(vk::DescriptorPoolCreateFlags::UPDATE_AFTER_BIND_EXT);

        let descriptor_pool = vulkan_context
            .device
            .create_descriptor_pool(&descriptor_pool_info, None)
            .unwrap();

        let mut bindings: Vec<vk::DescriptorSetLayoutBinding> = vec![];
        let mut set_layout_binding_flags = vec![];
        for i in 0..pool_sizes.len() {
            bindings.push(
                init::descriptor_set_layout_binding(
                    i as u32,
                    pool_sizes[i].ty,
                    pool_sizes[i].descriptor_count,
                    vk::ShaderStageFlags::ALL,
                )
                .clone(),
            );

            set_layout_binding_flags.push(
                vk::DescriptorBindingFlags::PARTIALLY_BOUND
                    | vk::DescriptorBindingFlags::UPDATE_AFTER_BIND,
            );
        }

        for binding in &bindings {
            println!(
                "Binding number: {:?}\nBinding type{:?}\n",
                binding.binding, binding.descriptor_type
            );
        }

        let mut set_layout_binding_f = vk::DescriptorSetLayoutBindingFlagsCreateInfo::default()
            .binding_flags(&set_layout_binding_flags);

        let layout_info = vk::DescriptorSetLayoutCreateInfo::default()
            .bindings(&bindings)
            .flags(vk::DescriptorSetLayoutCreateFlags::UPDATE_AFTER_BIND_POOL)
            .push_next(&mut set_layout_binding_f);

        let set_layout = vulkan_context
            .device
            .create_descriptor_set_layout(&layout_info, None)
            .unwrap();

        let descriptor_set = vulkan_context
            .device
            .allocate_descriptor_sets(
                &vk::DescriptorSetAllocateInfo::default()
                    .descriptor_pool(descriptor_pool)
                    .set_layouts(&[set_layout]),
            )
            .unwrap();

        util::create_shader(
            &vulkan_context,
            "shaders/spiv/gui.vert.spv".to_owned(),
            vk::ShaderStageFlags::VERTEX,
            set_layout,
        );
    }
}
