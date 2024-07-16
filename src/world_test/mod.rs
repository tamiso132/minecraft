const VOXEL_SCALE: f32 = 1.0;
const CHUNK_RESOLUTION: usize = 64;
const DEPTH: usize = 5;

type TextureID = u8;

fn calculate_lod_step(distance: f32) -> usize {
    1
}
#[derive(Default, Clone, Copy)]
struct Color {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}
impl Color {
    fn black() -> Self {
        Self { r: 0, g: 0, b: 0, a: 255 }
    }

    fn white() -> Self {
        Self { r: 255, g: 255, b: 255, a: 255 }
    }
}

struct MatArray {
    pub mats: [Color; CHUNK_RESOLUTION * CHUNK_RESOLUTION * CHUNK_RESOLUTION],
}

impl MatArray {
    fn new() -> MatArray {
        let mut mats = [Color::default(); CHUNK_RESOLUTION * CHUNK_RESOLUTION * CHUNK_RESOLUTION];
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
        Self { mats }
    }

    fn get_texture_id(x: usize, y: usize, z: usize) -> TextureID {
        todo!();
    }
}

struct Chunk {
    mats: MatArray,
    buffer: BufferIndex,
}
const CHECK: usize = size_of::<Chunk>();
impl Chunk {
    fn new(lod: usize) -> Self {
        let mats = MatArray::new();

        let mut current_offset = 0;

        for z in 0..CHUNK_RESOLUTION / lod {}

        Self { mats: MatArray::new(), buffer: 0 }
    }

    fn average_2x2(lod: usize, start_offset: usize, mats: &[Color; CHUNK_RESOLUTION * CHUNK_RESOLUTION * CHUNK_RESOLUTION]) -> Color {
        let mut r = 0;
        let mut g = 0;
        let mut b = 0;

        let per_voxel = lod * 2;
        for z in 0..per_voxel {
            let z_offset = Chunk::get_z_offset(z);
            for x in 0..per_voxel {
                let x_offset = Chunk::get_x_offset(x);

                r += mats[z_offset + start_offset + x_offset].r;
                g += mats[z_offset + start_offset + x_offset].g;
                b += mats[z_offset + start_offset + x_offset].b;
            }
        }
        let divide = 2 * 2 * lod;

        Color { r: r / divide, g: g / divide, b: b / divide, a: 255 }
    }

    fn draw_mesh() {}

    fn get_y_offset(y: usize) -> usize {
        y * CHUNK_RESOLUTION * CHUNK_RESOLUTION
    }

    fn get_z_offset(z: usize) -> usize {
        z * CHUNK_RESOLUTION
    }
    fn get_x_offset(x: usize) -> usize {
        x * CHUNK_RESOLUTION * CHUNK_RESOLUTION
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
        Chunk::new();
        let chunks = Vec::with_capacity(self.size * self.size);
        for z in 0..self.size {
            for x in 0..self.size {
                chunks.push(Chunk::new());
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
