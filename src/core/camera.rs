#[repr(C)]
#[derive(Clone, Copy)]
pub struct Camera {
    pub model: cgmath::Matrix4<f32>,
    pub view: cgmath::Matrix4<f32>,
    pub proj: cgmath::Matrix4<f32>,
}

unsafe impl bytemuck::Pod for Camera {}
unsafe impl bytemuck::Zeroable for Camera {}
impl Default for Camera {
    fn default() -> Self {
        Camera {
            model: cgmath::Matrix4 {
                x: cgmath::Vector4 { x: 0.0, y: 0.0, z: 0.0, w: 0.0 },
                y: cgmath::Vector4 { x: 0.0, y: 0.0, z: 0.0, w: 0.0 },
                z: cgmath::Vector4 { x: 0.0, y: 0.0, z: 0.0, w: 0.0 },
                w: cgmath::Vector4 { x: 0.0, y: 0.0, z: 0.0, w: 0.0 },
            },
            view: cgmath::Matrix4 {
                x: cgmath::Vector4 { x: 0.0, y: 0.0, z: 0.0, w: 0.0 },
                y: cgmath::Vector4 { x: 0.0, y: 0.0, z: 0.0, w: 0.0 },
                z: cgmath::Vector4 { x: 0.0, y: 0.0, z: 0.0, w: 0.0 },
                w: cgmath::Vector4 { x: 0.0, y: 0.0, z: 0.0, w: 0.0 },
            },
            proj: cgmath::Matrix4 {
                x: cgmath::Vector4 { x: 0.0, y: 0.0, z: 0.0, w: 0.0 },
                y: cgmath::Vector4 { x: 0.0, y: 0.0, z: 0.0, w: 0.0 },
                z: cgmath::Vector4 { x: 0.0, y: 0.0, z: 0.0, w: 0.0 },
                w: cgmath::Vector4 { x: 0.0, y: 0.0, z: 0.0, w: 0.0 },
            },
        }
    }
}
