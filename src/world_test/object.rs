use std::collections::HashMap;
use std::hash::Hash;

use dot_vox::{Color, DotVoxData};
use voxelengine::vulkan::resource::BufferIndex;

pub struct GlobalColor {
    pub colors: Vec<MyColor>,
    pub indices_taken: HashMap<MyColor, usize>,
    pub buffer: BufferIndex,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct MyColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

struct VoxelObject {}

pub fn load_model(s: &str, colors: &mut GlobalColor) -> dot_vox::DotVoxData {
    let s = format!("assets/{}", s);
    let mut v = dot_vox::load(s.as_str()).unwrap();

    for model in &mut v.models {
        for voxels in &mut model.voxels {
            let color = convert(&v.palette[voxels.i as usize + 1]);
            let index_overwrite;
            if colors.indices_taken.contains_key(&color) {
                index_overwrite = colors.indices_taken.get(&color).unwrap();
            } else {
                colors.indices_taken.insert(color.clone(), colors.colors.len());
                colors.colors.push(color);
                voxels.i = (colors.colors.len() - 1) as u8;
            }
        }
    }
    v
}

fn convert(external: &Color) -> MyColor {
    MyColor { r: external.r, g: external.g, b: external.b, a: external.a }
}
