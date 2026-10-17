use std::error::Error;
use std::sync::Arc;
use winit::event_loop::ActiveEventLoop;
use winit::window::Window;

pub fn create_arc_window(
    event_loop: &ActiveEventLoop,
    title: &str,
) -> Result<Arc<Window>, Box<dyn Error>> {
    let window = event_loop.create_window(Window::default_attributes()
        .with_title(title)
    )?;
    Ok(Arc::new(window))
}
pub fn get_surface_size(window: &Window) -> (u32, u32) {
    let size = window.inner_size();
    let width = size.width.max(1);
    let height = size.height.max(1);

    (width, height)
}