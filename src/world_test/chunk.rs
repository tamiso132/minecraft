use std::cmp::min;

use ash::vk;
use glm::Vec3;
use mesh::GPUQuad;
use object::GlobalColor;
use tgui::ImguiId;
use voxelengine::gui::struct_impl::*;
use voxelengine::vulkan::util::slice_as_u8;
use voxelengine::TImguiRender;
use voxelengine::{
    terrain::block::GPUBlock,
    vulkan::{
        resource::{self, AllocatedBuffer, BufferBuilder, BufferIndex, BufferStorage, BufferType, Memory},
        util::{self, slice_as_u8_vec},
        TKQueue,
    },
};
use voxelengine_proc::ImGuiFields;

use super::generation::{generate_height_map, NoiseParameters};
use super::node::ChunkQueue;
use super::{mesh, object, MatSize, CHUNK_RESOLUTION, CHUNK_SIZE};

pub(crate) fn get_y_offset(size: usize, y: f32) -> usize {
    (y * (size * size) as f32) as usize
}

pub(crate) fn get_z_offset(size: usize, z: f32) -> usize {
    (z * (size as f32)) as usize
}

pub(crate) fn get_x_offset(x: usize) -> usize {
    x
}

#[repr(C, align(16))]
#[derive(Default, ImGuiFields)]
struct ChunkConstant {
    pos: Vec3,
    pub scale: f32,
    pub cam_index: u32,
    pub quad_index: u32,
    pub color_index: u32,
    pub world_index: u32,
    pub chunk_size: u32,
}

#[derive(Default, ImGuiFields)]
pub struct ChunkMesh {
    #[ignore_field]
    chunk: Chunk,
    #[ignore_field]
    offset_position: Vec3,
    #[ignore_field]
    lod: u32,
    #[ignore_field]
    quad_len: usize,

    chunk_constant: ChunkConstant,

    #[ignore_field]
    chunk_queue: ChunkQueue,

    is_empty: bool,
}
impl ChunkMesh {
    pub fn new_test(res: &mut BufferStorage, graphic_queue: TKQueue, offset_position: Vec3, cmd: vk::CommandBuffer, lod: u32, chunk_queue: ChunkQueue) -> Self {
        // APPLY basic terrain

        let chunk = Chunk::new(res, cmd, graphic_queue, lod, offset_position);

        // APPLY TREE ISH

        let quads = mesh::mesh(&chunk.material);

        let mut size = quads.len() * size_of::<GPUQuad>();
        let mut is_empty = false;
        if quads.len() == 0 {
            size = 1;
            is_empty = true;
        }

        let buffers = BufferBuilder::new()
            .set_name("")
            .set_data(slice_as_u8_vec(&quads))
            .set_is_descriptor(true)
            .set_queue_family(graphic_queue)
            .set_memory(Memory::Local)
            .set_size(size as u64)
            .build_resource(res, cmd);

        let texture_buffer = res.get_buffer_ref(chunk.texture_buffer).index;

        let scale = 2u32.pow(lod) as f32;
        let quarter_size = (CHUNK_RESOLUTION as f32 * scale) / 4.0;
        Self {
            chunk,
            offset_position,
            lod,
            quad_len: quads.len(),
            chunk_constant: ChunkConstant {
                pos: offset_position,
                cam_index: 0,
                color_index: 0,
                chunk_size: 64,
                quad_index: res.get_buffer_ref(buffers[0]).index as u32,
                world_index: texture_buffer as u32,
                scale,
            },
            is_empty,
            chunk_queue,
        }
    }

    pub fn upload_to_gpu() {
        // do uploading to gpu here
    }

    pub fn draw(&mut self, device: &ash::Device, cmd: vk::CommandBuffer, layout: vk::PipelineLayout, cam_index: u32, g_color_index: u32) {
        if self.is_empty {
            return;
        }

        let scale = 2u32.pow(self.lod) as f32;
        let quarter_size = (CHUNK_RESOLUTION as f32 * scale) / 4.0;

        self.chunk_constant.cam_index = cam_index;
        self.chunk_constant.color_index = g_color_index;

        let slice = slice_as_u8(&self.chunk_constant);

        unsafe {
            device.cmd_push_constants(
                cmd,
                layout,
                vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT | vk::ShaderStageFlags::COMPUTE,
                0,
                slice_as_u8_vec(slice),
            )
        };
        unsafe { device.cmd_draw(cmd, 6, self.quad_len as u32, 0, 0) };
    }
}

const NOISE_PARAMETER: NoiseParameters = NoiseParameters::default();

#[derive(Debug, Default)]
struct Chunk {
    material: Vec<MatSize>,
    texture_buffer: BufferIndex,
}

impl Chunk {
    fn new(res: &mut BufferStorage, cmd: vk::CommandBuffer, graphic: TKQueue, lod: u32, global_pos: Vec3) -> Self {
        let mut material = vec![0; CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE];
        let chunk_size = CHUNK_SIZE;

        let surface_grid = generate_height_map(
            global_pos.x as i32,
            global_pos.z as i32,
            global_pos.y as i32,
            chunk_size,
            &mut material,
            &NOISE_PARAMETER,
        );

        let mut builder = BufferBuilder::new_storage_buffer();
        let mat = util::slice_as_u8_vec(&material);
        let texture_buffer = builder.set_size(mat.len() as u64).set_data(&mat).set_frames(1).set_queue_family(graphic).set_name("texture_buffer").build_resource(res, cmd)[0];
        Self { material, texture_buffer }
    }

    fn get_y_offset(size: usize, y: f32) -> usize {
        (y * (size * size) as f32) as usize
    }

    fn get_z_offset(size: usize, z: f32) -> usize {
        (z * (size as f32)) as usize
    }
    fn get_x_offset(x: usize) -> usize {
        x
    }

    fn get_chunk_render_data(&self) -> BufferIndex {
        // write to it before?
        todo!();
    }
}
