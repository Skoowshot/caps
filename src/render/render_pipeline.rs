use std::error::Error;
use std::fs;

pub struct RenderPipelines {
    pub voxel_pipeline: wgpu::RenderPipeline,
}

pub fn create_pipeline(
    device: &wgpu::Device,
    config: &wgpu::SurfaceConfiguration,
    buffers: &[wgpu::VertexBufferLayout],
    bind_group_layouts: &[&wgpu::BindGroupLayout],
    depth_stencil: bool,
    shader_path: &str,
) -> Result<wgpu::RenderPipeline, Box<dyn Error>> {
    let shader_source = fs::read_to_string(shader_path)?;

    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some(&format!("shader:{}", shader_path)),
        source: wgpu::ShaderSource::Wgsl(shader_source.into())
    });

    let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some(&format!("render_pipeline_layout:{}", shader_path)),
        bind_group_layouts,
        push_constant_ranges: &[],
    });

    let render_targets = create_render_target_states(config.format);

    let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some(&format!("render_pipeline:{}", shader_path)),
        layout: Some(&render_pipeline_layout),
        vertex: create_vertex_state(&shader, buffers),
        fragment: Some(create_fragment_state(&shader, &render_targets)),
        primitive: create_primitive_state(),
        depth_stencil: if depth_stencil { Some(create_depth_stencil_state()) } else { None },
        multisample: create_multisample_state(),
        multiview: None,
    });

    Ok(render_pipeline)
}

pub fn create_vertex_state<'a>(
    shader: &'a wgpu::ShaderModule,
    buffers: &'a [wgpu::VertexBufferLayout<'a>],
) -> wgpu::VertexState<'a> {
    wgpu::VertexState {
        module: shader,
        entry_point: "vs_main",
        compilation_options: wgpu::PipelineCompilationOptions::default(),
        buffers,
    }
}

pub fn create_render_target_states<'a>(
    texture_format: wgpu::TextureFormat,
) -> [Option<wgpu::ColorTargetState>; 1] {
    [Some(wgpu::ColorTargetState {
        format: texture_format,
        blend: Some(wgpu::BlendState::REPLACE),
        write_mask: wgpu::ColorWrites::ALL,
    })]
}

pub fn create_fragment_state<'a, 'b>(
    shader: &'a wgpu::ShaderModule,
    targets: &'a [Option<wgpu::ColorTargetState>],
) -> wgpu::FragmentState<'a> {
    wgpu::FragmentState {
        module: shader,
        entry_point: "fs_main",
        compilation_options: wgpu::PipelineCompilationOptions::default(),
        targets,
    }
}

pub fn create_primitive_state(
) -> wgpu::PrimitiveState {
    wgpu::PrimitiveState {
        topology: wgpu::PrimitiveTopology::TriangleList,
        strip_index_format: None,
        front_face: wgpu::FrontFace::Ccw,
        cull_mode: Some(wgpu::Face::Back),
        polygon_mode: wgpu::PolygonMode::Fill,
        unclipped_depth: false,
        conservative: false,
    }
}

pub fn create_depth_stencil_state(
) -> wgpu::DepthStencilState {
    wgpu::DepthStencilState {
        format: wgpu::TextureFormat::Depth32Float,
        depth_write_enabled: true,
        depth_compare: wgpu::CompareFunction::Less,
        stencil: wgpu::StencilState::default(),
        bias: wgpu::DepthBiasState::default(),
    }
}

pub fn create_multisample_state(
) -> wgpu::MultisampleState {
    wgpu::MultisampleState {
        count: 1,
        mask: !0,
        alpha_to_coverage_enabled: false,
    }
}