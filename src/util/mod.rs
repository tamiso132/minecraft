use std::{
    ffi::{c_void, CString},
    fs::File,
    io::Read,
    panic::UnwindSafe,
    sync::Arc,
};

use ash::vk::{
    self, AccessFlags, CommandBufferLevel, CommandPool, DependencyFlags, ImageAspectFlags,
    ImageLayout, Offset3D, ShaderStageFlags,
};
use loader::{DebugLoaderEXT, ShaderLoaderEXT};
use resource::AllocatedImage;
use vk_mem::Alloc;

use crate::vulkan::{self, VulkanContext};

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

pub fn create_fence(device: &ash::Device) -> vk::Fence {
    unsafe {
        device
            .create_fence(
                &vk::FenceCreateInfo::default().flags(vk::FenceCreateFlags::SIGNALED),
                None,
            )
            .unwrap()
    }
}

pub fn create_semphore(device: &ash::Device) -> vk::Semaphore {
    unsafe {
        device
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
    println!("{:?}", path);
    let mut file = File::open(path.clone()).expect(&format!("unable to read file {}", path));
    let mut buffer = vec![];
    file.read_to_end(&mut buffer).expect("unable to read file");

    return buffer;
}

pub fn pad_size_to_min_aligment(size: u32, min_aligment: u32) -> u32 {
    (size + min_aligment - 1) & !(min_aligment - 1)
}

pub fn create_unlinked_shader(
    context: &VulkanContext,
    shader_loader: ShaderLoaderEXT,
    path: String,
    shader_stage: vk::ShaderStageFlags,
    descriptor_layout: Vec<vk::DescriptorSetLayout>,
    push_constants: Vec<vk::PushConstantRange>,
) -> vk::ShaderEXT {
    let data = load_shader(path);
    let name = CString::new("main").unwrap();

    let layouts = descriptor_layout;
    let shader_info = init::shader_create_info(shader_stage)
        .code(&data)
        .name(&name)
        .push_constant_ranges(&push_constants)
        .flags(vk::ShaderCreateFlagsEXT::empty())
        .next_stage(vk::ShaderStageFlags::empty())
        .set_layouts(&layouts);

    let shader = shader_loader
        .create_shaders_ext(shader_info)
        .expect("failed to create a shader");
    shader
}

pub fn create_layout(
    device: Arc<ash::Device>,
    binding: u32,
    descriptor_type: Vec<vk::DescriptorType>,
    debug_loader: &DebugLoaderEXT,
    name: CString,
) -> vk::DescriptorSetLayout {
    let mut bindings: Vec<vk::DescriptorSetLayoutBinding> = vec![];

    for (index, descriptor) in descriptor_type.iter().enumerate() {
        bindings.push(init::descriptor_set_layout_binding(
            index as u32,
            descriptor.to_owned(),
            1000,
            vk::ShaderStageFlags::ALL,
        ))
    }

    let layout_flags = vec![
        vk::DescriptorBindingFlags::PARTIALLY_BOUND | vk::DescriptorBindingFlags::UPDATE_AFTER_BIND,
        vk::DescriptorBindingFlags::PARTIALLY_BOUND | vk::DescriptorBindingFlags::UPDATE_AFTER_BIND,
        vk::DescriptorBindingFlags::PARTIALLY_BOUND | vk::DescriptorBindingFlags::UPDATE_AFTER_BIND,
    ];

    let mut binding_flags =
        vk::DescriptorSetLayoutBindingFlagsCreateInfo::default().binding_flags(&layout_flags);

    let layout_info = vk::DescriptorSetLayoutCreateInfo::default()
        .bindings(&bindings)
        .flags(vk::DescriptorSetLayoutCreateFlags::UPDATE_AFTER_BIND_POOL)
        .push_next(&mut binding_flags);

    unsafe {
        let layout = device
            .create_descriptor_set_layout(&layout_info, None)
            .unwrap();

        debug_loader
            .set_debug_util_object_name_ext(
                vk::DebugUtilsObjectNameInfoEXT::default()
                    .object_handle(layout)
                    .object_name(&name),
            )
            .unwrap();
        layout
    }
}
pub fn create_shader(device: &ash::Device, path: String) -> vk::ShaderModule {
    let data = load_shader(path);

    assert!(data.len() % 4 == 0, "Must extend to a multiple of 4");

    let vec_u32: Vec<u32> = data
        .chunks_exact(4)
        .map(|chunk| u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
        .collect();

    let shader_info = vk::ShaderModuleCreateInfo::default().code(&vec_u32);
    unsafe { device.create_shader_module(&shader_info, None).unwrap() }
}

pub fn create_shader_ext(
    context: &VulkanContext,
    shader_loader: ShaderLoaderEXT,
    path: String,
    shader_stage: vk::ShaderStageFlags,
    descriptor_layout: vk::DescriptorSetLayout,
) -> vk::ShaderEXT {
    // compute shaders cannot be linked
    assert!(shader_stage == vk::ShaderStageFlags::COMPUTE);

    let data = load_shader(path);
    let name = CString::new("main").unwrap();

    let layouts = [descriptor_layout];
    println!("code size = {:?}\n", &data.len());
    let shader_info = init::shader_create_info(shader_stage)
        .code(&data)
        .name(&name)
        .set_layouts(&layouts);

    let shader = shader_loader
        .create_shaders_ext(shader_info)
        .expect("failed to create a shader");
    shader
}

pub fn transition_image_present(device: &ash::Device, cmd: vk::CommandBuffer, image: vk::Image) {
    let barrier = vec![init::image_barrier_info(
        image,
        vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
        vk::ImageLayout::PRESENT_SRC_KHR,
        vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
        vk::AccessFlags::MEMORY_READ,
    )];

    let (src_stage, dst_stage) = (
        vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
        vk::PipelineStageFlags::BOTTOM_OF_PIPE,
    );

    unsafe {
        device.cmd_pipeline_barrier(
            cmd,
            src_stage,
            dst_stage,
            vk::DependencyFlags::empty(),
            &vec![],
            &vec![],
            &barrier,
        )
    }
}

pub fn transition_image_general(device: &ash::Device, cmd: vk::CommandBuffer, image: vk::Image) {
    let barrier = vec![init::image_barrier_info(
        image,
        vk::ImageLayout::UNDEFINED,
        vk::ImageLayout::GENERAL,
        vk::AccessFlags::NONE_KHR,
        vk::AccessFlags::SHADER_READ | vk::AccessFlags::SHADER_WRITE,
    )];

    let (src_stage, dst_stage) = (
        vk::PipelineStageFlags::TOP_OF_PIPE,
        vk::PipelineStageFlags::COMPUTE_SHADER,
    );

    unsafe {
        device.cmd_pipeline_barrier(
            cmd,
            src_stage,
            dst_stage,
            vk::DependencyFlags::empty(),
            &vec![],
            &vec![],
            &barrier,
        )
    }
}

pub fn transition_image_color(device: &ash::Device, cmd: vk::CommandBuffer, image: vk::Image) {
    let barrier = vec![init::image_barrier_info(
        image,
        vk::ImageLayout::UNDEFINED,
        vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
        vk::AccessFlags::NONE_KHR,
        vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
    )];

    let (src_stage, dst_stage) = (
        vk::PipelineStageFlags::TOP_OF_PIPE,
        vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
    );

    unsafe {
        device.cmd_pipeline_barrier(
            cmd,
            src_stage,
            dst_stage,
            vk::DependencyFlags::empty(),
            &vec![],
            &vec![],
            &barrier,
        )
    }
}

pub fn copy_image(
    context: &VulkanContext,
    src_image: &AllocatedImage,
    dst_image: &AllocatedImage,
    extent: vk::Extent2D,
) {
    let sub_resource = init::image_subresource_info(vk::ImageAspectFlags::COLOR);

    let old_src_layout = vk::ImageLayout::GENERAL;

    let mut src_barrier = vk::ImageMemoryBarrier::default()
        .old_layout(old_src_layout)
        .new_layout(ImageLayout::TRANSFER_SRC_OPTIMAL)
        .subresource_range(sub_resource)
        .src_access_mask(AccessFlags::SHADER_WRITE)
        .dst_access_mask(AccessFlags::TRANSFER_READ)
        .image(src_image.image);

    let dst_barrier = vk::ImageMemoryBarrier::default()
        .old_layout(vk::ImageLayout::UNDEFINED)
        .new_layout(ImageLayout::TRANSFER_DST_OPTIMAL)
        .subresource_range(sub_resource)
        .image(dst_image.image)
        .src_access_mask(vk::AccessFlags::NONE_KHR)
        .dst_access_mask(AccessFlags::TRANSFER_WRITE);

    unsafe {
        context.device.cmd_pipeline_barrier(
            context.main_cmd,
            vk::PipelineStageFlags::FRAGMENT_SHADER,
            vk::PipelineStageFlags::TRANSFER,
            DependencyFlags::empty(),
            &vec![],
            &vec![],
            &vec![src_barrier],
        );

        context.device.cmd_pipeline_barrier(
            context.main_cmd,
            vk::PipelineStageFlags::TOP_OF_PIPE,
            vk::PipelineStageFlags::TRANSFER,
            DependencyFlags::empty(),
            &vec![],
            &vec![],
            &vec![dst_barrier],
        );

        let sub_resource_layer = vk::ImageSubresourceLayers::default()
            .aspect_mask(ImageAspectFlags::COLOR)
            .base_array_layer(0)
            .layer_count(1)
            .mip_level(0);

        let offsets = [
            Offset3D::default().x(0).y(0).z(0),
            Offset3D {
                x: extent.width as i32,
                y: extent.height as i32,
                z: 1,
            },
        ];
        let image_blit = vk::ImageBlit::default()
            .dst_offsets(offsets)
            .dst_subresource(sub_resource_layer)
            .src_offsets(offsets)
            .src_subresource(sub_resource_layer);

        context.device.cmd_blit_image(
            context.main_cmd,
            src_image.image,
            vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
            dst_image.image,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            &[image_blit],
            vk::Filter::LINEAR,
        );

        src_barrier = src_barrier
            .old_layout(ImageLayout::TRANSFER_SRC_OPTIMAL)
            .new_layout(ImageLayout::GENERAL)
            .src_access_mask(AccessFlags::TRANSFER_READ)
            .dst_access_mask(AccessFlags::SHADER_WRITE);

        context.device.cmd_pipeline_barrier(
            context.main_cmd,
            vk::PipelineStageFlags::TRANSFER,
            vk::PipelineStageFlags::COMPUTE_SHADER,
            DependencyFlags::empty(),
            &vec![],
            &vec![],
            &vec![src_barrier],
        );
    }
}
pub fn copy_image_immediate(
    context: &VulkanContext,
    src_image: &AllocatedImage,
    dst_image: &AllocatedImage,
    extent: vk::Extent2D,
) {
    // TODO
}