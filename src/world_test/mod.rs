pub mod chunk;
mod direction;
mod generation;
mod mesh;
mod node;

const VOXEL_SCALE: f32 = 1.0;
const CHUNK_RESOLUTION: usize = 64;
const DEPTH: usize = 2;

pub type Gridbits = u64;
type TextureID = u8;
type Chunkindex = u32;
