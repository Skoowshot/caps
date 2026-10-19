use std::sync::Arc;
use winit::window::Window;
use crate::bind_groups::BindGroups;
use crate::render::render_pipeline::RenderPipelines;
use crate::render::renderer::Renderer;

pub struct RunningState {
    pub graphics_state: GraphicsState,
    pub render_pipelines: RenderPipelines,
    pub renderer: Renderer,
    pub bind_groups: BindGroups,
    pub window: Arc<Window>,
}

pub struct GraphicsState {
    pub surface: wgpu::Surface<'static>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
}