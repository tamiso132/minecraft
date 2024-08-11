use libnoise::{Generator, Source};
use voxelengine::terrain::Chunk;

use super::*;

pub struct NoiseParameters {
    amplitude: f64,
    seed: u32,
    scale: [f64; 2],
    hill_effect: f64,
}

impl NoiseParameters {
    pub const fn default() -> Self {
        Self { amplitude: 10.0, seed: 51251351, scale: [0.2, 0.2], hill_effect: 15.0 }
    }
}

pub fn generate_height_map(global_x: i32, global_z: i32, chunk_resolution: usize, parameters: &NoiseParameters) -> Vec<u32> {
    let x_start = global_x;
    let z_start = global_z;
    let chunk_length = CHUNK_RESOLUTION;

    let mut grid = vec![0u32; chunk_length * chunk_length];
    let amplitude = parameters.amplitude;
    let seed = parameters.seed;
    let hill_effect = parameters.hill_effect;
    let scale = parameters.scale;

    let generator = Source::simplex(seed as u64).add(1.0).scale(scale);

    for z in 0..chunk_length {
        let z_offset = chunk_length * z;
        for x in 0..chunk_length {
            let nx = (x as f64 + x_start as f64) / chunk_length as f64;
            let nz = (z_offset as f64 + z_start as f64) / chunk_length as f64;
            grid[(z_offset + x as usize) as usize] = (((generator.sample([nx as f64, nz as f64]) * hill_effect).round() / hill_effect) * amplitude).round() as u32;
        }
    }
    grid
}
