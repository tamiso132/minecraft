use std::{any::Any, fs::OpenOptions, io::{Read, Write}, ptr::copy_nonoverlapping};

use awedio::*;
use sounds::MemorySound;
use voxelengine::core::asset::*;

use crate::world_test::object::MyColor;


fn test(){
    awedio::sounds::open_file("test.wav");
}



pub struct SoundLoader;
pub struct MaterialLoader;

pub struct TSound{
    sound: Box<dyn Sound>,
}


impl Loader for MaterialLoader{
    type T = Vec<MyColor>;

    fn load(file_str: &str) -> Box<dyn Any> {
        let mut file = OpenOptions::new().read(true).create(true).open(file_str).unwrap();
        let len = file.metadata().unwrap().len();
        if len > 0 {
            let mut buffer: Vec<u8> = Vec::with_capacity(len as usize);
            file.read_to_end(&mut buffer);
            let mut colors: Vec<MyColor> = Vec::with_capacity( len as usize / size_of::<MyColor>());
        unsafe{
            copy_nonoverlapping(buffer.as_ptr(), colors.as_mut_ptr().cast(), len as usize);
            return Box::new(colors);
        }
        }
        else{
            let v: Vec<MyColor> = vec![];
            return Box::new(v);
        }
    }

    fn save(file_str: &str, any: Box<dyn Any>) {
        let colors: Vec<MyColor> = *any.downcast().unwrap();
       
        let ptr = colors.as_ptr() as *const u8;
        let len = colors.len() * std::mem::size_of::<MyColor>();

        let mut file = OpenOptions::new().write(true).create(true).open(file_str).unwrap();
       
        unsafe{
            file.write_all(std::slice::from_raw_parts(ptr, len)).unwrap();
        }
    }

    fn reinterpret_bytes(any: Box<dyn Any>) -> Box<Vec<MyColor>> {
        any.downcast().unwrap()
    }

    fn get_full_path(file_str: &str) -> String{
        format!("assets/material/{}", file_str)
    }
}

impl Loader for SoundLoader {
    type T = MemorySound;

    fn load(file_str: &str) -> Box<dyn Any> {
        println!("file_str: {}", file_str);
        let memory_sound = awedio::sounds::open_file(file_str).unwrap().into_memory_sound().unwrap();
        Box::new(memory_sound) as Box<dyn Any>
        
    } 
    
    fn reinterpret_bytes(any: Box<dyn Any>) -> Box<MemorySound> {
        any.downcast().unwrap()
    }
    
    fn save(file_str: &str, any: Box<dyn Any>) {
        
    }
    
    fn get_full_path(file_str: &str) -> String {
        format!("assets/audio/{}", file_str)
    }
    
   
 
}





