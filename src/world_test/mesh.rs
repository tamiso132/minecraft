use std::{slice::Windows, thread::yield_now, u8};

use super::*;

use std::fmt::Debug;

pub fn mesh(y_axis: &[Gridbits]) -> Vec<GPUQuad> {
    let size = size_of::<Gridbits>() * 8;

    #[inline]
    fn insert_voxel_to_axis(x: usize, y: usize, z: usize, block: Gridbits, axis_cols: &mut [[Gridbits; CHUNK_SIZE]; CHUNK_SIZE]) {
        axis_cols[z][x] |= block << y as Gridbits;
    }

    // solid binary for  each axis
    // Starts at lowest point of the chunk
    let mut axis_cols: [[[Gridbits; CHUNK_SIZE]; CHUNK_SIZE]; 6] = [[[0; CHUNK_SIZE]; CHUNK_SIZE]; 6];

    // create_axis(grid);

    for y in 0..CHUNK_SIZE {
        let y_offset = y * CHUNK_SIZE * CHUNK_SIZE;
        for z in 0..CHUNK_SIZE {
            let z_offset = z * CHUNK_SIZE;
            for x in 0..CHUNK_SIZE {
                insert_voxel_to_axis(z, x, y, y_axis[y_offset + z_offset + x], &mut axis_cols[Axis::Right.get_raw() * 2]);
            }
        }
    }

    for y in 0..CHUNK_SIZE {
        let y_offset = y * CHUNK_SIZE * CHUNK_SIZE;
        for z in 0..CHUNK_SIZE {
            let z_offset = z * CHUNK_SIZE;
            for x in 0..CHUNK_SIZE {
                insert_voxel_to_axis(x, y, z, y_axis[y_offset + z_offset + x], &mut axis_cols[Axis::Up.get_raw() * 2]);
            }
        }
    }

    for y in 0..CHUNK_SIZE {
        let y_offset = y * CHUNK_SIZE * CHUNK_SIZE;
        for z in 0..CHUNK_SIZE {
            let z_offset = z * CHUNK_SIZE;
            for x in 0..CHUNK_SIZE {
                let block = (y_axis[y_offset + z_offset + x] & 1) == 1;
                insert_voxel_to_axis(x, z, y, y_axis[y_offset + z_offset + x], &mut axis_cols[Axis::Front.get_raw() * 2]);
            }
        }
    }

    // CULL FACES
    // ORDER don't matter as long as everything get culled
    for axis in 0..3 {
        for z in 0..size {
            for x in 0..size {
                let col = axis_cols[axis * 2][z][x];

                axis_cols[axis * 2][z][x] = col & !(col << 1);
                axis_cols[axis * 2 + 1][z][x] = col & !(col >> 1);
            }
        }
    }

    let mut quads = vec![];

    for face in 0..6 {
        let axis = Axis::from(face as u32 / 2);
        let add = ((face + 1) % 2 == 0) as u32;
        for z in 0..size {
            for x in 0..size {
                let mut column = axis_cols[face][z][x];

                // TODO, trade places on right and forward
                while column != 0 {
                    let mut y = column.trailing_zeros();
                    column &= !((1 as Gridbits) << y);
                    axis_cols[face][z][x] &= !((1 as Gridbits) << y);

                    let mut right_extend = 0;

                    // EXTEND TO RIGHT (in plane)
                    let mut is_extend;

                    loop {
                        let next_right = right_extend + 1;
                        // is a block to the right
                        if (x + next_right) >= size {
                            break;
                        }

                        is_extend = (axis_cols[face][z][x + next_right] >> y) & 1 == 1;

                        // is a face to the right
                        if !is_extend {
                            break;
                        }

                        axis_cols[face][z][x + next_right] &= !((1 as Gridbits) << y);
                        right_extend += 1;
                    }
                    let mut up_extend = 0;

                    // EXTEND UP (in plane)
                    loop {
                        let mut extend_up = true;
                        let mut next_up = up_extend + 1;
                        if (z + next_up) >= size {
                            break;
                        }
                        let up_bits = &mut axis_cols[face][z + next_up];

                        for right in 0..=right_extend {
                            if (up_bits[right + x] >> y) & 1 == 0 {
                                extend_up = false;
                                break;
                            }
                        }

                        if extend_up {
                            // clear all merged up bits
                            for right in 0..=right_extend {
                                up_bits[right + x] &= !((1 as Gridbits) << y);
                            }
                            up_extend += 1;
                            continue;
                        }

                        break;
                    }

                    let width = right_extend + 1;
                    let height = up_extend + 1;

                    let pos = axis.get_position(x as u32, y as u32 + add, z as u32);

                    quads.push(GPUQuad::new(pos.0 as u64, pos.1 as u64, pos.2 as u64, width as u64, height as u64, face as u64));
                    let x = 1;
                }
            }
        }
    }
    quads
}

#[repr(usize)]
#[derive(Clone, Copy)]
pub enum Axis {
    Right, // left --> right
    Front, // front --> back
    Up,    // bottom --> top
}

impl Axis {
    //  first is width
    // second is column
    // third is height

    pub fn get_position(&self, x: u32, y: u32, z: u32) -> (u32, u32, u32) {
        match self {
            Axis::Right => (y, z, x),
            Axis::Up => (x, y, z),
            Axis::Front => (x, z, y),
        }
    }

    pub fn get_raw(&self) -> usize {
        (*self).clone() as usize
    }
}

impl From<u32> for Axis {
    fn from(axis: u32) -> Self {
        match axis {
            0 => Self::Right,
            1 => Self::Front,
            2 => Self::Up,
            _ => panic!(),
        }
    }
}

#[repr(C, align(8))]
pub struct GPUQuad {
    data: u64,
}

impl GPUQuad {
    pub fn new(x: u64, y: u64, z: u64, w: u64, h: u64, face: u64) -> Self {
        let mask_6 = ((1 as u64) << 7) - 1;
        let mask_3 = ((1 as u64) << 3) - 1;
        let data = (x & mask_6) | ((y & mask_6) << 7) | ((z & mask_6) << 14) | ((w & mask_6) << 21) | ((h & mask_6) << 28) | ((face & mask_3) << 35);

        Self { data }
    }

    pub fn println(&self) {
        let mask_6 = ((1 as u64) << 7) - 1;
        println!("bits: {:b}", mask_6);
        let mask_3 = ((1 as u64) << 4) - 1;

        let x = self.data & mask_6;
        let y = (self.data >> 7) & mask_6;
        let z = (self.data >> 14) & mask_6;
        let w = (self.data >> 21) & mask_6;
        let h = (self.data >> 28) & mask_6;
        let f = (self.data >> 35) & mask_3;

        println!("x: {}\ny: {}\nz: {}\nw: {}\nh: {}\nf: {}", x, y, z, w, h, f);
    }
}

// x 6 bits
// y 6 bits
// z 6 bits
// w 6 bits
// h 6 bits
// face: 3 bits
