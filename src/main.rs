#![feature(inherent_associated_types)]

use std::{
    ffi::CString,
    os::unix::thread,
    thread::Thread,
    time::{Duration, Instant},
};

use ash::vk;
use env_logger::Builder;
use object::SimplexNoise;
use vulkan::{
    builder::{self, ComputePipelineBuilder},
    init,
    mesh::VertexBlock,
    resource::{AllocatedBuffer, AllocatedImage, BufferType, Memory},
    util, PushConstant, SkyBoxPushConstant, VulkanContext,
};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{EventLoop, EventLoopWindowTarget},
    window::WindowBuilder,
};
mod camera;
mod object;
mod vulkan;
pub const MAX_FRAMES_IN_FLIGHT: usize = 2;

/// There should only be application relevant information in these functions
struct Application {
    vulkan: VulkanContext,
    compute: vk::Pipeline,
    compute_images: Vec<AllocatedImage>,

    push_constant: SkyBoxPushConstant,
    last_frame: Instant,

    pipeline: vk::Pipeline,
    vertex_buffer: AllocatedBuffer,
}

extern crate ultraviolet as glm;

impl Application {
    const HZ_MAX: i64 = (1000.0 / 60.0) as i64;
    fn new(event_loop: &EventLoop<()>) -> Self {
        Builder::new().filter_level(log::LevelFilter::Info).init();

        let mut vulkan = VulkanContext::new(event_loop, MAX_FRAMES_IN_FLIGHT, true);

        let comp_skybox = util::create_shader(&vulkan.device, "shaders/spv/skybox.comp.spv".to_owned());
        let compute = ComputePipelineBuilder::new(comp_skybox).build(&vulkan.device, vulkan.pipeline_layout);
        let mut images = vec![];
        let mesh = VertexBlock::get_mesh();

        let vertex_buffer = vulkan.resources.create_buffer_non_descriptor(
            mesh.len() as u64 * size_of::<VertexBlock>() as u64,
            BufferType::Vertex,
            Memory::Local,
            vulkan.graphic.family,
            "vertexBuffer".to_owned(),
        );

        util::begin_cmd(&vulkan.device, vulkan.cmds[0]);

        vulkan
            .resources
            .write_to_buffer_local(vulkan.cmds[0], &vertex_buffer, util::slice_as_u8(&mesh));

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
        }

        util::end_cmd_and_submit(&vulkan.device, vulkan.cmds[0], vulkan.graphic, vec![], vec![], vk::Fence::null());
        unsafe { vulkan.device.device_wait_idle().unwrap() };

        let vertex = util::create_shader(&vulkan.device, "shaders/spv/colored_triangle.vert.spv".to_owned());
        let frag = util::create_shader(&vulkan.device, "shaders/spv/colored_triangle.frag.spv".to_owned());

        let pipeline = builder::PipelineBuilder::new()
            .add_layout(vulkan.pipeline_layout)
            .add_color_format(vulkan.get_swapchain_format())
            .add_depth(vulkan.get_depth_format(), true, true, vk::CompareOp::LESS_OR_EQUAL)
            .cull_mode(vk::CullModeFlags::BACK, vk::FrontFace::CLOCKWISE)
            .add_topology(vk::PrimitiveTopology::TRIANGLE_LIST)
            .build::<VertexBlock>(&vulkan.device, vertex, frag);

        Self {
            vulkan,
            compute,
            compute_images: images,
            push_constant: SkyBoxPushConstant::new(),
            last_frame: Instant::now(),
            pipeline,
            vertex_buffer,
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

        self.push_constant.image_index = self.compute_images[self.vulkan.current_frame].index as u32;

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

        self.vulkan.begin_rendering();

        device.cmd_bind_pipeline(cmd, vk::PipelineBindPoint::GRAPHICS, self.pipeline);

        device.cmd_bind_vertex_buffers(cmd, 0, &[self.vertex_buffer.buffer], &vec![0]);

        self.vulkan.end_rendering();

        let imgui = self.vulkan.imgui.as_mut().unwrap();

        let ui = imgui.get_draw_instance(&self.vulkan.window);

        ui.input_float4("Data1", &mut self.push_constant.data1).build();
        ui.input_float4("Data2", &mut self.push_constant.data2).build();
        ui.input_float4("Data3", &mut self.push_constant.data3).build();
        ui.input_float4("Data4", &mut self.push_constant.data4).build();

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

        let hz_diff = Self::HZ_MAX - diff.as_millis() as i64;

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
                _control_flow.set_control_flow(winit::event_loop::ControlFlow::Poll);
                match event {
                    Event::WindowEvent { event, .. } => match event {
                        WindowEvent::CloseRequested => {
                            _control_flow.exit();
                        }
                        WindowEvent::RedrawRequested => {
                            self.draw();
                        }

                        _ => {}
                    },
                    Event::AboutToWait => {}
                    // new frame
                    Event::NewEvents(_) => {}
                    Event::LoopExiting => {
                        // Cleanup resources here
                    }
                    _ => {}
                }

                let now = Instant::now();
                let delta_time = now - self.last_frame;

                self.vulkan.imgui.as_mut().unwrap().update_delta_time(delta_time);
                self.last_frame = now;
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
    let frequency = 0.10;
    let seed = 52;
    for x in 0..10 {
        for y in 0..10 {
            let noise = SimplexNoise::two_d(x as f32, y as f32);
            println!("Noise: {:?}", noise);
            let height = ((noise + 1.0) * 0.5 * 256.0).round();

            println!("({:?},{:?}) = {:?}", x, y, height);
        }
    }

    panic!();
    let event_loop = EventLoop::new().unwrap();
    let mut application = Application::new(&event_loop);
    unsafe {
        application.run(event_loop);
    }
}
