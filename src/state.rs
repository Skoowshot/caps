use std::sync::Arc;
use winit::window::Window;
use crate::{render_pipeline, renderer};
use crate::camera::{Camera, Projection};

pub struct RunningState {
    pub graphics_state: GraphicsState,
    pub render_pipelines: render_pipeline::RenderPipelines,
    pub renderer: renderer::Renderer,
    pub camera: CameraState,
    pub window: Arc<Window>,
}

pub struct GraphicsState {
    pub surface: wgpu::Surface<'static>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
}

pub struct CameraState {
    pub camera_bind_group_layout: wgpu::BindGroupLayout,
    pub camera_bind_group: wgpu::BindGroup,
    pub camera_buffer: wgpu::Buffer,
    pub camera: Camera,
    pub projection: Projection,
}