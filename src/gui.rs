use egui::{ClippedPrimitive, Context};
use egui_wgpu::{wgpu::{CommandEncoder, Device, Queue, TextureFormat}, Renderer, ScreenDescriptor};
use egui_winit::State;
use winit::{event::WindowEvent, window::Window};

pub struct GuiRenderer {
    state: State,
    pub renderer: Renderer,
}

impl GuiRenderer {
    pub fn new(
        device: &Device,
        output_color_format: TextureFormat,
        output_depth_format: Option<TextureFormat>,
        msaa_samples: u32,
        window: &Window,
    ) -> GuiRenderer {
        let egui_context = Context::default();

        let egui_state = State::new(
            egui_context,
            egui::viewport::ViewportId::ROOT,
            &window,
            Some(window.scale_factor() as f32),
            None,
            Some(2 * 1024),
        );

        let egui_renderer = Renderer::new(
            device,
            output_color_format,
            output_depth_format,
            msaa_samples,
            true,
        );

        GuiRenderer {
            state: egui_state,
            renderer: egui_renderer,
        }
    }

    pub fn handle_input(&mut self, window: &Window, event: &WindowEvent) -> bool {
        self.state.on_window_event(window, event).consumed
    }

    pub fn prepare(
        &mut self,
        device: &Device,
        queue: &Queue,
        window: &Window,
        encoder: &mut CommandEncoder,
        screen_descriptor: &ScreenDescriptor,
        mut gui: impl FnMut(&Context),
    ) -> Vec<ClippedPrimitive> {
        let raw_input = self.state.take_egui_input(window);
        self.state.egui_ctx().begin_pass(raw_input);
        gui(self.state.egui_ctx());

        let full_output = self.state.egui_ctx().end_pass();

        self.state.handle_platform_output(window, full_output.platform_output);

        let tris = self.state.egui_ctx().tessellate(full_output.shapes, self.state.egui_ctx().pixels_per_point());

        self.renderer.update_buffers(
            device,
            queue,
            encoder,
            &tris,
            screen_descriptor,
        );

        for (id, im_delta) in full_output.textures_delta.set {
            self.renderer.update_texture(device, queue, id, &im_delta);
        }

        for id in full_output.textures_delta.free {
            self.renderer.free_texture(&id);
        }

        tris
    }
}
