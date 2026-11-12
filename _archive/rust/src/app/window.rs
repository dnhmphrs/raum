use winit::window::{Icon, Window, WindowBuilder};
use winit::{dpi::PhysicalSize, event_loop::EventLoop};

// #[cfg(target_arch = "wasm32")]
// use web_sys::window;

#[cfg(target_arch = "wasm32")]
use winit::platform::web::WindowExtWebSys;

pub struct WindowSize {
    pub width: u32,
    pub height: u32,
}

pub fn create_window(event_loop: &EventLoop<()>, initial_size: WindowSize) -> Window {
    let img = include_bytes!("../../res/icon.png");
    let icon = image::load_from_memory(img).unwrap();

    let title = env!("CARGO_PKG_NAME").to_uppercase();
    let window = WindowBuilder::new()
        // .with_maximized(true)
        .with_title(title)
        .build(event_loop)
        .expect("Failed to create a window");

    //  #[cfg(target_arch = "wasm32")]
    {
        // Winit prevents sizing with CSS, so we have to set
        // the size manually when on web.
        window.set_inner_size(PhysicalSize::new(initial_size.width, initial_size.height));

        window.set_window_icon(Some(
            Icon::from_rgba(icon.to_rgba8().into_raw(), icon.width(), icon.height()).unwrap(),
        ));

        #[cfg(target_arch = "wasm32")]
        web_sys::window()
            .and_then(|win| win.document())
            .and_then(|doc| {
                let dst = doc.get_element_by_id("wasm-example")?;
                let canvas = web_sys::Element::from(window.canvas());
                dst.append_child(&canvas).ok()?;
                Some(())
            })
            .expect("Couldn't append canvas to document body.");

        // #[cfg(target_arch = "wasm32")]
        // if let Some(browser_window) = web_sys::window() {
        //     let width = browser_window.inner_width().unwrap().as_f64().unwrap() as u32;
        //     let height = browser_window.inner_height().unwrap().as_f64().unwrap() as u32;

        //     window.set_inner_size(PhysicalSize::new(width, height));
        // }
    }

    window
}

pub async fn create_surface_and_config(
    instance: &wgpu::Instance,
    window: &Window,
) -> (wgpu::Surface, wgpu::SurfaceConfiguration) {
    let surface = unsafe { instance.create_surface(window) }.unwrap();
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        })
        .await
        .unwrap();

    let surface_caps = surface.get_capabilities(&adapter);
    let surface_format = surface_caps
        .formats
        .iter()
        .copied()
        .find(|f| f.is_srgb())
        .unwrap_or(surface_caps.formats[0]);

    let config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: surface_format,
        width: window.inner_size().width,
        height: window.inner_size().height,
        present_mode: surface_caps.present_modes[0],
        alpha_mode: surface_caps.alpha_modes[0],
        view_formats: vec![],
    };

    (surface, config)
}

pub async fn create_device_and_queue(
    instance: &wgpu::Instance,
    surface: &wgpu::Surface,
) -> (wgpu::Device, wgpu::Queue) {
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        })
        .await
        .unwrap();

    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                features: wgpu::Features::empty(),
                limits: if cfg!(target_arch = "wasm32") {
                    wgpu::Limits::downlevel_webgl2_defaults()
                } else {
                    wgpu::Limits::default()
                },
            },
            None,
        )
        .await
        .unwrap();

    (device, queue)
}
