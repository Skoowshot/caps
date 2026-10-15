use std::error::Error;
use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowId};

struct App {
    state: Option<AppState>,
}

struct AppState {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    // move to specific pipelines struct
    render_pipeline: wgpu::RenderPipeline,
    window: Arc<Window>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.state.is_some() {
            return;
        }

        let window = Arc::new(
            event_loop
                .create_window(Window::default_attributes().with_title("WGPU Test"))
                .unwrap(),
        );

        pollster::block_on(async {
            let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::default());

            let surface = instance.create_surface(window.clone()).unwrap();
            let adapter = instance
                .request_adapter(&wgpu::RequestAdapterOptions {
                    power_preference: wgpu::PowerPreference::HighPerformance,
                    compatible_surface: Some(&surface),
                    force_fallback_adapter: false,
                })
                .await
                .expect("failed to find an appropriate adapter");

            let (device, queue) = adapter
                .request_device(&wgpu::DeviceDescriptor {
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    label: None
                }, None)
                .await
                .expect("failed to create device");

            let size = window.inner_size();
            let width = size.width.max(1);
            let height = size.height.max(1);

            let surface_caps = surface.get_capabilities(&adapter);
            let surface_format = surface_caps
                .formats
                .iter()
                .copied()
                .find(|f| f.is_srgb())
                .unwrap_or(surface_caps.formats[0]);

            let config = wgpu::SurfaceConfiguration {
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                format: surface_format,
                width,
                height,
                present_mode: surface_caps.present_modes[0],
                alpha_mode: surface_caps.alpha_modes[0],
                desired_maximum_frame_latency: 2,
                view_formats: vec![]
            };

            surface.configure(&device, &config);

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

            self.state = Some(AppState {
                window,
                surface,
                device,
                queue,
                config,
                render_pipeline,
            });
        });
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _window_id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit()
            }
            WindowEvent::Resized(size) => {
                let Some(state) = &mut self.state else { return; };

                let config = &mut state.config;
                let width = size.width.max(1);
                let height = size.height.max(1);

                config.width = width;
                config.height = height;
                state.surface.configure(&state.device, &state.config);
            }
            WindowEvent::RedrawRequested => {
                let Some(state) = &self.state else { return; };

                let frame = match state.surface.get_current_texture() {
                    Ok(frame) => frame,
                    Err(_) => {
                        state.surface.configure(&state.device, &state.config);
                        state.surface.get_current_texture().expect("failed to acquire next texture")
                    }
                };

                let view = frame
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());

                let mut encoder =
                    state.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
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

                    render_pass.set_pipeline(&state.render_pipeline);
                    render_pass.draw(0..3, 0..1);
                }

                state.queue.submit(std::iter::once(encoder.finish()));
                frame.present();

                state.window.request_redraw()
            },
            _ => (),
        }
    }
}

impl App {

}

fn main() -> Result<(), Box<dyn Error>> {
    let event_loop = EventLoop::new()?;
    let mut app = App {
        state: None
    };

    event_loop.run_app(&mut app)?;
    Ok(())
}
