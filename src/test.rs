#![feature(inherent_associated_types)]

use std::{
    collections::HashMap,
    mem::transmute,
    time::{Duration, Instant},
};

use ash::vk::{self};
use env_logger::Builder;
use voxelengine::{
    app::{App, ApplicationTrait},
    core::camera::{Camera, Controls, GPUCamera},
    terrain::{
        block::GPUBlock,
        World,
    },
    vulkan::{
        builder::{self},
        mesh::{Vertex, VertexBlock},
        resource::{AllocatedBuffer, BufferBuilder, BufferIndex, BufferType, Memory},
        util, VulkanContext,
    },
};
use winit::{
    event::{self, ElementState, Event, RawKeyEvent, WindowEvent},
    event_loop::{self, EventLoop},
    keyboard::KeyCode,
    window::CursorGrabMode,
};

pub const MAX_FRAMES_IN_FLIGHT: usize = 2;

#[repr(C, align(16))]
struct PushConstant {
    pub cam_index: u32,
    pub object_index: u32,
}
#[repr(C, align(16))]
struct NodeVertex {
    position: glm::Vec3,
}

impl Vertex for NodeVertex {
    fn get_vertex_attribute_desc() -> Vec<vk::VertexInputAttributeDescription> {
        [vk::VertexInputAttributeDescription::default()
            .binding(0)
            .location(0)
            .format(vk::Format::R32G32B32_SFLOAT)
            .offset(0)]
        .to_vec()
    }
}

impl NodeVertex {
    pub const fn new(position: glm::Vec3) -> Self {
        Self { position }
    }
}

const Vertices: [NodeVertex; 36] = [
    // right
    NodeVertex::new(glm::Vec3::new(0.5, 0.5, 0.5)),
    NodeVertex::new(glm::Vec3::new(0.5, 0.5, -0.5)),
    NodeVertex::new(glm::Vec3::new(0.5, -0.5, -0.5)),
    NodeVertex::new(glm::Vec3::new(0.5, -0.5, -0.5)),
    NodeVertex::new(glm::Vec3::new(0.5, -0.5, 0.5)),
    NodeVertex::new(glm::Vec3::new(0.5, 0.5, 0.5)),
    // Left face
    NodeVertex::new(glm::Vec3::new(-0.5, 0.5, 0.5)),
    NodeVertex::new(glm::Vec3::new(-0.5, -0.5, -0.5)),
    NodeVertex::new(glm::Vec3::new(-0.5, 0.5, -0.5)),
    NodeVertex::new(glm::Vec3::new(-0.5, -0.5, -0.5)),
    NodeVertex::new(glm::Vec3::new(-0.5, 0.5, 0.5)),
    NodeVertex::new(glm::Vec3::new(-0.5, -0.5, 0.5)),
    // Top face
    NodeVertex::new(glm::Vec3::new(-0.5, 0.5, -0.5)),
    NodeVertex::new(glm::Vec3::new(0.5, 0.5, -0.5)),
    NodeVertex::new(glm::Vec3::new(0.5, 0.5, 0.5)),
    NodeVertex::new(glm::Vec3::new(0.5, 0.5, 0.5)),
    NodeVertex::new(glm::Vec3::new(-0.5, 0.5, 0.5)),
    NodeVertex::new(glm::Vec3::new(-0.5, 0.5, -0.5)),
    // Bottom face
    NodeVertex::new(glm::Vec3::new(-0.5, -0.5, -0.5)),
    NodeVertex::new(glm::Vec3::new(0.5, -0.5, 0.5)),
    NodeVertex::new(glm::Vec3::new(0.5, -0.5, -0.5)),
    NodeVertex::new(glm::Vec3::new(0.5, -0.5, 0.5)),
    NodeVertex::new(glm::Vec3::new(-0.5, -0.5, -0.5)),
    NodeVertex::new(glm::Vec3::new(-0.5, -0.5, 0.5)),
    // Front face
    NodeVertex::new(glm::Vec3::new(-0.5, -0.5, 0.5)),
    NodeVertex::new(glm::Vec3::new(0.5, 0.5, 0.5)),
    NodeVertex::new(glm::Vec3::new(0.5, -0.5, 0.5)),
    NodeVertex::new(glm::Vec3::new(0.5, 0.5, 0.5)),
    NodeVertex::new(glm::Vec3::new(-0.5, -0.5, 0.5)),
    NodeVertex::new(glm::Vec3::new(-0.5, 0.5, 0.5)),
    // Back face
    NodeVertex::new(glm::Vec3::new(-0.5, -0.5, -0.5)),
    NodeVertex::new(glm::Vec3::new(0.5, -0.5, -0.5)),
    NodeVertex::new(glm::Vec3::new(0.5, 0.5, -0.5)),
    NodeVertex::new(glm::Vec3::new(0.5, 0.5, -0.5)),
    NodeVertex::new(glm::Vec3::new(-0.5, 0.5, -0.5)),
    NodeVertex::new(glm::Vec3::new(-0.5, -0.5, -0.5)),
];

/// There should only be application relevant information in these functions
pub struct TestApplication {
    vulkan: VulkanContext,

    last_frame: Instant,

    pipeline: Vec<vk::Pipeline>,
    vertex_buffer: Vec<BufferIndex>,

    cam: Camera,
    controls: Controls,

    focus: bool,

    object_count: u32,
    objects: Vec<GPUBlock>,
    resize: bool,

    world: World,
    is_frustum: bool,

    vertex_block: Vec<VertexBlock>,

    culled: Vec<GPUBlock>,
    pipeline_index: i32,
}

impl ApplicationTrait for TestApplication {
    fn on_new(event_loop: &EventLoop<()>) -> Self {
        Builder::new().filter_level(log::LevelFilter::Info).init();

        let mut vulkan = VulkanContext::new(&event_loop, MAX_FRAMES_IN_FLIGHT, true);

        let mesh = VertexBlock::get_mesh();

        let cam = Camera::new(vulkan.window_extent);
        let world = World::new(cam.get_pos(), 4);

        //let objects = world.get_culled();
        let objects = vec![];

        let cmd = vulkan.cmds[0];
        let mut buffer_builder = BufferBuilder::new();
        util::begin_cmd(&vulkan.device, vulkan.cmds[0]);

        /*Create Vulkan Buffers*/
        let res = vulkan.resources.get_buffer_storage();
        let vertex_buffer = buffer_builder
            .set_name("vertex-buffer")
            .set_is_descriptor(false)
            .set_size(objects.len() as u64 * size_of::<NodeVertex>() as u64)
            .set_memory(Memory::Local)
            .set_type(BufferType::Vertex)
            .set_frames(1)
            .set_queue_family(vulkan.graphic)
            .set_data(util::slice_as_u8(&objects))
            .build_resource(res, cmd);

        let cam_buffers = buffer_builder
            .set_frames(MAX_FRAMES_IN_FLIGHT as u32)
            .set_size(size_of::<GPUCamera>() as u64)
            .set_memory(Memory::Host)
            .set_type(BufferType::Uniform)
            .set_is_descriptor(true)
            .set_name("camera-buffer")
            .set_data(&[])
            .build_resource(res, cmd);

        let object_buffers = buffer_builder
            .set_size(objects.len() as u64 * size_of::<GPUBlock>() as u64)
            .set_type(BufferType::Storage)
            .set_name("object-buffer")
            .set_data(util::slice_as_u8(&objects))
            .build_resource(res, cmd);

        util::end_cmd_and_submit(&vulkan.device, vulkan.cmds[0], vulkan.graphic, vec![], vec![], vk::Fence::null());
        unsafe { vulkan.device.device_wait_idle().unwrap() };

        /*Create Vulkan Pipeline */
        let vertex = util::create_shader(&vulkan.device, "shaders/spv/colored_triangle.vert.spv".to_owned());
        let frag = util::create_shader(&vulkan.device, "shaders/spv/colored_triangle.frag.spv".to_owned());

        let pipelines = builder::PipelineBuilder::new()
            .add_layout(vulkan.pipeline_layout)
            .add_color_format(vulkan.get_swapchain_format())
            .add_depth(vulkan.get_depth_format(), true, true, vk::CompareOp::LESS_OR_EQUAL)
            .cull_mode(vk::CullModeFlags::BACK, vk::FrontFace::CLOCKWISE)
            .add_topology(vk::PrimitiveTopology::TRIANGLE_LIST)
            .add_wire()
            .build::<VertexBlock>(&vulkan.device, vertex, frag);

        vulkan.window.set_cursor_grab(CursorGrabMode::None).unwrap();
        vulkan.window.set_cursor_visible(true);
        vulkan.window.focus_window();

        vulkan.resources.set_frame(0);

        Self {
            cam,
            vulkan,
            last_frame: Instant::now(),
            pipeline: pipelines,
            vertex_buffer,
            controls: Controls::new(),
            focus: false,
            object_count: objects.len() as u32,
            resize: false,
            world,
            objects,
            is_frustum: false,
            culled: vec![],
            pipeline_index: 0,
            vertex_block: vec![],
        }
    }

    fn resize_event(&mut self) {
        self.resize = true;
    }

    fn on_draw(&mut self) {
        unsafe {
            self.vulkan.prepare_frame(&mut self.resize);

            if self.resize == true {
                self.recreate_swapchain();
                self.vulkan.prepare_frame(&mut self.resize);
            }

            let device = &self.vulkan.device;
            let frame_index = self.vulkan.current_frame;
            let swapchain_index = self.vulkan.swapchain.image_index;
            let cmd = self.vulkan.cmds[frame_index];

            // let data = &mut self.frame_data[frame_index];

            // device.cmd_bind_descriptor_sets(
            //     cmd,
            //     vk::PipelineBindPoint::COMPUTE,
            //     self.vulkan.pipeline_layout,
            //     0,
            //     &[self.vulkan.resources.set],
            //     &vec![],
            // );

            // util::transition_image_color(&device, cmd, self.vulkan.swapchain.images[swapchain_index as usize].image);

            // let gpu_cam = vec![self.cam.get_gpu_camera()];

            // self.vulkan
            //     .resources
            //     .write_to_buffer_host(&mut data.cam_buffer, util::slice_as_u8(&gpu_cam));

            // self.vulkan.begin_rendering(vk::AttachmentLoadOp::LOAD);

            // let pipeline = self.pipeline[self.pipeline_index as usize];

            // device.cmd_bind_pipeline(cmd, vk::PipelineBindPoint::GRAPHICS, pipeline);

            // device.cmd_bind_vertex_buffers(cmd, 0, &[self.vertex_buffer.buffer], &vec![0]);

            // device.cmd_bind_descriptor_sets(
            //     cmd,
            //     vk::PipelineBindPoint::GRAPHICS,
            //     self.vulkan.pipeline_layout,
            //     0,
            //     &vec![self.vulkan.resources.set],
            //     &vec![],
            // );

            // let push_constant = PushConstant {
            //     cam_index: self.frame_data[frame_index].cam_buffer.index as u32,
            //     object_index: self.frame_data[frame_index].objects.index as u32,
            // };

            // device.cmd_push_constants(
            //     cmd,
            //     self.vulkan.pipeline_layout,
            //     vk::ShaderStageFlags::COMPUTE | vk::ShaderStageFlags::FRAGMENT | vk::ShaderStageFlags::VERTEX,
            //     0,
            //     std::slice::from_raw_parts(&push_constant as *const _ as *const u8, std::mem::size_of::<PushConstant>()),
            // );

            // device.cmd_draw(cmd, self.vertex_block.len() as u32, 4, 0, 0);

            // self.vulkan.end_rendering();

            // // let imgui = self.vulkan.imgui.as_mut().unwrap();

            // // let ui = imgui.get_draw_instance(&self.vulkan.window);

            // // let set = self.vulkan.resources.set;

            // // imgui.render(
            // //     self.vulkan.window_extent,
            // //     &self.vulkan.swapchain.images[self.vulkan.swapchain.image_index as usize],
            // //     self.vulkan.current_frame,
            // //     &mut self.vulkan.resources,
            // //     cmd,
            // //     set,
            // // );

            if self.vulkan.end_frame_and_submit() {
                self.resize = true;
            }

            let now = Instant::now();

            let diff = now - self.last_frame;

            let hz_diff = Self::HZ_MAX - diff.as_millis() as i64;

            if hz_diff > 0 {
                std::thread::sleep(Duration::from_millis(hz_diff as u64));
            }

            self.vulkan.window.request_redraw();
        }
    }

    fn on_new_frame(&mut self, event: &Event<()>) {
        let now = Instant::now();
        let delta_time = now.duration_since(self.last_frame);

        self.vulkan.imgui.as_mut().unwrap().update_delta_time(delta_time);
        self.last_frame = now;

        self.vulkan.imgui.as_mut().unwrap().process_event_imgui(&self.vulkan.window, &event);
        self.cam.process_keyboard(&self.controls, delta_time.as_secs_f64());
    }

    fn on_mouse_motion(&mut self, delta: &(f64, f64)) {
        if self.focus {
            self.cam.process_mouse(*delta);
        }
    }

    fn on_key_press(&mut self, key_event: &RawKeyEvent, keycode: KeyCode) {
        self.controls.update_key(keycode, key_event.state == ElementState::Pressed);

        if key_event.state == ElementState::Pressed && keycode == KeyCode::Escape {
            unsafe {
                self.focus = !self.focus;
                let cursor_grab: CursorGrabMode = transmute(self.focus);
                self.vulkan.window.set_cursor_grab(cursor_grab).unwrap();
                self.vulkan.window.set_cursor_visible(!self.focus);
            }
        }
    }

    fn on_destroy(&mut self) {
        unsafe {
            self.vulkan.device.device_wait_idle().unwrap();

            // for i in 0..self.frame_data.len() {
            //     let frame = &mut self.frame_data[i];
            //     self.vulkan.allocator.destroy_buffer(frame.cam_buffer.buffer, &mut frame.cam_buffer.alloc);
            //     self.vulkan.allocator.destroy_buffer(frame.objects.buffer, &mut frame.objects.alloc);
            // }

            // for pipeline in self.pipeline.clone() {
            //     self.vulkan.device.destroy_pipeline(pipeline, None);
            // }

            // self.pipeline.clear();

            // self.vulkan
            //     .allocator
            //     .destroy_buffer(self.vertex_buffer.buffer, &mut self.vertex_buffer.alloc);
        }
        self.vulkan.destroy();
    }

    fn set_imgui_draw(&mut self, imgui_func: fn(ui: &mut imgui::Ui)) {
        todo!()
    }
}

#[repr(C, align(16))]
struct CMainPipeline {
    index: u32,
}

struct FrameData {
    cam_buffer: AllocatedBuffer,
    objects: AllocatedBuffer,
    push: PushConstant,
}

extern crate ultraviolet as glm;

impl TestApplication {
    const HZ_MAX: i64 = (1000.0 / 60.0) as i64;
    fn new(event_loop: &EventLoop<()>) -> Self {
        ApplicationTrait::on_new(event_loop)
    }

    // need to rebuild the swapchain and any resources that depend on the window extent
    fn recreate_swapchain(&mut self) {
        log::info!("Recreating swapchain");
        unsafe {
            self.vulkan.device.device_wait_idle().unwrap();
        }

        self.vulkan.recreate_swapchain();

        log::info!("recreate computation images");

        // TODO, recreate my general image

        self.cam.resize_window(self.vulkan.window_extent);
        self.resize = false;
    }
}
