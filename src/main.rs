use glam::{Mat4, Vec3, Vec4};
use render::Renderer;

mod render;
mod state;
mod types;

fn main() {
    let (width, height) = (1920, 1080);

    let position = Vec3::new(1.5139699, -16.080126, 2.293509);
    let camera = Mat4::from_cols(
        Vec4::new(0.98061466, 0.000000007450581, 0.19594617, 0.0),
        Vec4::new(0.1948367, -0.10626423, -0.9750625, 0.0),
        Vec4::new(0.020822048, 0.99433804, -0.1042043, 0.0),
        Vec4::new(0.0, 0.0, 0.0, 1.0),
    );

    // let position = Vec3::new(2.3167892, -11.112907, 0.96180904);
    // let camera = Mat4::from_cols(
    //     Vec4::new(0.9854079, 0.0, 0.1702095, 0.0),
    //     Vec4::new(0.17017217, 0.020942032, -0.9851917, 0.0),
    //     Vec4::new(-0.0035645217, 0.99978065, 0.02063644, 0.0),
    //     Vec4::new(0.0, 0.0, 0.0, 1.0),
    // );

    let mut renderer = Renderer::new(width, height);
    renderer.set_render_skybox(false);
    renderer.set_render_disc(true);
    renderer.set_frames(16);
    renderer.set_view(camera, position, 1.5);
    renderer.render();

    let data: Vec<_> = renderer
        .target()
        .chunks(4)
        .map(|bytes| {
            let b = <[u8; 4]>::try_from(bytes).unwrap();
            f32::from_ne_bytes(b)
        })
        .collect();

    let mut image: image::ImageBuffer<image::Rgba<u8>, Vec<u8>> = image::ImageBuffer::new(width, height);

    for (i, pixel) in data.chunks(4).enumerate() {
        let x = i as u32 % width;
        let y = i as u32 / width;

        let px_unorm: Vec<u8> = pixel.iter().map(|&c| (c.clamp(0.0, 1.0) * 255.0) as u8).collect();
        image.put_pixel(
            x,
            (height - 1) - y,
            image::Rgba(px_unorm.as_slice().try_into().unwrap()),
        );
    }

    image.save("black-hole.png").unwrap();
}
