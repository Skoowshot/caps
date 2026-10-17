mod app;
mod state;
mod gfx_init;
mod window_helpers;
mod render_pipeline;
mod renderer;
mod camera;

mod voxel {
    pub mod render;
}

use std::error::Error;
use winit::event_loop::{EventLoop};

fn main() -> Result<(), Box<dyn Error>> {
    let event_loop = EventLoop::new()?;
    let mut app = app::App {
        running_state: None,
        angle: 0.0,
    };

    event_loop.run_app(&mut app)?;
    Ok(())
}
