#[repr(C)]
#[derive(Clone, Copy)]
pub struct Camera {
    pub model: cgmath::Matrix4<f32>,
    pub view: cgmath::Matrix4<f32>,
    pub proj: cgmath::Matrix4<f32>,

}