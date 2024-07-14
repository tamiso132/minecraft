use std::{
    mem::transmute,
    time::{Duration, Instant},
};

use ash::vk;
use env_logger::Builder;
use voxelengine::{
    core::camera::{Camera, Controls, GPUCamera},
    terrain::World,
    vulkan::{
        builder,
        mesh::Vertex,
        resource::{AllocatedBuffer, BufferType, Memory},
        util, VulkanContext,
    },
    App::ApplicationTrait,
};
use winit::{
    event::{ElementState, Event, RawKeyEvent},
    event_loop::EventLoop,
    keyboard::KeyCode,
    window::CursorGrabMode,
};

use crate::{HZ_MAX, MAX_FRAMES_IN_FLIGHT};

#[derive(Clone, Copy)]
#[repr(C, align(16))]
pub struct NodeDebugVert {
    pub pos: glm::Vec3,
}

impl Default for NodeDebugVert {
    fn default() -> Self {
        Self { pos: Default::default() }
    }
}

impl NodeDebugVert {
    pub const fn new(pos: glm::Vec3) -> Self {
        Self { pos }
    }
}

impl Vertex for NodeDebugVert {
    fn get_vertex_attribute_desc() -> Vec<vk::VertexInputAttributeDescription> {
        [vk::VertexInputAttributeDescription::default()
            .binding(0)
            .location(0)
            .format(vk::Format::R32G32B32_SFLOAT)
            .offset(0)]
        .to_vec()
    }
}

impl NodeDebugVert {
    const Vertices: [NodeDebugVert; 36] = [
        // right
        NodeDebugVert::new(glm::Vec3::new(0.5, 0.5, 0.5)),
        NodeDebugVert::new(glm::Vec3::new(0.5, 0.5, -0.5)),
        NodeDebugVert::new(glm::Vec3::new(0.5, -0.5, -0.5)),
        NodeDebugVert::new(glm::Vec3::new(0.5, -0.5, -0.5)),
        NodeDebugVert::new(glm::Vec3::new(0.5, -0.5, 0.5)),
        NodeDebugVert::new(glm::Vec3::new(0.5, 0.5, 0.5)),
        // Left face
        NodeDebugVert::new(glm::Vec3::new(-0.5, 0.5, 0.5)),
        NodeDebugVert::new(glm::Vec3::new(-0.5, -0.5, -0.5)),
        NodeDebugVert::new(glm::Vec3::new(-0.5, 0.5, -0.5)),
        NodeDebugVert::new(glm::Vec3::new(-0.5, -0.5, -0.5)),
        NodeDebugVert::new(glm::Vec3::new(-0.5, 0.5, 0.5)),
        NodeDebugVert::new(glm::Vec3::new(-0.5, -0.5, 0.5)),
        // Top face
        NodeDebugVert::new(glm::Vec3::new(-0.5, 0.5, -0.5)),
        NodeDebugVert::new(glm::Vec3::new(0.5, 0.5, -0.5)),
        NodeDebugVert::new(glm::Vec3::new(0.5, 0.5, 0.5)),
        NodeDebugVert::new(glm::Vec3::new(0.5, 0.5, 0.5)),
        NodeDebugVert::new(glm::Vec3::new(-0.5, 0.5, 0.5)),
        NodeDebugVert::new(glm::Vec3::new(-0.5, 0.5, -0.5)),
        // Bottom face
        NodeDebugVert::new(glm::Vec3::new(-0.5, -0.5, -0.5)),
        NodeDebugVert::new(glm::Vec3::new(0.5, -0.5, 0.5)),
        NodeDebugVert::new(glm::Vec3::new(0.5, -0.5, -0.5)),
        NodeDebugVert::new(glm::Vec3::new(0.5, -0.5, 0.5)),
        NodeDebugVert::new(glm::Vec3::new(-0.5, -0.5, -0.5)),
        NodeDebugVert::new(glm::Vec3::new(-0.5, -0.5, 0.5)),
        // Front face
        NodeDebugVert::new(glm::Vec3::new(-0.5, -0.5, 0.5)),
        NodeDebugVert::new(glm::Vec3::new(0.5, 0.5, 0.5)),
        NodeDebugVert::new(glm::Vec3::new(0.5, -0.5, 0.5)),
        NodeDebugVert::new(glm::Vec3::new(0.5, 0.5, 0.5)),
        NodeDebugVert::new(glm::Vec3::new(-0.5, -0.5, 0.5)),
        NodeDebugVert::new(glm::Vec3::new(-0.5, 0.5, 0.5)),
        // Back face
        NodeDebugVert::new(glm::Vec3::new(-0.5, -0.5, -0.5)),
        NodeDebugVert::new(glm::Vec3::new(0.5, -0.5, -0.5)),
        NodeDebugVert::new(glm::Vec3::new(0.5, 0.5, -0.5)),
        NodeDebugVert::new(glm::Vec3::new(0.5, 0.5, -0.5)),
        NodeDebugVert::new(glm::Vec3::new(-0.5, 0.5, -0.5)),
        NodeDebugVert::new(glm::Vec3::new(-0.5, -0.5, -0.5)),
    ];
}
struct FrameData {
    obj: AllocatedBuffer,
    cam: AllocatedBuffer,
}

struct TestApplication {
    vulkan: VulkanContext,

    last_frame: Instant,

    pipeline: Vec<vk::Pipeline>,
    vertex_buffer: AllocatedBuffer,

    cam: Camera,
    controls: Controls,
    frame_data: Vec<FrameData>,

    focus: bool,

    object_count: u32,
    resize: bool,

    world: World,
}

impl TestApplication {
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

impl ApplicationTrait for TestApplication {
    fn on_new(event_loop: &EventLoop<()>) -> Self {
        Builder::new().filter_level(log::LevelFilter::Info).init();

        let mut vulkan = VulkanContext::new(&event_loop, MAX_FRAMES_IN_FLIGHT, true);

        let mesh = NodeDebugVert::Vertices;

        let cam = Camera::new(vulkan.window_extent);
        let world = World::new(cam.get_pos(), 4);

        //let objects = world.get_culled();
        todo!();
        // let objects = vec![];
        // let vertex_buffer = vulkan.resources.create_buffer_non_descriptor(
        //     objects.len() as u64 * size_of::<NodeDebugVert>() as u64,
        //     BufferType::Vertex,
        //     Memory::Local,
        //     vulkan.graphic.family,
        //     "vertexBuffer".to_owned(),
        // );

        // let mut frame_data = vec![];
        // //let objects = AreaGenerator::generate_around((0, 0));

        // let texture_loaded = util::load_texture_array("texture_atlas_0.png", 64);

        // let texture_atlas = vulkan.resources.create_texture_array(texture_loaded, "texture_atlas".to_owned());

        // util::begin_cmd(&vulkan.device, vulkan.cmds[0]);
        // vulkan
        //     .resources
        //     .write_to_buffer_local(vulkan.cmds[0], &vertex_buffer, util::slice_as_u8(&objects));

        // /*Should be outside of this initilize */
        // for i in 0..MAX_FRAMES_IN_FLIGHT {
        //     let cam_buffer_n = format!("camera_buffer{}", i);
        //     let object_buffer_n = format!("object_buffer{}", i);

        //     let cam_buffer = vulkan.resources.create_buffer(
        //         size_of::<GPUCamera>() as u64,
        //         BufferType::Uniform,
        //         Memory::Host,
        //         vulkan.graphic.family,
        //         cam_buffer_n,
        //     );

        //     let mut object = vulkan.resources.create_buffer(
        //         objects.len() as u64 * size_of::<GPUBlock>() as u64,
        //         BufferType::Storage,
        //         Memory::Host,
        //         vulkan.graphic.family,
        //         object_buffer_n,
        //     );

        //     vulkan.resources.write_to_buffer_host(&mut object, util::slice_as_u8(&objects));

        //     frame_data.push(FrameData { cam: cam_buffer, obj: object });
        // }

        // util::end_cmd_and_submit(&vulkan.device, vulkan.cmds[0], vulkan.graphic, vec![], vec![], vk::Fence::null());
        // unsafe { vulkan.device.device_wait_idle().unwrap() };

        // let vertex = util::create_shader(&vulkan.device, "shaders/spv/colored_triangle.vert.spv".to_owned());
        // let frag = util::create_shader(&vulkan.device, "shaders/spv/colored_triangle.frag.spv".to_owned());

        // let pipelines = builder::PipelineBuilder::new()
        //     .add_layout(vulkan.pipeline_layout)
        //     .add_color_format(vulkan.get_swapchain_format())
        //     .add_depth(vulkan.get_depth_format(), true, true, vk::CompareOp::LESS_OR_EQUAL)
        //     .cull_mode(vk::CullModeFlags::BACK, vk::FrontFace::CLOCKWISE)
        //     .add_topology(vk::PrimitiveTopology::TRIANGLE_LIST)
        //     .add_wire()
        //     .build::<NodeDebugVert>(&vulkan.device, vertex, frag);

        // vulkan.window.set_cursor_grab(CursorGrabMode::None).unwrap();
        // vulkan.window.set_cursor_visible(true);
        // vulkan.window.focus_window();

        // vulkan.resources.set_frame(0);

        // Self {
        //     cam,
        //     vulkan,
        //     last_frame: Instant::now(),
        //     pipeline: pipelines,
        //     vertex_buffer,
        //     controls: Controls::new(),
        //     frame_data,
        //     focus: false,
        //     object_count: objects.len() as u32,
        //     resize: false,
        //     world,
        // }
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

            let data = &mut self.frame_data[frame_index];

            util::transition_image_color(&device, cmd, self.vulkan.swapchain.images[swapchain_index as usize].image);

            // TODO HAVE TO DO A BARRIER FOR THE COMPUTE, SO IT IS DONE
            // TODO, make it write directly to the swapchain image
            /*Update Camera */
            let gpu_cam = vec![self.cam.get_gpu_camera()];
            let len;

            self.vulkan.resources.write_to_buffer_host(&mut data.cam, util::slice_as_u8(&gpu_cam));

            self.vulkan.begin_rendering(vk::AttachmentLoadOp::LOAD);

            // device.cmd_bind_pipeline(cmd, vk::PipelineBindPoint::GRAPHICS, pipeline);

            device.cmd_bind_vertex_buffers(cmd, 0, &[self.vertex_buffer.buffer], &vec![0]);

            device.cmd_bind_descriptor_sets(
                cmd,
                vk::PipelineBindPoint::GRAPHICS,
                self.vulkan.pipeline_layout,
                0,
                &vec![self.vulkan.resources.set],
                &vec![],
            );

            // device.cmd_push_constants(
            //     cmd,
            //     self.vulkan.pipeline_layout,
            //     vk::ShaderStageFlags::COMPUTE | vk::ShaderStageFlags::FRAGMENT | vk::ShaderStageFlags::VERTEX,
            //     0,
            //     std::slice::from_raw_parts(&push_main as *const _ as *const u8, std::mem::size_of::<CMainPipeline>()),
            // );

            //  device.cmd_draw(cmd, self.vertex_block.len() as u32, 4, 0, 0);

            self.vulkan.end_rendering();

            let set = self.vulkan.resources.set;

            if self.vulkan.end_frame_and_submit() {
                self.resize = true;
            }

            let now = Instant::now();

            let diff = now - self.last_frame;

            let hz_diff = HZ_MAX - diff.as_millis() as i64;

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

            for i in 0..self.frame_data.len() {
                let frame = &mut self.frame_data[i];
                self.vulkan.allocator.destroy_buffer(frame.cam.buffer, &mut frame.cam.alloc);
                self.vulkan.allocator.destroy_buffer(frame.obj.buffer, &mut frame.obj.alloc);
            }

            for pipeline in self.pipeline.clone() {
                self.vulkan.device.destroy_pipeline(pipeline, None);
            }

            self.pipeline.clear();

            self.vulkan
                .allocator
                .destroy_buffer(self.vertex_buffer.buffer, &mut self.vertex_buffer.alloc);
        }
        self.vulkan.destroy();
    }

    fn set_imgui_draw(&mut self, imgui_func: fn(ui: &mut imgui::Ui)) {
        todo!()
    }
}
