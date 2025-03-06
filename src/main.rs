use glam::{Mat4, Vec3};
use render::Renderer;

mod render;
mod state;
mod types;

fn main() {
    let (width, height) = (1920, 1080);

    let eye = Vec3::new(0.0, 0.0, 30.0);
    let target = Vec3::ZERO;

    let mut renderer = Renderer::new(width, height);
    renderer.set_render_skybox(false);
    renderer.set_render_disc(true);
    renderer.set_frames(16);
    renderer.set_view(Mat4::look_at_lh(Vec3::ZERO, target - eye, Vec3::Y), eye, 1.5);
    renderer.render();

    let data = renderer.target();
    let mut image: image::ImageBuffer<image::Rgba<u8>, Vec<u8>> = image::ImageBuffer::new(width, height);

    for (i, pixel) in data.chunks(4).enumerate() {
        let x = i as u32 % width;
        let y = i as u32 / width;

        image.put_pixel(x, y, image::Rgba(pixel.try_into().unwrap()));
    }

    image.save("black-hole.png").unwrap();
}
