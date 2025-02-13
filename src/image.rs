use glam::UVec2;

pub struct Image<const N: usize, T: Copy> {
    pixels: Vec<[T; N]>,
    resolution: UVec2,
}

impl<const N: usize, T: Copy> Image<N, T> {
    pub fn new_fill(resolution: UVec2, pixel: [T; N]) -> Image<N, T> {
        Image::<N, T> {
            pixels: vec![pixel; (resolution.x * resolution.y) as usize],
            resolution,
        }
    }

    #[inline]
    pub fn set(&mut self, x: u32, y: u32, pixel: [T; N]) {
        self.pixels[(y * self.resolution.x + x) as usize] = pixel;
    }

    #[inline]
    pub fn get(&self, x: u32, y: u32) -> [T; N] {
        self.pixels[(y * self.resolution.x + x) as usize]
    }
}
