use std::hash::Hash;
use std::{collections::HashMap, usize};

use dot_vox::{Color, DotVoxData};
use voxelengine::vulkan::resource::BufferIndex;

pub struct GlobalObjects {
    objects: Vec<VoxObject>,
}

pub struct GlobalColor {
    pub colors: Vec<MyColor>,
    pub indices_taken: HashMap<MyColor, usize>,
    pub buffer: BufferIndex,
}

impl GlobalColor {
    pub fn new() -> GlobalColor {
        Self { colors: vec![MyColor { r: 100, g: 100, b: 255, a: 255 }], indices_taken: HashMap::new(), buffer: 0 }
    }
}

pub struct MyVoxel {
    pub x: u8,
    pub y: u8,
    pub z: u8,
    pub i: usize,
}
impl MyVoxel {
    pub fn new(x: u8, y: u8, z: u8, i: usize) -> Self {
        Self { x, y, z, i }
    }
}

pub struct VoxObject {
    pub model: Vec<MyVoxel>,
}

#[repr(align(4))]
#[derive(Debug, Default, PartialEq, Eq, Hash, Clone)]
pub struct MyColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

struct VoxelObject {}

fn load_model(s: &str, g_colors: &mut GlobalColor) -> VoxObject {
    let s = format!("assets/{}", s);
    let mut v = dot_vox::load(s.as_str()).unwrap();
    let model = &mut v.models[0];

    let mut my_voxels = vec![];

    for voxels in model.voxels.iter() {
        let voxel_index;
        let color = convert(&v.palette[voxels.i as usize]);

        if g_colors.indices_taken.contains_key(&color) {
            voxel_index = *g_colors.indices_taken.get(&color).unwrap();
        } else {
            g_colors.colors.push(color.clone());
            voxel_index = g_colors.colors.len() - 1;
            g_colors.indices_taken.insert(color, voxel_index);
        }

        my_voxels.push(MyVoxel::new(voxels.x, voxels.z, voxels.y, voxel_index));
    }
    VoxObject { model: my_voxels }
}

pub fn init_models(g_colors: &mut GlobalColor) -> GlobalObjects {
    GlobalObjects { objects: vec![load_model("tree.vox", g_colors), load_model("chr_knight.vox", g_colors)] }
}

fn convert(external: &Color) -> MyColor {
    MyColor { r: external.r, g: external.g, b: external.b, a: external.a }
}
