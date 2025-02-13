use glam::{Mat4, UVec2, Vec2, Vec3, Vec4, Vec4Swizzles};
use image::Image;

mod image;

struct Camera {
    position: Vec3,
    rotation: Mat4,
    focal_length: f32,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            position: Vec3::new(0.0, 0.0, -30.0),
            rotation: Mat4::IDENTITY,
            focal_length: 1.5,
        }
    }
}

const A: f32 = 0.8;
const M: f32 = 1.0;
const Q: f32 = 0.0;
const EPS: f32 = 0.01;
const TIMESTEP: f32 = 0.15;

fn r_from_coords(x: Vec4) -> f32 {
    let p = x.yzw();
    let rho = p.dot(p) - A * A;
    let r2 = 0.5 * (rho + f32::sqrt(rho * rho + 4.0 * A * A * p.z * p.z));
    f32::sqrt(r2)
}

fn metric(x: Vec4) -> Mat4 {
    let p = x.yzw();
    let r = r_from_coords(x);
    let r2 = r * r;
    let k = Vec4::new(
        1.0,
        (r * p.x + A * p.y) / (r2 + A * A),
        (r * p.y - A * p.x) / (r2 + A * A),
        p.z / r,
    );
    let f = r2 * (2.0 * M * r - Q * Q) / (r2 * r2 + A * A * p.z * p.z);
    f * Mat4::from_cols(k.x * k, k.y * k, k.z * k, k.w * k) + Mat4::from_diagonal(Vec4::new(-1.0, 1.0, 1.0, 1.0))
}

fn hamiltonian(x: Vec4, p: Vec4) -> f32 {
    let g_inv = metric(x).inverse();
    0.5 * (g_inv * p).dot(p)
}

fn hamiltonian_gradient(x: Vec4, p: Vec4) -> Vec4 {
    (Vec4::new(
        hamiltonian(x + Vec4::new(EPS, 0.0, 0.0, 0.0), p),
        hamiltonian(x + Vec4::new(0.0, EPS, 0.0, 0.0), p),
        hamiltonian(x + Vec4::new(0.0, 0.0, EPS, 0.0), p),
        hamiltonian(x + Vec4::new(0.0, 0.0, 0.0, EPS), p),
    ) - hamiltonian(x, p))
        / EPS
}

fn trace(ro: Vec3, rd: Vec3) -> Vec3 {
    let time = 0.0;
    let mut x = Vec4::new(time, ro.x, ro.y, ro.z);
    let mut p = metric(x) * Vec4::new(1.0, rd.x, rd.y, rd.z);
    let mut captured = false;

    let steps = 256;
    for i in 0..steps {
        p -= TIMESTEP * hamiltonian_gradient(x, p);
        x += TIMESTEP * metric(x).inverse() * p;

        let r = r_from_coords(x);
        captured = r < 1.0 + f32::sqrt(1.0 - A * A);
        if captured {
            break;
        }
    }

    let dxdt = metric(x).inverse() * p;
    let dir = dxdt.yzw().normalize();
    let pos = x.yzw();
    let time = x.x;

    dir * if captured { 0.0 } else { 1.0 }
}

const IMAGE_SIZE: UVec2 = UVec2::new(512, 512);

fn main() {
    let mut image = Image::new_fill(IMAGE_SIZE, [0.0; 3]);
    let camera = Camera::default();

    for x in 0..IMAGE_SIZE.x {
        for y in 0..IMAGE_SIZE.y {
            let frag_coord = Vec2::new(x as f32, y as f32);
            let p = (2.0 * frag_coord - IMAGE_SIZE.as_vec2()) / IMAGE_SIZE.y as f32;

            let rd = (camera.rotation * Vec3::new(p.x, p.y, camera.focal_length).normalize().extend(1.0))
                .normalize()
                .xyz();

            let ro = camera.position;

            let col = trace(ro, rd);

            image.set(x, y, col.to_array())
        }
    }

    exr::prelude::write_rgba_file(
        "black-hole.exr",
        IMAGE_SIZE.x as usize,
        IMAGE_SIZE.y as usize,
        |x, y| {
            let c = image.get(x as u32, y as u32);
            (c[0], c[1], c[2], 1.0)
        },
    )
    .unwrap();
}
