#![feature(inherent_associated_types)]

use std::{
    ffi::CString,
    os::unix::thread,
    thread::Thread,
    time::{Duration, Instant},
};

use ash::vk;
use vulkan::{
    builder::{self, ComputePipelineBuilder},
    resource::AllocatedImage,
    util, PushConstant, SkyBoxPushConstant, VulkanContext,
};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{EventLoop, EventLoopWindowTarget},
    window::WindowBuilder,
};
mod camera;
mod vulkan;
pub const MAX_FRAMES_IN_FLIGHT: usize = 2;

/// There should only be application relevant information in these functions
struct Application {
    vulkan: VulkanContext,
    compute: vk::Pipeline,
    compute_images: Vec<AllocatedImage>,

    push_constant: SkyBoxPushConstant,
    last_frame: Instant,
}

extern crate ultraviolet as glm;

impl Application {
    const HZ_MAX: u128 = (1000.0 / 60.0) as u128;
    fn new(event_loop: &EventLoop<()>) -> Self {
        let mut vulkan = VulkanContext::new(event_loop, MAX_FRAMES_IN_FLIGHT, true);

        let comp_skybox = util::create_shader(&vulkan.device, "shaders/spv/skybox.comp.spv".to_owned());
        let compute = ComputePipelineBuilder::new(comp_skybox).build(&vulkan.device, vulkan.pipeline_layout);
        let mut images = vec![];
        util::begin_cmd(&vulkan.device, vulkan.cmds[0]);
        /*Should be outside of this initilize */
        for i in 0..MAX_FRAMES_IN_FLIGHT {
            let name = format!("{}_{}", "compute_skybox", i);
            images.push(vulkan.resources.create_storage_image(
                vulkan.window_extent,
                4,
                vk::MemoryPropertyFlags::DEVICE_LOCAL,
                vk::Format::R8G8B8A8_UNORM,
                vk::ImageUsageFlags::TRANSFER_SRC
                    | vk::ImageUsageFlags::TRANSFER_DST
                    | vk::ImageUsageFlags::STORAGE
                    | vk::ImageUsageFlags::COLOR_ATTACHMENT,
                std::ffi::CString::new(name).unwrap(),
            ));

            util::transition_image_general(&vulkan.device, vulkan.cmds[0], images.last().unwrap().image);
        }

        util::end_cmd_and_submit(&vulkan.device, vulkan.cmds[0], vulkan.graphic, vec![], vec![], vk::Fence::null());
        unsafe { vulkan.device.device_wait_idle().unwrap() };

        Self {
            vulkan,
            compute,
            compute_images: images,
            push_constant: SkyBoxPushConstant::new(),
            last_frame: Instant::now(),
        }
    }

    unsafe fn draw(&mut self) {
        self.vulkan.prepare_frame();

        let device = &self.vulkan.device;
        let frame_index = self.vulkan.current_frame;
        let swapchain_index = self.vulkan.swapchain.image_index;
        let cmd = self.vulkan.cmds[frame_index];

        device.cmd_bind_descriptor_sets(
            cmd,
            vk::PipelineBindPoint::COMPUTE,
            self.vulkan.pipeline_layout,
            0,
            &[self.vulkan.resources.set],
            &vec![],
        );

        device.cmd_bind_pipeline(cmd, vk::PipelineBindPoint::COMPUTE, self.compute);

        self.push_constant.image_index = self.compute_images[self.vulkan.current_frame].index;

        device.cmd_push_constants(
            cmd,
            self.vulkan.pipeline_layout,
            vk::ShaderStageFlags::COMPUTE | vk::ShaderStageFlags::FRAGMENT | vk::ShaderStageFlags::VERTEX,
            0,
            std::slice::from_raw_parts(&self.push_constant as *const _ as *const u8, std::mem::size_of::<SkyBoxPushConstant>()),
        );

        device.cmd_dispatch(cmd, self.vulkan.window_extent.width / 16, self.vulkan.window_extent.height / 16, 1);

        util::copy_to_image_from_image(
            &device,
            cmd,
            &self.compute_images[frame_index],
            &self.vulkan.swapchain.images[swapchain_index as usize],
            self.vulkan.window_extent,
        );

        util::transition_image_color(&device, cmd, self.vulkan.swapchain.images[swapchain_index as usize].image);

        /*Start Rendering*/

        let imgui = self.vulkan.imgui.as_mut().unwrap();

        let u1 = imgui.get_draw_instance(&self.vulkan.window);
        u1.button("hello");

        let set = self.vulkan.resources.set;

        imgui.render(
            self.vulkan.window_extent,
            &self.vulkan.swapchain.images[self.vulkan.swapchain.image_index as usize],
            self.vulkan.current_frame,
            &mut self.vulkan.resources,
            cmd,
            &self.vulkan.window,
            set,
        );

        self.vulkan.end_frame_and_submit();

        let now = Instant::now();

        let diff = now - self.last_frame;

        let hz_diff = Self::HZ_MAX - diff.as_millis();

        if hz_diff > 0 {
            std::thread::sleep(Duration::from_millis(hz_diff as u64));
        }

        self.vulkan.window.request_redraw();
    }

    unsafe fn run(&mut self, event_loop: EventLoop<()>) {
        self.last_frame = Instant::now();

        event_loop
            .run(move |event, _control_flow| {
                self.vulkan.imgui.as_mut().unwrap().process_event_imgui(&self.vulkan.window, &event);

                match event {
                    Event::WindowEvent { event, .. } => match event {
                        WindowEvent::CloseRequested => {
                            _control_flow.exit();
                        }
                        WindowEvent::RedrawRequested => {}

                        _ => {}
                    },
                    Event::AboutToWait => {
                        self.draw();
                        // std::thread::sleep(Duration::from_secs(1));
                    }

                    // new frame
                    Event::NewEvents(_) => {
                        let now = Instant::now();
                        let delta_time = now - self.last_frame;

                        self.vulkan.imgui.as_mut().unwrap().update_delta_time(delta_time);
                        self.last_frame = now;
                    }
                    Event::LoopExiting => {
                        // Cleanup resources here
                    }
                    _ => {}
                }
            })
            .unwrap();
    }

    // need to rebuild the swapchain and any resources that depend on the window extent
    pub fn recreate_swapchain(&mut self) {
        unsafe {
            self.vulkan.device.device_wait_idle().unwrap();
        }
        let window_extent_physical = self.vulkan.window.inner_size();
        let extent = vk::Extent2D { width: window_extent_physical.width, height: window_extent_physical.height };

        self.vulkan.recreate_swapchain(extent);

        // TODO, recreate my general image
        for image in &mut self.compute_images {
            self.vulkan.resources.resize_image(image, extent);
        }
    }
}

fn main() {
    let event_loop = EventLoop::new().unwrap();
    let mut application = Application::new(&event_loop);
    unsafe {
        application.run(event_loop);
    }
}
