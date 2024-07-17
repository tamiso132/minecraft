const VOXEL_SCALE: f32 = 1.0;
const CHUNK_RESOLUTION: usize = 64;
const DEPTH: usize = 2;

type TextureID = u8;

fn calculate_lod_step(distance: f32) -> usize {
    1
}
#[derive(Default, Clone, Copy, Debug)]
struct Color {
    pub rgba: [u8; 4],
}
impl Color {
    fn black() -> Self {
        Self { rgba: [0, 0, 0, 255] }
    }

    fn white() -> Self {
        Self { rgba: [255, 255, 255, 255] }
    }
}

#[derive(Debug)]
struct MatArray {
    pub mats: Vec<Color>,
}

impl MatArray {
    fn new(size: usize) -> MatArray {
        let mut mats = vec![Color::black(); size * size * size];
        let colors = [Color::black(), Color::white()];

        for y in 0..size {
            let y_offset = Chunk::get_y_offset(size, y as f32);
            for z in 0..size {
                let z_offset = Chunk::get_z_offset(size, z as f32);

                let current = z % 2;
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
#[derive(Debug)]
struct ChunkMesh {
    chunk: Chunk,
    center: Vec3,
    scale: f32,
}

impl ChunkMesh {
    pub fn new(center: Vec3, lod: usize) -> Self {
        let size = 2usize.pow(lod as u32 - 1) as f32 * CHUNK_RESOLUTION as f32 * VOXEL_SCALE;
        let chunks = Self::generate_chunks(center - Vec3::new(size, 0.0, size), lod);
        let chunk = Self::generate_lod_chunk(lod, chunks);

        let target_size = CHUNK_RESOLUTION >> lod;
        let scale = target_size as f32 / CHUNK_RESOLUTION as f32;
        Self { center, scale, chunk }
    }

    fn generate_chunks(bot_left: glm::Vec3, lod: usize) -> Vec<Chunk> {
        let chunk_amount = 2usize.pow(lod as u32 - 1);
        let mut chunks = vec![];
        for y in 0..chunk_amount {
            for z in 0..chunk_amount {
                for x in 0..chunk_amount {
                    chunks.push(Chunk::new());
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

                    Chunk::generate_lod(&mut chunk, lod_offset, lod, &chunks[chunk_offset].mats.mats);
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
const CHECK: usize = size_of::<Chunk>();
impl Chunk {
    fn new() -> Self {
        let mats = MatArray::new(CHUNK_RESOLUTION);

        Self { mats }
    }

    fn generate_lod(lod_chunk: &mut Chunk, lod_chunk_offset: usize, lod: usize, mats: &Vec<Color>) {
        let target_size = CHUNK_RESOLUTION >> lod;
        let scale = CHUNK_RESOLUTION / target_size;

        for y in 0..target_size {
            let y_target_offset = Chunk::get_y_offset(target_size, y as f32);
            for sz in 0..target_size {
                let z_source_offset = Chunk::get_z_offset(CHUNK_RESOLUTION, (sz * scale) as f32);
                let z_target_offset = Chunk::get_z_offset(target_size, sz as f32);

                for sx in 0..target_size {
                    let x_source_offset = Chunk::get_x_offset(sx * scale);
                    let x_target_offset = Chunk::get_x_offset(sx);

                    lod_chunk.mats.mats[z_target_offset + x_target_offset + y_target_offset + lod_chunk_offset] = Self::decompress(mats, scale, z_source_offset + x_source_offset);
                }
            }
        }
    }

    fn decompress(mats: &Vec<Color>, scale: usize, source_offset: usize) -> Color {
        let mut color_sum = [0; 3];
        for y in 0..scale {
            let y_offset = Chunk::get_y_offset(CHUNK_RESOLUTION, y as f32);
            for z in 0..scale {
                let z_offset = Chunk::get_z_offset(CHUNK_RESOLUTION, z as f32);
                for x in 0..scale {
                    let x_offset = Chunk::get_x_offset(x);
                    let color = mats[y_offset + z_offset + x_offset + source_offset].rgba;

                    for i in 0..3 {
                        color_sum[i] += color[i] as u32;
                    }
                }
            }
        }
        let mut out_color = Color::black();
        let divide = scale * scale * scale;
        for i in 0..3 {
            color_sum[i] /= divide as u32;
            out_color.rgba[i] = color_sum[i] as u8;
        }

        out_color
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

use std::{ops::Div, ptr};

use ash::vk;
use glm::{Vec2, Vec3};
use libnoise::Scale;
use voxelengine::vulkan::resource::{AllocatedBuffer, BufferIndex, BufferStorage};

// lazily allocate them
type Chunkindex = u32;

struct Range {
    pub start: usize,
    pub end: usize,
}
impl Range {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }
}

pub struct Node {
    pos: glm::Vec3,
    size: usize,
    parent: *mut Node,
    nodes: [*mut Node; 4],
    depth: usize,
    buffer: BufferIndex,

    mesh: ChunkMesh,
}

impl Node {
    fn new(res: &mut BufferStorage, size: usize, center_pos: glm::Vec3, parent: *mut Node, depth: usize) -> Self {
        //TODO generate chunk data
        let mesh = ChunkMesh::new(center_pos, depth);
        Self { pos: center_pos, mesh, size, parent, nodes: [std::ptr::null_mut(); 4], depth, buffer: 0 }
    }

    fn render_node() {}
}

impl Node {}

pub struct Octree {
    root: Node,
}

impl Octree {
    pub fn new(res: &mut BufferStorage, pos: Vec3) -> Octree {
        let size_in_voxels = 2usize.pow(DEPTH as u32 - 1) * (CHUNK_RESOLUTION);
        println!("size in voxels: {}", size_in_voxels);
        let size = (CHUNK_RESOLUTION as f32) * VOXEL_SCALE;

        let root = Node::new(res, size_in_voxels, Vec3::new(pos.x, pos.y, pos.z), ptr::null_mut(), DEPTH);

        Self { root }
    }
}
