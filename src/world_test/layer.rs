use std::{
    cmp::{max, min},
    collections::HashMap,
};

use libnoise::{Generator, Source};

use crate::world_test::CHUNK_RESOLUTION;

use super::{
    chunk::{get_x_offset, get_z_offset},
    generation::NoiseParameters,
    Range,
};

struct Ret<T> {
    pub data: T,
}

trait Layer<T> {
    fn y_range(&self) -> Range<i32>;
    fn apply(&self, chunk: &Vec<u32>, global_x: i32, global_y: i32, global_z: i32, lod_scale: i32) -> Ret<T>;
}

struct HeightMap {
    base_surface: i32,
    param: NoiseParameters,
    y_range: Range<i32>,
}

pub fn new(noise: &NoiseParameters) {}
impl Layer<Vec<u32>> for HeightMap {
    fn y_range(&self) -> Range<i32> {
        todo!()
    }

    fn apply(&self, chunk: &Vec<u32>, global_x: i32, global_y: i32, global_z: i32, lod_scale: i32) -> Ret<Vec<u32>> {
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
        Ret { data: grid }
    }
}
