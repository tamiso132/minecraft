#![feature(inherent_associated_types)]

use std::{
    collections::HashMap,
    mem::transmute,
    time::{Duration, Instant},
};

use ash::vk::{self};
use env_logger::Builder;
use test::TestApplication;
use voxelengine::{
    app::{App, ApplicationTrait},
    core::camera::{Camera, Controls, Frustum, GPUCamera},
    terrain::block::{GPUBlock, GPUTexture, Materials},
    vulkan::{
        builder::{self, ComputePipelineBuilder},
        mesh::VertexBlock,
        resource::{self, AllocatedBuffer, AllocatedImage, BufferBuilder, BufferType, Memory},
        util, SkyBoxPushConstant, VulkanContext,
    },
};
use winit::{
    event::{self, ElementState, Event, RawKeyEvent, WindowEvent},
    event_loop::{self, EventLoop},
    keyboard::KeyCode,
    window::CursorGrabMode,
};
extern crate ultraviolet as glm;
extern crate voxelengine_gui as tgui;
extern crate dot_vox as vox;

mod test;
mod loader;
mod prelude;

pub const MAX_FRAMES_IN_FLIGHT: usize = 2;
mod world_test;
fn main() {
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(event_loop::ControlFlow::Poll);

    let mut application: App<TestApplication> = App::new(&event_loop);

    application.run(event_loop);
}
