use std::{
    cmp::{max, min},
    collections::HashMap,
    sync::Arc,
};

use libnoise::{Generator, Simplex, Source};

use crate::world_test::{
    chunk::{get_x_offset, get_y_offset, get_z_offset},
    generation::NoiseParameters,
    MatSize, Range, CHUNK_RESOLUTION,
};

pub trait Layer<T, O> {
    fn y_range(&self) -> Range<i32>;
    fn generate(&self, input: T) -> O;
}

pub struct HeightMap {
    pub(crate) base_surface: i32,
    pub(crate) param: NoiseParameters,
}

pub fn new(noise: &NoiseParameters) {}

type GlobalX = i32;
type GlobalY = i32;
type GlobalZ = i32;
type LodScale = i32;
type Chunk = Vec<u32>;

type SurfaceGrid = Vec<u32>;

pub(crate) struct ChunkParameter<'a> {
    pub global_x: i32,
    pub global_y: i32,
    pub global_z: i32,
    pub lod_scale: i32,
    pub chunk: &'a Vec<u32>,
}

impl<'a> Layer<(&ChunkParameter<'a>), SurfaceGrid> for HeightMap {
    fn y_range(&self) -> Range<i32> {
        todo!()
    }

    fn generate(&self, input: &ChunkParameter) -> SurfaceGrid {
        let chunks = input.chunk;
        let global_x = input.global_x;
        let global_y = input.global_y;
        let global_z = input.global_z;
        let lod_scale = input.lod_scale;

        let chunk_length = CHUNK_RESOLUTION;

        let mut grid = vec![0u32; chunk_length * chunk_length];

        let hill_effect = self.param.hill_effect;

        let generator = Source::simplex(self.param.seed as u64).add(1.0).scale(self.param.scale);

        let highest_voxel = CHUNK_RESOLUTION as i32 * lod_scale as i32 + global_y;
        // filled chunk
        let mut max_amplitude = 0;
        let mut base_height = chunk_length as usize;

        if highest_voxel > self.base_surface {
            // surface variation hight
            max_amplitude = min((highest_voxel - self.base_surface).abs(), self.param.amplitude as i32);
            base_height = ((self.base_surface - global_y).abs() / lod_scale) as usize;
        }

        if lod_scale > 1 {
            let higher_res_scale = lod_scale as i32 / 2;

            for z in 0..CHUNK_RESOLUTION {
                let curr_global_z = global_z + z as i32 * lod_scale as i32;
                let chunk_z_offset = get_z_offset(z);

                for x in 0..CHUNK_RESOLUTION {
                    let curr_global_x = global_x * x as i32 * lod_scale as i32;
                    let chunk_x_offset = get_x_offset(x);

                    let mut sums = 0;
                    for zz in 0..2 {
                        let higher_scale_z = curr_global_z + zz * higher_res_scale;
                        let nz = higher_scale_z;
                        for xx in 0..2 {
                            let higher_scale_x = curr_global_x + xx * higher_res_scale;
                            let nx = higher_scale_x;

                            let surface_y = ((((generator.sample([nx as f64, nz as f64]) * hill_effect).round() / hill_effect) * max_amplitude as f64).round() as i32) + base_height as i32;
                            sums += surface_y;
                        }
                    }
                    sums /= 4;

                    grid[chunk_x_offset + chunk_z_offset] = sums as u32;
                }
            }
        }
        grid
    }
}

pub(crate) struct BlockLayer {
    y_range: Range<i32>,
    block_types: Vec<(MatSize)>,
    scale: [f64; 3],
    seed: u64,
}

impl BlockLayer {
    pub fn new(y_range: Range<i32>, block_types: Vec<MatSize>, scale: [f64; 3], seed: u64) -> Self {
        Self { y_range, block_types, scale, seed }
    }
}

impl<'a> Layer<(&'a ChunkParameter<'a>, &SurfaceGrid), (Vec<MatSize>, i32, i32)> for BlockLayer {
    fn y_range(&self) -> Range<i32> {
        todo!()
    }

    fn generate(&self, input: (&'a ChunkParameter, &SurfaceGrid)) -> (Vec<MatSize>, i32, i32) {
        let chunks = input.0.chunk;
        let global_x = input.0.global_x;
        let global_y = input.0.global_y;
        let global_z = input.0.global_z;
        let lod_scale = input.0.lod_scale;
        let chunk_length = CHUNK_RESOLUTION;

        let generator = Simplex::new(5315).add(1.0).scale(self.scale);

        let highest_voxel = lod_scale * CHUNK_RESOLUTION as i32 + global_y;

        let is_inside = (self.y_range.max > global_y) & (self.y_range.min < highest_voxel);

        if !is_inside {
            return (vec![], 0, 0);
        }

        let y_max = min(self.y_range.max, highest_voxel);
        let y_min = max(self.y_range.min, global_y);
        let y_length = (y_max - y_min) / lod_scale;

        let mut grid = vec![0 as MatSize; y_length as usize];

        let mut higher_res_scale = lod_scale / 2;

        let mut lod_length = 2;

        if (lod_scale as f32 / 2.0) < 1.0 {
            lod_length = 1;
            higher_res_scale = lod_scale;
        }

        let start_y = (y_min - global_y) / lod_scale;
        for y in 0..y_length {
            let real_y = y as i32 * lod_scale + y_min;
            let chunk_y_offset = get_y_offset(y as usize);

            for z in 0..chunk_length {
                let real_z = z as i32 * lod_scale + global_z;
                let chunk_z_offset = get_z_offset(z);
                for x in 0..chunk_length {
                    let chunk_x_offset = get_x_offset(x);
                    let real_x = x as i32 * lod_scale + global_x;

                    let mut sum_avg = 0.0;

                    for yy in 0..lod_length {
                        let high_res_y = real_y + yy * higher_res_scale;
                        for zz in 0..lod_length {
                            let high_res_z = real_z + zz * higher_res_scale;
                            for xx in 0..lod_length {
                                let high_res_x = real_x + xx * higher_res_scale;
                                sum_avg += generator.sample([real_x as f64, real_y as f64, real_z as f64]) * (self.block_types.len() - 1) as f64;
                            }
                        }
                    }

                    sum_avg /= (lod_length * lod_length * lod_length) as f64;

                    let mat_index = sum_avg.round() as MatSize;
                    grid[chunk_y_offset + chunk_x_offset + chunk_z_offset] = mat_index;
                }
            }
        }
        (grid, start_y, start_y + y_length)
    }
}

pub(crate) struct TemperatureLayer {
    pub(crate) base_surface: i32,
    pub(crate) param: NoiseParameters,
}

impl<'a> Layer<(&ChunkParameter<'a>), Vec<f32>> for TemperatureLayer {
    fn y_range(&self) -> Range<i32> {
        todo!()
    }

    fn generate(&self, input: (&ChunkParameter<'a>)) -> Vec<f32> {
        let chunks = input.chunk;
        let global_x = input.global_x;
        let global_y = input.global_y;
        let global_z = input.global_z;
        let lod_scale = input.lod_scale as usize;
        // 0 means 1 chunk
        // 1 means 4 chunks
        // 2 means 8 chunks

        let generator = Simplex::new(self.param.seed as u64).scale(self.param.scale).mul(self.param.amplitude as f64);

        let mut grid: Vec<f32> = vec![0.0; (CHUNK_RESOLUTION * CHUNK_RESOLUTION) as usize];

        generate_2d_map(input, &mut grid, |nx, nz| generator.sample([nx as f64, nz as f64]) as f32);

        let chunk_length = CHUNK_RESOLUTION / lod_scale;

        let mut temp_chunk_avg_grid = vec![];
        for chunk_z in 0..lod_scale {
            for chunk_x in 0..lod_scale {
                // can be multithreaded
                temp_chunk_avg_grid.push(calculate_average_2d(chunk_x, chunk_z, chunk_length, &grid));
            }
        }

        temp_chunk_avg_grid
    }
}

fn generate_2d_map<'a, F, S>(input: &ChunkParameter<'a>, grid: &mut [S], sample_func: F)
where
    F: Fn(i32, i32) -> f32,
    S: num::FromPrimitive + num::ToPrimitive,
{
    let chunks = input.chunk;
    let global_x = input.global_x;
    let global_y = input.global_y;
    let global_z = input.global_z;
    let lod_scale = input.lod_scale;

    let mut higher_res_scale = lod_scale / 2;
    let mut lod_length = 2;

    if (lod_scale as f32 / 2.0) < 1.0 {
        lod_length = 1;
        higher_res_scale = lod_scale;
    }

    for z in 0..CHUNK_RESOLUTION {
        let curr_global_z = global_z + z as i32 * lod_scale as i32;
        let chunk_z_offset = get_z_offset(z);

        for x in 0..CHUNK_RESOLUTION {
            let curr_global_x = global_x * x as i32 * lod_scale as i32;
            let chunk_x_offset = get_x_offset(x);

            let mut sums = 0.0 as f32;
            for zz in 0..lod_length {
                let higher_scale_z = curr_global_z + zz * higher_res_scale;
                let nz = higher_scale_z;
                for xx in 0..lod_length {
                    let higher_scale_x = curr_global_x + xx * higher_res_scale;
                    let nx = higher_scale_x;

                    sums += sample_func(nx, nz);
                }
            }
            sums /= (lod_length * lod_length) as f32;

            grid[chunk_x_offset + chunk_z_offset] = S::from_f32(sums).unwrap();
        }
    }
}

/// x,y the chunk coordinates
/// grid is the noise
/// chunk_length is the lod length for 1 chunk
fn calculate_average_2d(chunk_x: usize, chunk_z: usize, chunk_length: usize, grid: &[f32]) -> f32 {
    let mut sum = 0.0;
    let start_x = chunk_x * chunk_length;
    let start_z = chunk_z * chunk_length;
    for z in 0..chunk_length {
        let z_offset = get_z_offset(start_z + z);
        for x in 0..chunk_length {
            let x_offset = get_x_offset(start_x + x);
            sum += grid[z_offset + x_offset];
        }
    }
    sum / (chunk_length * chunk_length) as f32
}
