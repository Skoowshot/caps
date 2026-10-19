use std::error::Error;
use std::sync::Arc;
use cgmath::{Deg, Point3, Rad, Vector3};
use cgmath::num_traits::real::Real;
use wgpu::BindGroupLayout;
use wgpu::util::DeviceExt;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowId};
use crate::{wgpu_utilities, state, window_utils};
use crate::bind_groups::BindGroups;
use crate::buffer_bind_group::BufferBindGroup;
use crate::camera::{Camera, CameraUniform};
use crate::render::renderer::Renderer;
use crate::render::render_pipeline;
use crate::voxel::render::VoxelInstance;
use crate::wgpu_utilities::create_depth_texture;

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

                let device = &graphics_state.device;
                let config = &mut graphics_state.config;
                let (width, height) = window_utils::get_surface_size(&running_state.window);

                config.width = width;
                config.height = height;

                running_state.renderer.resize(device, (width, height));
                graphics_state.surface.configure(device, config)
            }
            WindowEvent::RedrawRequested => {
                let Some(running_state) = &mut self.running_state else { return; };
                let graphics_state = &running_state.graphics_state;

                let _ = running_state.renderer.redraw(
                    &graphics_state.surface,
                    &graphics_state.config,
                    &graphics_state.device,
                    &graphics_state.queue,
                    &running_state.render_pipelines,
                    &running_state.bind_groups
                );

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
        let window_arc = window_utils::create_arc_window(event_loop, "WGPU Test")?;
        let graphics_state = self.create_graphics_state(window_arc.clone(), event_loop)?;
        let bind_groups = self.create_bind_groups(&graphics_state.device)?;
        let render_pipelines = self.create_graphics_pipelines(&graphics_state.device, &graphics_state.config, &bind_groups)?;
        let renderer = Renderer::new(&graphics_state.device, window_utils::get_surface_size(&window_arc));

        Ok(state::RunningState {
            graphics_state,
            render_pipelines,
            bind_groups,
            renderer,
            window: window_arc,
        })
    }

    pub fn create_graphics_state(&self, window: Arc<Window>, _event_loop: &ActiveEventLoop) -> Result<state::GraphicsState, Box<dyn Error>> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::default());
        pollster::block_on(async {
            let surface = instance.create_surface(window.clone())?;
            let adapter = wgpu_utilities::create_adapter(&instance, &surface).await?;
            let (device, queue) = wgpu_utilities::create_device_and_queue(&adapter).await?;
            let (width, height) = window_utils::get_surface_size(&window);
            let surface_capabilities = surface.get_capabilities(&adapter);
            let surface_format = wgpu_utilities::find_surface_format(&surface_capabilities);
            let config = wgpu_utilities::create_surface_configuration(&surface_capabilities, surface_format, width, height);

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
        bind_groups: &BindGroups,
    ) -> Result<render_pipeline::RenderPipelines, Box<dyn Error>> {
        let voxel_pipeline = render_pipeline::create_pipeline(
            &device,
            &config,
            &[VoxelInstance::desc()],
            &[&bind_groups.camera.bind_group_layout],
            true,
            "shaders/voxel.wgsl"
        )?;

        Ok(render_pipeline::RenderPipelines {
            voxel_pipeline
        })
    }

    pub fn create_bind_groups(
        &mut self,
        device: &wgpu::Device,
    ) -> Result<BindGroups, Box<dyn Error>> {
        let camera_bind_group = BufferBindGroup::new_uniform(
            device,
            size_of::<CameraUniform>() as u64,
            Some("camera")
        );

        Ok(BindGroups {
            camera: camera_bind_group,
        })
    }
}