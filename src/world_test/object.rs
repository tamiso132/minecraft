use std::hash::Hash;
use std::{collections::HashMap, usize};

use dot_vox::{Color, DotVoxData};
use voxelengine::concurrency::ThreadPool;
use voxelengine::vulkan::resource::BufferIndex;

pub struct PtrWrapper<T> {
    data: *mut T,
}

impl<T> Clone for PtrWrapper<T>{
fn clone(&self) -> Self {
    Self { data: (self.data as usize).clone() as *mut T }   
}
}

impl<T> PtrWrapper<T>{
    pub fn new(data: *mut T) -> Self{
        Self{data}
    }
}

unsafe impl<T> Send for PtrWrapper<T> {}

pub struct GlobalObjects {
    objects: Vec<VoxObject>,
}

unsafe impl Send for VoxObject{}
unsafe impl Sync for VoxObject{}

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

fn load_colors_into_hashmap(g_colors: &mut GlobalColor){
    
}

fn multi_thread_load_color(model: &str, global_colors: &mut Vec<GlobalColor>)
{
    global_colors.push(GlobalColor::new());

    let last_elem_index = global_colors.len() - 1;
    let mut ptr_color = &mut global_colors[last_elem_index] as *mut GlobalColor;

}

pub fn init_models(g_colors: &mut GlobalColor, thread_pool:&mut ThreadPool) -> GlobalObjects {

    let mut global_objects = GlobalObjects{objects: vec![]};
    global_objects.objects.push(VoxObject::default());
    global_objects.objects.push(VoxObject::default());
  
    let mut global_colors = vec![];
    global_colors.push(GlobalColor::new());
    

    let mut ptr_objects = (&mut global_objects.objects[0]) as *mut VoxObject;
    let mut ptr_colors = (&mut global_colors[global_colors.len() - 1]) as *mut GlobalColor;

    let handle = std::thread::spawn({
        
        let mut ptr_object = PtrWrapper::new(ptr_objects);
        let mut ptr_color = PtrWrapper::new(ptr_colors);

        move || {
            let mut ptr_object = ptr_object;
            let mut ptr_color = ptr_color;
            unsafe {
            (*ptr_object.data) = load_model("a", &mut *ptr_color.data);
            }
        }
    });


    GlobalObjects { objects: vec![load_model("tree.vox", g_colors), load_model("chr_knight.vox", g_colors)] }
}

fn convert(external: &Color) -> MyColor {
    MyColor { r: external.r, g: external.g, b: external.b, a: external.a }
}
