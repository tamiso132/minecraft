use core::f64;
use std::{cmp::{max, min}, usize};

use chunk::{get_x_offset, get_y_offset, get_z_offset};
use libnoise::{Generator, Source};
use voxelengine::terrain::Chunk;

use super::*;

const SURFACE_LEVEL:f32 = 0.0;

pub struct NoiseParameters {
    amplitude: u32,
    seed: u32,
    scale: [f64; 2],
    hill_effect: f64,
}

impl NoiseParameters {
    pub const fn default() -> Self {
        Self { amplitude: 10, seed: 51251351, scale: [0.2, 0.2], hill_effect: 15.0 }
    }
}

pub(crate) fn generate_height_map(global_x: i32, global_z: i32, global_y: i32, chunk_resolution: usize, mat_vec: &mut Vec<MatSize>, parameters: &NoiseParameters) -> Vec<u32> {
    let chunk_length = CHUNK_RESOLUTION;

    let mut grid = vec![0u32; chunk_length * chunk_length];
    let amplitude = parameters.amplitude;
    let seed = parameters.seed;
    let hill_effect = parameters.hill_effect;
    let scale = parameters.scale;

    let generator = Source::simplex(seed as u64).add(1.0).scale(scale);

    for z in 0..chunk_length{
        let z_offset = get_z_offset(chunk_length, z as f32);
        for x in 0..chunk_length{
            let nx = (x as f64 + global_x as f64) / chunk_length as f64;
            let nz = (z as f64 + global_z as f64) / chunk_length as f64;
            let mut sum = 0;

            if global_y < 0 {
                sum = global_y.abs();
            }
        
            let min_sum = 64 - sum;

            if min_sum  > 0{
                let max_amplitude = max(min(min_sum, amplitude as i32), 1) as f64;

                grid[(z_offset + x as usize) as usize] = ((((generator.sample([nx as f64, nz as f64]) * hill_effect).round() / hill_effect) * max_amplitude).round() as u32) + sum as u32;
            }

        }
    }

    for z in 0..chunk_length {
        
        let z_offset = get_z_offset(chunk_length, z as f32);
        
        for x in 0..chunk_length {
        
            let x_offset = get_x_offset(x);

            let height = grid[(z_offset + x as usize) as usize];
            for y in 0..height{
                let y_offset = get_y_offset(chunk_length, y as f32);
                mat_vec[z_offset + x_offset + y_offset] = 4;
            }
        }
    }
    grid
}
