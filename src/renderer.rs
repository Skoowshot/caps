use std::error::Error;
use rand::Rng;
use wgpu::util::DeviceExt;
use crate::render_pipeline;
use crate::voxel::render::VoxelInstance;

pub struct Renderer {
    pub depth_texture_view: Option<wgpu::TextureView>,
    pub voxel_buffer: Option<wgpu::Buffer>,
    pub num_voxels: u32,
}

impl Renderer {

    pub fn redraw(
        &mut self,
        surface: &wgpu::Surface,
        config: &wgpu::SurfaceConfiguration,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        render_pipelines: &render_pipeline::RenderPipelines,
        camera_bind_group: &wgpu::BindGroup,
    ) -> Result<(), Box<dyn Error>> {
        if self.depth_texture_view.is_none() {
            self.depth_texture_view = Some(render_pipeline::create_depth_texture(device, config))
        }
        if self.voxel_buffer.is_none() {
            let mut rng = rand::thread_rng();
            let count = 3000000;

            let voxels: Vec<VoxelInstance> = (0..count)
                .map(|_| {
                    VoxelInstance {
                        position: [
                            rng.gen_range(-300.0..300.0),
                            rng.gen_range(-300.0..300.0),
                            rng.gen_range(-300.0..300.0),
                        ],
                        size: rng.gen_range(0.05..0.1),
                        color: [
                            rng.r#gen::<f32>(),
                            rng.r#gen::<f32>(),
                            rng.r#gen::<f32>(),
                            1.0,
                        ],
                    }
                })
                .collect();

            let voxel_buffer = device.create_buffer_init(
                &wgpu::util::BufferInitDescriptor {
                    label: Some("Voxel Instance Buffer"),
                    contents: bytemuck::cast_slice(&*voxels),
                    usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                }
            );

            self.voxel_buffer = Some(voxel_buffer);
            self.num_voxels = voxels.len() as u32;
        }
        let frame = surface.get_current_texture()?;

        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
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
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: self.depth_texture_view.as_ref().unwrap(),
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            let voxel_buffer = self.voxel_buffer.as_ref().unwrap();
        render_pass.set_pipeline(&render_pipelines.voxel_pipeline);
        render_pass.set_bind_group(0, camera_bind_group, &[]);
        render_pass.set_vertex_buffer(0, voxel_buffer.slice(..));
        render_pass.draw(0..36, 0..self.num_voxels);
        }
        queue.submit(std::iter::once(encoder.finish()));
        frame.present();

        Ok(())
    }
}