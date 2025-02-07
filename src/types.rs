use bytemuck::{Pod, Zeroable};
use glam::Mat3;

#[repr(C)]
#[derive(Pod, Zeroable, Copy, Clone)]
pub struct View {
    pub camera: Mat3,
    pub focal_length: f32,
}
