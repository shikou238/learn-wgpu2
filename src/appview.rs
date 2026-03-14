use crate::window::WindowWrapper;

use std::sync::Arc;

use winit::{
     event_loop::ActiveEventLoop, keyboard::KeyCode, window::{self, Window}
};


// This will store the state of our game
pub struct AppView {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub render_pipeline: wgpu::RenderPipeline,
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

        let surface_caps = surface.get_capabilities(&adapter);
        // Shader code in this tutorial assumes an sRGB surface texture. Using a different
        // one will result in all the colors coming out darker. If you want to support non
        // sRGB surfaces, you'll need to account for that when drawing to the frame.
        let surface_format = surface_caps.formats.iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

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

        

        let render_pipeline = match Self::constuct_render_pipeline(&device, surface_format){
            Ok(pipeline) => pipeline,
            Err(e) => {
                log::error!("Failed to construct render pipeline: {:?}", e);
                return Err(e);
            }
        };
        
        let window_wrappers = windows.into_iter().map(|window| WindowWrapper::new(window, &instance, &adapter, surface_format)).collect();
        
        Ok(Self {
            device,
            queue,
            render_pipeline,
            windows: window_wrappers,
        })
    }
    pub fn reconstruct_render_pipeline(&mut self) -> anyhow::Result<()> {
        log::debug!("Reconstructing render pipeline...--------------------------------------------------------------");
        let surface_format = self.windows[0].config.format;
        self.render_pipeline = match Self::constuct_render_pipeline(&self.device, surface_format){
            Ok(pipeline) => pipeline,
            Err(e) => {
                log::error!("Failed to reconstruct render pipeline: {:?}", e);
                return Err(e);
            }
        };
        log::debug!("Render pipeline reconstructed successfully.");
        Ok(())
    }
    fn constuct_render_pipeline(device: &wgpu::Device,surface_format: wgpu::TextureFormat) -> anyhow::Result<wgpu::RenderPipeline> {
        let s = std::fs::read_to_string("src/shader.wgsl")?;
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(s.into()),
        });
        //let shader = device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[],
                immediate_size: 0,
            });
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"), // 1.
                buffers: &[], // 2.
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState { // 3.
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState { // 4.
                    format: surface_format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
             primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList, // 1.
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw, // 2.
                cull_mode: Some(wgpu::Face::Back),
                // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                polygon_mode: wgpu::PolygonMode::Fill,
                // Requires Features::DEPTH_CLIP_CONTROL
                unclipped_depth: false,
                // Requires Features::CONSERVATIVE_RASTERIZATION
                conservative: false,
            },
             depth_stencil: None, // 1.
            multisample: wgpu::MultisampleState {
                count: 1, // 2.
                mask: !0, // 3.
                alpha_to_coverage_enabled: false, // 4.
            },
            multiview_mask: None, // 5.
            cache: None, // 6.compile result
        });
        Ok(render_pipeline)
    }

    pub fn close(&mut self, window_id: winit::window::WindowId) -> bool{
        self.windows.retain(|w| w.nature.id() != window_id);
        self.windows.is_empty()
    }
}
