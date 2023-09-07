#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
use winit::event_loop::EventLoop;

pub mod camera;
pub mod geometry;
pub mod renderer;
pub mod texture;
use renderer::Renderer;

mod app;

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub async fn run() {
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
            console_log::init_with_level(log::Level::Warn).expect("Could't initialize logger");
        } else {
            env_logger::init();
        }
    }

    let event_loop = EventLoop::new();
    let window = app::window::create_window(&event_loop);
    let mut state = Renderer::new(window).await;

    event_loop.run(move |event, _, control_flow| {
        app::event::handle_event(event, &mut state, control_flow);
    });
}
