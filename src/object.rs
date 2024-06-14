use glm::Vec3;

#[repr(C)]
struct Block {
    pos: glm::Mat4,
    texture_index: u32,
}

impl Block {}

#[repr(C)]
struct GPUTexture {
    face_indices: [u32; 6],
    ambient: Vec3,
    shininess: f32,
    diffuse: Vec3,
    specular: Vec3,
}

struct Chunk {}

pub struct SimplexNoise {}
impl SimplexNoise {
    const PERM: [u8; 256] = [
        151, 160, 137, 91, 90, 15, 131, 13, 201, 95, 96, 53, 194, 233, 7, 225, 140, 36, 103, 30, 69, 142, 8, 99, 37, 240, 21, 10, 23, 190, 6, 148,
        247, 120, 234, 75, 0, 26, 197, 62, 94, 252, 219, 203, 117, 35, 11, 32, 57, 177, 33, 88, 237, 149, 56, 87, 174, 20, 125, 136, 171, 168, 68,
        175, 74, 165, 71, 134, 139, 48, 27, 166, 77, 146, 158, 231, 83, 111, 229, 122, 60, 211, 133, 230, 220, 105, 92, 41, 55, 46, 245, 40, 244,
        102, 143, 54, 65, 25, 63, 161, 1, 216, 80, 73, 209, 76, 132, 187, 208, 89, 18, 169, 200, 196, 135, 130, 116, 188, 159, 86, 164, 100, 109,
        198, 173, 186, 3, 64, 52, 217, 226, 250, 124, 123, 5, 202, 38, 147, 118, 126, 255, 82, 85, 212, 207, 206, 59, 227, 47, 16, 58, 17, 182, 189,
        28, 42, 223, 183, 170, 213, 119, 248, 152, 2, 44, 154, 163, 70, 221, 153, 101, 155, 167, 43, 172, 9, 129, 22, 39, 253, 19, 98, 108, 110, 79,
        113, 224, 232, 178, 185, 112, 104, 218, 246, 97, 228, 251, 34, 242, 193, 238, 210, 144, 12, 191, 179, 162, 241, 81, 51, 145, 235, 249, 14,
        239, 107, 49, 192, 214, 31, 181, 199, 106, 157, 184, 84, 204, 176, 115, 121, 50, 45, 127, 4, 150, 254, 138, 236, 205, 93, 222, 114, 67, 29,
        24, 72, 243, 141, 128, 195, 78, 66, 215, 61, 156, 180,
    ];

    fn hash(i: u32) -> u8 {
        Self::PERM[i as u8 as usize]
    }
    fn grad(hash: u32, x: f32) -> f32 {
        let h = hash & 0x0F;
        let mut grad = 1.0 + (h & 7) as f32;

        if (h & 8) != 0 {
            grad *= -1.0;
        }

        grad * x
    }

    fn grad_2d(hash: u32, x: f32, y: f32) -> f32 {
        let h = hash & 0x3F;
        let (mut u, v) = {
            if h < 4 {
                (y, x)
            } else {
                (x, y)
            }
        };

        if h & 1 == 1 {
            u *= -1.0;
        }

        let mut v_multi = 2.0;

        if h & 2 == 2 {
            v_multi = -2.0;
        }

        u + v * v_multi
    }

    pub fn noise_2d(x: u32, y: u32, frequency: f32, seed: u32) -> f32 {
        Self::two_d(x as f32 + frequency + seed as f32, y as f32 + frequency + seed as f32)
    }

    pub fn generate_noise(x: f32, y: f32, octaves: u32, persistence: f32) -> f32 {
        let mut total = 0.0;
        let mut frequency = 1.0;
        let mut amplitude = 1.0;
        let mut max_value = 0.0;

        for _ in 0..octaves {
            total += Self::two_d(x * frequency, y * frequency) * amplitude;
            max_value += amplitude;
            amplitude *= persistence;
            frequency *= 2.0;
        }

        (total / max_value + 1.0) * 0.5
    }

    pub fn one_d(x: f32) -> f32 {
        let (n0, n1);

        // Corners coordinates (nearest integer values):
        let i0 = x.floor();
        let i1 = i0 + 1.0;
        // Distances to corners (between 0 and 1):
        let x0 = x - i0;
        let x1 = x0 - 1.0;

        let mut t0 = 1.0 - x0 * x0;
        t0 *= t0;
        n0 = t0 * t0 * Self::grad(Self::PERM[i0 as usize] as u32, x0);

        let mut t1 = 1.0 - x1 * x1;
        t1 *= t1;
        n1 = t1 * t1 * Self::grad(Self::PERM[i1 as usize] as u32, x1);

        0.395 * (n0 + n1)
    }

    pub fn two_d(x: f32, y: f32) -> f32 {
        const F2: f32 = 0.366025403;
        const G2: f32 = 0.211324865;

        // Skew the input space to determine which simplex cell we're in
        let s: f32 = (x + y) * F2;
        let xs: f32 = x + s;
        let ys: f32 = y + s;
        let i = xs.floor();
        let j = ys.floor();

        // Unskew the cell origin back to (x,y) space
        let t = (i + j) as f32 * G2;
        let _x0 = i - t;
        let _y0 = j - t;
        let x0 = x - _x0;
        let y0 = y - _y0;

        // For the 2D case, the simplex shape is an equilateral triangle.
        // Determine which simplex we are in.
        let (mut i1, mut j1) = (0, -1);

        if x0 > y0 {
            i1 = 1;
            j1 = 0;
        }

        let x1 = x0 - (i1 as f32) + G2;
        let y1 = y0 - (j1 as f32) + G2;
        let x2 = x0 - 1.0 + 2.0 * G2;
        let y2 = y0 - 1.0 + 2.0 * G2;

        let gi0_hash = i as u32 + Self::hash(j as u32) as u32;
        let gi1_hash = (i + i1 as f32) as u32 + Self::hash((j + j1 as f32) as u32) as u32;
        let gi2_hash = (i + 1.0) as u32 + Self::hash((j + 1.0) as u32) as u32;

        let gi0 = Self::hash(gi0_hash as u32);
        let gi1 = Self::hash(gi1_hash as u32);
        let gi2 = Self::hash(gi2_hash as u32);

        let (n0, n1, n2);

        // Calculate the contribution
        let mut t0 = 0.5 - x0 * x0 - y0 * y0;

        if t0 < 0.0 {
            n0 = 0.0;
        } else {
            t0 *= t0;
            n0 = t0 * t0 * Self::grad_2d(gi0 as u32, x0, y0);
        }
        // Calculate the contribution
        let mut t1 = 0.5 - x1 * x1 - y1 * y1;

        if t1 < 0.0 {
            n1 = 0.0;
        } else {
            t1 *= t1;
            n1 = t1 * t1 * Self::grad_2d(gi1 as u32, x1, y1);
        }
        // Calculate the contribution
        let mut t2 = 0.5 - x2 * x2 - y2 * y2;

        if t2 < 0.0 {
            n2 = 0.0;
        } else {
            t2 *= t2;
            n2 = t2 * t2 * Self::grad_2d(gi2 as u32, x2, y2);
        }

        45.23065 * (n0 + n1 + n2)
    }
}
