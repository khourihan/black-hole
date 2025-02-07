use std::sync::Arc;

use egui_wgpu::ScreenDescriptor;
use glam::{Mat3, Vec3};
use winit::{application::ApplicationHandler, dpi::PhysicalSize, event::WindowEvent, event_loop::ActiveEventLoop, window::Window};

use crate::{input::InputManager, state::State};

pub struct RenderContext {
    camera: Mat3,
    pub focal_length: f32,
}

impl RenderContext {
    pub fn new() -> Self {
        Self {
            camera: Mat3::ZERO,
            focal_length: 1.5,
        }
    }

    pub fn set_camera(&mut self, position: Vec3, target: Vec3, up: Vec3) {
        let front = (target - position).normalize();
        let right = front.cross(up).normalize();
        let up = right.cross(front).normalize();

        self.camera = Mat3::from_cols(right, up, front)
    }
}

pub trait Renderer {
    fn input(&mut self, input: &InputManager);

    fn render(&mut self, ctx: &mut RenderContext);

    fn gui(&mut self, ctx: &egui::Context);
}

pub struct App<R: Renderer> {
    width: u32,
    height: u32,
    instance: wgpu::Instance,
    state: Option<State>,
    window: Option<Arc<Window>>,
    input: InputManager,
    render_ctx: RenderContext,
    renderer: R,
}

impl<R: Renderer> App<R> {
    pub fn new(renderer: R) -> Self {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::default());

        Self {
            width: 1360,
            height: 768,
            instance,
            state: None,
            window: None,
            input: InputManager::new(),
            render_ctx: RenderContext::new(),
            renderer,
        }
    }

    async fn set_window(&mut self, window: Window) {
        let window = Arc::new(window);

        let _ = window.request_inner_size(PhysicalSize::new(self.width, self.height));

        let surface = self
            .instance
            .create_surface(window.clone())
            .expect("failed to create surface.");

        let state = State::new(
            &self.instance,
            surface,
            &window,
            self.width,
            self.height,
        ).await;

        self.window.get_or_insert(window);
        self.state.get_or_insert(state);
    }

    fn handle_resized(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.state.as_mut().unwrap().resize_surface(width, height);
        }
    }

    fn handle_redraw(&mut self) -> Result<(), wgpu::SurfaceError> {
        if let Some(window) = self.window.as_ref() {
            if let Some(min) = window.is_minimized() {
                if min {
                    return Ok(());
                }
            }
        }

        let state = self.state.as_mut().unwrap();

        let screen_descriptor = ScreenDescriptor {
            size_in_pixels: [state.surface_config.width, state.surface_config.height],
            pixels_per_point: self.window.as_ref().unwrap().scale_factor() as f32 * state.scale_factor,
        };

        let surface_texture = state.surface.get_current_texture()?;

        let surface_view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = state
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        let window = self.window.as_ref().unwrap();

        let clipped_primitives = state.gui.prepare(
            &state.device,
            &state.queue,
            window,
            &mut encoder,
            &screen_descriptor,
            |ctx| self.renderer.gui(ctx),
        );

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("render_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &surface_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            pass.set_pipeline(&state.render_pipeline);
            pass.set_bind_group(0, &state.last_frame_bind_group, &[]);
            pass.set_bind_group(1, &state.view_bind_group, &[]);
            pass.draw(0..4, 0..1);

            state.gui.renderer.render(&mut pass.forget_lifetime(), &clipped_primitives, &screen_descriptor);
        }

        encoder.copy_texture_to_texture(surface_texture.texture.as_image_copy(), state.last_frame_texture.as_image_copy(), surface_texture.texture.size());

        state.queue.submit(std::iter::once(encoder.finish()));
        surface_texture.present();

        Ok(())
    }
}

impl<R: Renderer> ApplicationHandler for App<R> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = event_loop
            .create_window(Window::default_attributes())
            .unwrap();

        pollster::block_on(self.set_window(window));
    }

    fn device_event(
        &mut self,
        _event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::event::DeviceId,
        event: winit::event::DeviceEvent,
    ) {
        self.input.process_device_event(&event);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        if !self.state
            .as_mut()
            .unwrap()
            .gui
            .handle_input(self.window.as_ref().unwrap(), &event)
        {
            self.input.process_window_event(&event);
        }

        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            },
            WindowEvent::RedrawRequested => {
                self.renderer.input(&self.input);
                self.renderer.render(&mut self.render_ctx);

                let state = self.state.as_mut().unwrap();

                match self.handle_redraw() {
                    Err(wgpu::SurfaceError::Lost) => self.handle_resized(self.width, self.height),
                    Err(wgpu::SurfaceError::OutOfMemory) => event_loop.exit(),
                    Err(e) => eprintln!("{:?}", e),
                    Ok(_) => (),
                }

                self.window.as_ref().unwrap().request_redraw();
            },
            WindowEvent::Resized(new_size) => {
                self.handle_resized(new_size.width, new_size.height);
            },
            _ => (),
        }
    }

    fn new_events(&mut self, event_loop: &ActiveEventLoop, cause: winit::event::StartCause) {
        self.input.step();
    }
}
