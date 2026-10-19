mod app;
mod state;
mod wgpu_utilities;
mod window_utils;
mod camera;
mod buffer_bind_group;
mod bind_groups;

mod voxel {
    pub mod render;
}

mod render {
    pub mod renderer;
    pub mod render_pipeline;
}

use std::error::Error;
use winit::event_loop::{EventLoop};

fn main() -> Result<(), Box<dyn Error>> {
    let event_loop = EventLoop::new()?;
    let mut app = app::App {
        running_state: None,
    };

    event_loop.run_app(&mut app)?;
    Ok(())
}
