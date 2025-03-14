use glam::{Mat4, Vec3, Vec4};
use render::Renderer;

mod render;
mod state;
mod types;

fn step(edge: Vec3, x: Vec3) -> Vec3 {
    Vec3::new(
        if x.x < edge.x { 0.0 } else { 1.0 },
        if x.y < edge.y { 0.0 } else { 1.0 },
        if x.z < edge.z { 0.0 } else { 1.0 },
    )
}

fn smoothstep(edge0: Vec3, edge1: Vec3, mut x: Vec3) -> Vec3 {
    x = Vec3::clamp((x - edge0) / (edge1 - edge0), Vec3::ZERO, Vec3::ONE);
    x * x * (3.0 - 2.0 * x)
}

fn mix(a: Vec3, b: Vec3, t: Vec3) -> Vec3 {
    a * (1.0 - t) + b * t
}

fn linear_to_srgb(c: Vec3) -> Vec3 {
    Vec3::clamp(
        mix(
            1.055 * Vec3::powf(c, 1.0 / 2.4) - Vec3::splat(0.055),
            c * 12.92,
            step(c, Vec3::splat(0.0031308)),
        ),
        Vec3::ZERO,
        Vec3::ONE,
    )
}

fn tonemap(color: Vec3) -> Vec3 {
    smoothstep(Vec3::ZERO, Vec3::ONE, 1.0 - Vec3::exp(-color * 1.0))
}

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

    let mut image: image::ImageBuffer<image::Rgb<u8>, Vec<u8>> = image::ImageBuffer::new(width, height);

    for (i, pixel) in data.chunks(4).enumerate() {
        let x = i as u32 % width;
        let y = i as u32 / width;

        let mut col = Vec3::new(pixel[0], pixel[1], pixel[2]);
        col.x /= pixel[3];
        col.y /= pixel[3];
        col.z /= pixel[3];

        col = linear_to_srgb(tonemap(col));

        let col_unorm = (col.clamp(Vec3::ZERO, Vec3::ONE) * 255.0).as_u8vec3();
        image.put_pixel(x, (height - 1) - y, image::Rgb(col_unorm.to_array()));
    }

    image.save("black-hole.png").unwrap();
}
