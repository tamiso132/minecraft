#[repr(usize)]
#[derive(Clone, Copy)]
pub enum Axis {
    Right, // left --> right
    Up,    // bottom --> top
    Front, // front --> back
}
impl Axis {
    pub fn get_position(&self, x: u32, y: u32, z: u32) -> (u32, u32, u32) {
        match self {
            Axis::Right => (z, y, x),
            Axis::Up => (z, x, y),
            Axis::Front => (x, y, z),
        }
    }

    pub fn get_quad(&self, x: u32, y: u32, z: u32, w: u32, h: u32) -> GPUQuad {
        match self {
            Axis::Right => {
                let b_l = (z, y, x);
                let b_r = (z, y, x + w);

                let t_l = (z, y + h, x);
                let t_r = (z, y + h, x + w);
            }

            Axis::Up => {
                let b_l = (z, y, x);
                let b_r = (z, y, x + w);

                let t_l = (z, y + h, x);
                let t_r = (z, y + h, x + w);
            }

            Axis::Front => {
                let b_l = (x, y, z);
                let b_r = (x + w, y, z);

                let t_l = (x, y + h, z);
                let t_r = (x + w, y + h, z);
            }
        }
        todo!()
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

pub struct Quad {
    b_l: u32,
    b_r: u32,
    t_l: u32,
    t_r: u32,
}

pub struct GPUQuad {
    data: u64,
}

impl GPUQuad {
    pub fn new(x: usize, y: usize, z: usize, w: usize, h: usize, face: usize) {
        let mask_6 = ((1 as usize) << 7) - 1;
        let mask_3 = ((1 as usize) << 4) - 1;
        let data = (x & mask_6) | ((y & mask_6) << 6) | ((z & mask_6) << 12) | ((w & mask_6) << 18) | ((w & mask_6) << 24) | ((face & mask_3) << 30);
    }
}

// x 6 bits
// y 6 bits
// z 6 bits
// w 6 bits
// h 6 bits
// face: 3 bits
