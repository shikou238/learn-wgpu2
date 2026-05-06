
use crate::{appview::AppView, window::WindowWrapper};

use std::sync::Arc;

use winit::{
    application::ApplicationHandler, event::*, event_loop::{ActiveEventLoop, EventLoop}, keyboard::{KeyCode, PhysicalKey}, window::Window
};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;


pub struct App {
    #[cfg(target_arch = "wasm32")]
    proxy: Option<winit::event_loop::EventLoopProxy<AppView>>,
    state: Option<AppView>,
}

impl App {
    pub fn new(#[cfg(target_arch = "wasm32")] event_loop: &EventLoop<AppView>) -> Self {
        #[cfg(target_arch = "wasm32")]
        let proxy = Some(event_loop.create_proxy());
        Self {
            state: None,
            #[cfg(target_arch = "wasm32")]
            proxy,
        }
    }
}


impl ApplicationHandler<AppView> for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        #[allow(unused_mut)]
        let mut window_attributes = Window::default_attributes();

        #[cfg(target_arch = "wasm32")]
        {
            use wasm_bindgen::JsCast;
            use winit::platform::web::WindowAttributesExtWebSys;
            
            const CANVAS_ID: &str = "canvas";

            let window = wgpu::web_sys::window().unwrap();
            let document = window.document().unwrap();
            let canvas = document.get_element_by_id(CANVAS_ID).unwrap();
            let html_canvas_element = canvas.unchecked_into();
            window_attributes = window_attributes.with_canvas(Some(html_canvas_element));
        }

        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());

        let mut window_attributes2 = Window::default_attributes();
        let _window2 = Arc::new(event_loop.create_window(window_attributes2).unwrap());

        #[cfg(not(target_arch = "wasm32"))]
        {
            // If we are not on web we can use pollster to
            // await the 
            self.state = Some(pollster::block_on(AppView::new(vec![window, _window2])).unwrap());
        }

        #[cfg(target_arch = "wasm32")]
        {
            // Run the future asynchronously and use the
            // proxy to send the results to the event loop
            if let Some(proxy) = self.proxy.take() {
                wasm_bindgen_futures::spawn_local(async move {
                    assert!(proxy
                        .send_event(
                            AppView::new(vec![window])
                                .await
                                .expect("Unable to create canvas!!!")
                        )
                        .is_ok())
                });
            }
        }
    }

    #[allow(unused_mut)]
    fn user_event(&mut self, _event_loop: &ActiveEventLoop, mut event: AppView) {
        // This is where proxy.send_event() ends up
        #[cfg(target_arch = "wasm32")]
        {
            use winit::window;

            let mut window = &mut event.windows[0];
            window.nature.request_redraw();
            window.resize(
                window.nature.inner_size().width,
                window.nature.inner_size().height,
                &event.device
            );
        }
        self.state = Some(event);
    }

     fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {

        let _canvas = match &mut self.state {
            Some(canvas) => canvas,
            None => return,
        };
        let window_wrapper = {
            let window_wrapper = _canvas.windows.iter_mut().find(|w| w.nature.id() == window_id);
            match window_wrapper {
                Some(wrapper) => wrapper,
                None => return,
            }
        };
        let (device, queue,render_pipeline) = (&_canvas.device, &_canvas.queue, &_canvas.render_pipeline);
        match event {
            WindowEvent::CloseRequested => {
                if _canvas.close(window_id){
                    event_loop.exit()
                }
                return
            },
            WindowEvent::Resized(size) => window_wrapper.resize(size.width, size.height, device),
            WindowEvent::RedrawRequested => {
                window_wrapper.render(device, queue, render_pipeline, &_canvas.model_buffer, &_canvas.diffuse_bind_group).unwrap();
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(code),
                        state: key_state,
                        ..
                    },
                ..
            } => match (code, key_state.is_pressed()) {
                (KeyCode::Escape, true) => event_loop.exit(),
                (KeyCode::KeyR, true) => {_canvas.reload();},
                _ => {}
            },
            WindowEvent::RedrawRequested => {
                window_wrapper.update();
                match window_wrapper.render(device, queue, render_pipeline, &_canvas.model_buffer, &_canvas.diffuse_bind_group) {
                    Ok(_) => {}
                    // Reconfigure the surface if it's lost or outdated
                    Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                        let size = window_wrapper.nature.inner_size();
                        window_wrapper.resize(size.width, size.height, device);
                    }
                    Err(e) => {
                        log::error!("Unable to render {}", e);
                    }
                }
            },
            WindowEvent::CursorMoved { device_id, position } => {
                window_wrapper.cursor_moved(position);
            }
            _ => {}
        }
    }
}
