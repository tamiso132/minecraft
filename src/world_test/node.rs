use std::mem::MaybeUninit;

use ash::vk::{self, CommandBuffer};
use glm::Vec3;
use voxelengine::vulkan::{
    resource::{BufferIndex, BufferStorage},
    TKQueue,
};

use crate::world_test::{CHUNK_RESOLUTION, DEPTH, VOXEL_SCALE};

use super::chunk::ChunkMesh;

pub struct Node {
    center_pos: glm::Vec3,
    size: usize,
    parent: *mut Node,
    nodes: [*mut Node; 8],
    depth: usize,
    buffer: BufferIndex,

    mesh: ChunkMesh,
}

impl Node {
    fn new(res: &mut BufferStorage, cmd: vk::CommandBuffer, queue: TKQueue, size: usize, center_pos: glm::Vec3, parent: *mut Node, depth: usize) -> Self {
        //TODO generate chunk data
        let mesh = ChunkMesh::new_test(res, queue, center_pos, cmd, depth as u32);
        
        unsafe {
            Self {
                center_pos,
                mesh,
                size,
                parent,
                nodes: [MaybeUninit::<*mut Node>::zeroed().assume_init(); 8],
                depth,
                buffer: 0,
            }
        }
    }

    fn split(&mut self, res: &mut BufferStorage, cmd: vk::CommandBuffer, queue: TKQueue) {
        let half_size = self.size as f32 / 2.0;
        let quarter_size = self.size as f32 / 4.0;
        // TOP
        let front_left_pos = Vec3::new(self.center_pos.x - quarter_size, self.center_pos.y + quarter_size, self.center_pos.z + quarter_size);
        let back_left_pos = Vec3::new(self.center_pos.x - quarter_size, self.center_pos.y + quarter_size, self.center_pos.z - quarter_size);

        let front_right_pos = Vec3::new(self.center_pos.x + quarter_size, self.center_pos.y + quarter_size, self.center_pos.z + quarter_size);
        let back_right_pos = Vec3::new(self.center_pos.x + quarter_size, self.center_pos.y + quarter_size, self.center_pos.z - quarter_size);

        // BOT
        let front_left_bot_pos = Vec3::new(self.center_pos.x - quarter_size, self.center_pos.y - half_size, self.center_pos.z + half_size);
        let back_left_bot_pos = Vec3::new(self.center_pos.x - quarter_size, self.center_pos.y - quarter_size, self.center_pos.z - quarter_size);

        let front_right_bot_pos = Vec3::new(self.center_pos.x + quarter_size, self.center_pos.y - quarter_size, self.center_pos.z + quarter_size);
        let back_right_bot_pos = Vec3::new(self.center_pos.x + quarter_size, self.center_pos.y - quarter_size, self.center_pos.z - quarter_size);

        Box::new(10);

        let pos = [front_left_pos, back_left_pos, front_right_pos, back_right_pos, front_left_bot_pos, back_left_bot_pos, front_right_bot_pos, back_right_bot_pos];

        for i in 0..8 {
            self.nodes[0] = Box::into_raw(Box::new(Node::new(res, cmd, queue, half_size as usize, pos[i], self as *mut Node, self.depth - 1)));
        }
    }

    fn render_node(&mut self, device: &ash::Device, cmd: vk::CommandBuffer, layout: vk::PipelineLayout, cam_index: u32, g_color_index: u32) {
        if self.nodes[0].is_null() {
            // DRAW THIS
            self.mesh.draw(device, cmd, layout, cam_index, g_color_index);
        } else {
            // draw children
            for child in self.nodes {
                unsafe {
                    (*child).render_node(device, cmd, layout, cam_index, g_color_index);
                }
            }
        }
    }
}

impl Node {}

pub struct Octree {
    root: Node,
}

impl Octree {
    pub fn new(res: &mut BufferStorage, cmd: vk::CommandBuffer, queue: TKQueue, pos: Vec3) -> Octree {
        let size_in_voxels = 2usize.pow(DEPTH as u32 - 1) * (CHUNK_RESOLUTION);
        let size = (CHUNK_RESOLUTION as f32) * VOXEL_SCALE;
        unsafe {
            let root = Node::new(
                res,
                cmd,
                queue,
                size_in_voxels,
                Vec3::new(pos.x, pos.y, pos.z),
                MaybeUninit::<*mut Node>::zeroed().assume_init(),
                DEPTH,
            );

            Self { root }
        }
    }

    pub fn draw(&mut self, device: &ash::Device, cmd: vk::CommandBuffer, layout: vk::PipelineLayout, cam_index: u32, g_color_index: u32){
        self.root.render_node(device, cmd, layout, cam_index, g_color_index);
    }
}
