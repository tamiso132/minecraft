use ash::vk;
use glm::Vec3;
use mesh::GPUQuad;
use object::GlobalColor;
use voxelengine::{
    terrain::block::GPUBlock,
    vulkan::{
        resource::{self, AllocatedBuffer, BufferBuilder, BufferIndex, BufferStorage, BufferType, Memory},
        util::{self, slice_as_u8},
        TKQueue,
    },
};

use super::{mesh, object, MatSize, CHUNK_RESOLUTION, CHUNK_SIZE};

#[repr(C, align(16))]
#[derive(Default)]
struct ChunkConstant {
    pos: Vec3,
    pub cam_index: u32,
    pub quad_index: u32,
    pub color_index: u32,
    pub world_index: u32,
    pub chunk_size: u32,
}

pub struct ChunkMesh {
    chunk: Chunk,
    center: Vec3,
    scale: f32,
    quad_len: usize,
    chunk_constant: [ChunkConstant; 1],
}
impl ChunkMesh {
    pub fn new_test(res: &mut BufferStorage, global_color: &mut GlobalColor, graphic_queue: TKQueue, cmd: vk::CommandBuffer) -> Self {
        let chunk = Chunk::new(global_color, res, cmd, graphic_queue);

        let quads = mesh::mesh(&chunk.material);

        let buffers = BufferBuilder::new()
            .set_name("ChunkData-1")
            .set_data(slice_as_u8(&quads))
            .set_is_descriptor(true)
            .set_queue_family(graphic_queue)
            .set_size((quads.len() * size_of::<GPUQuad>()) as u64)
            .build_resource(res, cmd);

        let texture_buffer = chunk.texture_buffer;

        Self {
            chunk,
            center: Vec3::zero(),
            scale: 1.0,
            quad_len: quads.len(),
            chunk_constant: [ChunkConstant {
                pos: Vec3::zero(),
                cam_index: 0,
                color_index: res.get_buffer_ref(global_color.buffer).index as u32,
                chunk_size: 64,
                quad_index: res.get_buffer_ref(buffers[0]).index as u32,
                world_index: res.get_buffer_ref(texture_buffer).index as u32,
            }],
        }
    }

    pub fn draw(&mut self, device: &ash::Device, res: &mut BufferStorage, cmd: vk::CommandBuffer, layout: vk::PipelineLayout, cam_index: u32, g_color_index: u32) {
        self.chunk_constant[0].cam_index = cam_index;
        self.chunk_constant[0].color_index = g_color_index;
        unsafe {
            device.cmd_push_constants(
                cmd,
                layout,
                vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT | vk::ShaderStageFlags::COMPUTE,
                0,
                slice_as_u8(&self.chunk_constant),
            )
        };
        unsafe { device.cmd_draw(cmd, 6, self.quad_len as u32, 0, 0) };
    }
}

#[derive(Debug)]
struct Chunk {
    material: Vec<MatSize>,
    texture_buffer: BufferIndex,
}
impl Chunk {
    fn new(global: &mut GlobalColor, res: &mut BufferStorage, cmd: vk::CommandBuffer, graphic: TKQueue) -> Self {
        let mut material = vec![0; CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE];

        let v = object::load_model("chr_knight.vox", global);
        for voxel in v.model.iter() {
            let y_offset = Chunk::get_y_offset(CHUNK_SIZE, voxel.y as f32);
            let x_offset = Chunk::get_x_offset(voxel.x as usize);
            let z_offset = Chunk::get_z_offset(CHUNK_SIZE, voxel.z as f32);

            material[y_offset + z_offset + x_offset] = voxel.i as MatSize;
        }

        let mut builder = BufferBuilder::new_storage_buffer();
        let mat = util::slice_as_u8(&material);
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
