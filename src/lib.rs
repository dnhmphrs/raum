#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
use winit::event_loop::EventLoop;

pub mod camera;
pub mod renderer;
pub mod texture;
use app::window::WindowSize;
use renderer::Renderer;

mod app;

const INITIAL_WINDOW_WIDTH: u32 = 800;
const INITIAL_WINDOW_HEIGHT: u32 = 600;

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

    let initial_size = WindowSize {
        width: INITIAL_WINDOW_WIDTH,
        height: INITIAL_WINDOW_HEIGHT,
    };

    let window = app::window::create_window(&event_loop, initial_size);
    let mut renderer = Renderer::new(window).await;

    event_loop.run(move |event, _, control_flow| {
        app::event::handle_event(event, &mut renderer, control_flow);
    });
}
