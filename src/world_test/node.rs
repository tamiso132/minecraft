use glm::Vec3;
use voxelengine::vulkan::resource::{BufferIndex, BufferStorage};

use crate::world_test::{CHUNK_RESOLUTION, DEPTH, VOXEL_SCALE};

use super::chunk::ChunkMesh;

pub struct Node {
    pos: glm::Vec3,
    size: usize,
    parent: *mut Node,
    nodes: [*mut Node; 4],
    depth: usize,
    buffer: BufferIndex,

    mesh: ChunkMesh,
}

impl Node {
    fn new(res: &mut BufferStorage, size: usize, center_pos: glm::Vec3, parent: *mut Node, depth: usize) -> Self {
        //TODO generate chunk data
        let mesh = ChunkMesh::new(center_pos, depth);
        Self { pos: center_pos, mesh, size, parent, nodes: [std::ptr::null_mut(); 4], depth, buffer: 0 }
    }

    fn render_node() {}
}

impl Node {}

pub struct Octree {
    root: Node,
}

impl Octree {
    pub fn new(res: &mut BufferStorage, pos: Vec3) -> Octree {
        let size_in_voxels = 2usize.pow(DEPTH as u32 - 1) * (CHUNK_RESOLUTION);
        println!("size in voxels: {}", size_in_voxels);
        let size = (CHUNK_RESOLUTION as f32) * VOXEL_SCALE;

        let root = Node::new(res, size_in_voxels, Vec3::new(pos.x, pos.y, pos.z), std::ptr::null_mut(), DEPTH);

        Self { root }
    }
}
