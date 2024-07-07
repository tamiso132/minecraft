#![feature(inherent_associated_types)]

use std::{
    collections::HashMap,
    mem::transmute,
    time::{Duration, Instant},
};

use ash::vk::{self};
use env_logger::Builder;
use voxelengine::{
    application::{Application, ApplicationTrait},
    core::camera::{Camera, Controls, Frustum, GPUCamera},
    terrain::{
        block::{GPUBlock, GPUTexture, Materials},
        World,
    },
    vulkan::{
        builder::{self, ComputePipelineBuilder},
        mesh::VertexBlock,
        resource::{AllocatedBuffer, AllocatedImage, BufferType, Memory},
        util, SkyBoxPushConstant, VulkanContext,
    },
};
use winit::{
    event::{self, ElementState, Event, RawKeyEvent, WindowEvent},
    event_loop::{self, EventLoop},
    keyboard::KeyCode,
    window::CursorGrabMode,
};

pub const MAX_FRAMES_IN_FLIGHT: usize = 2;

/// There should only be application relevant information in these functions
struct GameApplication {
    vulkan: VulkanContext,
    compute: vk::Pipeline,

    push_constant: SkyBoxPushConstant,
    last_frame: Instant,

    pipeline: Vec<vk::Pipeline>,
    vertex_buffer: AllocatedBuffer,

    key_pressed: HashMap<KeyCode, bool>,

    cam: Camera,
    controls: Controls,
    frame_data: Vec<FrameData>,

    focus: bool,

    object_count: u32,
    objects: Vec<GPUBlock>,
    resize: bool,

    texture_atlas: AllocatedImage,
    material_buffer: AllocatedBuffer,

    world: World,
    is_frustum: bool,

    vertex_block: Vec<VertexBlock>,

    culled: Vec<GPUBlock>,
    pipeline_index: i32,
}

impl ApplicationTrait for GameApplication {
    fn on_new() -> Self {
        let event_loop = EventLoop::new().unwrap();
        Builder::new().filter_level(log::LevelFilter::Info).init();

        let mut vulkan = VulkanContext::new(&event_loop, MAX_FRAMES_IN_FLIGHT, true);

        let comp_skybox = util::create_shader(&vulkan.device, "shaders/spv/skybox.comp.spv".to_owned());
        let compute = ComputePipelineBuilder::new(comp_skybox).build(&vulkan.device, vulkan.pipeline_layout);
        let mesh = VertexBlock::get_mesh();

        let cam = Camera::new(vulkan.window_extent);
        let world = World::new(cam.get_pos(), 4);
        //let objects = world.get_culled();
        let objects = vec![];

        let vertex_buffer = vulkan.resources.create_buffer_non_descriptor(
            objects.len() as u64 * size_of::<VertexBlock>() as u64,
            BufferType::Vertex,
            Memory::Local,
            vulkan.graphic.family,
            "vertexBuffer".to_owned(),
        );

        let mut frame_data = vec![];
        //let objects = AreaGenerator::generate_around((0, 0));

        let texture_loaded = util::load_texture_array("texture_atlas_0.png", 64);

        let texture_atlas = vulkan.resources.create_texture_array(texture_loaded, "texture_atlas".to_owned());

        let mats = Materials::get_all();
        let material_buffer = vulkan.resources.create_buffer(
            mats.len() as u64 * size_of::<GPUTexture>() as u64,
            BufferType::Storage,
            Memory::Local,
            vulkan.graphic.family,
            "materialbuffer".to_owned(),
        );

        util::begin_cmd(&vulkan.device, vulkan.cmds[0]);
        vulkan
            .resources
            .write_to_buffer_local(vulkan.cmds[0], &vertex_buffer, util::slice_as_u8(&objects));

        vulkan
            .resources
            .write_to_buffer_local(vulkan.cmds[0], &material_buffer, &util::slice_as_u8(&mats));

        /*Should be outside of this initilize */
        for i in 0..MAX_FRAMES_IN_FLIGHT {
            let name = format!("{}_{}", "compute_skybox", i);
            let cam_buffer_n = format!("camera_buffer{}", i);
            let object_buffer_n = format!("object_buffer{}", i);
            let indice_buffer_n = format!("indice_buffer{}", i);
            let compute_image = vulkan.resources.create_storage_image(
                vulkan.window_extent,
                4,
                vk::MemoryPropertyFlags::DEVICE_LOCAL,
                vk::Format::R8G8B8A8_UNORM,
                vk::ImageUsageFlags::TRANSFER_SRC
                    | vk::ImageUsageFlags::TRANSFER_DST
                    | vk::ImageUsageFlags::STORAGE
                    | vk::ImageUsageFlags::COLOR_ATTACHMENT,
                name,
            );

            let cam_buffer = vulkan.resources.create_buffer(
                size_of::<GPUCamera>() as u64,
                BufferType::Uniform,
                Memory::Host,
                vulkan.graphic.family,
                cam_buffer_n,
            );

            let mut object = vulkan.resources.create_buffer(
                objects.len() as u64 * size_of::<GPUBlock>() as u64,
                BufferType::Storage,
                Memory::Host,
                vulkan.graphic.family,
                object_buffer_n,
            );

            let indice = vulkan.resources.create_buffer(
                size_of::<GPUIndex>() as u64,
                BufferType::Uniform,
                Memory::Local,
                vulkan.graphic.family,
                indice_buffer_n,
            );
            let gpu_index = GPUIndex {
                cam: cam_buffer.index as u32,
                object: object.index as u32,
                texture: texture_atlas.index as u32,
                normal: 0,
                material: material_buffer.index as u32,
            };
            vulkan.resources.write_to_buffer_host(&mut object, util::slice_as_u8(&objects));

            vulkan
                .resources
                .write_to_buffer_local(vulkan.cmds[0], &indice, util::slice_as_u8(&vec![gpu_index]));

            frame_data.push(FrameData { cam_buffer, objects: object, compute_image, indices_buffer: indice });
        }

        util::end_cmd_and_submit(&vulkan.device, vulkan.cmds[0], vulkan.graphic, vec![], vec![], vk::Fence::null());
        unsafe { vulkan.device.device_wait_idle().unwrap() };

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
            compute,
            push_constant: SkyBoxPushConstant::new(),
            last_frame: Instant::now(),
            pipeline: pipelines,
            vertex_buffer,
            key_pressed: HashMap::new(),
            controls: Controls::new(),
            frame_data,
            focus: false,
            object_count: objects.len() as u32,
            resize: false,
            texture_atlas,
            material_buffer,
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

            let data = &mut self.frame_data[frame_index];

            device.cmd_bind_descriptor_sets(
                cmd,
                vk::PipelineBindPoint::COMPUTE,
                self.vulkan.pipeline_layout,
                0,
                &[self.vulkan.resources.set],
                &vec![],
            );

            device.cmd_bind_pipeline(cmd, vk::PipelineBindPoint::COMPUTE, self.compute);

            self.push_constant.image_index = data.compute_image.index as u32;

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
                &data.compute_image,
                &self.vulkan.swapchain.images[swapchain_index as usize],
                self.vulkan.window_extent,
            );

            util::transition_image_color(&device, cmd, self.vulkan.swapchain.images[swapchain_index as usize].image);

            // TODO HAVE TO DO A BARRIER FOR THE COMPUTE, SO IT IS DONE
            // TODO, make it write directly to the swapchain image
            /*Update Camera */
            let gpu_cam = vec![self.cam.get_gpu_camera()];
            let len;
            if self.is_frustum {
                let frustum = Frustum::new(&self.cam);
                self.culled.clear();
                for i in 0..self.objects.len() {
                    let object = self.objects[i].clone();
                    if frustum.is_inside(object.position) {
                        self.culled.push(object)
                    }
                }
                len = self.culled.len();
                self.vulkan
                    .resources
                    .write_to_buffer_host(&mut data.objects, util::slice_as_u8(&self.culled));
            } else {
                len = self.objects.len();
                self.vulkan
                    .resources
                    .write_to_buffer_host(&mut data.objects, util::slice_as_u8(&self.objects));
            }

            self.vulkan
                .resources
                .write_to_buffer_host(&mut data.cam_buffer, util::slice_as_u8(&gpu_cam));

            self.vulkan.begin_rendering(vk::AttachmentLoadOp::LOAD);

            let mut viewport = vk::Viewport::default();
            viewport.height = self.vulkan.window_extent.height as f32;
            viewport.width = self.vulkan.window_extent.width as f32;
            viewport.min_depth = 0.0;
            viewport.max_depth = 1.0;

            let scissor = vk::Rect2D::default().extent(self.vulkan.window_extent);

            let pipeline = self.pipeline[self.pipeline_index as usize];

            device.cmd_set_viewport(cmd, 0, &[viewport]);
            device.cmd_set_scissor(cmd, 0, &[scissor]);

            device.cmd_bind_pipeline(cmd, vk::PipelineBindPoint::GRAPHICS, pipeline);

            device.cmd_bind_vertex_buffers(cmd, 0, &[self.vertex_buffer.buffer], &vec![0]);

            device.cmd_bind_descriptor_sets(
                cmd,
                vk::PipelineBindPoint::GRAPHICS,
                self.vulkan.pipeline_layout,
                0,
                &vec![self.vulkan.resources.set],
                &vec![],
            );
            let push_main = CMainPipeline { index: data.indices_buffer.index as u32 };

            device.cmd_push_constants(
                cmd,
                self.vulkan.pipeline_layout,
                vk::ShaderStageFlags::COMPUTE | vk::ShaderStageFlags::FRAGMENT | vk::ShaderStageFlags::VERTEX,
                0,
                std::slice::from_raw_parts(&push_main as *const _ as *const u8, std::mem::size_of::<CMainPipeline>()),
            );

            device.cmd_draw(cmd, self.vertex_block.len() as u32, 4, 0, 0);

            self.vulkan.end_rendering();

            let imgui = self.vulkan.imgui.as_mut().unwrap();

            let ui = imgui.get_draw_instance(&self.vulkan.window);

            ui.input_float4("Data1", &mut self.push_constant.data1).build();
            ui.input_float4("Data2", &mut self.push_constant.data2).build();
            ui.input_float4("Data3", &mut self.push_constant.data3).build();
            ui.input_float4("Data4", &mut self.push_constant.data4).build();
            ui.checkbox("Frustum", &mut self.is_frustum);
            ui.input_int("pipeline index", &mut self.pipeline_index).build();

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

            for i in 0..self.frame_data.len() {
                let frame = &mut self.frame_data[i];
                self.vulkan.allocator.destroy_buffer(frame.cam_buffer.buffer, &mut frame.cam_buffer.alloc);
                self.vulkan.allocator.destroy_buffer(frame.objects.buffer, &mut frame.objects.alloc);
                self.vulkan
                    .allocator
                    .destroy_buffer(frame.indices_buffer.buffer, &mut frame.indices_buffer.alloc);

                self.vulkan
                    .allocator
                    .destroy_image(frame.compute_image.image, &mut frame.compute_image.alloc.as_mut().unwrap());
                self.vulkan.device.destroy_image_view(frame.compute_image.view, None);
            }

            for pipeline in self.pipeline.clone() {
                self.vulkan.device.destroy_pipeline(pipeline, None);
            }

            self.pipeline.clear();

            self.vulkan
                .allocator
                .destroy_image(self.texture_atlas.image, &mut self.texture_atlas.alloc.as_mut().unwrap());
            self.vulkan.device.destroy_image_view(self.texture_atlas.view, None);
            self.vulkan.device.destroy_sampler(self.texture_atlas.sampler, None);

            self.vulkan
                .allocator
                .destroy_buffer(self.vertex_buffer.buffer, &mut self.vertex_buffer.alloc);
        }
        self.vulkan.destroy();
    }
}

#[repr(C, align(16))]
struct GPUIndex {
    cam: u32,
    object: u32,
    texture: u32,
    normal: u32,
    material: u32,
}

#[repr(C, align(16))]
struct CMainPipeline {
    index: u32,
}

struct FrameData {
    cam_buffer: AllocatedBuffer,
    objects: AllocatedBuffer,
    compute_image: AllocatedImage,
    /// Uniform buffer that has all indices
    indices_buffer: AllocatedBuffer,
}

extern crate ultraviolet as glm;

impl GameApplication {
    const HZ_MAX: i64 = (1000.0 / 60.0) as i64;
    fn new() -> Self {
        ApplicationTrait::on_new()
    }

    // need to rebuild the swapchain and any resources that depend on the window extent
    fn recreate_swapchain(&mut self) {
        log::info!("Recreating swapchain");
        unsafe {
            self.vulkan.device.device_wait_idle().unwrap();
        }
        let window_extent_physical = self.vulkan.window.inner_size();
        let extent = vk::Extent2D { width: window_extent_physical.width, height: window_extent_physical.height };

        self.vulkan.recreate_swapchain(extent);

        log::info!("recreate computation images");

        // TODO, recreate my general image
        for image in &mut self.frame_data {
            self.vulkan.resources.resize_image(&mut image.compute_image, extent);
        }

        self.cam.resize_window(extent);
        self.resize = false;
    }
}

fn main() {
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(event_loop::ControlFlow::Poll);

    let mut application: Application<GameApplication> = Application::new();

    application.run(event_loop);
}
