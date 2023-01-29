use std::mem::size_of;

use ash::vk;
use memoffset::offset_of;
use serde::{Serialize, Deserialize};

#[repr(C)]
#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct Vertex {
    pub color: cgmath::Vector3<f32>,
    pub pos: cgmath::Vector2<f32>,
}

impl Vertex {
    pub fn get_descriptions() -> (
        vk::VertexInputBindingDescription,
        [vk::VertexInputAttributeDescription; 2],
    ) {
        let binding_description = vk::VertexInputBindingDescription::builder()
            .stride(size_of::<Vertex>() as u32)
            .binding(0)
            .input_rate(vk::VertexInputRate::VERTEX);

        let position_attrib = vk::VertexInputAttributeDescription::builder()
            .binding(0)
            .location(0)
            .format(vk::Format::R32G32_SFLOAT)
            .offset(offset_of!(Vertex, pos) as u32);

        let uv_attrib = vk::VertexInputAttributeDescription::builder()
            .binding(0)
            .location(1)
            .format(vk::Format::R32G32B32_SFLOAT)
            .offset(offset_of!(Vertex, color) as u32);
        (*binding_description, [*position_attrib, *uv_attrib])
    }
}

unsafe impl bytemuck::Pod for Vertex {}
unsafe impl bytemuck::Zeroable for Vertex {}
