pub mod biome;
pub mod chunk;
mod generation;
mod mesh;
pub mod node;
pub mod object;

const CHUNK_RESOLUTION: usize = 64;
const DEPTH: usize = 1;

const VOXEL_SCALE: f32 = 1.0;
const CHUNK_SIZE: usize = size_of::<Gridbits>() * 8;
const DISTANCE_THRESHOLD: f32 = 128.0;
const OCTREE_LENGTH: f32 = 2u32.pow(DEPTH as u32) as f32 * CHUNK_SIZE as f32 * VOXEL_SCALE;

pub type Gridbits = u64;
type MatSize = u32;
type TextureID = u8;
type Chunkindex = u32;

#[derive(PartialOrd, PartialEq)]
pub(crate) struct Vec3Wrapper {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}
#[derive(Clone, Copy)]
pub struct Range<T> {
    pub min: T,
    pub max: T,
}
impl<T> Range<T> {
    pub fn new(min: T, max: T) -> Self {
        Self { min, max }
    }
}
