pub mod chunk;
mod generation;
mod mesh;
mod node;

const CHUNK_RESOLUTION: usize = 64;
const DEPTH: usize = 2;

const VOXEL_SCALE: f32 = 1.0;
const CHUNK_SIZE: usize = size_of::<Gridbits>() * 8;

pub type Gridbits = u8;
type TextureID = u8;
type Chunkindex = u32;
