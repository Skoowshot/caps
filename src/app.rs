use std::error::Error;
use std::sync::Arc;
use wgpu::BindGroupLayout;
use wgpu::util::DeviceExt;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowId};
use crate::{gfx_init, render_pipeline, state, window_helpers};
use crate::camera::{Camera, CameraUniform, Projection};
use crate::renderer::Renderer;
use crate::state::CameraState;
use crate::voxel::render::VoxelInstance;

pub struct App {
    pub running_state: Option<state::RunningState>,
    pub angle: f32,
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
                let (width, height) = window_helpers::get_surface_size(&running_state.window);

                config.width = width;
                config.height = height;

                graphics_state.surface.configure(
                    &graphics_state.device,
                    &graphics_state.config
                )
            }
            WindowEvent::RedrawRequested => {
                let Some(running_state) = &mut self.running_state else { return; };
                let graphics_state = &running_state.graphics_state;

                self.angle += 0.001;

                let radius = 5.0;

                running_state.camera.camera.position.x = self.angle.cos() * radius;
                running_state.camera.camera.position.z = self.angle.sin() * radius;
                running_state.camera.camera.position.y = 3.0;

                running_state.camera.camera.target.x += self.angle.cos() * 0.0001;

                let mut camera_uniform = CameraUniform::new();
                camera_uniform.update_view_proj(&running_state.camera.camera, &running_state.camera.projection);

                running_state.graphics_state.queue.write_buffer(
                    &running_state.camera.camera_buffer,
                    0,
                    bytemuck::cast_slice(&[camera_uniform]),
                );


                running_state.renderer.redraw(
                    &graphics_state.surface,
                    &graphics_state.config,
                    &graphics_state.device,
                    &graphics_state.queue,
                    &running_state.render_pipelines,
                    &running_state.camera.camera_bind_group
                ).expect("failed to redraw");

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

    pub fn create_running_state(&mut self, event_loop: &ActiveEventLoop) -> Result<state::RunningState, Box<dyn Error>> {
        let window_arc = window_helpers::create_arc_window(event_loop, "WGPU Test")?;
        let graphics_state = self.create_graphics_state(window_arc.clone(), event_loop)?;
        let camera_state = self.create_camera_state(&graphics_state.device);
        let render_pipelines = self.create_graphics_pipelines(&graphics_state.device, &graphics_state.config, &camera_state.camera_bind_group_layout)?;
        let renderer = Renderer {
            depth_texture_view: None,
            voxel_buffer: None,
            num_voxels: 0,
        };

        Ok(state::RunningState {
            graphics_state,
            camera: camera_state,
            render_pipelines,
            renderer,
            window: window_arc,
        })
    }

    pub fn create_graphics_state(&self, window: Arc<Window>, _event_loop: &ActiveEventLoop) -> Result<state::GraphicsState, Box<dyn Error>> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::default());
        pollster::block_on(async {
            let surface = instance.create_surface(window.clone())?;
            let adapter = gfx_init::create_adapter(&instance, &surface).await?;
            let (device, queue) = gfx_init::create_device_and_queue(&adapter).await?;
            let (width, height) = window_helpers::get_surface_size(&window);
            let surface_capabilities = surface.get_capabilities(&adapter);
            let surface_format = gfx_init::find_surface_format(&surface_capabilities);
            let config = gfx_init::create_surface_configuration(&surface_capabilities, surface_format, width, height);

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
        &mut self,
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        camera_bind_group_layout: &BindGroupLayout,
    ) -> Result<render_pipeline::RenderPipelines, Box<dyn Error>> {
        let voxel_pipeline = render_pipeline::create_pipeline(
            &device,
            &config,
            &[VoxelInstance::desc()],
            &[camera_bind_group_layout],
            "shaders/voxel.wgsl"
        )?;

        Ok(render_pipeline::RenderPipelines {
            voxel_pipeline
        })
    }

    pub fn create_camera_state(&self, device: &wgpu::Device) -> state::CameraState {
        let mut camera_uniform = CameraUniform::new();
        let camera = Camera {
            position: (0.0, 3.0, 5.0).into(),
            target: (0.0, 0.0, 0.0).into(),
            up: cgmath::Vector3::unit_y(),
        };
        let projection = Projection::new(800, 600, 45.0, 0.1, 100.0);
        camera_uniform.update_view_proj(&camera, &projection);

        let camera_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Camera Buffer"),
                contents: bytemuck::cast_slice(&[camera_uniform]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        );
        let camera_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
            label: Some("camera_bind_group_layout"),
        });
        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera_buffer.as_entire_binding(),
                }
            ],
            label: Some("camera_bind_group"),
        });
        CameraState {
            camera,
            camera_buffer,
            projection,
            camera_bind_group_layout,
            camera_bind_group
        }
    }
}