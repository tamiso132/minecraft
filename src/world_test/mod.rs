pub mod chunk;
mod generation;
mod mesh;
pub mod node;
pub mod object;

const CHUNK_RESOLUTION: usize = 64;
const DEPTH: usize = 1;

const VOXEL_SCALE: f32 = 0.015625;
const CHUNK_SIZE: usize = size_of::<Gridbits>() * 8;
const DISTANCE_THRESHOLD: f32 = 128.0;

pub type Gridbits = u64;
type MatSize = u32;
type TextureID = u8;
type Chunkindex = u32;
