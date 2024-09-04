use super::{
    object::{MyColor, VoxObject},
    Range,
};

trait TBiome {
    fn threshold_range();
    fn load_biome();
    fn generate_biome(&self, chunk: &mut Vec<u32>, global_x: i32, global_y: i32, global_z: i32, lod_scale: i32);
}

enum BiomeType {
    Forest,
}

enum BlockType {
    Grass,
    Dirt,
    Stone,
}

struct SurfaceObject {
    object: VoxObject,
    frequency: f32,
}

struct BiomeBuilder {
    temp: Range<f32>,
    rainfall: Range<f32>,
    altitude: usize,
}

struct Flatland {
    temp: Range<f32>,
    rainfall: Range<f32>,
    altitude: usize,
}

impl TBiome for Flatland {
    fn threshold_range() {
        todo!()
    }

    fn load_biome() {
        todo!()
    }

    fn generate_biome(&self, chunk: &mut Vec<u32>, global_x: i32, global_y: i32, global_z: i32, lod_scale: i32) {
        todo!()
    }
}

impl BiomeType {}

/// saves all variations
struct Block {}
