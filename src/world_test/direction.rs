use std::fmt::Debug;

#[repr(usize)]
#[derive(Clone, Copy)]
pub enum Axis {
    Right, // left --> right
    Up,    // bottom --> top
    Front, // front --> back
}
impl Axis {
    //  first is width
    // second is column
    // third is height

    pub fn get_position(&self, x: u32, y: u32, z: u32) -> (u32, u32, u32) {
        match self {
            Axis::Right => (z, x, y),
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
            1 => Self::Up,
            2 => Self::Front,
            _ => panic!(),
        }
    }
}
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
