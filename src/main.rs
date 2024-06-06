#![feature(inherent_associated_types)]

use std::{
    borrow::BorrowMut,
    ffi::{c_char, CStr, CString},
    time::Duration,
};

use ash::vk::{self, Extent2D, PresentModeKHR};
use util::{
    builder::{self, ComputePipelineBuilder},
    init,
};
use vulkan::{PushConstant, SkyBoxPushConstant, VulkanContext};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{self, ControlFlow, EventLoop},
    raw_window_handle::{HasDisplayHandle, HasWindowHandle},
    window::WindowBuilder,
};

mod util;
mod vulkan;

const APPLICATION_NAME: &'static str = "Vulkan App";
const DEBUG_EXT: &'static str = "VK_EXT_debug_utils";
const VALIDATION_LAYER: &'static str = "VK_LAYER_KHRONOS_validation";

extern crate nalgebra_glm as glm;
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

        let (device, physical, graphic_queue, transfer_queue) = builder::DeviceBuilder::new()
            .ext_dynamic_rendering()
            .ext_image_cube_array()
            .ext_sampler_anisotropy()
            .ext_bindless_descriptors()
            .select_physical_device(&instance)
            .build(&instance);

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

        let push_constant = SkyBoxPushConstant::new();

        let comp_skybox = util::create_shader(
            &vulkan_context.device,
            "shaders/spv/skybox.comp.spv".to_owned(),
        );

        let push_vec = vec![push_constant.push_constant_range()];
        let layout_vec = vec![vulkan_context.resources.layout];

        let vk_pipeline = vk::PipelineLayoutCreateInfo::default()
            .flags(vk::PipelineLayoutCreateFlags::empty())
            .push_constant_ranges(&push_vec)
            .set_layouts(&layout_vec);

        vulkan_context.pipeline_layout = vulkan_context
            .device
            .create_pipeline_layout(&vk_pipeline, None)
            .unwrap();

        let pipeline = ComputePipelineBuilder::new(comp_skybox)
            .build(&vulkan_context, vulkan_context.pipeline_layout);

        let mut images = vec![];

        vulkan_context
            .device
            .reset_command_buffer(
                vulkan_context.main_cmd,
                vk::CommandBufferResetFlags::empty(),
            )
            .unwrap();

        vulkan_context
            .device
            .begin_command_buffer(
                vulkan_context.main_cmd,
                &vk::CommandBufferBeginInfo::default()
                    .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT),
            )
            .unwrap();

        for i in 0..vulkan_context.swapchain_images.len() {
            let name = format!("{}_{}", "compute_skybox", i);
            images.push(vulkan_context.resources.create_storage_image(
                vulkan_context.window_extent,
                4,
                vk::MemoryPropertyFlags::DEVICE_LOCAL,
                vk::Format::R8G8B8A8_UNORM,
                vk::ImageUsageFlags::TRANSFER_SRC
                    | vk::ImageUsageFlags::TRANSFER_DST
                    | vk::ImageUsageFlags::STORAGE
                    | vk::ImageUsageFlags::COLOR_ATTACHMENT,
                CString::new(name).unwrap(),
            ));

            util::transition_image_general(
                &vulkan_context.device,
                vulkan_context.main_cmd,
                images.last().unwrap().image,
            );
        }

        vulkan_context
            .device
            .end_command_buffer(vulkan_context.main_cmd)
            .unwrap();

        vulkan_context
            .device
            .queue_submit(
                vulkan_context.graphic_queue.queue,
                &[vk::SubmitInfo::default().command_buffers(&[vulkan_context.main_cmd])],
                vk::Fence::null(),
            )
            .unwrap();

        vulkan_context.device.device_wait_idle().unwrap();

        // let gui_vert_shader = util::create_shader(
        //     &vulkan_context,
        //     "shaders/spiv/gui.vert.spv".to_owned(),
        //     vk::ShaderStageFlags::VERTEX,
        //     set_layout,
        // );

        // // let gui_frag_shader = util::create_shader(
        //     &vulkan_context,
        //     "shaders/spiv/gui.vert.spv".to_owned(),
        //     vk::ShaderStageFlags::FRAGMENT,
        //     set_layout,
        // );
        event_loop
            .run(move |event, _control_flow| match event {
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::CloseRequested => {
                        _control_flow.exit();
                    }
                    _ => {}
                },
                _ => {
                    let descriptor_sets = vec![vulkan_context.resources.set];

                    let device = vulkan_context.device.clone();
                    let cmd = vulkan_context.main_cmd.clone();

                    let (swapchain_index, _) = vulkan_context
                        .swapchain_loader
                        .acquire_next_image(
                            vulkan_context.swapchain,
                            100000,
                            vulkan_context.image_aquired_semp,
                            vk::Fence::null(),
                        )
                        .unwrap();

                    let swapchain_image = vulkan_context.swapchain_images[swapchain_index as usize]
                        .image
                        .clone();

                    device
                        .reset_command_buffer(cmd, vk::CommandBufferResetFlags::empty())
                        .unwrap();

                    device
                        .begin_command_buffer(
                            cmd,
                            &vk::CommandBufferBeginInfo::default()
                                .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT),
                        )
                        .expect("failed to begin cmd");

                    device.cmd_bind_descriptor_sets(
                        cmd,
                        vk::PipelineBindPoint::COMPUTE,
                        vulkan_context.pipeline_layout,
                        0,
                        &descriptor_sets,
                        &vec![],
                    );

                    device.cmd_bind_pipeline(cmd, vk::PipelineBindPoint::COMPUTE, pipeline);

                    device.cmd_push_constants(
                        cmd,
                        vulkan_context.pipeline_layout,
                        vk::ShaderStageFlags::COMPUTE,
                        0,
                        std::slice::from_raw_parts(
                            &push_constant as *const _ as *const u8,
                            std::mem::size_of::<SkyBoxPushConstant>(),
                        ),
                    );

                    device.cmd_dispatch(
                        cmd,
                        vulkan_context.window_extent.width / 16,
                        vulkan_context.window_extent.height / 16,
                        1,
                    );

                    util::copy_image(
                        &vulkan_context,
                        &images[0],
                        &vulkan_context.swapchain_images[swapchain_index as usize],
                        vulkan_context.window_extent,
                    );

                    util::transition_image_color(&device, cmd, swapchain_image);

                    // fragment rendering will happen here

                    util::transition_image_present(&device, cmd, swapchain_image);

                    vulkan_context
                        .device
                        .end_command_buffer(vulkan_context.main_cmd)
                        .unwrap();

                    let aquire_is_ready = vec![vulkan_context.image_aquired_semp];
                    let render_is_done = vec![vulkan_context.render_is_done];
                    let swapchain_indices = vec![swapchain_index];
                    let wait_mask = vec![vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
                    let cmds = vec![vulkan_context.main_cmd];
                    let swapchain = vec![vulkan_context.swapchain];

                    let submit_info = vec![vk::SubmitInfo::default()
                        .command_buffers(&cmds)
                        .wait_semaphores(&aquire_is_ready)
                        .signal_semaphores(&render_is_done)
                        .wait_dst_stage_mask(&wait_mask)];

                    let present_info = vk::PresentInfoKHR::default()
                        .swapchains(&swapchain)
                        .wait_semaphores(&render_is_done)
                        .image_indices(&swapchain_indices);

                    vulkan_context
                        .device
                        .queue_submit(
                            vulkan_context.graphic_queue.queue,
                            &submit_info,
                            vk::Fence::null(),
                        )
                        .unwrap();

                    vulkan_context
                        .swapchain_loader
                        .queue_present(vulkan_context.graphic_queue.queue, &present_info)
                        .unwrap();

                    vulkan_context.device.device_wait_idle().unwrap();
                }
            })
            .unwrap();
    }
}
