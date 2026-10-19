pub struct BufferBindGroup {
    pub buffer: wgpu::Buffer,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
}

impl BufferBindGroup {
    pub fn new_uniform(
        device: &wgpu::Device,
        size: u64,
        label: Option<&str>
    ) -> Self {
        let buffer_label = label.map(|f| format!("buffer:{}", f));
        let bind_group_layout_label = label.map(|f| format!("bind_group_layout:{}", f));
        let bind_group_label = label.map(|f| format!("bind_group:{}", f));

        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: buffer_label.as_deref(),
            size,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: bind_group_layout_label.as_deref(),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }]
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: bind_group_label.as_deref(),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding()
            }]
        });

        Self {
            buffer,
            bind_group_layout,
            bind_group,
        }
    }
}