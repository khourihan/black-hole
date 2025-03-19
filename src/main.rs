use std::f32::consts::{PI, TAU};

use glam::{EulerRot, Quat, Vec2, Vec3};
use input::InputManager;
use types::BlackHole;
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
    black_hole: BlackHole,
    skybox_enabled: bool,
    disc_enabled: bool,
    info_window_open: bool,
    is_new: bool,
    is_changed: bool,
}

impl app::Renderer for Renderer {
    fn input(&mut self, input: &InputManager) {
        self.info_window_open ^= input.key_pressed(KeyCode::KeyT);

        if input.key_pressed(KeyCode::KeyP) {
            println!("{}", self.position);
            println!("{}", glam::Mat4::from_quat(self.rotation));
        }

        let (sdx, sdy) = input.scroll_diff();
        let (mdx, mdy) = input.mouse_diff();
        let scroll_diff = Vec2::new(sdx, sdy);
        let mouse_diff = Vec2::new(mdx, -mdy);

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

            self.is_changed = true
        } else {
            self.is_changed = false;
        }

        self.is_new = false;
    }

    fn render(&mut self, ctx: &mut app::RenderContext) {
        ctx.set_camera(self.position, self.rotation);
        ctx.set_render_skybox();
        ctx.set_render_disc();
        ctx.focal_length = self.focal_length;
        ctx.reset_frame_count = self.is_changed;
        ctx.black_hole = self.black_hole;

        if self.skybox_enabled {
            ctx.set_render_skybox();
        } else {
            ctx.unset_render_skybox();
        }

        if self.disc_enabled {
            ctx.set_render_disc();
        } else {
            ctx.unset_render_disc();
        }
    }

    fn gui(&mut self, ctx: &egui::Context) {
        egui::Window::new("").open(&mut self.info_window_open).show(ctx, |ui| {
            ui.checkbox(&mut self.skybox_enabled, "Skybox");
            ui.checkbox(&mut self.disc_enabled, "Accretion Disc");

            ui.horizontal(|ui| {
                ui.label("cdist");
                ui.add(egui::DragValue::new(&mut self.black_hole.cdist));
            });

            ui.horizontal(|ui| {
                ui.label("a");
                ui.add(egui::DragValue::new(&mut self.black_hole.a).range(0f32..=1f32));
            });

            ui.horizontal(|ui| {
                ui.label("m");
                ui.add(egui::DragValue::new(&mut self.black_hole.m));
            });

            ui.horizontal(|ui| {
                ui.label("q");
                ui.add(egui::DragValue::new(&mut self.black_hole.q).range(0f32..=128f32));
            });

            ui.horizontal(|ui| {
                ui.label("dt min");
                ui.add(egui::DragValue::new(&mut self.black_hole.dt_min).range(0f32..=10f32));
            });

            ui.horizontal(|ui| {
                ui.label("dt max");
                ui.add(egui::DragValue::new(&mut self.black_hole.dt_max).range(0f32..=100f32));
            });

            ui.horizontal(|ui| {
                ui.label("steps");
                ui.add(egui::DragValue::new(&mut self.black_hole.steps).range(1u32..=1024u32));
            });

            if self.disc_enabled {
                ui.horizontal(|ui| {
                    ui.label("disc radius");
                    ui.add(egui::DragValue::new(&mut self.black_hole.disc_radius).range(0f32..=f32::MAX));
                });

                ui.horizontal(|ui| {
                    ui.label("disc height");
                    ui.add(egui::DragValue::new(&mut self.black_hole.disc_height).range(0f32..=f32::MAX));
                });

                ui.horizontal(|ui| {
                    ui.label("disc falloff (radial:");
                    ui.add(egui::DragValue::new(&mut self.black_hole.disc_falloff.x).range(0f32..=f32::MAX));
                    ui.label(", vertical:");
                    ui.add(egui::DragValue::new(&mut self.black_hole.disc_falloff.y).range(0f32..=f32::MAX));
                    ui.label(")");
                });

                ui.horizontal(|ui| {
                    ui.label("disc emission falloff (radial:");
                    ui.add(egui::DragValue::new(&mut self.black_hole.disc_emission_falloff.x).range(0f32..=f32::MAX));
                    ui.label(", vertical:");
                    ui.add(egui::DragValue::new(&mut self.black_hole.disc_emission_falloff.y).range(0f32..=f32::MAX));
                    ui.label(")");
                });

                ui.horizontal(|ui| {
                    ui.label("disc temperature scale");
                    ui.add(egui::DragValue::new(&mut self.black_hole.disc_temperature_scale).range(0f32..=f32::MAX));
                });

                ui.horizontal(|ui| {
                    ui.label("disc temperature offset");
                    ui.add(egui::DragValue::new(&mut self.black_hole.disc_temperature_offset).range(0f32..=f32::MAX));
                });

                ui.horizontal(|ui| {
                    ui.label("disc radial scale");
                    ui.add(egui::DragValue::new(&mut self.black_hole.disc_radial_scale).range(0f32..=f32::MAX));
                });
            }
        });
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
            is_changed: false,
            black_hole: BlackHole::default(),
            skybox_enabled: true,
            disc_enabled: true,
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
