use std::mem;

use ash::vk;
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
            VertexBlock {
                pos: glm::Vec3::new(0.5, 0.5, 0.5),
                norm: glm::Vec3::new(1.0, 0.0, 0.0),
                uv: glm::Vec2::new(1.0, 1.0),
                face_index: 0,
            },
            VertexBlock {
                pos: glm::Vec3::new(0.5, 0.5, -0.5),
                norm: glm::Vec3::new(1.0, 0.0, 0.0),
                uv: glm::Vec2::new(0.0, 1.0),
                face_index: 0,
            },
            VertexBlock {
                pos: glm::Vec3::new(0.5, -0.5, -0.5),
                norm: glm::Vec3::new(1.0, 0.0, 0.0),
                uv: glm::Vec2::new(0.0, 0.0),
                face_index: 0,
            },
            VertexBlock {
                pos: glm::Vec3::new(0.5, -0.5, -0.5),
                norm: glm::Vec3::new(1.0, 0.0, 0.0),
                uv: glm::Vec2::new(0.0, 0.0),
                face_index: 0,
            },
            VertexBlock {
                pos: glm::Vec3::new(0.5, -0.5, 0.5),
                norm: glm::Vec3::new(1.0, 0.0, 0.0),
                uv: glm::Vec2::new(1.0, 0.0),
                face_index: 0,
            },
            VertexBlock {
                pos: glm::Vec3::new(0.5, 0.5, 0.5),
                norm: glm::Vec3::new(1.0, 0.0, 0.0),
                uv: glm::Vec2::new(1.0, 1.0),
                face_index: 0,
            },
            // Left face
            VertexBlock {
                pos: glm::Vec3::new(-0.5, 0.5, 0.5),
                norm: glm::Vec3::new(-1.0, 0.0, 0.0),
                uv: glm::Vec2::new(1.0, 0.0),
                face_index: 1,
            },
            VertexBlock {
                pos: glm::Vec3::new(-0.5, -0.5, -0.5),
                norm: glm::Vec3::new(-1.0, 0.0, 0.0),
                uv: glm::Vec2::new(0.0, 1.0),
                face_index: 1,
            },
            VertexBlock {
                pos: glm::Vec3::new(-0.5, 0.5, -0.5),
                norm: glm::Vec3::new(-1.0, 0.0, 0.0),
                uv: glm::Vec2::new(0.0, 0.0),
                face_index: 1,
            },
            VertexBlock {
                pos: glm::Vec3::new(-0.5, -0.5, -0.5),
                norm: glm::Vec3::new(-1.0, 0.0, 0.0),
                uv: glm::Vec2::new(0.0, 1.0),
                face_index: 1,
            },
            VertexBlock {
                pos: glm::Vec3::new(-0.5, 0.5, 0.5),
                norm: glm::Vec3::new(-1.0, 0.0, 0.0),
                uv: glm::Vec2::new(1.0, 0.0),
                face_index: 1,
            },
            VertexBlock {
                pos: glm::Vec3::new(-0.5, -0.5, 0.5),
                norm: glm::Vec3::new(-1.0, 0.0, 0.0),
                uv: glm::Vec2::new(1.0, 1.0),
                face_index: 1,
            },
            // Top face
            VertexBlock {
                pos: glm::Vec3::new(-0.5, 0.5, -0.5),
                norm: glm::Vec3::new(0.0, 1.0, 0.0),
                uv: glm::Vec2::new(0.0, 1.0),
                face_index: 2,
            },
            VertexBlock {
                pos: glm::Vec3::new(0.5, 0.5, -0.5),
                norm: glm::Vec3::new(0.0, 1.0, 0.0),
                uv: glm::Vec2::new(1.0, 1.0),
                face_index: 2,
            },
            VertexBlock {
                pos: glm::Vec3::new(0.5, 0.5, 0.5),
                norm: glm::Vec3::new(0.0, 1.0, 0.0),
                uv: glm::Vec2::new(1.0, 0.0),
                face_index: 2,
            },
            VertexBlock {
                pos: glm::Vec3::new(0.5, 0.5, 0.5),
                norm: glm::Vec3::new(0.0, 1.0, 0.0),
                uv: glm::Vec2::new(1.0, 0.0),
                face_index: 2,
            },
            VertexBlock {
                pos: glm::Vec3::new(-0.5, 0.5, 0.5),
                norm: glm::Vec3::new(0.0, 1.0, 0.0),
                uv: glm::Vec2::new(0.0, 0.0),
                face_index: 2,
            },
            VertexBlock {
                pos: glm::Vec3::new(-0.5, 0.5, -0.5),
                norm: glm::Vec3::new(0.0, 1.0, 0.0),
                uv: glm::Vec2::new(0.0, 1.0),
                face_index: 2,
            },
            // Bottom face
            VertexBlock {
                pos: glm::Vec3::new(-0.5, -0.5, -0.5),
                norm: glm::Vec3::new(0.0, -1.0, 0.0),
                uv: glm::Vec2::new(0.0, 1.0),
                face_index: 3,
            },
            VertexBlock {
                pos: glm::Vec3::new(0.5, -0.5, 0.5),
                norm: glm::Vec3::new(0.0, -1.0, 0.0),
                uv: glm::Vec2::new(1.0, 0.0),
                face_index: 3,
            },
            VertexBlock {
                pos: glm::Vec3::new(0.5, -0.5, -0.5),
                norm: glm::Vec3::new(0.0, -1.0, 0.0),
                uv: glm::Vec2::new(1.0, 1.0),
                face_index: 3,
            },
            VertexBlock {
                pos: glm::Vec3::new(0.5, -0.5, 0.5),
                norm: glm::Vec3::new(0.0, -1.0, 0.0),
                uv: glm::Vec2::new(1.0, 0.0),
                face_index: 3,
            },
            VertexBlock {
                pos: glm::Vec3::new(-0.5, -0.5, -0.5),
                norm: glm::Vec3::new(0.0, -1.0, 0.0),
                uv: glm::Vec2::new(0.0, 1.0),
                face_index: 3,
            },
            VertexBlock {
                pos: glm::Vec3::new(-0.5, -0.5, 0.5),
                norm: glm::Vec3::new(0.0, -1.0, 0.0),
                uv: glm::Vec2::new(0.0, 0.0),
                face_index: 3,
            },
            // Front face
            VertexBlock {
                pos: glm::Vec3::new(-0.5, 0.5, 0.5),
                norm: glm::Vec3::new(0.0, 0.0, 1.0),
                uv: glm::Vec2::new(0.0, 1.0),
                face_index: 4,
            },
            VertexBlock {
                pos: glm::Vec3::new(0.5, 0.5, 0.5),
                norm: glm::Vec3::new(0.0, 0.0, 1.0),
                uv: glm::Vec2::new(1.0, 1.0),
                face_index: 4,
            },
            VertexBlock {
                pos: glm::Vec3::new(0.5, -0.5, 0.5),
                norm: glm::Vec3::new(0.0, 0.0, 1.0),
                uv: glm::Vec2::new(1.0, 0.0),
                face_index: 4,
            },
            VertexBlock {
                pos: glm::Vec3::new(0.5, -0.5, 0.5),
                norm: glm::Vec3::new(0.0, 0.0, 1.0),
                uv: glm::Vec2::new(1.0, 0.0),
                face_index: 4,
            },
            VertexBlock {
                pos: glm::Vec3::new(-0.5, -0.5, 0.5),
                norm: glm::Vec3::new(0.0, 0.0, 1.0),
                uv: glm::Vec2::new(0.0, 0.0),
                face_index: 4,
            },
            VertexBlock {
                pos: glm::Vec3::new(-0.5, 0.5, 0.5),
                norm: glm::Vec3::new(0.0, 0.0, 1.0),
                uv: glm::Vec2::new(0.0, 1.0),
                face_index: 4,
            },
            // Back face
            VertexBlock {
                pos: glm::Vec3::new(-0.5, 0.5, -0.5),
                norm: glm::Vec3::new(0.0, 0.0, -1.0),
                uv: glm::Vec2::new(0.0, 1.0),
                face_index: 5,
            },
            VertexBlock {
                pos: glm::Vec3::new(0.5, 0.5, -0.5),
                norm: glm::Vec3::new(0.0, 0.0, -1.0),
                uv: glm::Vec2::new(1.0, 1.0),
                face_index: 5,
            },
            VertexBlock {
                pos: glm::Vec3::new(0.5, -0.5, -0.5),
                norm: glm::Vec3::new(0.0, 0.0, -1.0),
                uv: glm::Vec2::new(1.0, 0.0),
                face_index: 5,
            },
            VertexBlock {
                pos: glm::Vec3::new(0.5, -0.5, -0.5),
                norm: glm::Vec3::new(0.0, 0.0, -1.0),
                uv: glm::Vec2::new(1.0, 0.0),
                face_index: 5,
            },
            VertexBlock {
                pos: glm::Vec3::new(-0.5, -0.5, -0.5),
                norm: glm::Vec3::new(0.0, 0.0, -1.0),
                uv: glm::Vec2::new(0.0, 0.0),
                face_index: 5,
            },
            VertexBlock {
                pos: glm::Vec3::new(-0.5, 0.5, -0.5),
                norm: glm::Vec3::new(0.0, 0.0, -1.0),
                uv: glm::Vec2::new(0.0, 1.0),
                face_index: 5,
            },
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
