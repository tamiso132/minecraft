use std::{marker::PhantomData, sync::Arc};

pub(crate) mod layer;
use super::{
    chunk::get_y_offset,
    generation::NoiseParameters,
    object::{MyColor, VoxObject},
    MatSize, Range, CHUNK_RESOLUTION,
};
use layer::{BlockLayer, ChunkParameter, HeightMap, Layer};

pub struct Block {
    variations: Vec<u8>,
}

pub struct AllBiomes {
    pub flatland: Flatland,
    /// used so other crates cannot intantiate it
    phantom: PhantomData<()>,
}
impl AllBiomes {
    fn new() -> Self {
        // TODO, read from a file

        // LOAD all colors

        // let mut flatland_builder = BiomeBuilder::default();
        // {
        //     let temp_range: Range<f32> = super::Range::new(0.0, 20.0);
        //     let rain_density = super::Range::new(0.0, 0.4);

        //     let base_surface = 50;

        //     let amplitude = 10;
        //     let scale: [f64; 2] = [0.01, 0.01];
        //     let hill_effect = 15.0;
        //     let param = NoiseParameters { amplitude: amplitude, seed: 5315314314, scale, hill_effect };

        //     flatland_builder.add_temp(temp_range).add_rainfall_freq(rain_density).add_height_map(HeightMap { base_surface, param }).add_block_layer(BlockLayer::new(Range::new(-500 500), block_types, scale, seed));
        // }
        todo!()
        //Self { flatland: Flatland::new(flatland_builder), phantom: PhantomData::default() }
    }
}

pub trait TBiome {
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

pub struct BiomeBuilder {
    temp: Range<f32>,
    rainfall: Range<f32>,
    height_map: HeightMap,
    blocks: Vec<BlockLayer>,
}

impl BiomeBuilder {
    pub fn default() -> Self {
        let default_range = Range { max: 0.0, min: 0.0 };

        Self {
            temp: default_range,
            rainfall: default_range,
            height_map: HeightMap { base_surface: 0, param: NoiseParameters::default() },
            blocks: vec![],
        }
    }
    pub fn add_temp(&mut self, temp: Range<f32>) -> &mut Self {
        self.temp = temp;
        self
    }

    pub fn add_rainfall_freq(&mut self, frequency: Range<f32>) -> &mut Self {
        self.rainfall = frequency;
        self
    }

    pub fn add_height_map(&mut self, height_map: HeightMap) -> &mut Self {
        self.height_map = height_map;
        self
    }

    pub fn add_block_layer(&mut self, block_layer: BlockLayer) -> &mut Self {
        self.blocks.push(block_layer);
        self
    }
}

pub struct Flatland {
    temp: Range<f32>,
    rainfall: Range<f32>,

    height_map: HeightMap,
    blocks: Vec<BlockLayer>,
}

impl Flatland {
    pub fn new(builder: BiomeBuilder) -> Self {
        Self {
            temp: builder.temp,
            rainfall: builder.rainfall,
            height_map: builder.height_map,
            blocks: builder.blocks,
        }
    }
    pub fn add_block_layer(block_layer: BlockLayer) {}
}

impl TBiome for Flatland {
    fn threshold_range() {
        todo!()
    }

    fn generate_biome(&self, chunk: &mut Vec<u32>, global_x: i32, global_y: i32, global_z: i32, lod_scale: i32) {
        let chunk_length = CHUNK_RESOLUTION;
        let chunk_parameter = ChunkParameter { global_x, global_y, global_z, lod_scale, chunk };
        let height_map = self.height_map.generate(&chunk_parameter);

        let mut block_handler = vec![];

        let height_ref = &height_map;
        let chunk_param_ref = &chunk_parameter;

        for i in 0..self.blocks.len() {
            block_handler.push(tasc::sync::scoped(move || self.blocks[i].generate((chunk_param_ref, height_ref))));
        }

        // Join all terrain changes
        let mut blocks = vec![];
        for handler in block_handler {
            blocks.push(handler.wait().unwrap());
        }

        // apply changes to the chunk
        for block in blocks {
            if !block.0.is_empty() {
                let start_y = block.1;
                let end_y = block.2;

                unsafe {
                    let offset_y = get_y_offset(start_y as usize);
                    std::ptr::copy_nonoverlapping::<MatSize>(&block.0[0], &mut chunk[offset_y], block.0.len());
                }
            }
        }
    }

    fn load_biome() {
        todo!()
    }
}

impl BiomeType {}
