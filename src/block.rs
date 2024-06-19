use std::marker::PhantomData;

use glm::{Mat4, Vec3, Vec4};

#[repr(u32)]
pub enum BlockType {
    Dirt,
    Grass,
    Stone,
    AcaciaL,
}
impl BlockType {
    pub const fn variants() -> usize {
        4
    }
}

#[repr(C, align(16))]
pub struct GPUBlock {
    model: glm::Mat4,
    texture_index: u32,
}

impl GPUBlock {
    pub fn new(position: Vec3, block_type: BlockType) -> Self {
        Self { model: Mat4::from_translation(position), texture_index: block_type as u32 }
    }

    pub fn test_random_positions() -> Vec<GPUBlock> {
        let mut blocks = vec![];
        for i in 0..100 {
            let mut translation = glm::Mat4::identity();
            translation = translation * Mat4::from_translation(Vec3::new(i as f32, 0.0, 0.0));
            blocks.push(GPUBlock { model: translation, texture_index: 0 })
        }

        blocks
    }

    pub fn from_position(position: Vec3) -> Self {
        Self { model: Mat4::from_translation(position), texture_index: 0 }
    }
}
#[repr(C, align(16))]
#[derive(Clone, Copy)]
pub struct GPUTexture {
    ambient: Vec3,
    shininess: f32,
    diffuse: Vec4,
    specular: Vec3,
    face_indices: [u32; 6],
}

impl GPUTexture {
    pub fn new(ambient: Vec3, diffuse: Vec3, specular: Vec3, face_indices: [u32; 6]) -> Self {
        let d = Vec4::from(diffuse);

        Self { face_indices, ambient, shininess: 0.0, diffuse: d, specular }
    }

    pub fn from_face_indices(face_indices: [u32; 6]) -> Self {
        Self { face_indices, ..Default::default() }
    }
}

impl Default for GPUTexture {
    fn default() -> Self {
        let n_ambient = Vec3::new(0.1, 0.1, 0.1);
        let n_diffuse = Vec3::new(0.5, 0.5, 0.5);
        let specular = Vec3::new(0.4, 0.4, 0.4);

        Self {
            face_indices: Default::default(),
            ambient: n_ambient,
            shininess: 0.0,
            diffuse: Vec4::from(n_diffuse),
            specular,
        }
    }
}

pub struct Materials {}

impl Materials {
    pub fn get_all() -> Vec<GPUTexture> {
        let atlas_width = 29;

        let mut gpu_textures = [GPUTexture::default(); BlockType::variants()];
        let dirt_index = 8 * atlas_width + 16;

        let grass_side_index = 10 * atlas_width + 16;
        let grass_top_index = 14 * atlas_width + 16 + 1;

        let stone_index = 12;

        let acacia_top = 2;
        let acacia_side = 1;

        // right-> left -> top -> bot -> front -> back

        let dirt_block = [dirt_index; 6];

        let mut grass_block = [grass_side_index; 6];
        grass_block[2] = grass_top_index;
        grass_block[3] = dirt_index;

        let stone_block = [stone_index; 6];

        let mut acacia_block = [acacia_side; 6];
        acacia_block[2] = acacia_top;
        acacia_block[3] = acacia_top;

        gpu_textures[BlockType::Dirt as usize] = GPUTexture::from_face_indices(dirt_block);
        gpu_textures[BlockType::Grass as usize] = GPUTexture::from_face_indices(grass_block);
        gpu_textures[BlockType::Stone as usize] = GPUTexture::from_face_indices(stone_block);
        gpu_textures[BlockType::AcaciaL as usize] = GPUTexture::from_face_indices(acacia_block);

        gpu_textures.to_vec()
    }
}
