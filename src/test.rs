#![feature(inherent_associated_types)]

use std::{
    any::Any,
    collections::HashMap,
    mem::transmute,
    time::{Duration, Instant},
};

use ash::vk::{self, FrontFace};
use env_logger::Builder;
use tgui::ImguiId;
use voxelengine::{
    app::ApplicationTrait,
    core::camera::{Camera, Controls, GPUCamera},
    terrain::{block::GPUBlock, World},
    vulkan::{
        builder::{self},
        mesh::{EmptyVertex, Vertex, VertexBlock},
        resource::{BufferBuilder, BufferIndex, BufferType, Memory},
        util, VulkanContext,
    },
    TImguiRender,
};
use voxelengine_proc::ImGuiFields;
use winit::{
    event::{ElementState, Event, RawKeyEvent},
    event_loop::EventLoop,
    keyboard::KeyCode,
    window::CursorGrabMode,
};

use crate::world_test::{chunk::ChunkMesh, object::GlobalColor};
use voxelengine::gui::*;

pub const MAX_FRAMES_IN_FLIGHT: usize = 2;

/// There should only be application relevant information in these functions
pub struct TestApplication {
    vulkan: VulkanContext,

    last_frame: Instant,

    pipeline: Vec<vk::Pipeline>,

    cam: Camera,
    cam_buffers: Vec<BufferIndex>,

    controls: Controls,

    focus: bool,

    resize: bool,

    chunk_mesh: ChunkMesh,

    variables: ImguiVariables,

    imgui_id: ImguiId,

    global_color: GlobalColor,
}

#[derive(ImGuiFields, Default)]
pub struct ImguiVariables {
    #[slider(0, 1)]
    pipeline_index: usize,
    is_frustum: bool,
    #[slider(1.0, 5.0)]
    float_test: f32,
}

impl ApplicationTrait for TestApplication {
    fn on_new(event_loop: &EventLoop<()>) -> Self {
        Builder::new().filter_level(log::LevelFilter::Info).init();
        let mut vulkan = VulkanContext::new(&event_loop, MAX_FRAMES_IN_FLIGHT, true);
        //  Octree::new(&mut vulkan.resources.get_buffer_storage(), Vec3::zero());
        let cam = Camera::new(vulkan.window_extent);

        let mut global_color = GlobalColor { colors: vec![], indices_taken: HashMap::new(), buffer: 0 };

        let cmd = vulkan.cmds[0];
        let mut buffer_builder = BufferBuilder::new();
        util::begin_cmd(&vulkan.device, vulkan.cmds[0]);

        /*Create Vulkan Buffers*/
        let res = vulkan.resources.get_buffer_storage();
        let x = 5;
        let cam_buffers = buffer_builder
            .set_frames(MAX_FRAMES_IN_FLIGHT as u32)
            .set_size(size_of::<GPUCamera>() as u64)
            .set_memory(Memory::Host)
            .set_type(BufferType::Uniform)
            .set_is_descriptor(true)
            .set_name("camera-buffer")
            .set_data(&[])
            .build_resource(res, cmd);

        let chunk_mesh = ChunkMesh::new_test(res, &mut global_color, vulkan.graphic, vulkan.cmds[0]);
        let data = util::slice_as_u8(&global_color.colors);

        global_color.buffer = buffer_builder
            .set_frames(1)
            .set_size(global_color.colors.len() as u64 * 4)
            .set_memory(Memory::Local)
            .set_type(BufferType::Storage)
            .set_is_descriptor(true)
            .set_data(data)
            .set_name("global_color")
            .build_resource(res, cmd)[0];

        for i in 0..global_color.colors.len() {
            println!("{:?}", global_color.colors[i]);
        }

        util::end_cmd_and_submit(&vulkan.device, vulkan.cmds[0], vulkan.graphic, vec![], vec![], vk::Fence::null());
        unsafe { vulkan.device.device_wait_idle().unwrap() };

        /*Create Vulkan Pipeline */
        let vertex = util::create_shader(&vulkan.device, "shaders/spv/chunk.vert.spv".to_owned());
        let frag = util::create_shader(&vulkan.device, "shaders/spv/chunk.frag.spv".to_owned());

        let pipelines = builder::PipelineBuilder::new()
            .add_layout(vulkan.pipeline_layout)
            .add_color_format(vulkan.get_swapchain_format())
            .add_depth(vulkan.get_depth_format(), true, true, vk::CompareOp::LESS_OR_EQUAL)
            .cull_mode(vk::CullModeFlags::BACK, FrontFace::CLOCKWISE)
            .add_topology(vk::PrimitiveTopology::TRIANGLE_LIST)
            .add_wire()
            .build::<EmptyVertex>(&vulkan.device, vertex, frag);

        vulkan.window.set_cursor_grab(CursorGrabMode::None).unwrap();
        vulkan.window.set_cursor_visible(true);
        vulkan.window.focus_window();

        vulkan.resources.set_frame(0);
        let variables = ImguiVariables::default();
        Self {
            cam,
            vulkan,
            last_frame: Instant::now(),
            pipeline: pipelines,
            controls: Controls::new(),
            focus: false,
            resize: false,
            variables,
            cam_buffers,
            chunk_mesh,
            imgui_id: ImguiId::new(50),
            global_color,
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

            util::transition_image_color(&device, cmd, self.vulkan.swapchain.images[swapchain_index as usize].image);

            let gpu_cam = vec![self.cam.get_gpu_camera()];
            self.vulkan.resources.get_buffer_storage().write_to_buffer_host(self.cam_buffers[frame_index], util::slice_as_u8(&gpu_cam));

            self.vulkan.begin_rendering(vk::AttachmentLoadOp::CLEAR);

            let pipeline = self.pipeline[self.variables.pipeline_index as usize];

            device.cmd_bind_pipeline(cmd, vk::PipelineBindPoint::GRAPHICS, pipeline);

            device.cmd_bind_descriptor_sets(
                cmd,
                vk::PipelineBindPoint::GRAPHICS,
                self.vulkan.pipeline_layout,
                0,
                &vec![self.vulkan.resources.set],
                &vec![],
            );

            let cam_index = self.vulkan.resources.get_buffer_storage().get_buffer_ref(self.cam_buffers[frame_index]).index;

            self.chunk_mesh.draw(
                &self.vulkan.device,
                self.vulkan.resources.get_buffer_storage(),
                cmd,
                self.vulkan.pipeline_layout,
                cam_index as u32,
                self.global_color.buffer as u32,
            );

            self.vulkan.end_rendering();

            let imgui = self.vulkan.imgui.as_mut().unwrap();
            self.imgui_id.start_frame();

            let ui = imgui.get_draw_instance(&self.vulkan.window);
            let set = self.vulkan.resources.set;

            self.variables.display_imgui(ui, &mut self.imgui_id);
            self.cam.display_imgui(ui, &mut self.imgui_id);

            imgui.render(
                self.vulkan.window_extent,
                &self.vulkan.swapchain.images[self.vulkan.swapchain.image_index as usize],
                self.vulkan.current_frame,
                &mut self.vulkan.resources.get_buffer_storage(),
                cmd,
                set,
            );

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
