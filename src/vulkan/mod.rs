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

#[repr(C, align(16))]
pub struct SkyBoxPushConstant {
    data1: glm::Vec4,
    data2: glm::Vec4,
    data3: glm::Vec4,
    data4: glm::Vec4,
    pub image_index: u32,
}

impl SkyBoxPushConstant {
    pub fn new() -> Self {
        Self {
            data1: glm::vec4(0.5, 0.5, 0.5, 0.5),
            data2: glm::vec4(0.5, 0.5, 0.5, 0.5),
            data3: glm::vec4(1.0, 1.0, 1.0, 1.0),
            data4: glm::vec4(1.0, 1.0, 1.0, 1.0),
            image_index: 0,
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
                .add_color_format(vulkan.swapchain.images[0].format)
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
    pub swap: vk::SwapchainKHR,
    pub images: Vec<AllocatedImage>,
    pub depth: AllocatedImage,
    pub image_index: u32,
}

pub struct VulkanContext {
    pub entry: Arc<ash::Entry>,
    pub instance: Arc<ash::Instance>,
    pub device: Arc<ash::Device>,
    pub physical: vk::PhysicalDevice,
    /// Don't forget to clean this one up
    pub allocator: Arc<vk_mem::Allocator>,

    pub window_extent: vk::Extent2D,
    pub window: Arc<winit::window::Window>,

    pub swapchain: Swapchain,

    pub cmds: Vec<vk::CommandBuffer>,
    pub pools: Vec<vk::CommandPool>,

    pub graphic: TKQueue,
    pub transfer: TKQueue,

    pub debug_messenger: vk::DebugUtilsMessengerEXT,

    pub swapchain_loader: ash::khr::swapchain::Device,
    pub surface_loader: ash::khr::surface::Instance,
    pub debug_loader: Option<ash::ext::debug_utils::Instance>,

    pub debug_loader_ext: DebugLoaderEXT,

    pub pipeline_layout: vk::PipelineLayout,
    pub resources: Resource,

    pub queue_done: Vec<vk::Fence>,

    pub aquired_semp: Vec<vk::Semaphore>,
    pub render_done_signal: Vec<vk::Semaphore>,

    pub current_frame: usize,

    pub max_frames_in_flight: usize,
}

impl VulkanContext {
    const APPLICATION_NAME: &'static str = "Vulkan App";

    pub fn new(event_loop: &EventLoop<()>, max_frames_in_flight: usize) -> Self {
        unsafe {
            // should remove all must do things from here or keep it here and move the not must do things to fn main

            let window = Arc::new(
                WindowBuilder::new()
                    .with_title(Self::APPLICATION_NAME)
                    .with_inner_size(winit::dpi::LogicalSize::new(f64::from(1920.0), f64::from(1080.0)))
                    .build(event_loop)
                    .unwrap(),
            );

            let (instance, entry, debug_callback, debug_loader) = builder::InstanceBuilder::new()
                .enable_debug()
                .set_required_version(1, 3, 0)
                .set_app_name("Vulkan App")
                .set_xlib_ext()
                .build();
            let (device, physical, graphic, transfer) = builder::DeviceBuilder::new()
                .ext_dynamic_rendering()
                .ext_image_cube_array()
                .ext_sampler_anisotropy()
                .ext_bindless_descriptors()
                .select_physical_device(&instance)
                .build(&instance);

            let instance = Arc::new(instance);
            let entry = Arc::new(entry);
            let device = Arc::new(device);

            /*Create Allocator */
            let mut allocator_info = vk_mem::AllocatorCreateInfo::new(&instance, &device, physical);
            allocator_info.flags |= vk_mem::AllocatorCreateFlags::BUFFER_DEVICE_ADDRESS;

            let allocator = Arc::new(Allocator::new(allocator_info).expect("failed to create vma allocator"));

            let mut swapchain_images = vec![];
            let mut depth_image = AllocatedImage::default();
            let window_extent = vk::Extent2D { width: window.inner_size().width, height: window.inner_size().height };

            let (swapchain_loader, swapchain, surface_loader, surface) =
                builder::SwapchainBuilder::new(entry.clone(), device.clone(), instance.clone(), physical, allocator.clone(), window.clone())
                    .add_extent(window_extent)
                    .select_image_format(vk::Format::B8G8R8A8_SRGB)
                    .select_sharing_mode(vk::SharingMode::EXCLUSIVE)
                    .select_presentation_mode(vk::PresentModeKHR::MAILBOX)
                    .build(&mut swapchain_images, &mut depth_image);

            let debug_loader_ext = DebugLoaderEXT::new(instance.clone(), device.clone());

            let resources = Resource::new(instance.clone(), device.clone(), graphic, allocator.clone(), debug_loader_ext.clone());

            let push_vec = vec![vk::PushConstantRange::default()
                .size(128)
                .stage_flags(ShaderStageFlags::VERTEX | ShaderStageFlags::FRAGMENT | ShaderStageFlags::COMPUTE)];

            let layout_vec = vec![resources.layout];

            let vk_pipeline = vk::PipelineLayoutCreateInfo::default()
                .flags(vk::PipelineLayoutCreateFlags::empty())
                .push_constant_ranges(&push_vec)
                .set_layouts(&layout_vec);

            let pipeline_layout = device.create_pipeline_layout(&vk_pipeline, None).unwrap();

            let mut present_done = vec![];
            let mut aquired_semp = vec![];
            let mut render_done = vec![];
            let mut cmds = vec![];
            let mut pools = vec![];

            for i in 0..max_frames_in_flight {
                present_done.push(util::create_fence(&device));
                aquired_semp.push(util::create_semphore(&device));
                render_done.push(util::create_semphore(&device));

                let main_pool = util::create_pool(&device, graphic.get_family());
                cmds.push(util::create_cmd(&device, main_pool));
                pools.push(main_pool);
            }

            Self {
                entry,
                instance,
                allocator,
                window,
                device,
                window_extent,
                physical,

                cmds,
                pools,

                graphic,
                transfer,

                swapchain_loader,
                surface_loader,

                debug_messenger: debug_callback,
                debug_loader: Some(debug_loader),

                debug_loader_ext,
                pipeline_layout,

                resources,
                queue_done: present_done,
                aquired_semp,
                render_done_signal: render_done,

                swapchain: Swapchain { surface, swap: swapchain, images: swapchain_images, depth: depth_image, image_index: 0 },
                current_frame: 0,
                max_frames_in_flight,
            }
        }
    }
    // TODO, fix default syncing things
    pub fn prepare_frame(&mut self) {
        unsafe {
            self.device
                .wait_for_fences(&[self.queue_done[self.current_frame]], true, u64::MAX - 1)
                .unwrap();
            self.device.reset_fences(&[self.queue_done[self.current_frame]]).unwrap();

            let signal_image_aquired = self.aquired_semp[self.current_frame];

            (self.swapchain.image_index, _) = self
                .swapchain_loader
                .acquire_next_image(self.swapchain.swap, 100000, signal_image_aquired, vk::Fence::null())
                .unwrap();

            util::begin_cmd(&self.device, self.cmds[self.current_frame]);
        }
    }

    // TODO, fix default syncing and submitting
    pub fn end_frame_and_submit(&mut self) {
        let cmd = self.cmds[self.current_frame];

        util::transition_image_present(&self.device, cmd, self.swapchain.images[self.swapchain.image_index as usize].image);

        util::end_cmd_and_submit(
            &self.device,
            cmd,
            self.graphic,
            vec![self.render_done_signal[self.current_frame]],
            vec![self.aquired_semp[self.current_frame]],
            self.queue_done[self.current_frame],
        );
        util::present_submit(
            &self.swapchain_loader,
            self.graphic,
            self.swapchain.swap,
            self.swapchain.image_index,
            vec![self.render_done_signal[self.current_frame]],
        );

        self.current_frame = (self.current_frame + 1) % self.max_frames_in_flight;
        self.swapchain.image_index = (self.swapchain.image_index + 1) % self.swapchain.images.len() as u32;
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
