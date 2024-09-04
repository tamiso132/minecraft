use core::f64;
use std::{
    cmp::{max, min},
    collections::HashMap,
    usize,
};

use chunk::{get_x_offset, get_y_offset, get_z_offset};
use libnoise::{Generator, Source};
use voxelengine::terrain::Chunk;

use super::*;

const SURFACE_LEVEL: f32 = 0.0;

pub struct NoiseParameters {
    pub amplitude: u32,
    pub seed: u32,
    // FREQUENCY
    pub scale: [f64; 2],
    pub hill_effect: f64,
}

impl NoiseParameters {
    pub const fn default() -> Self {
        Self { amplitude: 10, seed: 51251351, scale: [0.2, 0.2], hill_effect: 15.0 }
    }
}

pub(crate) fn noise_map_2d(noise_param: &NoiseParameters) {}

/// independent from other CHUNK_RESOLUTION
pub(crate) fn generate_basic_terrain(global_x: i32, global_y: i32, global_z: i32, lod_scale: usize, mat_vec: &mut Vec<MatSize>, parameters: &NoiseParameters) -> Vec<u32> {
    let chunk_length = CHUNK_RESOLUTION;

    let mut grid = vec![0u32; chunk_length * chunk_length];
    let amplitude = parameters.amplitude;
    let seed = parameters.seed;
    let hill_effect = parameters.hill_effect;
    let scale = parameters.scale;
    let generator = Source::simplex(seed as u64).add(1.0).scale(scale);
    let mat_generator = Source::simplex(53159491 as u64).add(1.0).scale([100.0]);
    // when surface begins
    let min_surface = {
        if global_y < SURFACE_LEVEL as i32 {
            (SURFACE_LEVEL as i32 - global_y).abs()
        } else {
            0
        }
    };

    // for z in 0..chunk_length {
    //     let z_offset = get_z_offset(chunk_length, z as f32);
    //     for x in 0..chunk_length {
    //         let nx = (x as f64 * lod_scale as f64 + global_x as f64) / chunk_length as f64;
    //         let nz = (z as f64 * lod_scale as f64 + global_z as f64) / chunk_length as f64;

    //         let surface_y = ((((generator.sample([nx as f64, nz as f64]) * hill_effect).round() / hill_effect) * amplitude as f64).round() as u32);

    //         grid[z_offset + x as usize] = max(min_surface as u32, surface_y);
    //     }
    // }
    // for z in 0..chunk_length {
    //     let z_offset = get_z_offset(chunk_length, z as f32);

    //     for x in 0..chunk_length {
    //         let x_offset = get_x_offset(x);

    //         let height = grid[(z_offset + x as usize) as usize];
    //         for y in 0..height {
    //             let nx = (x as f64 + global_x as f64) / chunk_length as f64;
    //             let nz = (z as f64 + global_z as f64) / chunk_length as f64;
    //             let ny = (y as f64 + global_y as f64) / chunk_length as f64;

    //             let y_offset = get_y_offset(chunk_length, y as f32);
    //             let mat = mat_generator.sample([nx as f64 + nz as f64 + ny as f64]) * 10.0;
    //             mat_vec[z_offset + x_offset + y_offset] = mat.round() as MatSize;
    //         }
    //     }
    // }
    todo!();
}

pub(crate) fn generate_surface(grid: &Vec<u32>, lod_scale: usize, chunk_resolution: usize, global_x: i32, global_y: i32, global_z: i32) -> Vec<u32> {
    let chunk_length = CHUNK_RESOLUTION;

    // do a check if it is possible

    let highest_point = CHUNK_RESOLUTION as i32 * lod_scale as i32 + global_y;

    if highest_point < SURFACE_LEVEL as i32 {
        // everything is solid

        todo!();
        // return
    }

    if lod_scale > 1 {
        let higher_res_scale = lod_scale as i32 / 2;

        let mut frequency: HashMap<usize, usize> = HashMap::new();

        for z in 0..CHUNK_RESOLUTION {
            let curr_global_z = global_z + z as i32 * lod_scale as i32;
            for x in 0..CHUNK_RESOLUTION {
                let curr_global_x = global_x * x as i32 * lod_scale as i32;

                for zz in 0..2 {
                    let higher_scale_z = curr_global_z + zz * higher_res_scale;
                    for xx in 0..2 {
                        let higher_scale_x = curr_global_x * xx * higher_res_scale;
                    }
                }

                frequency.clear();
            }
        }
    }
    todo!();
}

fn is_full_solid() {}
