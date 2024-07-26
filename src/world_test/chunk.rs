use super::*;
use ash::vk::{self, PFN_vkAllocateCommandBuffers, PFN_vkCmdDrawMultiEXT};
use generation::NoiseParameters;
use glm::Vec3;
use voxelengine::{
    terrain::block::GPUBlock,
    vulkan::{
        resource::{AllocatedBuffer, BufferBuilder, BufferIndex, BufferStorage, BufferType, Memory},
        util::slice_as_u8,
        TKQueue,
    },
};

fn generate_chunk(global_x: i32, global_y: i32, global_z: i32, chunk_resolution: usize) -> Chunk {
    let x_start = global_x;
    let y_start = global_y;
    let z_start = global_z;
    let chunk_length = chunk_resolution;

    let grid = generation::generate_height_map(global_x, global_z, chunk_resolution, &NoiseParameters::default());

    let mut chunk = Chunk::new();

    for z in 0..chunk_length {
        let z_offset = chunk_resolution * z;
        for x in 0..chunk_length {
            let height = grid[x + z_offset] as usize;
            for y in 0..height {
                let y_offset = chunk_resolution * chunk_resolution * y;
                chunk.mats.mats[y_offset + z_offset + x] = 1;
            }
        }
    }

    chunk
}

#[derive(Debug)]
pub struct MatArray {
    pub mats: Vec<Gridbits>,
}

impl MatArray {
    fn new(size: usize) -> MatArray {
        let mut mats = vec![0; size * size * size];
        let colors = [0, 1];

        for y in 0..size {
            let y_offset = Chunk::get_y_offset(size, y as f32);
            for z in 0..size {
                let z_offset = Chunk::get_z_offset(size, z as f32);

                let current = (z) % 2;
                for x in 0..size {
                    let x_offset = Chunk::get_x_offset(x);

                    mats[y_offset + z_offset + x_offset] = colors[current];
                }
            }
        }
        Self { mats: mats.to_vec() }
    }

    fn get_texture_id(x: usize, y: usize, z: usize) -> TextureID {
        todo!();
    }
}
#[repr(C, align(16))]
#[derive(Default)]
struct ChunkConstant {
    pos: Vec3,
    cam_index: u32,
    quad_index: u32,
}

pub struct ChunkMesh {
    chunk: Chunk,
    center: Vec3,
    scale: f32,
    quad_len: usize,
    draw_commands: Option<Vec<BufferIndex>>,
    chunk_constant: [ChunkConstant; 1],
}

impl ChunkMesh {
    pub fn new_test(res: &mut BufferStorage, graphic_queue: TKQueue, cmd: vk::CommandBuffer) -> Self {
        let chunk = Chunk::new();

        let quads = mesh::mesh(&chunk.mats.mats);

        let buffers = BufferBuilder::new()
            .set_name("ChunkData-1")
            .set_data(slice_as_u8(&quads))
            .set_is_descriptor(true)
            .set_queue_family(graphic_queue)
            .set_size((quads.len() * size_of::<GPUBlock>()) as u64)
            .set_memory(Memory::Local)
            .set_type(BufferType::Storage)
            .build_resource(res, cmd);
        let chunk_constant = [ChunkConstant { pos: Vec3::zero(), cam_index: 0, quad_index: 0 }];
        Self {
            chunk,
            center: Vec3::zero(),
            scale: 1.0,
            draw_commands: Some(buffers),
            quad_len: quads.len(),
            chunk_constant: chunk_constant,
        }
    }

    pub unsafe fn draw(&self, device: &ash::Device, res: &BufferStorage, cmd: vk::CommandBuffer, layout: vk::PipelineLayout, cam_index: u32) {
        let shader_index = res.get_buffer_ref(self.draw_commands.as_ref().unwrap().clone()[0]).index;

        device.cmd_push_constants(
            cmd,
            layout,
            vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT | vk::ShaderStageFlags::COMPUTE,
            0,
            slice_as_u8(&self.chunk_constant),
        );
        device.cmd_draw(cmd, 6, self.quad_len as u32, 0, 0);
    }

    pub fn new(center: Vec3, lod: usize) -> Self {
        let size = 2usize.pow(lod as u32 - 1) as f32 * CHUNK_RESOLUTION as f32 * VOXEL_SCALE;
        let chunks = Self::generate_chunks(center - Vec3::new(size, 0.0, size), lod);
        let chunk = Self::generate_lod_chunk(lod, chunks);

        let target_size = CHUNK_RESOLUTION >> lod;
        let scale = target_size as f32 / CHUNK_RESOLUTION as f32;
        Self { center, scale, chunk, draw_commands: None, quad_len: 0, chunk_constant: [ChunkConstant::default()] }
    }

    fn generate_chunks(bot_left: glm::Vec3, lod: usize) -> Vec<Chunk> {
        let chunk_amount = 2usize.pow(lod as u32 - 1);
        let size = 2usize.pow(lod as u32 - 1) as f32 * CHUNK_RESOLUTION as f32 * VOXEL_SCALE;
        let mut chunks = vec![];
        for y in 0..chunk_amount {
            for z in 0..chunk_amount {
                for x in 0..chunk_amount {
                    chunks.push(generate_chunk(bot_left.x as i32, (bot_left.y - size) as i32, bot_left.z as i32, CHUNK_RESOLUTION));
                }
            }
        }
        chunks
    }

    fn generate_lod_chunk(lod: usize, chunks: Vec<Chunk>) -> Chunk {
        let chunk_amount = 2usize.pow(lod as u32 - 1);
        let target_size = CHUNK_RESOLUTION >> lod;

        let scale = target_size as f32 / CHUNK_RESOLUTION as f32;
        let mut chunk = Chunk::new();
        for y in 0..chunk_amount {
            // Offsets
            let y_offset = Chunk::get_y_offset(CHUNK_RESOLUTION, y as f32 * scale);
            let y_chunk_offset = y * chunk_amount * chunk_amount;

            for z in 0..chunk_amount {
                // Offsets
                let z_offset = Chunk::get_z_offset(CHUNK_RESOLUTION, z as f32 * scale);
                let z_chunk_offset = z * chunk_amount;

                for x in 0..chunk_amount {
                    // Offsets
                    let x_offset = target_size * x;
                    let lod_offset = y_offset + z_offset + x_offset;
                    let chunk_offset = y_chunk_offset + z_chunk_offset + x;

                    //    Chunk::generate_lod(&mut chunk, lod_offset, lod, &chunks[chunk_offset].mats.mats);
                }
            }
        }
        chunk
    }
}
#[derive(Debug)]
struct Chunk {
    mats: MatArray,
}
impl Chunk {
    fn new() -> Self {
        let mats = MatArray::new(CHUNK_RESOLUTION);

        Self { mats }
    }

    fn generate_lod(lod_chunk: &mut Chunk, lod_chunk_offset: usize, lod: usize, mats: &Vec<u64>) {
        // let target_size = CHUNK_RESOLUTION >> lod;
        // let scale = CHUNK_RESOLUTION / target_size;
        // for y in 0..target_size {
        //     let y_target_offset = Chunk::get_y_offset(target_size, y as f32);
        //     for sz in 0..target_size {
        //         let z_source_offset = Chunk::get_z_offset(CHUNK_RESOLUTION, (sz * scale) as f32);
        //         let z_target_offset = Chunk::get_z_offset(target_size, sz as f32);

        //         for sx in 0..target_size {
        //             let x_source_offset = Chunk::get_x_offset(sx * scale);
        //             let x_target_offset = Chunk::get_x_offset(sx);

        //             lod_chunk.mats.mats[z_target_offset + x_target_offset + y_target_offset + lod_chunk_offset] = Self::decompress(mats, scale, z_source_offset + x_source_offset);
        //         }
        //     }
        // }
    }

    // flexible enough to create other functions to use instead
    /// Decompresses by getting the biggest count of material
    fn decompress(mats: &Vec<u8>, scale: usize, source_offset: usize) -> u8 {
        let mut indices = [0u32; u8::MAX as usize];
        let mut index = 0;

        for y in 0..scale {
            let y_offset = Chunk::get_y_offset(CHUNK_RESOLUTION, y as f32);
            for z in 0..scale {
                let z_offset = Chunk::get_z_offset(CHUNK_RESOLUTION, z as f32);
                for x in 0..scale {
                    let x_offset = Chunk::get_x_offset(x);
                    let color = mats[y_offset + z_offset + x_offset + source_offset];

                    indices[color as usize] += 1;

                    if indices[index as usize] < indices[color as usize] {
                        index = color;
                    }
                }
            }
        }
        index
    }

    fn draw_mesh() {}

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
