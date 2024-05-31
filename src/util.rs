use std::ffi::CString;

use ash::vk::{self, CommandBufferLevel, CommandPool};

use crate::{builder::VulkanContext, init};

pub fn create_cmd(
    context: &VulkanContext,
    queue_family: u32,
    pool: CommandPool,
) -> vk::CommandBuffer {
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
        (context.set_debug_util_object_name_ext)(context.device.handle(), &debug_info)
            .result()
            .expect("failed to set name");
    }
}
