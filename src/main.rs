use winit::event_loop::EventLoop;
use bespoke_engine::window::{Surface, SurfaceContext};

mod window;

use crate::window::Window;

#[tokio::main]
async fn main() {
    env_logger::init();

    let event_loop = EventLoop::new().unwrap();

    let ready = &|surface_context: &SurfaceContext| {
        let _ = surface_context.window.set_cursor_grab(winit::window::CursorGrabMode::Locked);
        Window::new(&surface_context.device, &surface_context.queue, surface_context.config.format, surface_context.window.inner_size())
    };

    let mut surface = Surface::new(ready).await;

    event_loop.run_app(&mut surface).unwrap();
}
