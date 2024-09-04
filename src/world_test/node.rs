use std::{
    collections::HashMap,
    hash::Hash,
    mem::MaybeUninit,
    sync::{Arc, Mutex},
};

use ash::vk::{self, CommandBuffer};
use glm::Vec3;
use voxelengine::{
    t_thread::{self, MutPtr, ThreadPool},
    vulkan::{
        resource::{BufferIndex, BufferStorage},
        TKQueue,
    },
    TImguiRender,
};
use voxelengine_proc::ImGuiFields;

use crate::world_test::{CHUNK_RESOLUTION, DEPTH, VOXEL_SCALE};

use super::{chunk::ChunkMesh, object::MyVoxel, Vec3Wrapper, CHUNK_SIZE, DISTANCE_THRESHOLD, OCTREE_LENGTH};

use voxelengine::gui::struct_impl::*;
use voxelengine_gui::ImguiId;

fn distance(player: Vec3, center_pos: Vec3) -> f32 {
    let mut dist = 0.0;
    for i in 0..3 {
        dist += (player[i] - center_pos[i]).powf(2.0);
    }

    dist.sqrt()
}

fn is_point_inside_cube(point: Vec3, min_corner: Vec3, max_corner: Vec3) -> bool {
    return (point.x >= min_corner.x && point.x <= max_corner.x) && (point.y >= min_corner.y && point.y <= max_corner.y) && (point.z >= min_corner.z && point.z <= max_corner.z);
}

fn distance_from_cube_to_point(point: Vec3, min_corner: Vec3, max_corner: Vec3) -> f32 {
    let closest_outer_cube = point.clamped(min_corner, max_corner);
    distance(point, closest_outer_cube)
}

#[derive(ImGuiFields, Default)]
pub struct Node {
    center_pos: glm::Vec3,
    size: usize,
    scale: f32,
    #[ignore_field]
    parent: Option<*mut Node>,
    #[ignore_field]
    nodes: Option<[*mut Node; 8]>,
    depth: usize,
    #[ignore_field]
    buffer: BufferIndex,
    mesh: ChunkMesh,
    #[ignore_field]
    chunk_queue: ChunkQueue,
}

impl Node {
    fn new(res: &mut BufferStorage, cmd: vk::CommandBuffer, queue: TKQueue, size: usize, center_pos: glm::Vec3, parent: Option<*mut Node>, scale: f32, depth: usize, player: Vec3, chunk_queue: ChunkQueue) -> Self {
        //TODO generate chunk data
        let half_size = Vec3::new(size as f32 / 2.0, size as f32 / 2.0, size as f32 / 2.0);
        let offset_position = center_pos - Vec3::new(size as f32 / 2.0, size as f32 / 2.0, size as f32 / 2.0);
        unsafe {
            let mut node = Self {
                center_pos,
                mesh: ChunkMesh::default(),
                size,
                parent,
                nodes: None,
                depth,
                buffer: 0,
                scale,
                chunk_queue,
            };

            if depth > 0 && distance_from_cube_to_point(player, center_pos - half_size, center_pos + half_size) < DISTANCE_THRESHOLD {
                node.split(res, cmd, queue, player);
            } else {
                node.mesh = ChunkMesh::new_test(res, queue, offset_position, cmd, depth as u32, node.chunk_queue.clone());
            }

            node
        }
    }

    pub fn raw_new(size: usize, center_pos: glm::Vec3, parent: Option<*mut Node>, scale: f32, depth: usize, chunk_queue: ChunkQueue) -> Self {
        let half_size = Vec3::new(size as f32 / 2.0, size as f32 / 2.0, size as f32 / 2.0);
        let offset_position = center_pos - Vec3::new(size as f32 / 2.0, size as f32 / 2.0, size as f32 / 2.0);
        unsafe {
            Self {
                center_pos,
                mesh: ChunkMesh::default(),
                size,
                parent,
                nodes: None,
                depth,
                buffer: 0,
                scale,
                chunk_queue,
            }
        }
    }

    fn should_split(&mut self, res: &mut BufferStorage, cmd: vk::CommandBuffer, queue: TKQueue, player: Vec3) {
        let offset_position = self.center_pos - Vec3::new(self.size as f32 / 2.0, self.size as f32 / 2.0, self.size as f32 / 2.0);

        let half_size = Vec3::new(self.size as f32 / 2.0, self.size as f32 / 2.0, self.size as f32 / 2.0);
        if self.depth > 0 && distance_from_cube_to_point(player, self.center_pos - half_size, self.center_pos + half_size) < DISTANCE_THRESHOLD {
            self.split(res, cmd, queue, player);
        } else {
            self.mesh = ChunkMesh::new_test(res, queue, offset_position, cmd, self.depth as u32, self.chunk_queue.clone());
        }
    }

    fn split(&mut self, res: &mut BufferStorage, cmd: vk::CommandBuffer, queue: TKQueue, player: Vec3) {
        let half_size = self.size as f32 / 2.0;
        let quarter_size = self.size as f32 / 4.0;
        // TOP
        let front_left_pos = Vec3::new(self.center_pos.x - quarter_size, self.center_pos.y + quarter_size, self.center_pos.z + quarter_size);
        let back_left_pos = Vec3::new(self.center_pos.x - quarter_size, self.center_pos.y + quarter_size, self.center_pos.z - quarter_size);

        let front_right_pos = Vec3::new(self.center_pos.x + quarter_size, self.center_pos.y + quarter_size, self.center_pos.z + quarter_size);
        let back_right_pos = Vec3::new(self.center_pos.x + quarter_size, self.center_pos.y + quarter_size, self.center_pos.z - quarter_size);

        // BOT
        let front_left_bot_pos = Vec3::new(self.center_pos.x - quarter_size, self.center_pos.y - quarter_size, self.center_pos.z + quarter_size);
        let back_left_bot_pos = Vec3::new(self.center_pos.x - quarter_size, self.center_pos.y - quarter_size, self.center_pos.z - quarter_size);

        let front_right_bot_pos = Vec3::new(self.center_pos.x + quarter_size, self.center_pos.y - quarter_size, self.center_pos.z + quarter_size);
        let back_right_bot_pos = Vec3::new(self.center_pos.x + quarter_size, self.center_pos.y - quarter_size, self.center_pos.z - quarter_size);

        let pos = [front_left_pos, back_left_pos, front_right_pos, back_right_pos, front_left_bot_pos, back_left_bot_pos, front_right_bot_pos, back_right_bot_pos];
        self.nodes = Some([std::ptr::null_mut(); 8]);
        for i in 0..8 {
            self.nodes.as_mut().unwrap()[i] = Box::into_raw(Box::new(Node::raw_new(
                half_size as usize,
                pos[i],
                Some(self as *mut Node),
                self.scale / 2.0,
                self.depth - 1,
                self.chunk_queue.clone(),
            )));

            let ptr = MutPtr::new(self.nodes.as_mut().unwrap()[i]);

            ThreadPool::execute(|| {
                let mut ptr = ptr;

                //(*ptr.data).should_split(res, cmd, queue, player)
            });
        }
    }

    fn render_node(&mut self, device: &ash::Device, cmd: vk::CommandBuffer, layout: vk::PipelineLayout, cam_index: u32, g_color_index: u32, player: Vec3) {
        if self.distance_to_object(player) < DISTANCE_THRESHOLD {
            if self.nodes.is_some() {
                for child in 0..8 {
                    unsafe {
                        (*self.nodes.as_mut().unwrap()[child]).render_node(device, cmd, layout, cam_index, g_color_index, player);
                    }
                }
            } else {
                self.mesh.draw(device, cmd, layout, cam_index, g_color_index);
            }
        }
    }

    fn render_imgui(&mut self, ui: &mut imgui::Ui, imgui_id: &mut ImguiId) {
        if self.nodes.is_some() {
            for child in 0..8 {
                unsafe {
                    (*self.nodes.as_mut().unwrap()[child]).render_imgui(ui, imgui_id);
                }
            }
        } else {
            self.display_imgui(ui, imgui_id);
        }
    }

    fn distance_to_object(&self, player: Vec3) -> f32 {
        let half_size = Vec3::new(self.size as f32 / 2.0, self.size as f32 / 2.0, self.size as f32 / 2.0);
        distance_from_cube_to_point(player, self.center_pos - half_size, self.center_pos + half_size)
    }
}

type OctreeOffset = Vec3Wrapper;

impl Vec3Wrapper {
    pub fn new(v: &Vec3) -> Self {
        Self { x: v.x, y: v.y, z: v.z }
    }
}

impl Hash for Vec3Wrapper {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.x.to_bits().hash(state);
        self.y.to_bits().hash(state);
        self.z.to_bits().hash(state);
    }
}

impl Eq for Vec3Wrapper {}
pub(crate) type ChunkQueue = Arc<Mutex<HashMap<Vec3Wrapper, Vec<MyVoxel>>>>;
pub struct World {
    root_indices: HashMap<OctreeOffset, usize>,
    /// adds all the voxels from neighbor chunks
    chunk_add_queue: ChunkQueue,

    roots: Vec<Octree>,
}
impl World {
    pub fn new(res: &mut BufferStorage, cmd: vk::CommandBuffer, queue: TKQueue, player: Vec3) -> Self {
        let octree_player_in = player / Vec3::new(OCTREE_LENGTH, OCTREE_LENGTH, OCTREE_LENGTH);

        // get octrees around player
        let left = octree_player_in + Vec3::new(-1.0, 0.0, 0.0);
        let right = octree_player_in + Vec3::new(1.0, 0.0, 0.0);

        let left_z_up = octree_player_in + Vec3::new(-1.0, 0.0, 1.0);
        let left_z_down = octree_player_in + Vec3::new(-1.0, 0.0, -1.0);

        let right_z_up = octree_player_in + Vec3::new(1.0, 0.0, 1.0);
        let right_z_down = octree_player_in + Vec3::new(1.0, 0.0, -1.0);

        let z_up = octree_player_in + Vec3::new(0.0, 0.0, 1.0);
        let z_down = octree_player_in + Vec3::new(0.0, 0.0, -1.0);

        let z_up_right = octree_player_in + Vec3::new(1.0, 0.0, 1.0);
        let z_up_left = octree_player_in + Vec3::new(-1.0, 0.0, 1.0);

        let z_down_right = octree_player_in + Vec3::new(1.0, 0.0, -1.0);
        let z_down_left = octree_player_in + Vec3::new(-1.0, 0.0, -1.0);

        let all_octrees_pos = [octree_player_in, left, right, left_z_down, left_z_up, right_z_down, right_z_up, z_down, z_up, z_down_left, z_down_right, z_up_left, z_up_right];

        let mut roots = vec![];
        let mut chunk_add_queue: ChunkQueue = Arc::new(Mutex::new(HashMap::new()));
        let mut root_indices = HashMap::new();
        for root_pos in &all_octrees_pos {
            root_indices.insert(Vec3Wrapper::new(root_pos), roots.len());
            roots.push(Octree::new(res, cmd, queue, *root_pos, player, chunk_add_queue.clone()));
        }

        Self { roots, chunk_add_queue, root_indices }
    }
    pub fn draw(&mut self, device: &ash::Device, cmd: vk::CommandBuffer, layout: vk::PipelineLayout, cam_index: u32, g_color_index: u32, player: Vec3) {}

    pub fn render_imgui(&mut self, ui: &mut imgui::Ui, imgui_id: &mut ImguiId) {}
}

pub struct Octree {
    root: Node,
    add_queue: ChunkQueue,
}

impl Octree {
    pub fn new(res: &mut BufferStorage, cmd: vk::CommandBuffer, queue: TKQueue, octree_offset: Vec3, player: Vec3, add_queue: ChunkQueue) -> Octree {
        let actual_offset = octree_offset * Vec3::new(OCTREE_LENGTH, OCTREE_LENGTH, OCTREE_LENGTH);
        unsafe {
            let root = Node::new(
                res,
                cmd,
                queue,
                OCTREE_LENGTH as usize,
                actual_offset,
                None,
                VOXEL_SCALE,
                DEPTH,
                player,
                add_queue.clone(),
            );

            Self { root, add_queue }
        }
    }

    pub fn draw(&mut self, device: &ash::Device, cmd: vk::CommandBuffer, layout: vk::PipelineLayout, cam_index: u32, g_color_index: u32, player: Vec3) {
        self.root.render_node(device, cmd, layout, cam_index, g_color_index, player);
    }

    pub fn render_imgui(&mut self, ui: &mut imgui::Ui, imgui_id: &mut ImguiId) {
        self.root.render_imgui(ui, imgui_id)
    }
}
