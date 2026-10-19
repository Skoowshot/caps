#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
    pub fn new() -> Self {
        use cgmath::SquareMatrix;
        Self {
            view_proj: cgmath::Matrix4::identity().into(),
        }
    }

    pub fn set_view_proj(&mut self, view_proj_matrix: cgmath::Matrix4<f32>) {
        self.view_proj = view_proj_matrix.into();
    }

    pub fn set_view_proj_from_camera(&mut self, camera: &Camera) {
        self.view_proj = camera.calc_matrix().into();
    }
}

pub struct Camera {
    pub position: cgmath::Point3<f32>,
    pub target: cgmath::Point3<f32>,
    pub up: cgmath::Vector3<f32>,

    pub aspect: f32,
    pub fov_y: cgmath::Rad<f32>,
    pub z_near: f32,
    pub z_far: f32,
}

impl Camera {
    pub fn calc_view_matrix(&self) -> cgmath::Matrix4<f32> {
        cgmath::Matrix4::look_at_rh(self.position, self.target, self.up)
    }

    pub fn calc_projection_matrix(&self) -> cgmath::Matrix4<f32> {
        cgmath::perspective(self.fov_y, self.aspect, self.z_near, self.z_far)
    }

    pub fn calc_matrix(&self) -> cgmath::Matrix4<f32> {
        self.calc_projection_matrix() * self.calc_view_matrix()
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.aspect = width as f32 / height as f32;
    }
}