use std::{
    ffi::{c_void, CString},
    fs::File,
    io::Read,
    panic::UnwindSafe,
};

use ash::vk::{self, CommandBufferLevel, CommandPool, ShaderStageFlags};
use vk_mem::Alloc;

use crate::vulkan::VulkanContext;

pub mod builder;
pub mod init;
pub mod loader;
pub mod mesh;
pub mod resource;

pub fn create_cmd(context: &VulkanContext, pool: CommandPool) -> vk::CommandBuffer {
    let cmd_info = vk::CommandBufferAllocateInfo::default()
        .command_pool(pool)
        .level(CommandBufferLevel::PRIMARY)
        .command_buffer_count(1);

    unsafe { context.device.allocate_command_buffers(&cmd_info).unwrap()[0] }
}

pub fn create_pool(context: &VulkanContext, queue_family: u32) -> vk::CommandPool {
    unsafe {
        context
            .device
            .create_command_pool(&init::command_pool_info(queue_family), None)
            .unwrap()
    }
}

pub fn create_fence(context: &VulkanContext) -> vk::Fence {
    unsafe {
        context
            .device
            .create_fence(
                &vk::FenceCreateInfo::default().flags(vk::FenceCreateFlags::SIGNALED),
                None,
            )
            .unwrap()
    }
}

pub fn create_semphore(context: &VulkanContext) -> vk::Semaphore {
    unsafe {
        context
            .device
            .create_semaphore(&vk::SemaphoreCreateInfo::default(), None)
            .unwrap()
    }
}

pub fn debug_object_set_name(
    context: &VulkanContext,
    raw_object_handle: u64,
    object_type: vk::ObjectType,
    name: String,
) {
    let raw_name = CString::new(name).unwrap();

    let mut debug_info = vk::DebugUtilsObjectNameInfoEXT::default().object_name(&raw_name);
    debug_info.object_handle = raw_object_handle;
    debug_info.object_type = object_type;

    unsafe {
        context
            .debug_loader_ext
            .set_debug_util_object_name_ext(debug_info);
    }
}

pub fn load_shader(path: String) -> Vec<u8> {
    let mut file = File::open(path.clone()).expect(&format!("unable to read file {}", path));
    let mut buffer = vec![];
    file.read_to_end(&mut buffer).expect("unable to read file");

    return buffer;
}

pub fn create_shader(
    context: &VulkanContext,
    path: String,
    shader_stage: vk::ShaderStageFlags,
    descriptor_layout: vk::DescriptorSetLayout,
) -> vk::ShaderEXT {
    let data = load_shader(path);
    let name = CString::new("main").unwrap();

    let layouts = [descriptor_layout];
    println!("code size = {:?}\n", &data.len());
    let shader_info = init::shader_create_info(shader_stage)
        .code(&data)
        .name(&name)
        .set_layouts(&layouts);

    let shader = context
        .shader_loader_ext
        .create_shaders_ext(shader_info)
        .expect("failed to create a shader");
    shader
}
