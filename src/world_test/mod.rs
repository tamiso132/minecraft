const VOXEL_SCALE: f32 = 1.0;
const CHUNK_RESOLUTION: usize = 64;
const DEPTH: usize = 2;

type TextureID = u8;

fn calculate_lod_step(distance: f32) -> usize {
    1
}

struct NoiseParameters {
    amplitude: f64,
    seed: u32,
    scale: [f64; 2],
    hill_effect: f64,
}
impl NoiseParameters {
    const fn default() -> Self {
        Self { amplitude: 10.0, seed: 51251351, scale: [0.2, 0.2], hill_effect: 15.0 }
    }
}

fn generate_height_map(global_x: i32, global_z: i32, chunk_resolution: usize, parameters: &NoiseParameters) -> Vec<u32> {
    let x_start = global_x;
    let z_start = global_z;
    let chunk_length = CHUNK_RESOLUTION;

    let mut grid = vec![0u32; chunk_length * chunk_length];
    let amplitude = parameters.amplitude;
    let seed = parameters.seed;
    let hill_effect = parameters.hill_effect;
    let scale = parameters.scale;

    let generator = Source::simplex(seed as u64).add(1.0).scale(scale);

    for z in 0..chunk_length {
        let z_offset = chunk_length * z;
        for x in 0..chunk_length {
            let nx = (x as f64 + x_start as f64) / chunk_length as f64;
            let nz = (z_offset as f64 + z_start as f64) / chunk_length as f64;
            grid[(z_offset + x as usize) as usize] = (((generator.sample([nx as f64, nz as f64]) * hill_effect).round() / hill_effect) * amplitude).round() as u32;
        }
    }
    grid
}

fn generate_chunk(global_x: i32, global_y: i32, global_z: i32, chunk_resolution: usize) -> Chunk {
    let x_start = global_x;
    let y_start = global_y;
    let z_start = global_z;
    let chunk_length = chunk_resolution;

    let grid = generate_height_map(global_x, global_z, chunk_resolution, &NoiseParameters::default());

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

fn generate_occupancy_grid() {
    let u: u64 = 0;
    u.trailing_ones();
}

#[derive(Debug)]
struct MatArray {
    pub mats: Vec<u8>,
}

impl MatArray {
    fn new(size: usize) -> MatArray {
        let mut mats = vec![0; size * size * size];
        let colors = [0, 1];

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
    occupancy_grid: Vec<u64>,
}
const CHECK: usize = size_of::<Chunk>();
impl Chunk {
    fn new() -> Self {
        let mats = MatArray::new(CHUNK_RESOLUTION);

        Self { mats, occupancy_grid: todo!() }
    }

    fn generate_lod(lod_chunk: &mut Chunk, lod_chunk_offset: usize, lod: usize, mats: &Vec<u8>) {
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

use std::{ops::Div, ptr};

use ash::vk;
use glm::{Vec2, Vec3};
use libnoise::{Generator, Scale, Source};
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
