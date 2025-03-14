use std::mem;

use glam::{Mat4, Vec3};

use crate::state::State;

pub struct Renderer {
    width: u32,
    height: u32,
    state: State,
    target: Vec<u8>,
    frames: u32,
    frame_count: usize,
}

impl Renderer {
    pub fn new(width: u32, height: u32) -> Self {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());
        let target = Vec::<u8>::with_capacity((width * height * 4) as usize * mem::size_of::<f32>());

        Self {
            width,
            height,
            state: pollster::block_on(State::new(&instance, &target, width, height)),
            target,
            frames: 1,
            frame_count: 0,
        }
    }

    pub fn set_view(&mut self, camera: Mat4, position: Vec3, focal_length: f32) {
        self.state.view.camera = camera.to_cols_array();
        self.state.view.position = position.to_array();
        self.state.view.focal_length = focal_length;

        self.state
            .queue
            .write_buffer(&self.state.view_buffer, 0, bytemuck::cast_slice(&[self.state.view]));
    }

    pub fn set_render_skybox(&mut self, v: bool) {
        if v {
            self.state.view.flags |= 1;
        } else {
            self.state.view.flags &= !1;
        }
    }

    pub fn set_render_disc(&mut self, v: bool) {
        if v {
            self.state.view.flags |= 0b10;
        } else {
            self.state.view.flags &= !0b10;
        }
    }

    pub fn set_frames(&mut self, frames: u32) {
        self.frames = frames;
    }

    pub fn render(&mut self) {
        for _ in 0..self.frames {
            self.state.view.frame_count = self.frame_count as u32;
            self.state
                .queue
                .write_buffer(&self.state.view_buffer, 0, bytemuck::cast_slice(&[self.state.view]));

            self.render_frame();
        }

        let mut encoder = self
            .state
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture: &self.state.last_frame_textures[0],
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &self.state.output_staging_buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(self.width * 4 * mem::size_of::<f32>() as u32),
                    rows_per_image: Some(self.height),
                },
            },
            wgpu::Extent3d {
                width: self.width,
                height: self.height,
                depth_or_array_layers: 1,
            },
        );

        self.state.queue.submit(std::iter::once(encoder.finish()));

        let buffer_slice = self.state.output_staging_buffer.slice(..);
        let (sender, receiver) = flume::bounded(1);
        buffer_slice.map_async(wgpu::MapMode::Read, move |r| sender.send(r).unwrap());
        self.state.device.poll(wgpu::Maintain::wait()).panic_on_timeout();
        pollster::block_on(receiver.recv_async()).unwrap().unwrap();

        {
            let view = buffer_slice.get_mapped_range();
            self.target.extend_from_slice(&view[..]);
        }

        self.state.output_staging_buffer.unmap();
    }

    pub fn render_frame(&mut self) {
        let mut encoder = self
            .state
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

        let current_frame_view = &self.state.last_frame_views[self.frame_count % 2];

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("render_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: current_frame_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            pass.set_pipeline(&self.state.render_pipeline);
            pass.set_bind_group(0, &self.state.last_frame_bind_groups[(self.frame_count + 1) % 2], &[]);
            pass.set_bind_group(1, &self.state.view_bind_group, &[]);
            pass.set_bind_group(2, &self.state.sky_bind_group, &[]);
            pass.draw(0..3, 0..1);
        }

        encoder.copy_texture_to_texture(
            self.state.last_frame_textures[self.frame_count % 2].as_image_copy(),
            self.state.last_frame_textures[(self.frame_count + 1) % 2].as_image_copy(),
            self.state.last_frame_textures[0].size(),
        );

        self.state.queue.submit(std::iter::once(encoder.finish()));

        self.frame_count += 1;
    }

    pub fn target(self) -> Vec<u8> {
        self.target
    }
}
