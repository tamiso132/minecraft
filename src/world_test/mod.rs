//pub mod
pub mod biome;
pub mod chunk;
pub mod node;
pub mod object;
pub mod vkmulti;

// private mod
mod gen;
mod mesh;


// project crates
use voxelengine::vulkan::resource::*;
use crate::prelude::*;
use voxelengine::vulkan::*;

// external libs
use libnoise::{Generator, Source};

// type alias
/// Voxels per chunk, bit per voxel.
type Gridbits = u64;
/// Size of material index
type MatSize = u32;
/// Size of chunk index
type Chunkindex = u32;


/// Voxels per chunk length
const CHUNK_RESOLUTION: usize = size_of::<Gridbits>() * 8;
/// Octree depth
const DEPTH: usize = 1;
/// TODO, Voxelscale, should be written to a buffer and used in the shader
const VOXEL_SCALE: f32 = 1.0;
/// Distance till the node, renders parent instead.
const DISTANCE_THRESHOLD: f32 = 128.0;
/// Size of one Octree
const OCTREE_LENGTH: f32 = 2u32.pow(DEPTH as u32) as f32 * CHUNK_RESOLUTION as f32 * VOXEL_SCALE;

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
