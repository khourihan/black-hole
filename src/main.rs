use input::InputManager;
use winit::{event_loop::{ControlFlow, EventLoop}, keyboard::KeyCode};

mod gui;
mod state;
mod app;
mod input;

struct Renderer {
    info_window_open: bool,
}

impl app::Renderer for Renderer {
    fn input(&mut self, input: &InputManager) {
        self.info_window_open ^= input.key_pressed(KeyCode::KeyT);
    }

    fn render(&mut self, ctx: &mut app::RenderContext) {

    }

    fn gui(&mut self, ctx: &egui::Context) {
        egui::Window::new("")
            .open(&mut self.info_window_open)
            .show(ctx, |ui| {

            });
    }
}

impl Default for Renderer {
    fn default() -> Self {
        Self {
            info_window_open: true,
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
