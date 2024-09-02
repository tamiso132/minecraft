use core::panic;
use std::cell::UnsafeCell;
use std::hash::Hash;
use std::mem::{ManuallyDrop, MaybeUninit};
use std::{collections::HashMap, usize};

use dot_vox::{Color, DotVoxData};
use voxelengine::t_thread::{self, MutPtr, Ptr, ThreadPool};
use voxelengine::vulkan::resource::BufferIndex;

pub struct GlobalObjects {
    pub objects: Vec<VoxObject>,
    pub g_colors: GlobalColor,
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
    pub fn with_capacity(len: usize) -> Self {
        let colors = Vec::with_capacity(len);
        let indices_taken = HashMap::with_capacity(len);

        Self { colors, indices_taken, buffer: 0 }
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
#[derive(Default)]
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

fn load_colors_into_hashmap(g_colors: &mut GlobalColor) {}

fn load(model: &str) -> DotVoxData {
    let asset_path = format!("assets/{}", model);
    dot_vox::load(asset_path.as_str()).unwrap()
}

fn multi_thread_load_color(model: &DotVoxData, global_colors: &mut GlobalColor) {
    let mut ptr_color = MutPtr::new(global_colors);
    let mut ptr_model = Ptr::new(model as *const DotVoxData);
    let my_closure = move || {
        ptr_color = ptr_color;
        ptr_model = ptr_model;

        unsafe {
            for voxels in (*ptr_model.data).models[0].voxels.iter() {
                let color = convert(&(*ptr_model.data).palette[voxels.i as usize]);
                if !(*ptr_color.data).indices_taken.contains_key(&color) {
                    (*ptr_color.data).indices_taken.insert(color.clone(), 0);
                    (*ptr_color.data).colors.push(color);
                }
            }
        }
    };
    ThreadPool::execute(my_closure);
}

fn join_all_colors(global_colors: &mut Vec<GlobalColor>) -> GlobalColor {
    let mut capacity = 0;
    for c in global_colors.iter() {
        capacity += c.colors.len();
    }

    let mut real_g = GlobalColor::with_capacity(capacity);

    for c in global_colors.iter() {
        for color in c.colors.iter() {
            if !real_g.indices_taken.contains_key(&color) {
                real_g.colors.push(color.clone());
                let len = real_g.colors.len() - 1;
                real_g.indices_taken.insert(color.clone(), len);
            }
        }
    }
    return real_g;
}

fn multi_thread_objects(models: &Vec<UnsafeCell<DotVoxData>>, g_colors: &GlobalColor) -> Vec<VoxObject> {
    let mut objects = Vec::with_capacity(models.len());
    for i in 0..models.len() {
        objects.push(VoxObject::default());

        let g_color_ptr = Ptr::new(g_colors);
        let model_ptr = Ptr::new(models[i].get().cast_const());
        let voxel_ptr = MutPtr::new(&mut objects[i]);

        let closure = (|| {
            let model_ptr = model_ptr;
            let g_color_ptr = g_color_ptr;

            let mut my_voxels = vec![];
            unsafe {
                for voxels in &(*model_ptr.data).models[0].voxels {
                    let color = convert(&(*model_ptr.data).palette[voxels.i as usize]);

                    if g_colors.indices_taken.contains_key(&color) {
                        let voxel_index = *g_colors.indices_taken.get(&color).unwrap();
                        my_voxels.push(MyVoxel::new(voxels.x, voxels.z, voxels.y, voxel_index));
                    } else {
                        panic!("all the colors should already have been loaded");
                    }
                }
            }
        });
    }
    ThreadPool::join_tasks();
    objects
}

pub fn multi_thread_init() -> GlobalObjects {
    let model_names = ["tree.vox", "chr_knight.vox"];

    let mut dots: Vec<UnsafeCell<DotVoxData>> = Vec::with_capacity(model_names.len());
    unsafe {
        dots.set_len(model_names.len());
    }

    for i in 0..model_names.len() {
        let dox_ptr = MutPtr::new(dots[i].get());
        let str_ptr = Ptr::new(&model_names[i]);

        ThreadPool::execute(|| {
            let dox_ptr = dox_ptr;
            let str_ptr = str_ptr;

            unsafe { (dox_ptr).data.write(load(*str_ptr.data)) };
        });
    }
    ThreadPool::join_tasks();
    
    let mut threaded_colors = Vec::with_capacity(dots.len());
    for i in 0..dots.len() {
        threaded_colors.push(GlobalColor::new());
        multi_thread_load_color(unsafe { &*dots[i].get().cast_const() }, &mut threaded_colors[i]);
    }

    ThreadPool::join_tasks();
    let g_colors = join_all_colors(&mut threaded_colors);
    let objects = multi_thread_objects(&dots, &g_colors);

    GlobalObjects { objects, g_colors }
}

fn convert(external: &Color) -> MyColor {
    MyColor { r: external.r, g: external.g, b: external.b, a: external.a }
}
