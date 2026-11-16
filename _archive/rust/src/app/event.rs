use crate::renderer::Renderer;
use winit::{event::*, event_loop::ControlFlow};

#[allow(unreachable_patterns)]
pub fn handle_event(event: Event<()>, renderer: &mut Renderer, control_flow: &mut ControlFlow) {
    *control_flow = ControlFlow::Poll;

    match event {
        Event::MainEventsCleared => renderer.window().request_redraw(),
        Event::DeviceEvent {
            event: DeviceEvent::MouseMotion { delta },
            ..
        } => {
            if renderer.mouse_pressed {
                renderer.camera_controller().process_mouse(delta.0, delta.1)
            } else if renderer.mouse_released {
                renderer.mouse_released = false;
            }
        }
        Event::WindowEvent {
            ref event,
            window_id,
        } if window_id == renderer.window().id() && !renderer.input(event) => match event {
            #[cfg(not(target_arch = "wasm32"))]
            WindowEvent::CloseRequested
            | WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        state: ElementState::Pressed,
                        virtual_keycode: Some(VirtualKeyCode::Escape),
                        ..
                    },
                ..
            } => *control_flow = ControlFlow::Exit,
            WindowEvent::Resized(physical_size) => renderer.resize(*physical_size),
            WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                renderer.resize(**new_inner_size)
            }
            _ => {}
        },
        Event::RedrawRequested(window_id) if window_id == renderer.window().id() => {
            renderer.update();
            match renderer.render() {
                Ok(_) => {}
                Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                    renderer.resize(renderer.size)
                }
                Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                Err(wgpu::SurfaceError::Timeout) => log::warn!("Surface timeout"),
                _ => {}
            }
        }
        _ => {}
    }
}