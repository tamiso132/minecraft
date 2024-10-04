use std::{mem::ManuallyDrop, ptr::copy_nonoverlapping, sync::Arc};

use awedio::sounds::MemorySound;
use voxelengine::core::asset::{ISave, ISaveType};

use super::ConstRes;



pub struct Sound{
    sound:  ConstRes<awedio::sounds::MemorySound>
}

impl Default for Sound{
    fn default() -> Self {
        Self { sound:  ConstRes { data:  MemorySound::from_samples(Arc::new(Vec::new()), 0, 0) } }
    }
}


pub struct SoundLoad;

impl<T: Default + Clone + 'static> ISave<ConstRes<T>> for SoundLoad{
    type Param = ();

    fn save_to_memory(v: &mut Vec<u8>, param: Self::Param, data: &<ConstRes<T> as ISaveType>::DataType) {
        panic!("You cannot use this method for ConstRes");
    }

    fn save_to_file(data: &Vec<<ConstRes<T> as ISaveType>::DataType>, bufwriter: &mut std::fs::File) {
        panic!("You cannot use this method for ConstRes");
    }

    fn load_from_file(file_str: &str) -> Vec<u8> {
        use awedio::Sound;

        let sound = ManuallyDrop::new(awedio::sounds::open_file(file_str).unwrap().into_memory_sound().unwrap());
        let mut vec:Vec<u8> = Vec::with_capacity(size_of::<MemorySound>());
       
        unsafe{
            copy_nonoverlapping((&sound as *const ManuallyDrop<MemorySound>).cast(), vec.as_mut_ptr() , size_of::<MemorySound>());
            vec.set_len(size_of::<MemorySound>());  
            vec
        }

    }
}