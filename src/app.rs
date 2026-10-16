use std::error::Error;
use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowId};
use crate::state;

pub struct App {
    pub running_state: Option<state::RunningState>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.is_running() { return; }

        self.running_state = Some(self.create_running_state(event_loop).expect("failed to create running state"));

        let Some(state) = &self.running_state else { return; };
        state.window.request_redraw();
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _window_id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::Resized(_size) => {
                let Some(running_state) = &mut self.running_state else { return; };
                let graphics_state = &mut running_state.graphics_state;

                let config = &mut graphics_state.config;
                let (width, height) = get_surface_size(&running_state.window);

                config.width = width;
                config.height = height;

                graphics_state.surface.configure(
                    &graphics_state.device,
                    &graphics_state.config
                )
            }
            WindowEvent::RedrawRequested => {
                let Some(running_state) = &self.running_state else { return; };
                let graphics_state = &running_state.graphics_state;

                let frame = match graphics_state.surface.get_current_texture() {
                    Ok(frame) => frame,
                    Err(_) => {
                        graphics_state.surface.configure(&graphics_state.device, &graphics_state.config);
                        graphics_state.surface.get_current_texture().expect("failed to acquire next texture")
                    }
                };

                let view = frame
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());

                let mut encoder =
                    graphics_state.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                        label: Some("Render Encoder")
                    });

                {
                    let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("Render Pass"),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color {
                                    r: 0.01,
                                    g: 0.01,
                                    b: 0.015,
                                    a: 1.0
                                }),
                                store: wgpu::StoreOp::Store
                            }
                        })],
                        depth_stencil_attachment: None,
                        timestamp_writes: None,
                        occlusion_query_set: None,
                    });

                    render_pass.set_pipeline(&running_state.render_pipelines.render_pipeline);
                    render_pass.draw(0..3, 0..1);
                }

                graphics_state.queue.submit(std::iter::once(encoder.finish()));
                frame.present();

                running_state.window.request_redraw()
            }
            _ => {}
        }
    }
}

impl App {
    pub fn is_running(&self) -> bool {
        self.running_state.is_some()
    }

    pub fn create_running_state(&self, event_loop: &ActiveEventLoop) -> Result<state::RunningState, Box<dyn Error>> {
        let window = event_loop
            .create_window(Window::default_attributes().with_title("WGPU Test"))?;
        let window_arc = Arc::new(window);

        let graphics_state = self.create_graphics_state(window_arc.clone(), event_loop)?;
        let render_pipelines = self.create_graphics_pipelines(&graphics_state.device, &graphics_state.config)?;

        Ok(state::RunningState {
            graphics_state,
            render_pipelines,
            window: window_arc,
        })
    }

    pub fn create_graphics_state(&self, window: Arc<Window>, _event_loop: &ActiveEventLoop) -> Result<state::GraphicsState, Box<dyn Error>> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::default());
        pollster::block_on(async {
            let surface = instance.create_surface(window.clone())?;
            let adapter = self.create_adapter(&instance, &surface).await?;
            let (device, queue) = self.create_device_and_queue(&adapter).await?;
            let (width, height) = get_surface_size(&window);
            let surface_capabilities = surface.get_capabilities(&adapter);
            let surface_format = self.find_surface_format(&surface_capabilities);
            let config = self.create_surface_configuration(&surface_capabilities, surface_format, width, height);

            surface.configure(&device, &config);

            Ok(state::GraphicsState {
                surface,
                device,
                queue,
                config,
            })
        })
    }

    pub fn create_graphics_pipelines(
        &self,
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
    ) -> Result<state::RenderPipelines, Box<dyn Error>> {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Triangle Shader"),
            source: wgpu::ShaderSource::Wgsl(
                r#"
                    struct VertexOutput {
                        @builtin(position) position: vec4<f32>,
                        @location(0) color: vec3<f32>,
                    };

                    @vertex
                    fn vs_main(@builtin(vertex_index) in_vertex_index: u32) -> VertexOutput {
                        var pos = array<vec2<f32>, 3>(
                            vec2<f32>(0,  0.6),
                            vec2<f32>(-0.6, -0.6),
                            vec2<f32>( 0.6, -0.6)
                        );

                        var colors = array<vec3<f32>, 3>(
                            vec3<f32>(1.0, 0.0, 0.0),
                            vec3<f32>(0.0, 1.0, 0.0),
                            vec3<f32>(0.0, 0.0, 1.0)
                        );

                        var indices = array<u32, 3>(
                            0, 1, 2
                        );

                        let vertex_index = indices[in_vertex_index];

                        var out: VertexOutput;
                        out.position = vec4<f32>(pos[vertex_index], 0.0, 1.0);
                        out.color = colors[vertex_index];
                        return out;
                    }

                    @fragment
                    fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
                        return vec4<f32>(in.color, 1.0);
                    }
                    "#
                    .into(),
            ),
        });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        Ok(state::RenderPipelines {
            render_pipeline
        })
    }

    pub async fn create_adapter<'a>(&self, instance: &wgpu::Instance, surface: &wgpu::Surface<'a>) -> Result<wgpu::Adapter, Box<dyn Error>> {
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(surface),
                force_fallback_adapter: false,
            })
            .await
            .ok_or("failed to find an appropriate adapter")?;

        Ok(adapter)
    }

    pub async fn create_device_and_queue(&self, adapter: &wgpu::Adapter) -> Result<(wgpu::Device, wgpu::Queue), Box<dyn Error>> {
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                label: None
            }, None)
            .await?;

        Ok((device, queue))
    }

    pub fn find_surface_format(&self, surface_capabilities: &wgpu::SurfaceCapabilities) -> wgpu::TextureFormat {
        surface_capabilities
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_capabilities.formats[0])
    }

    pub fn create_surface_configuration(
        &self,
        surface_capabilities: &wgpu::SurfaceCapabilities,
        surface_format: wgpu::TextureFormat,
        width: u32,
        height: u32
    ) -> wgpu::SurfaceConfiguration {
        wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width,
            height,
            present_mode: surface_capabilities.present_modes[0],
            alpha_mode: surface_capabilities.alpha_modes[0],
            desired_maximum_frame_latency: 2,
            view_formats: vec![]
        }
    }
}

pub fn get_surface_size(window: &Window) -> (u32, u32) {
    let size = window.inner_size();
    let width = size.width.max(1);
    let height = size.height.max(1);

    (width, height)
}