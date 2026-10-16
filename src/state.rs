use std::sync::Arc;
use winit::window::Window;

pub struct RunningState {
    pub graphics_state: GraphicsState,
    pub render_pipelines: RenderPipelines,
    pub window: Arc<Window>,
}

pub struct GraphicsState {
    pub surface: wgpu::Surface<'static>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
}

pub struct RenderPipelines {
    pub render_pipeline: wgpu::RenderPipeline,
}