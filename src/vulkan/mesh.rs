use std::mem;

use ash::vk;
use glm::{Vec2, Vec3};
use imgui::draw_list;

pub trait Vertex {
    fn get_vertex_attribute_desc() -> Vec<vk::VertexInputAttributeDescription>;
    fn get_vertex_binding_desc() -> Vec<vk::VertexInputBindingDescription>
    where
        Self: Sized,
    {
        [vk::VertexInputBindingDescription::default()
            .binding(0)
            .stride(mem::size_of::<Self>() as u32)
            .input_rate(vk::VertexInputRate::VERTEX)]
        .to_vec()
    }
}
#[repr(C, align(16))]
pub struct VertexBlock {
    pos: glm::Vec3,
    norm: glm::Vec3,
    uv: glm::Vec2,
    face_index: u32,
}

impl VertexBlock {
    pub fn new(pos: glm::Vec3, norm: Vec3, uv: Vec2, face_index: u32) -> Self {
        Self { pos, norm, uv, face_index }
    }
}

impl Default for VertexBlock {
    fn default() -> Self {
        Self {
            pos: Default::default(),
            face_index: Default::default(),
            norm: Default::default(),
            uv: Default::default(),
        }
    }
}

impl Vertex for VertexBlock {
    fn get_vertex_attribute_desc() -> Vec<vk::VertexInputAttributeDescription> {
        [
            vk::VertexInputAttributeDescription::default()
                .binding(0)
                .location(0)
                .format(vk::Format::R32G32B32_SFLOAT)
                .offset(0),
            vk::VertexInputAttributeDescription::default()
                .binding(0)
                .location(1)
                .format(vk::Format::R32G32B32_SFLOAT)
                .offset(memoffset::offset_of!(VertexBlock, norm) as u32),
            vk::VertexInputAttributeDescription::default()
                .binding(0)
                .location(2)
                .format(vk::Format::R32G32_SFLOAT)
                .offset(memoffset::offset_of!(VertexBlock, uv) as u32),
            vk::VertexInputAttributeDescription::default()
                .binding(0)
                .location(3)
                .format(vk::Format::R32_UINT)
                .offset(memoffset::offset_of!(VertexBlock, face_index) as u32),
        ]
        .to_vec()
    }
}

impl VertexBlock {
    pub fn get_mesh() -> Vec<VertexBlock> {
        vec![
            // Right face
            VertexBlock::new(glm::Vec3::new(0.5, 0.5, 0.5), glm::Vec3::new(1.0, 0.0, 0.0), glm::Vec2::new(1.0, 1.0), 0),
            VertexBlock::new(glm::Vec3::new(0.5, 0.5, -0.5), glm::Vec3::new(1.0, 0.0, 0.0), glm::Vec2::new(0.0, 1.0), 0),
            VertexBlock::new(glm::Vec3::new(0.5, -0.5, -0.5), glm::Vec3::new(1.0, 0.0, 0.0), glm::Vec2::new(0.0, 0.0), 0),
            VertexBlock::new(glm::Vec3::new(0.5, -0.5, -0.5), glm::Vec3::new(1.0, 0.0, 0.0), glm::Vec2::new(0.0, 0.0), 0),
            VertexBlock::new(glm::Vec3::new(0.5, -0.5, 0.5), glm::Vec3::new(1.0, 0.0, 0.0), glm::Vec2::new(1.0, 0.0), 0),
            VertexBlock::new(glm::Vec3::new(-0.5, -0.5, 0.5), glm::Vec3::new(-1.0, 0.0, 0.0), glm::Vec2::new(1.0, 1.0), 1),
            // Left face
            VertexBlock::new(glm::Vec3::new(-0.5, 0.5, 0.5), glm::Vec3::new(-1.0, 0.0, 0.0), glm::Vec2::new(1.0, 0.0), 1),
            VertexBlock::new(glm::Vec3::new(-0.5, -0.5, -0.5), glm::Vec3::new(-1.0, 0.0, 0.0), glm::Vec2::new(0.0, 1.0), 1),
            VertexBlock::new(glm::Vec3::new(-0.5, 0.5, -0.5), glm::Vec3::new(-1.0, 0.0, 0.0), glm::Vec2::new(0.0, 0.0), 1),
            VertexBlock::new(glm::Vec3::new(-0.5, -0.5, -0.5), glm::Vec3::new(-1.0, 0.0, 0.0), glm::Vec2::new(0.0, 1.0), 1),
            VertexBlock::new(glm::Vec3::new(-0.5, 0.5, 0.5), glm::Vec3::new(-1.0, 0.0, 0.0), glm::Vec2::new(1.0, 0.0), 1),
            VertexBlock::new(glm::Vec3::new(-0.5, -0.5, 0.5), glm::Vec3::new(-1.0, 0.0, 0.0), glm::Vec2::new(1.0, 1.0), 1),
            // Top face
            VertexBlock::new(glm::Vec3::new(-0.5, 0.5, -0.5), glm::Vec3::new(0.0, 1.0, 0.0), glm::Vec2::new(0.0, 1.0), 2),
            VertexBlock::new(glm::Vec3::new(0.5, 0.5, -0.5), glm::Vec3::new(0.0, 1.0, 0.0), glm::Vec2::new(1.0, 1.0), 2),
            VertexBlock::new(glm::Vec3::new(0.5, 0.5, 0.5), glm::Vec3::new(0.0, 1.0, 0.0), glm::Vec2::new(1.0, 0.0), 2),
            VertexBlock::new(glm::Vec3::new(0.5, 0.5, 0.5), glm::Vec3::new(0.0, 1.0, 0.0), glm::Vec2::new(1.0, 0.0), 2),
            VertexBlock::new(glm::Vec3::new(-0.5, 0.5, 0.5), glm::Vec3::new(0.0, 1.0, 0.0), glm::Vec2::new(0.0, 0.0), 2),
            VertexBlock::new(glm::Vec3::new(-0.5, 0.5, -0.5), glm::Vec3::new(0.0, 1.0, 0.0), glm::Vec2::new(0.0, 1.0), 2),
            // Bottom face
            VertexBlock::new(glm::Vec3::new(-0.5, -0.5, -0.5), glm::Vec3::new(0.0, -1.0, 0.0), glm::Vec2::new(0.0, 1.0), 3),
            VertexBlock::new(glm::Vec3::new(0.5, -0.5, 0.5), glm::Vec3::new(0.0, -1.0, 0.0), glm::Vec2::new(1.0, 0.0), 3),
            VertexBlock::new(glm::Vec3::new(0.5, -0.5, -0.5), glm::Vec3::new(0.0, -1.0, 0.0), glm::Vec2::new(1.0, 1.0), 3),
            VertexBlock::new(glm::Vec3::new(0.5, -0.5, 0.5), glm::Vec3::new(0.0, -1.0, 0.0), glm::Vec2::new(1.0, 0.0), 3),
            VertexBlock::new(glm::Vec3::new(-0.5, -0.5, -0.5), glm::Vec3::new(0.0, -1.0, 0.0), glm::Vec2::new(0.0, 1.0), 3),
            VertexBlock::new(glm::Vec3::new(-0.5, -0.5, 0.5), glm::Vec3::new(0.0, -1.0, 0.0), glm::Vec2::new(0.0, 0.0), 3),
            // Front face
            VertexBlock::new(glm::Vec3::new(-0.5, -0.5, 0.5), glm::Vec3::new(0.0, 0.0, 1.0), glm::Vec2::new(1.0, 1.0), 5),
            VertexBlock::new(glm::Vec3::new(0.5, 0.5, 0.5), glm::Vec3::new(0.0, 0.0, 1.0), glm::Vec2::new(0.0, 0.0), 5),
            VertexBlock::new(glm::Vec3::new(0.5, -0.5, 0.5), glm::Vec3::new(0.0, 0.0, 1.0), glm::Vec2::new(0.0, 1.0), 5),
            VertexBlock::new(glm::Vec3::new(0.5, 0.5, 0.5), glm::Vec3::new(0.0, 0.0, 1.0), glm::Vec2::new(0.0, 0.0), 5),
            VertexBlock::new(glm::Vec3::new(-0.5, -0.5, 0.5), glm::Vec3::new(0.0, 0.0, 1.0), glm::Vec2::new(1.0, 1.0), 5),
            VertexBlock::new(glm::Vec3::new(-0.5, 0.5, 0.5), glm::Vec3::new(0.0, 0.0, 1.0), glm::Vec2::new(1.0, 0.0), 5),
            // Back face
            VertexBlock::new(glm::Vec3::new(-0.5, -0.5, -0.5), glm::Vec3::new(0.0, 0.0, -1.0), glm::Vec2::new(1.0, 1.0), 4),
            VertexBlock::new(glm::Vec3::new(0.5, -0.5, -0.5), glm::Vec3::new(0.0, 0.0, -1.0), glm::Vec2::new(0.0, 1.0), 4),
            VertexBlock::new(glm::Vec3::new(0.5, 0.5, -0.5), glm::Vec3::new(0.0, 0.0, -1.0), glm::Vec2::new(0.0, 0.0), 4),
            VertexBlock::new(glm::Vec3::new(0.5, 0.5, -0.5), glm::Vec3::new(0.0, 0.0, -1.0), glm::Vec2::new(0.0, 0.0), 4),
            VertexBlock::new(glm::Vec3::new(-0.5, 0.5, -0.5), glm::Vec3::new(0.0, 0.0, -1.0), glm::Vec2::new(1.0, 0.0), 4),
            VertexBlock::new(glm::Vec3::new(-0.5, -0.5, -0.5), glm::Vec3::new(0.0, 0.0, -1.0), glm::Vec2::new(1.0, 1.0), 4),
        ]
    }
}

#[repr(C)]
pub struct MeshImGui {
    pos: glm::Vec2,
    coords: glm::Vec2,
    color: (u8, u8, u8, u8),
}

impl MeshImGui {
    pub fn create_mesh(draw_data: &imgui::DrawData) -> (Vec<imgui::DrawVert>, Vec<u16>) {
        let mut vertices = Vec::with_capacity(draw_data.total_vtx_count as usize);
        let mut indices = Vec::with_capacity(draw_data.total_idx_count as usize);

        for draw_list in draw_data.draw_lists() {
            vertices.extend_from_slice(draw_list.vtx_buffer());
            indices.extend_from_slice(draw_list.idx_buffer());
        }
        (vertices, indices)
    }
}

impl Vertex for MeshImGui {
    fn get_vertex_attribute_desc() -> Vec<vk::VertexInputAttributeDescription> {
        [
            vk::VertexInputAttributeDescription::default()
                .binding(0)
                .location(0)
                .format(vk::Format::R32G32_SFLOAT)
                .offset(0),
            vk::VertexInputAttributeDescription::default()
                .binding(0)
                .location(1)
                .format(vk::Format::R32G32_SFLOAT)
                .offset(memoffset::offset_of!(MeshImGui, coords) as u32),
            vk::VertexInputAttributeDescription::default()
                .binding(0)
                .location(2)
                .format(vk::Format::R8G8B8A8_UNORM)
                .offset(memoffset::offset_of!(MeshImGui, color) as u32),
        ]
        .to_vec()
    }
}
