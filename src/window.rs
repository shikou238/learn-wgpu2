use std::sync::Arc;

use wgpu::wgc::device;
use winit::{
    application::ApplicationHandler, event::*, event_loop::{ActiveEventLoop, EventLoop}, keyboard::{KeyCode, PhysicalKey}, window::Window
};

use crate::hsv_to_rgb;

pub struct WindowWrapper{
    pub nature: Arc<Window>,
    // todo why lifetime is needed here?
    pub surface: wgpu::Surface<'static>,
    pub config: wgpu::SurfaceConfiguration,
    pub is_surface_configured: bool,
    pub last_cursor_position: Option<winit::dpi::PhysicalPosition<f64>>,
}

impl WindowWrapper {
    pub fn new(window: Arc<Window>, instance: &wgpu::Instance, adapter: &wgpu::Adapter,surface_format: wgpu::TextureFormat) -> Self {
        let size = window.inner_size();

        let surface = instance.create_surface(window.clone()).unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        Self {
            nature: window,
            surface,
            config,
            is_surface_configured: false,
            last_cursor_position: None,
        }
    }
    pub fn resize(&mut self, _width: u32, _height: u32,device: &wgpu::Device) {
        let width = _width.max(1);
        let height = _height.max(1);
        if width > 0 && height > 0 {
            self.config.width = width;
            self.config.height = height;
            self.surface.configure(device, &self.config);
            self.is_surface_configured = true;
        }
    }
    // impl State
    pub fn handle_key(&self, event_loop: &ActiveEventLoop, code: KeyCode, is_pressed: bool) {
        match (code, is_pressed) {
            (KeyCode::Escape, true) => event_loop.exit(),
            _ => {}
        }
    }
    pub fn update(&mut self) {
        // remove `todo!()`
    }
    pub fn cursor_moved(&mut self, position: winit::dpi::PhysicalPosition<f64>) {
        self.last_cursor_position = Some(position);        
    }
    
    pub fn render(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, render_pipeline: &wgpu::RenderPipeline) -> Result<(), wgpu::SurfaceError>{
        self.nature.request_redraw();

        // We can't render unless the surface is configured
        if !self.is_surface_configured {
            return Ok(());
        }
            
        let output = self.surface.get_current_texture()?;
        // We'll do more stuff here in the next tutorial

        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear( 
                            match self.last_cursor_position{
                                None => wgpu::Color{r: 0.1, g: 0.2, b: 0.3, a: 1.0},
                                Some(winit::dpi::PhysicalPosition{x, y}) => {
                                    let size = self.nature.inner_size();
                                    hsv_to_rgb(x / size.width as f64, 0.5, y / size.height as f64)
                                }}),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
                multiview_mask: None,
            });
            render_pass.set_pipeline(render_pipeline); // 2.
            render_pass.draw(0..3, 0..1); // 3.
        }

        // submit will accept anything that implements IntoIter
        queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}
