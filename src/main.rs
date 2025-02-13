use std::f32::consts::{PI, TAU};

use glam::{EulerRot, Quat, Vec2, Vec3};
use input::InputManager;
use winit::{
    event::MouseButton,
    event_loop::{ControlFlow, EventLoop},
    keyboard::KeyCode,
};

mod app;
mod gui;
mod input;
mod state;
mod types;

struct Renderer {
    position: Vec3,
    center: Vec3,
    radius: f32,
    yaw: f32,
    pitch: f32,
    rotation: Quat,
    focal_length: f32,
    info_window_open: bool,
    is_new: bool,
}

impl app::Renderer for Renderer {
    fn input(&mut self, input: &InputManager) {
        self.info_window_open ^= input.key_pressed(KeyCode::KeyT);

        let (sdx, sdy) = input.scroll_diff();
        let (mdx, mdy) = input.mouse_diff();
        let scroll_diff = Vec2::new(sdx, sdy);
        let mouse_diff = Vec2::new(mdx, mdy);

        let mut total_pan = Vec2::ZERO;
        let mut total_orbit = Vec2::ZERO;
        let mut total_zoom = Vec2::ZERO;

        if input.mouse_held(MouseButton::Right) {
            total_pan -= mouse_diff * 0.001;
        }

        if input.mouse_held(MouseButton::Left) {
            total_orbit -= mouse_diff * 0.0017453293; // 0.1°
        }

        total_zoom -= scroll_diff * 0.01;

        let mut any = false;

        if total_zoom != Vec2::ZERO {
            any = true;
            self.radius *= (-total_zoom.y).exp();
        }

        if total_orbit != Vec2::ZERO {
            any = true;
            self.yaw += total_orbit.x;
            self.pitch += total_orbit.y;

            if self.yaw > PI {
                self.yaw -= TAU;
            }
            if self.yaw < -PI {
                self.yaw += TAU;
            }
            if self.pitch > PI {
                self.pitch -= TAU;
            }
            if self.pitch < -PI {
                self.pitch += TAU;
            }
        }

        if total_pan != Vec2::ZERO {
            any = true;
            let radius = self.radius;
            self.center += (self.rotation * Vec3::X) * total_pan.x * radius;
            self.center += (self.rotation * Vec3::Y) * total_pan.y * radius;
        }

        if any || self.is_new {
            self.rotation = Quat::from_euler(EulerRot::YXZ, self.yaw, self.pitch, 0.0);
            self.position = self.center + (self.rotation * Vec3::NEG_Z) * self.radius;
        }

        self.is_new = false;
    }

    fn render(&mut self, ctx: &mut app::RenderContext) {
        ctx.set_camera(self.position, self.rotation);
        ctx.focal_length = self.focal_length;
    }

    fn gui(&mut self, ctx: &egui::Context) {
        // egui::Window::new("")
        //     .open(&mut self.info_window_open)
        //     .show(ctx, |ui| {
        //
        //     });
    }
}

impl Default for Renderer {
    fn default() -> Self {
        Self {
            center: Vec3::ZERO,
            radius: 30.0,
            pitch: 0.0,
            yaw: 0.0,
            rotation: Quat::IDENTITY,
            position: Vec3::new(0.0, 0.0, 0.0),
            focal_length: 1.5,
            info_window_open: true,
            is_new: true,
        }
    }
}

fn main() {
    let event_loop = EventLoop::new().unwrap();

    event_loop.set_control_flow(ControlFlow::Poll);

    let renderer = Renderer::default();

    let mut app = app::App::new(renderer);

    event_loop.run_app(&mut app).expect("failed to run app.");
}
