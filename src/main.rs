#![feature(inherent_associated_types)]

use std::ffi::CString;

use ash::vk;
use vulkan::{
    builder::{self, ComputePipelineBuilder},
    util, PushConstant, SkyBoxPushConstant, VulkanContext,
};
use winit::{
    event::{Event, WindowEvent},
    event_loop::EventLoop,
    window::WindowBuilder,
};

mod vulkan;

extern crate nalgebra_glm as glm;
extern crate vk_mem;

fn main() {
    println!("Hello, world!");
    unsafe {
        
    }
}
