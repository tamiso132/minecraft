const VOXEL_SCALE: f32 = 1.0;
const CHUNK_RESOLUTION: usize = 64;
const DEPTH: usize = 5;

type TextureID = u8;

fn calculate_lod_step(distance: f32) -> usize {
    1
}
#[derive(Default, Clone, Copy)]
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

struct MatArray {
    pub mats: Vec<Color>,
}

impl MatArray {
    fn new(size: usize) -> MatArray {
        let mut mats = [Color::default(); usize];
        let colors = [Color::black(), Color::white()];

        for y in 0..CHUNK_RESOLUTION {
            let y_offset = Chunk::get_y_offset(y);
            for z in 0..CHUNK_RESOLUTION {
                let z_offset = Chunk::get_z_offset(z);

                let current = z % 2;
                for x in 0..CHUNK_RESOLUTION {
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

struct Chunk {
    mats: MatArray,
    size: usize,
    buffer: BufferIndex,
}
const CHECK: usize = size_of::<Chunk>();
impl Chunk {
    fn new(lod: usize) -> Self {
        let mats = MatArray::new(CHUNK_RESOLUTION * CHUNK_RESOLUTION * CHUNK_RESOLUTION);

        let mut current_offset = 0;
        todo!();
    }

    fn generate_lod(lod: usize, start_offset: usize, mats: &[Color; CHUNK_RESOLUTION * CHUNK_RESOLUTION * CHUNK_RESOLUTION]) -> Chunk {
        let target_size = CHUNK_RESOLUTION >> lod;
        let scale = 2 * lod;
        let mut target_chunk = Chunk { mats: MatArray::new(target_size), buffer: 0, size: target_size };
        for sz in 0..target_size {
            let z_source_offset = Chunk::get_z_offset(CHUNK_RESOLUTION,sz);
            let z_target_offset = Chunk::get_z_offset(target_size, sz);
            for sx in 0..target_size {
                let x_source_offset = Chunk::get_x_offset(sx);
                let mut color_sum = [0u32; 4];
                let mut count = 0;

                target_chunk.mats.mats[];

                let target_index = tx + ty * target_size + tz * target_size * target_size;
                for i in 0..4 {
                    target_chunk.voxels[target_index].color[i] = (color_sum[i] / count) as u8;
                }
            }
        }
        target_chunk
    }

    fn decompress(mats: &[Color; CHUNK_RESOLUTION * CHUNK_RESOLUTION * CHUNK_RESOLUTION], scale: usize, source_offset: usize) -> Color {
        let mut color_sum = [0; 3];
        for z in 0..scale {
            let z_offset = Chunk::get_z_offset(CHUNK_RESOLUTION, z);
            for x in 0..scale {
                let x_offset = Chunk::get_x_offset(x);

                let color = mats[z_offset + x_offset + source_offset].rgba;

                for i in 0..3 {
                    color_sum[i] += color[i] as u32;
                }
            }
        }
        let mut out_color = Color::black();
        let divide = scale * scale;
        for i in 0..3 {
            color_sum[i] /= divide as u32;
            out_color.rgba[i] = color_sum[i] as u8;
        }

        out_color
    }

    fn draw_mesh() {}

    fn get_y_offset(size: usize, y: usize) -> usize {
        y * size * size
    }

    fn get_z_offset(size: usize, z: usize) -> usize {
        z * size
    }
    fn get_x_offset(x: usize) -> usize {
        x
    }

    fn get_chunk_render_data(&self) -> BufferIndex {
        // write to it before?
        self.buffer
    }
}

use std::{ops::Div, ptr};

use ash::vk;
use glm::{Vec2, Vec3};
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
    pos: glm::Vec2,
    size: usize,
    parent: *mut Node,
    nodes: [*mut Node; 4],
    depth: usize,
    buffer: BufferIndex,
}

impl Node {
    fn new(res: &mut BufferStorage, size: usize, pos: glm::Vec2, parent: *mut Node, depth: usize) -> Self {
        //TODO generate chunk data

        Self { pos, size, parent, nodes: [std::ptr::null_mut(); 4], depth, buffer: 0 }
    }

    fn generate_chunk_data_lod(&mut self) {
        let size = self.size as f32;

        let bot_left_pos = self.pos - Vec2::new(size / 2.0, size / 2.0);
        Chunk::new(self.depth);
        let chunks = Vec::with_capacity(self.size * self.size);
        for z in 0..self.size {
            for x in 0..self.size {
                chunks.push(Chunk::new(self.depth));
            }
        }
    }

    fn render_node() {}
}

impl Node {}

struct Octree {
    root: Node,
}

impl Octree {
    pub fn new(res: &mut BufferStorage, pos: Vec3) -> Octree {
        let size_in_voxels = 2usize.pow(DEPTH as u32 - 1) * (CHUNK_RESOLUTION);
        println!("size in voxels: {}", size_in_voxels);
        let size = (CHUNK_RESOLUTION as f32) * VOXEL_SCALE;

        let root = Node::new(res, size_in_voxels, Vec2::new(pos.x, pos.z), ptr::null_mut(), DEPTH);

        Self { root }
    }
}
