mod app;
mod state;

use std::error::Error;
use winit::event_loop::{EventLoop};

fn main() -> Result<(), Box<dyn Error>> {
    let event_loop = EventLoop::new()?;
    let mut app = app::App {
        running_state: None
    };

    event_loop.run_app(&mut app)?;
    Ok(())
}
