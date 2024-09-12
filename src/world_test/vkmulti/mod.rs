use ash::vk::{self, SubmitInfo};
use lazy_static::lazy_static;
use std::{
    cell::OnceCell,
    sync::{Arc, Mutex},
};
use voxelengine::vulkan::{util, TKQueue};

use super::biome::layer::new;

struct GlobalRecorder {
    pub available_recorder: Vec<vk::CommandBuffer>,
    pub all_pools: Vec<vk::CommandPool>,
    pub in_use_recorder: Vec<vk::CommandBuffer>,
    pub device: Option<Arc<ash::Device>>,
    pub tqueue: TKQueue,

    pub submit_all: vk::Fence,
}
impl GlobalRecorder {
    const fn new() -> Self {
        Self {
            available_recorder: vec![],
            in_use_recorder: vec![],
            device: unsafe { None },
            tqueue: TKQueue { queue: vk::Queue::null(), family: 0 },
            all_pools: vec![],
            submit_all: vk::Fence::null(),
        }
    }
}

pub fn initilize(device: Arc<ash::Device>, tqueue: TKQueue, capacity: u32) {
    unsafe {
        let mut global = GLOBAL.lock().unwrap();
        if global.available_recorder.len() != 0 {
            panic!("has been initialized multiple times");
        }

        for i in 0..capacity {
            let cmd_pool = util::create_pool(&device, tqueue.family);
            let cmd = util::create_cmd(&device, cmd_pool);

            global.available_recorder.push(cmd);
            global.all_pools.push(cmd_pool);
        }

        global.submit_all = util::create_fence(&device);
        global.device = Some(device);
        global.tqueue = tqueue;
    }
}

pub fn get_cmd() -> vk::CommandBuffer {
    let mut global = GLOBAL.lock().unwrap();
    let available;
    unsafe {
        if global.available_recorder.len() > 0 {
            available = global.available_recorder.pop().unwrap_unchecked();
            global.in_use_recorder.push(available);
        } else {
            let cmd_pool = util::create_pool(global.device.as_ref().unwrap(), global.tqueue.family);
            available = util::create_cmd(global.device.as_ref().unwrap(), cmd_pool);

            global.all_pools.push(cmd_pool);

            global.in_use_recorder.push(available);
        }

        available
    }
}

pub fn end_cmd(cmd: vk::CommandBuffer) {
    unsafe { GLOBAL.lock().unwrap().device.as_ref().unwrap().end_command_buffer(cmd) };
}

pub fn submit_all() -> vk::Fence {
    let mut global = GLOBAL.lock().unwrap();
    let mut submit_info = [SubmitInfo::default().command_buffers(&global.in_use_recorder)];
    unsafe {
        global.device.as_ref().unwrap().queue_submit(global.tqueue.queue, &submit_info, global.submit_all);
    }
    global.submit_all
}

lazy_static! {
    static ref GLOBAL: Mutex<GlobalRecorder> = Mutex::new(GlobalRecorder::new());
}
