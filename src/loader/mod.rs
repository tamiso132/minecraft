use core::panic;
use std::{io::*, mem::ManuallyDrop, ptr::copy_nonoverlapping, sync::Arc};

use awedio::{sounds::MemorySound, Sound};
use serde::{de::Visitor, Deserialize, Serialize};
use voxelengine::core::asset::*;


mod sound;

const SOUND_PATH: &'static str = "assets/audio";

pub struct FileWrite {
    offset: u64,
    size: u64,
}

pub struct Res<T>{
    data: T,
}

pub struct ConstRes<T>{
    data: T,
}

impl<T> ISaveType for Res<T>
where 
T: Clone + 'static + Default + Serialize + for<'a> Deserialize<'a>,
{
    type DataType = T;
    
    fn size() -> usize {
        size_of::<T>()
    }
}

impl<T> ISaveType for ConstRes<T>
where
T: Default + Clone +'static
{
    type DataType = T;
    
    fn size() -> usize {
        size_of::<T>()
    }
}




pub struct MutRes;

impl<T> ISave<Res<T>> for MutRes
where 
T: Clone + 'static + Default + Serialize + for<'a> Deserialize<'a>,
{
    
    type Param = FileWrite;

    fn save_to_memory(v: &mut Vec<u8>, param: Self::Param, data: &<Res<T> as ISaveType>::DataType) {
        let diff = (param.offset + param.size) as usize - v.len();
       if  diff > 0 {
        for i in 0..diff{
            v.push(i as u8);
        }
       }
       unsafe{
        copy_nonoverlapping((data as *const T) as *const u8, &mut v[param.offset as usize], param.size as usize);
       }
    }

    fn save_to_file(data: &Vec<<Res<T> as ISaveType>::DataType>, bufwriter: &mut std::fs::File) {
        let src = bincode::serialize(data).unwrap();
        bufwriter.write(&src).unwrap();
    }

    fn load_from_file(file_str: &str) -> Vec<u8> {
       
        let mut file = std::fs::OpenOptions::new().create(true).read(true).open(file_str).unwrap();
        let mut buffer = vec![];

        file.read_to_end(&mut buffer).unwrap();

        let buffer = bincode::deserialize(&buffer).unwrap();

        return buffer;
    }
 
}








