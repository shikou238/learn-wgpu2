use crate::window::WindowWrapper;

use std::sync::Arc;

use winit::{
     event_loop::ActiveEventLoop, keyboard::KeyCode, window::{self, Window}
};


// This will store the state of our game
pub struct AppView {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub windows: Vec<WindowWrapper>,
}

impl AppView {
    // We don't need this to be async right now,
    // but we will in the next tutorial
    pub async fn new(windows: Vec<Arc<Window>>) -> anyhow::Result<Self> {
        //todo : support multiple windows
        let window = &windows[0];

        let size = window.inner_size();

        // The instance is a handle to our GPU
        // BackendBit::PRIMARY => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            #[cfg(not(target_arch = "wasm32"))]
            backends: wgpu::Backends::PRIMARY,
            #[cfg(target_arch = "wasm32")]
            backends: wgpu::Backends::GL,
            ..Default::default()
        });

        let surface = instance.create_surface(window.clone()).unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await?;

         let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                experimental_features: wgpu::ExperimentalFeatures::disabled(),
                // WebGL doesn't support all of wgpu's features, so if
                // we're building for the web we'll have to disable some.
                required_limits: if cfg!(target_arch = "wasm32") {
                    wgpu::Limits::downlevel_webgl2_defaults()
                } else {
                    wgpu::Limits::default()
                },
                memory_hints: Default::default(),
                trace: wgpu::Trace::Off,
            })
            .await?;
        
        let window_wrappers = windows.into_iter().map(|window| WindowWrapper::new(window, &instance, &adapter)).collect();
        
        Ok(Self {
            device,
            queue,
            windows: window_wrappers,
        })
    }

    pub fn close(&mut self, window_id: winit::window::WindowId) -> bool{
        self.windows.retain(|w| w.nature.id() != window_id);
        self.windows.is_empty()
    }
}
