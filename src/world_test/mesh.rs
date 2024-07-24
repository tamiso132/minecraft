use std::{slice::Windows, thread::yield_now, u8};

use direction::Axis;

use super::*;

const AXIS_COUNT: usize = 6;

const CHUNK_SIZE: usize = size_of::<Gridbits>() * 8;

fn mesh(y_axis: &[Gridbits]) {
    let size = size_of::<Gridbits>() * 8;

    #[inline]
    fn insert_voxel_to_axis(x: usize, y: usize, z: usize, block: Gridbits, axis_cols: &mut [[Gridbits; CHUNK_SIZE]; CHUNK_SIZE]) {
        axis_cols[x][y] |= block << z as Gridbits;
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
                insert_voxel_to_axis(z, y, x, y_axis[y_offset + z_offset + x], &mut axis_cols[Axis::Right.get_raw() * 2]);
            }
        }
    }

    for y in 0..CHUNK_SIZE {
        let y_offset = y * CHUNK_SIZE * CHUNK_SIZE;
        for z in 0..CHUNK_SIZE {
            let z_offset = z * CHUNK_SIZE;
            for x in 0..CHUNK_SIZE {
                insert_voxel_to_axis(z, x, y, y_axis[y_offset + z_offset + x], &mut axis_cols[Axis::Up.get_raw() * 2]);
            }
        }
    }

    for y in 0..CHUNK_SIZE {
        let y_offset = y * CHUNK_SIZE * CHUNK_SIZE;
        for z in 0..CHUNK_SIZE {
            let z_offset = z * CHUNK_SIZE;
            for x in 0..CHUNK_SIZE {
                insert_voxel_to_axis(x, y, z, y_axis[y_offset + z_offset + x], &mut axis_cols[Axis::Front.get_raw() * 2]);
            }
        }
    }

    // CULL FACES
    // ORDER don't matter as long as everything get culled
    for axis in 0..3 {
        for y in 0..size {
            for x in 0..size {
                let col = axis_cols[axis][y][x];

                axis_cols[axis][y][x] = col & !(col << 1);
                axis_cols[axis + 1][y][x] = col & !(col >> 1);
            }
        }
    }

    for face in 0..6 {
        let axis = Axis::from(face as u32 / 2);

        for z in 0..size {
            for x in 0..size {
                let mut column = axis_cols[face][z][x];

                let mut right_bits = 0;
                if (x + 1) < size {
                    right_bits = axis_cols[face][z][x + 1];
                }

                // TODO, trade places on right and forward
                while column != 0 {
                    let y = column.trailing_zeros();
                    column &= column - 1;
                    let mut right_extend = 1;

                    // EXTEND TO RIGHT (in plane)
                    let mut is_extend = (axis_cols[face][z][x + right_extend] >> y) & 1 == 1;

                    while is_extend {
                        is_extend = false;
                        if (x + 1) < size {
                            is_extend = (axis_cols[face][z][x + right_extend] >> y) & 1 == 1;
                            // clear the bit we extending to
                            axis_cols[face][z][x + right_extend] &= !((1 as Gridbits) << y);
                        }
                        right_extend += 1;
                    }

                    let mut up_extend = 1;
                    let mut up_bits = &mut axis_cols[face][z + up_extend];

                    // EXTEND UP (in plane)
                    loop {
                        let mut extend_up = true;
                        for right in 0..right_extend {
                            if (up_bits[right] >> y) & 1 == 0 {
                                extend_up = false;
                                break;
                            }
                        }

                        if extend_up {
                            // clear all merged up bits
                            for right in 0..right_extend {
                                up_bits[right] &= !((1 as Gridbits) << y);
                            }
                            up_extend += 1;
                            up_bits = &mut axis_cols[face][z + up_extend];
                        }

                        break;
                    }

                    let start_x = x;
                    let start_up = z;

                    let width = right_extend;
                    let height = up_extend;

                    axis.get_position(x as u32, y as u32, z as u32);

                    // TODO, know how to calculate real voxel positons
                }
            }
        }
    }
}

fn cull_hidden_faces(back_axis: &mut Vec<Gridbits>, up_axis: &mut Vec<Gridbits>, right_axis: &mut Vec<Gridbits>) {
    let chunk_length = size_of::<Gridbits>() * 8;

    // Cull away hidden faces 64 at a time
    for i in 0..chunk_length * chunk_length {
        // get the directions from the front and from the back and cull away all hidden
        back_axis[i] = (!(back_axis[i] << 1) & back_axis[i]) | (!(back_axis[i] >> 1) & back_axis[i]);

        up_axis[i] = (!(up_axis[i] << 1) & up_axis[i]) | (!(up_axis[i] >> 1) & up_axis[i]);

        right_axis[i] = (!(right_axis[i] << 1) & right_axis[i]) | (!(right_axis[i] >> 1) & right_axis[i]);
    }
}

fn create_mask(num_bits: u32) -> u8 {
    let x = 1 << (num_bits);
    if x > 0 {
        return x - 1;
    }
    return x;
}

fn create_mask_unchecked(num_bits: Gridbits) -> Gridbits {
    ((1 as Gridbits) << num_bits) - 1
}
