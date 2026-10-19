use std::error::Error;

pub async fn create_adapter<'a>(instance: &wgpu::Instance, surface: &wgpu::Surface<'a>) -> Result<wgpu::Adapter, Box<dyn Error>> {
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

pub async fn create_device_and_queue(adapter: &wgpu::Adapter) -> Result<(wgpu::Device, wgpu::Queue), Box<dyn Error>> {
    let (device, queue) = adapter
        .request_device(&wgpu::DeviceDescriptor {
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::default(),
            label: None
        }, None)
        .await?;

    Ok((device, queue))
}

pub fn find_surface_format(surface_capabilities: &wgpu::SurfaceCapabilities) -> wgpu::TextureFormat {
    surface_capabilities
        .formats
        .iter()
        .copied()
        .find(|f| f.is_srgb())
        .unwrap_or(surface_capabilities.formats[0])
}

pub fn create_surface_configuration(
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

pub fn create_depth_texture(device: &wgpu::Device, dims: (u32, u32)) -> wgpu::TextureView {
    let size = wgpu::Extent3d {
        width: dims.0,
        height: dims.1,
        depth_or_array_layers: 1,
    };

    let desc = wgpu::TextureDescriptor {
        label: Some("texture:depth"),
        size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Depth32Float,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    };

    let texture = device.create_texture(&desc);
    texture.create_view(&wgpu::TextureViewDescriptor::default())
}