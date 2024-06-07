use std::sync::{Arc, Mutex};

use ash::{
    khr::dynamic_rendering,
    vk::{self, BlendFactor, BlendOp, DescriptorType, PrimitiveTopology, QueueFlags, ShaderStageFlags},
};
use builder::{ComputePipelineBuilder, PipelineBuilder};
use glm::Mat4;
use imgui::{FontConfig, FontSource};
use imgui_rs_vulkan_renderer::Renderer;
use imgui_winit_support::{HiDpiMode, WinitPlatform};
use loader::DebugLoaderEXT;
use mesh::MeshImGui;
use resource::{AllocatedImage, Resource};
use vk_mem::{Alloc, Allocator};
use winit::{event_loop::EventLoop, window::WindowBuilder};

pub mod builder;
pub mod init;
pub mod loader;
pub mod mesh;
pub mod resource;
pub mod util;

pub trait PushConstant {
    fn size(&self) -> u64;
    fn stage_flag(&self) -> vk::ShaderStageFlags;
    fn push_constant_range(&self) -> vk::PushConstantRange;
}

#[repr(C)]
pub struct SkyBoxPushConstant {
    data1: glm::Vec4,
    data2: glm::Vec4,
    data3: glm::Vec4,
    data4: glm::Vec4,
}

impl SkyBoxPushConstant {
    pub fn new() -> Self {
        Self {
            data1: glm::vec4(0.5, 0.5, 0.5, 0.5),
            data2: glm::vec4(0.5, 0.5, 0.5, 0.5),
            data3: glm::vec4(1.0, 1.0, 1.0, 1.0),
            data4: glm::vec4(1.0, 1.0, 1.0, 1.0),
        }
    }
}

impl PushConstant for SkyBoxPushConstant {
    fn size(&self) -> u64 {
        std::mem::size_of::<SkyBoxPushConstant>() as u64
    }

    fn stage_flag(&self) -> vk::ShaderStageFlags {
        vk::ShaderStageFlags::COMPUTE
    }

    fn push_constant_range(&self) -> vk::PushConstantRange {
        vk::PushConstantRange::default()
            .size(self.size() as u32)
            .offset(0)
            .stage_flags(self.stage_flag())
    }
}

pub struct ImguiContext {
    pub set: vk::DescriptorSet,
    pub layout: vk::DescriptorSetLayout,
    pub imgui: imgui::Context,
    pub winit_platform: WinitPlatform,

    pub imgui_pool: vk::DescriptorPool,
    pub pipeline: vk::Pipeline,
}

impl ImguiContext {
    fn new(vulkan: VulkanContext, device: ash::Device, instance: ash::Instance) -> Self {
        let mut imgui = imgui::Context::create();
        imgui.set_ini_filename(None);

        let mut platform = WinitPlatform::init(&mut imgui);
        let hidpi_factor = platform.hidpi_factor();
        let font_size = (13.0 * hidpi_factor) as f32;

        imgui
            .fonts()
            .add_font(&[FontSource::DefaultFontData { config: Some(FontConfig { size_pixels: font_size, ..FontConfig::default() }) }]);

        imgui.io_mut().font_global_scale = (1.0 / hidpi_factor) as f32;
        platform.attach_window(imgui.io_mut(), &vulkan.window, HiDpiMode::Rounded);

        let pool_sizes = vec![
            init::descriptor_pool_size(vk::DescriptorType::SAMPLER, 500),
            init::descriptor_pool_size(vk::DescriptorType::COMBINED_IMAGE_SAMPLER, 500),
            init::descriptor_pool_size(vk::DescriptorType::SAMPLED_IMAGE, 500),
            init::descriptor_pool_size(vk::DescriptorType::STORAGE_IMAGE, 500),
        ];

        let descriptor_pool_info = vk::DescriptorPoolCreateInfo::default().pool_sizes(&pool_sizes).max_sets(500);

        let descriptor_pool = unsafe { device.create_descriptor_pool(&descriptor_pool_info, None).unwrap() };

        let bindings = vec![vk::DescriptorSetLayoutBinding::default()
            .binding(0)
            .descriptor_type(DescriptorType::COMBINED_IMAGE_SAMPLER)
            .descriptor_count(1)
            .stage_flags(ShaderStageFlags::FRAGMENT)];

        unsafe {
            let layout = device
                .create_descriptor_set_layout(&vk::DescriptorSetLayoutCreateInfo::default().bindings(&bindings), None)
                .unwrap();

            let push_const_range =
                [vk::PushConstantRange { stage_flags: ShaderStageFlags::VERTEX, offset: 0, size: std::mem::size_of::<Mat4>() as u32 }];

            let layouts = vec![layout];

            let alloc_info = vk::DescriptorSetAllocateInfo::default()
                .descriptor_pool(descriptor_pool)
                .set_layouts(&layouts);

            let set = vulkan.device.allocate_descriptor_sets(&alloc_info).unwrap()[0];

            let fonts = imgui.fonts();
            let atlas_texture = fonts.build_rgba32_texture();
            //let image_info = vec![vk::DescriptorImageInfo::default()]

            let pipeline_layout = device
                .create_pipeline_layout(
                    &vk::PipelineLayoutCreateInfo::default()
                        .set_layouts(&layouts)
                        .push_constant_ranges(&push_const_range),
                    None,
                )
                .unwrap();
            let blend_state = vk::PipelineColorBlendAttachmentState::default()
                .color_write_mask(vk::ColorComponentFlags::R | vk::ColorComponentFlags::G | vk::ColorComponentFlags::B | vk::ColorComponentFlags::A)
                .blend_enable(true)
                .src_alpha_blend_factor(BlendFactor::SRC1_ALPHA)
                .dst_alpha_blend_factor(BlendFactor::ONE_MINUS_CONSTANT_ALPHA)
                .color_blend_op(BlendOp::ADD)
                .src_alpha_blend_factor(BlendFactor::ONE)
                .dst_alpha_blend_factor(BlendFactor::ONE_MINUS_SRC_ALPHA)
                .alpha_blend_op(BlendOp::ADD);

            let shader_frag = util::create_shader(&device, "shaders/spv/imgui_shader.frag.spv".to_owned());
            let shader_vert = util::create_shader(&device, "shaders/spv/imgui_shader.vert.spv".to_owned());

            let pipeline = PipelineBuilder::new()
                .add_color_format(vulkan.swapchain.swapchain_images[0].format)
                .add_pipeline_layout(pipeline_layout)
                .add_topology(PrimitiveTopology::TRIANGLE_LIST)
                .add_blend(blend_state)
                .build::<MeshImGui>(&vulkan, shader_vert, shader_frag);

            device.destroy_shader_module(shader_frag, None);
            device.destroy_shader_module(shader_vert, None);

            Self { set: todo!(), layout, imgui, winit_platform: todo!(), imgui_pool: descriptor_pool, pipeline }
        };
    }
}

pub struct Swapchain {
    pub surface: vk::SurfaceKHR,
    pub swapchain: vk::SwapchainKHR,
    pub swapchain_images: Vec<AllocatedImage>,
    pub depth_image: AllocatedImage,
    pub max_swapchain_images: u32,
    pub swapchain_index: u32,
}

pub struct VulkanContext<'a> {
    pub entry: ash::Entry,
    pub instance: Arc<ash::Instance>,
    pub device: ash::Device,
    pub physical: vk::PhysicalDevice,
    /// Don't forget to clean this one up
    pub allocator: Arc<vk_mem::Allocator>,

    pub window_extent: vk::Extent2D,
    pub window: winit::window::Window,

    pub swapchain: Swapchain,

    pub main_cmd: vk::CommandBuffer,
    pub main_pool: vk::CommandPool,

    pub graphic_queue: TKQueue,
    pub transfer: TKQueue,

    pub debug_messenger: vk::DebugUtilsMessengerEXT,

    pub swapchain_loader: ash::khr::swapchain::Device,
    pub surface_loader: ash::khr::surface::Instance,
    pub debug_loader: Option<ash::ext::debug_utils::Instance>,

    pub debug_loader_ext: DebugLoaderEXT<'a>,

    pub pipeline_layout: vk::PipelineLayout,
    pub resources: Resource<'a>,

    pub queue_is_done_fen: vk::Fence,

    pub image_aquired_semp: vk::Semaphore,
    pub render_is_done: vk::Semaphore,
}

impl<'a> VulkanContext<'a> {
    const APPLICATION_NAME: &'static str = "Vulkan App";

    pub fn new() -> Self {
        unsafe {
            // should remove all must do things from here or keep it here and move the not must do things to fn main

            let event_loop = EventLoop::new().unwrap();
            let window = WindowBuilder::new()
                .with_title(Self::APPLICATION_NAME)
                .with_inner_size(winit::dpi::LogicalSize::new(f64::from(1920.0), f64::from(1080.0)))
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

            let instance = Arc::new(instance);

            /*Create Allocator */
            let mut allocator_info = vk_mem::AllocatorCreateInfo::new(&instance, &device, physical);
            allocator_info.flags |= vk_mem::AllocatorCreateFlags::BUFFER_DEVICE_ADDRESS;

            let allocator = Arc::new(Allocator::new(allocator_info).expect("failed to create vma allocator"));

            let mut swapchain_images = vec![];
            let mut depth_image = AllocatedImage::default();

            let (swapchain_loader, swapchain, surface_loader, surface) =
                builder::SwapchainBuilder::new(&entry, &device, &instance, physical, &allocator, &window)
                    .select_image_format(vk::Format::B8G8R8A8_SRGB)
                    .select_sharing_mode(vk::SharingMode::EXCLUSIVE)
                    .select_presentation_mode(vk::PresentModeKHR::MAILBOX)
                    .build(&mut swapchain_images, &mut depth_image);

            let window_extent = vk::Extent2D { width: window.inner_size().width, height: window.inner_size().height };

            let debug_loader_ext = DebugLoaderEXT::new(&instance, &device);

            let main_pool = util::create_pool(&device, graphic_queue.get_family());

            let main_cmd = util::create_cmd(&device, main_pool);

            let mut resources = Resource::new(&instance, &device, physical, main_cmd, graphic_queue, &allocator, debug_loader_ext);

            /*After here is basically non general code but the only relevant code we need to look at */
            let push_constant = SkyBoxPushConstant::new();

            let comp_skybox = util::create_shader(&device, "shaders/spv/skybox.comp.spv".to_owned());

            let push_vec = vec![push_constant.push_constant_range()];
            let layout_vec = vec![resources.layout];

            let vk_pipeline = vk::PipelineLayoutCreateInfo::default()
                .flags(vk::PipelineLayoutCreateFlags::empty())
                .push_constant_ranges(&push_vec)
                .set_layouts(&layout_vec);

            let pipeline_layout = device.create_pipeline_layout(&vk_pipeline, None).unwrap();

            let compute_pipeline = ComputePipelineBuilder::new(comp_skybox).build(&device, pipeline_layout);

            let mut images = vec![];

            util::begin_cmd(&device, main_cmd);

            /*Should be outside of this initilize */
            for i in 0..swapchain_images.len() {
                let name = format!("{}_{}", "compute_skybox", i);
                images.push(resources.create_storage_image(
                    window_extent,
                    4,
                    vk::MemoryPropertyFlags::DEVICE_LOCAL,
                    vk::Format::R8G8B8A8_UNORM,
                    vk::ImageUsageFlags::TRANSFER_SRC
                        | vk::ImageUsageFlags::TRANSFER_DST
                        | vk::ImageUsageFlags::STORAGE
                        | vk::ImageUsageFlags::COLOR_ATTACHMENT,
                    std::ffi::CString::new(name).unwrap(),
                ));

                util::transition_image_general(&device, main_cmd, images.last().unwrap().image);
            }

            util::end_cmd_and_submit(&device, main_cmd, graphic_queue, vec![], vec![], vk::Fence::null());
            device.device_wait_idle().unwrap();

            Self {
                entry,
                instance,
                allocator,
                window,
                device: device.clone(),
                window_extent,
                physical: Default::default(),

                main_cmd: Default::default(),
                main_pool: Default::default(),

                graphic_queue: Default::default(),
                transfer: Default::default(),

                swapchain_loader,
                surface_loader,

                debug_messenger: Default::default(),
                debug_loader: None,

                debug_loader_ext,
                pipeline_layout: vk::PipelineLayout::null(),

                resources,
                queue_is_done_fen: util::create_fence(&device),
                image_aquired_semp: util::create_semphore(&device),
                render_is_done: util::create_semphore(&device),

                swapchain: Swapchain {
                    surface: Default::default(),
                    swapchain: Default::default(),
                    swapchain_images: Default::default(),
                    depth_image: Default::default(),
                    max_swapchain_images: 0,
                    swapchain_index: 0,
                },
            }
        }
    }

    pub fn run(&self) {
        event_loop
            .run(move |event, _control_flow| match event {
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::CloseRequested => {
                        _control_flow.exit();
                    }
                    _ => {}
                },
                _ => {
                    let descriptor_sets = vec![self.resources.set];

                    let device = self.device.clone();
                    let cmd = self.main_cmd.clone();

                    let (swapchain_index, _) = self.
                        .swapchain_loader
                        .acquire_next_image(self.swapchain.swapchain, 100000, self.image_aquired_semp, vk::Fence::null())
                        .unwrap();

                    let swapchain_image = self.swapchain.swapchain_images[swapchain_index as usize].image.clone();
                    unsafe {
                    device.reset_command_buffer(cmd, vk::CommandBufferResetFlags::empty()).unwrap();

                    device
                        .begin_command_buffer(cmd, &vk::CommandBufferBeginInfo::default().flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT))
                        .expect("failed to begin cmd");

                    device.cmd_bind_descriptor_sets(
                        cmd,
                        vk::PipelineBindPoint::COMPUTE,
                        self.pipeline_layout,
                        0,
                        &descriptor_sets,
                        &vec![],
                    );

                    device.cmd_bind_pipeline(cmd, vk::PipelineBindPoint::COMPUTE, pipeline);

                    device.cmd_push_constants(
                        cmd,
                        self.pipeline_layout,
                        vk::ShaderStageFlags::COMPUTE,
                        0,
                        std::slice::from_raw_parts(&push_constant as *const _ as *const u8, std::mem::size_of::<SkyBoxPushConstant>()),
                    );

                    device.cmd_dispatch(cmd, vulkan_context.window_extent.width / 16, vulkan_context.window_extent.height / 16, 1);

                    util::copy_to_image_from_image(
                        &vulkan_context,
                        &images[0],
                        &vulkan_context.swapchain.swapchain_images[swapchain_index as usize],
                        vulkan_context.window_extent,
                    );

                    util::transition_image_color(&device, cmd, swapchain_image);

                    // fragment rendering will happen here

                    util::transition_image_present(&device, cmd, swapchain_image);

                    vulkan_context.device.end_command_buffer(vulkan_context.main_cmd).unwrap();

                    let aquire_is_ready = vec![vulkan_context.image_aquired_semp];
                    let render_is_done = vec![vulkan_context.render_is_done];
                    let swapchain_indices = vec![swapchain_index];
                    let wait_mask = vec![vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
                    let cmds = vec![vulkan_context.main_cmd];
                    let swapchain = vec![vulkan_context.swapchain.swapchain];

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
                        .queue_submit(vulkan_context.graphic_queue.queue, &submit_info, vk::Fence::null())
                        .unwrap();

                    vulkan_context
                        .swapchain_loader
                        .queue_present(vulkan_context.graphic_queue.queue, &present_info)
                        .unwrap();

                    vulkan_context.device.device_wait_idle().unwrap();
                }
                }
            })
            .unwrap();
    }
}
#[derive(Debug, Clone, Copy)]
pub struct TKQueue {
    pub queue: vk::Queue,
    pub family: u32,
}

impl Default for TKQueue {
    fn default() -> Self {
        Self { queue: Default::default(), family: Default::default() }
    }
}

impl TKQueue {
    pub fn get_family(&self) -> u32 {
        self.family
    }
    pub fn get_queue(&self) -> vk::Queue {
        self.queue
    }

    pub fn find_queue(instance: ash::Instance, physical: vk::PhysicalDevice, queue_flag: QueueFlags) -> Option<Self> {
        unsafe {
            let queues = instance.get_physical_device_queue_family_properties(physical);
            let mut queue: Option<TKQueue> = None;

            for (index, family) in queues.iter().enumerate() {
                if family.queue_flags.contains(queue_flag) {
                    let tk_queue = TKQueue { queue: vk::Queue::null(), family: index as u32 };
                    queue = Some(tk_queue);
                    break;
                }
            }
            queue
        }
    }
    pub fn find_transfer_only(instance: ash::Instance, physical: vk::PhysicalDevice) -> Option<Self> {
        let queues = unsafe { instance.get_physical_device_queue_family_properties(physical) };
        let mut transfer_queue: Option<TKQueue> = None;

        for (index, family) in queues.iter().enumerate() {
            if !family.queue_flags.contains(QueueFlags::GRAPHICS) && family.queue_flags.contains(QueueFlags::TRANSFER) {
                let tk_queue = TKQueue { queue: vk::Queue::null(), family: index as u32 };
                transfer_queue = Some(tk_queue);
                break;
            }
        }
        transfer_queue
    }
}
