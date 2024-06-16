use core::slice;
use std::{
    mem,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use ash::{
    khr::dynamic_rendering,
    vk::{self, BlendFactor, BlendOp, DescriptorType, Extent2D, Offset2D, PrimitiveTopology, QueueFlags, ShaderStageFlags},
};
use builder::{ComputePipelineBuilder, PipelineBuilder, SwapchainBuilder};
use imgui::{draw_list, FontConfig, FontSource, TextureId};
use imgui_rs_vulkan_renderer::Renderer;
use imgui_winit_support::{HiDpiMode, WinitPlatform};
use loader::DebugLoaderEXT;
use mesh::MeshImGui;
use resource::{AllocatedBuffer, AllocatedImage, BufferType, Memory, Resource};
use vk_mem::{Alloc, Allocator};
use winit::{
    event::Event,
    event_loop::EventLoop,
    window::{Window, WindowBuilder},
};

use crate::{camera::Camera, MAX_FRAMES_IN_FLIGHT};

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
    pub data1: [f32; 4],
    pub data2: [f32; 4],
    pub data3: [f32; 4],
    pub data4: [f32; 4],
    pub image_index: u32,
}

impl SkyBoxPushConstant {
    pub fn new() -> Self {
        Self {
            data1: [0.0, 0.1, 1.0, 0.980],
            data2: [0.5, 0.5, 0.5, 0.5],
            data3: [0.5, 0.5, 0.5, 0.5],
            data4: [0.5, 0.5, 0.5, 0.5],
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

#[repr(C, align(16))]
struct ImguiPushConstant {
    ortho_mat: glm::Mat4,
    texture_index: u32,
}

pub struct ImguiContext {
    pub device: Arc<ash::Device>,

    pub imgui: imgui::Context,
    pub platform: WinitPlatform,

    pub pipeline: vk::Pipeline,
    pub layout: vk::PipelineLayout,
    pub texture_atlas: AllocatedImage,

    pub texture: imgui::Textures<vk::DescriptorSet>,

    pub vertex_buffers: Vec<AllocatedBuffer>,
    pub index_buffers: Vec<AllocatedBuffer>,

    pub graphic_queue: TKQueue,
}

impl ImguiContext {
    fn new(
        window: &winit::window::Window,
        device: Arc<ash::Device>,
        instance: Arc<ash::Instance>,
        resource: &mut Resource,
        layout: vk::PipelineLayout,
        swapchain_format: vk::Format,
        graphic: TKQueue,
    ) -> Self {
        let mut imgui = imgui::Context::create();
        imgui.set_ini_filename(None);

        let mut platform = WinitPlatform::init(&mut imgui);
        let hidpi_factor = platform.hidpi_factor();
        let font_size = (13.0 * hidpi_factor) as f32;

        imgui
            .fonts()
            .add_font(&[FontSource::DefaultFontData { config: Some(FontConfig { size_pixels: font_size, ..FontConfig::default() }) }]);

        imgui.io_mut().font_global_scale = (1.0 / hidpi_factor) as f32;
        platform.attach_window(imgui.io_mut(), window, HiDpiMode::Rounded);

        unsafe {
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
                .add_color_format(swapchain_format)
                .add_layout(layout)
                .add_topology(PrimitiveTopology::TRIANGLE_LIST)
                .add_blend(blend_state)
                .build::<MeshImGui>(&device, shader_vert, shader_frag);

            device.destroy_shader_module(shader_frag, None);
            device.destroy_shader_module(shader_vert, None);

            let fonts_texture = {
                let fonts = imgui.fonts();
                let atlas_texture = fonts.build_rgba32_texture();

                resource.create_texture_image(Extent2D { width: atlas_texture.width, height: atlas_texture.height }, atlas_texture.data)
            };

            let fonts = imgui.fonts();
            fonts.tex_id = TextureId::from(usize::MAX);

            let texture = imgui::Textures::new();

            // TODO create vertex/index buffers
            let mut vertex_buffers = vec![];
            let mut index_buffers = vec![];

            for i in 0..MAX_FRAMES_IN_FLIGHT {
                let vertex_name = format!("ImguiVertex{:?}", i);
                let index_name = format!("ImguiIndex{:?}", i);

                let starter_vertex_size = mem::size_of::<MeshImGui>() as u64 * 1000;
                let starter_index_size = mem::size_of::<u16>() as u64 * 100;

                let vertex =
                    resource.create_buffer_non_descriptor(starter_vertex_size, BufferType::Vertex, Memory::Host, graphic.family, vertex_name);

                let index = resource.create_buffer_non_descriptor(starter_index_size, BufferType::Index, Memory::Host, graphic.family, index_name);

                vertex_buffers.push(vertex);
                index_buffers.push(index);
            }

            log::info!("Imgui Context Initialized");

            Self {
                imgui,
                platform,
                pipeline,
                texture_atlas: fonts_texture,
                texture,
                vertex_buffers,
                index_buffers,
                graphic_queue: graphic,
                device,
                layout,
            }
        }
    }

    pub fn get_draw_instance(&mut self, window: &Window) -> &mut imgui::Ui {
        self.platform.prepare_frame(self.imgui.io_mut(), window).expect("failed to prepare imgui");
        self.imgui.frame()
    }

    pub fn render(
        &mut self,
        extent: vk::Extent2D,
        present_image: &AllocatedImage,
        frame_index: usize,
        res: &mut Resource,
        cmd: vk::CommandBuffer,
        window: &Window,
        set: vk::DescriptorSet,
    ) {
        unsafe {
            let draw_data = self.imgui.render();

            /*Updating buffers */
            let (vertices, indices) = MeshImGui::create_mesh(draw_data);

            let current_vertex_size = self.vertex_buffers[frame_index].size;
            let needed_vertex_size = vertices.len() as u64 * mem::size_of::<MeshImGui>() as u64;

            if needed_vertex_size > current_vertex_size {
                res.resize_buffer_non_descriptor(&mut self.vertex_buffers[frame_index], needed_vertex_size);
            }

            let current_index_size = self.index_buffers[frame_index].size;
            let needed_index_size = indices.len() as u64 * 2;

            if needed_index_size > current_index_size {
                res.resize_buffer_non_descriptor(&mut self.index_buffers[frame_index], needed_index_size);
            }

            let slice: &[u8] = slice::from_raw_parts(vertices.as_ptr() as *const u8, vertices.len() * mem::size_of::<imgui::DrawVert>() as usize);
            res.write_to_buffer_host(&mut self.vertex_buffers[frame_index], slice);

            let index_slice = slice::from_raw_parts(indices.as_ptr() as *const u8, indices.len() * 2);
            res.write_to_buffer_host(&mut self.index_buffers[frame_index], index_slice);

            /*RENDERING */
            let offset = vk::Offset2D::default().x(0).y(0);
            let attachment = vk::RenderingAttachmentInfo::default()
                .clear_value(vk::ClearValue::default())
                .image_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
                .image_view(present_image.view)
                .store_op(vk::AttachmentStoreOp::STORE)
                .load_op(vk::AttachmentLoadOp::LOAD);

            // start rendering
            self.device.cmd_begin_rendering(
                cmd,
                &vk::RenderingInfo::default()
                    .color_attachments(&[attachment])
                    .layer_count(1)
                    .render_area(vk::Rect2D { offset, extent }),
            );

            let view_port = vk::Viewport::default()
                .height(extent.height as f32)
                .width(extent.width as f32)
                .max_depth(1.0)
                .min_depth(0.0);

            self.device.cmd_set_viewport(cmd, 0, &[view_port]);

            self.device.cmd_bind_pipeline(cmd, vk::PipelineBindPoint::GRAPHICS, self.pipeline);

            self.device
                .cmd_bind_index_buffer(cmd, self.index_buffers[frame_index].buffer, 0, vk::IndexType::UINT16);
            self.device
                .cmd_bind_vertex_buffers(cmd, 0, &[self.vertex_buffers[frame_index].buffer], &[0]);

            let push_constant =
                [ImguiPushConstant { ortho_mat: Camera::ortho(draw_data.display_size[0], -draw_data.display_size[1]), texture_index: 0 }];

            let slice = { slice::from_raw_parts(push_constant.as_ptr() as *const u8, mem::size_of::<ImguiPushConstant>()) };

            self.device
                .cmd_bind_descriptor_sets(cmd, vk::PipelineBindPoint::GRAPHICS, self.layout, 0, &[set], &[]);

            self.device.cmd_push_constants(
                cmd,
                self.layout,
                vk::ShaderStageFlags::COMPUTE | vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT,
                0,
                slice,
            );

            let mut index_offset = 0;
            let mut vertex_offset = 0;
            let mut current_texture_id: Option<TextureId> = None;
            let clip_offset = draw_data.display_pos;
            let clip_scale = draw_data.framebuffer_scale;

            for draw_list in draw_data.draw_lists() {
                for command in draw_list.commands() {
                    match command {
                        imgui::DrawCmd::Elements { count, cmd_params: imgui::DrawCmdParams { clip_rect, texture_id, vtx_offset, idx_offset } } => {
                            let clip_x = (clip_rect[0] - clip_offset[0]) * clip_scale[0];
                            let clip_y = (clip_rect[1] - clip_offset[1]) * clip_scale[1];
                            let clip_w = (clip_rect[2] - clip_offset[0]) * clip_scale[0] - clip_x;
                            let clip_h = (clip_rect[3] - clip_offset[1]) * clip_scale[1] - clip_y;

                            let scissors = [vk::Rect2D {
                                offset: vk::Offset2D { x: (clip_x as i32).max(0), y: (clip_y as i32).max(0) },
                                extent: vk::Extent2D { width: clip_w as _, height: clip_h as _ },
                            }];

                            self.device.cmd_set_scissor(cmd, 0, &scissors);

                            if Some(texture_id) != current_texture_id {
                                if current_texture_id.is_some() {
                                    println!("multiple ones");
                                }
                                current_texture_id = Some(texture_id);
                            }

                            self.device
                                .cmd_draw_indexed(cmd, count as _, 1, index_offset + idx_offset as u32, vertex_offset + vtx_offset as i32, 0)
                        }
                        imgui::DrawCmd::ResetRenderState => todo!(),
                        imgui::DrawCmd::RawCallback { callback, raw_cmd } => todo!(),
                    }
                }

                index_offset += draw_list.idx_buffer().len() as u32;
                vertex_offset += draw_list.vtx_buffer().len() as i32;
            }

            self.device.cmd_end_rendering(cmd);
        } // Update both
    }

    pub fn update_delta_time(&mut self, delta_time: Duration) {
        self.imgui.io_mut().update_delta_time(delta_time);
    }

    pub fn process_event_imgui(&mut self, window: &winit::window::Window, event: &Event<()>) {
        self.platform.handle_event(self.imgui.io_mut(), window, event);
    }

    pub fn recreate_swapchain(&mut self) {}
}
pub struct Swapchain {
    pub surface: vk::SurfaceKHR,
    pub swap: vk::SwapchainKHR,
    pub images: Vec<AllocatedImage>,
    pub depth: AllocatedImage,
    pub image_index: u32,
    pub present_mode: vk::PresentModeKHR,
}

///Initialization all of Vulkan and has some default syncing and submitting
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

    pub imgui: Option<ImguiContext>,
}

impl VulkanContext {
    const APPLICATION_NAME: &'static str = "Vulkan App";

    pub fn new(event_loop: &EventLoop<()>, max_frames_in_flight: usize, is_imgui: bool) -> Self {
        unsafe {
            // should remove all must do things from here or keep it here and move the not must do things to fn main
            let window = Arc::new(
                WindowBuilder::new()
                    .with_title(Self::APPLICATION_NAME)
                    .with_inner_size(winit::dpi::LogicalSize::new(f64::from(1024), f64::from(768)))
                    .build(event_loop)
                    .unwrap(),
            );

            let (instance, entry, debug_callback, debug_loader) = builder::InstanceBuilder::new()
                .enable_debug()
                .set_required_version(1, 3, 0)
                .set_app_name("Vulkan App")
                .set_xlib_ext()
                .build();

            log::info!("Vulkan instance is built");
            let (device, physical, graphic, transfer) = builder::DeviceBuilder::new()
                .ext_dynamic_rendering()
                .ext_image_cube_array()
                .ext_sampler_anisotropy()
                .ext_bindless_descriptors()
                .select_physical_device(&instance)
                .build(&instance);
            log::info!("device instance is built");

            let instance = Arc::new(instance);
            let entry = Arc::new(entry);
            let device = Arc::new(device);

            /*Create Allocator */
            let mut allocator_info = vk_mem::AllocatorCreateInfo::new(&instance, &device, physical);
            allocator_info.flags |= vk_mem::AllocatorCreateFlags::BUFFER_DEVICE_ADDRESS;
            let allocator = Arc::new(Allocator::new(allocator_info).expect("failed to create vma allocator"));

            let debug_loader_ext = DebugLoaderEXT::new(instance.clone(), device.clone());

            let window_extent = vk::Extent2D { width: window.inner_size().width, height: window.inner_size().height };

            let mut resources = Resource::new(instance.clone(), device.clone(), graphic, allocator.clone(), debug_loader_ext.clone());
            log::info!("Resources intialized");
            let mut swapchain_images = vec![];
            let mut depth_image = AllocatedImage::default();
            let present_mode = vk::PresentModeKHR::MAILBOX;

            let (swapchain_loader, swapchain, surface_loader, surface) =
                builder::SwapchainBuilder::new(entry.clone(), device.clone(), instance.clone(), physical, allocator.clone(), window.clone())
                    .add_extent(window_extent)
                    .select_image_format(vk::Format::B8G8R8A8_SRGB)
                    .select_sharing_mode(vk::SharingMode::EXCLUSIVE)
                    .select_presentation_mode(vk::PresentModeKHR::MAILBOX)
                    .build(&mut resources, &mut swapchain_images, &mut depth_image);

            log::info!("swapchain initialized");

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

            for _ in 0..max_frames_in_flight {
                present_done.push(util::create_fence(&device));
                aquired_semp.push(util::create_semphore(&device));
                render_done.push(util::create_semphore(&device));

                let main_pool = util::create_pool(&device, graphic.get_family());
                cmds.push(util::create_cmd(&device, main_pool));
                pools.push(main_pool);
            }
            let imgui = {
                if is_imgui {
                    Some(ImguiContext::new(
                        &window,
                        device.clone(),
                        instance.clone(),
                        &mut resources,
                        pipeline_layout,
                        swapchain_images[0].format,
                        graphic,
                    ))
                } else {
                    None
                }
            };
            log::info!("Vulkan context initialized");
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

                swapchain: Swapchain { surface, swap: swapchain, images: swapchain_images, depth: depth_image, image_index: 0, present_mode },
                current_frame: 0,
                max_frames_in_flight,
                imgui,
            }
        }
    }

    pub fn recreate_swapchain(&mut self, new_extent: vk::Extent2D) {
        self.window_extent = new_extent;
        unsafe {
            let builder = SwapchainBuilder::new(
                self.entry.clone(),
                self.device.clone(),
                self.instance.clone(),
                self.physical,
                self.allocator.clone(),
                self.window.clone(),
            )
            .add_extent(new_extent)
            .select_image_format(self.swapchain.images[0].format)
            .select_presentation_mode(self.swapchain.present_mode)
            .select_sharing_mode(vk::SharingMode::EXCLUSIVE);

            self.swapchain_loader.destroy_swapchain(self.swapchain.swap, None);
            for image in &mut self.swapchain.images {
                self.device.destroy_image(image.image, None);
                self.device.destroy_image_view(image.view, None);
            }

            self.swapchain.images.clear();
            self.allocator
                .destroy_image(self.swapchain.depth.image, &mut self.swapchain.depth.alloc.as_mut().unwrap());

            builder.build(&mut self.resources, &mut self.swapchain.images, &mut self.swapchain.depth);
        }
    }

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

    pub fn begin_rendering(&self, load: vk::AttachmentLoadOp) {
        unsafe {
            let attachment = vk::RenderingAttachmentInfo::default()
                .image_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
                .load_op(load)
                .store_op(vk::AttachmentStoreOp::STORE)
                .image_view(self.get_swapchain_image().view);

            let depth_attachment = vk::RenderingAttachmentInfo::default()
                .image_layout(vk::ImageLayout::DEPTH_ATTACHMENT_OPTIMAL)
                .load_op(vk::AttachmentLoadOp::CLEAR)
                .store_op(vk::AttachmentStoreOp::STORE)
                .image_view(self.swapchain.depth.view);

            self.device.cmd_begin_rendering(
                self.cmds[self.current_frame],
                &vk::RenderingInfo::default()
                    .color_attachments(&[attachment])
                    .depth_attachment(&depth_attachment)
                    .layer_count(1)
                    .render_area(vk::Rect2D { offset: Offset2D::default(), extent: self.window_extent }),
            )
        }
    }

    pub fn end_rendering(&self) {
        unsafe { self.device.cmd_end_rendering(self.cmds[self.current_frame as usize]) };
    }

    pub fn process_imgui_event(&mut self, event: &Event<()>) {
        self.imgui.as_mut().unwrap().process_event_imgui(&self.window, event);
    }

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

    pub fn get_swapchain_format(&self) -> vk::Format {
        self.swapchain.images[0].format
    }

    pub fn get_swapchain_image(&self) -> &AllocatedImage {
        &self.swapchain.images[self.swapchain.image_index as usize]
    }

    pub fn get_depth_format(&self) -> vk::Format {
        self.swapchain.depth.format
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
