use std::mem;

use ash::vk;

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

#[repr(C)]
pub struct MeshImGui {
    pos: glm::Vec2,
    coords: glm::Vec2,
    color: glm::Vec2,
}

impl Vertex for MeshImGui {
    fn get_vertex_attribute_desc() -> Vec<vk::VertexInputAttributeDescription> {
        // TODO check so
        [
            vk::VertexInputAttributeDescription::default().binding(0).location(0).format(vk::Format::R32G32_SFLOAT).offset(0),
            vk::VertexInputAttributeDescription::default()
                .binding(0)
                .location(1)
                .format(vk::Format::R32G32_SFLOAT)
                .offset(memoffset::offset_of!(MeshImGui, pos) as u32),
            vk::VertexInputAttributeDescription::default()
                .binding(0)
                .location(2)
                .format(vk::Format::R8G8B8A8_UNORM)
                .offset(memoffset::offset_of!(MeshImGui, color) as u32),
        ]
        .to_vec()
    }
}
