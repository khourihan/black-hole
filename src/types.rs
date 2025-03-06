use bytemuck::{Pod, Zeroable};

#[repr(C, align(16))]
#[derive(Pod, Zeroable, Clone, Copy)]
pub struct View {
    pub camera: [f32; 16],
    pub position: [f32; 3],
    pub focal_length: f32,
    pub resolution: [u32; 2],
    pub flags: u32,
    pub frames: u32,
}
