use std::error::Error;
use cgmath::{Deg, Point3, Vector3};
use rand::Rng;
use wgpu::util::DeviceExt;
use crate::bind_groups::BindGroups;
use crate::render::render_pipeline::RenderPipelines;
use crate::voxel::render::VoxelInstance;
use crate::camera::{Camera, CameraUniform};
use crate::wgpu_utilities::create_depth_texture;

pub struct Renderer {
    pub depth_texture_view: wgpu::TextureView,
    pub voxel_buffer: Option<wgpu::Buffer>,
    pub num_voxels: u32,
}

impl Renderer {
    pub fn new(
        device: &wgpu::Device,
        surface_size: (u32, u32),
    ) -> Self {
        Self {
            depth_texture_view: create_depth_texture(device, surface_size),
            voxel_buffer: None,
            num_voxels: 0,
        }
    }

    pub fn redraw(
        &mut self,
        surface: &wgpu::Surface,
        config: &wgpu::SurfaceConfiguration,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        render_pipelines: &RenderPipelines,
        bind_groups: &BindGroups
    ) -> Result<(), Box<dyn Error>> {
        if self.voxel_buffer.is_none() {
            let mut rng = rand::thread_rng();
            let count = 5000000;

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
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            let radius = 5.0;
            let (width, height) = (800, 600);
            let mut camera_uniform = CameraUniform::new();
            let camera = Camera {
                aspect: width as f32 / height as f32,
                target: Point3::new(0.0,0.0,0.0),
                position: Point3::new(radius, 5.0, radius),
                up: Vector3::unit_y(),
                z_near: 0.01,
                z_far: 100.0,
                fov_y: Deg(50.0).into()
            };
            camera_uniform.set_view_proj_from_camera(&camera);

            queue.write_buffer(
                &bind_groups.camera.buffer,
                0,
                bytemuck::cast_slice(&[camera_uniform]),
            );

            let voxel_buffer = self.voxel_buffer.as_ref().unwrap();
        render_pass.set_pipeline(&render_pipelines.voxel_pipeline);
        render_pass.set_bind_group(0, &bind_groups.camera.bind_group, &[]);
        render_pass.set_vertex_buffer(0, voxel_buffer.slice(..));
        render_pass.draw(0..36, 0..self.num_voxels);
        }
        queue.submit(std::iter::once(encoder.finish()));
        frame.present();

        Ok(())
    }

    pub fn resize(&mut self, device: &wgpu::Device, dims: (u32, u32)) {
        self.depth_texture_view = create_depth_texture(device, dims);
    }
}