use std::iter;

use winit::{dpi::PhysicalSize, event::*, window::Window};

use crate::app::window as app_window;
use crate::scenes::{Scene, wave_plane::WavePlaneScene, raymarch_volume::RaymarchVolumeScene};
use crate::texture;

pub mod shared;
use shared::SharedResources;

pub struct Renderer {
    window: Window,
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    pub size: PhysicalSize<u32>,
    depth_texture: texture::Texture,
    
    // Shared resources
    shared: SharedResources,
    
    // Current scene
    current_scene_index: usize,
    scenes: Vec<Box<dyn Scene>>,
    
    // Input state
    pub mouse_pressed: bool,
    pub mouse_released: bool,
}

impl Renderer {
    pub async fn new(window: Window) -> Self {
        let size = window.inner_size();
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            dx12_shader_compiler: Default::default(),
        });
        
        let (surface, config) = app_window::create_surface_and_config(&instance, &window).await;
        let (device, queue) = app_window::create_device_and_queue(&instance, &surface).await;
        
        surface.configure(&device, &config);
        
        let depth_texture = texture::Texture::create_depth_texture(&device, &config, "depth_texture");
        let shared = SharedResources::new(&device);
        
        // Initialize scenes
        let wave_plane = WavePlaneScene::new(&device, &config, &shared);
        let raymarch = RaymarchVolumeScene::new(&device, &config, &shared);

        let scenes: Vec<Box<dyn Scene>> = vec![
            Box::new(wave_plane),
            Box::new(raymarch),
        ];
        
        Self {
            window,
            surface,
            device,
            queue,
            config,
            size,
            depth_texture,
            shared,
            current_scene_index: 0,
            scenes,
            mouse_pressed: false,
            mouse_released: false,
        }
    }
    
    pub fn window(&self) -> &Window {
        &self.window
    }
    
    pub fn current_scene(&self) -> &dyn Scene {
        self.scenes[self.current_scene_index].as_ref()
    }
    
    pub fn current_scene_mut(&mut self) -> &mut Box<dyn Scene> {
        &mut self.scenes[self.current_scene_index]
    }
    
    pub fn next_scene(&mut self) {
        self.current_scene_index = (self.current_scene_index + 1) % self.scenes.len();
        log::info!("Switched to scene: {}", self.current_scene().name());
    }
    
    pub fn prev_scene(&mut self) {
        if self.current_scene_index == 0 {
            self.current_scene_index = self.scenes.len() - 1;
        } else {
            self.current_scene_index -= 1;
        }
        log::info!("Switched to scene: {}", self.current_scene().name());
    }
    
    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.shared.resize(new_size.width, new_size.height);
            self.surface.configure(&self.device, &self.config);
            self.depth_texture = texture::Texture::create_depth_texture(
                &self.device,
                &self.config,
                "depth_texture",
            );
            
            // Notify current scene
            self.scenes[self.current_scene_index].resize(&self.device, &self.queue, &self.config);
        }
    }
    
    pub fn input(&mut self, event: &WindowEvent) -> bool {
        match event {
            // Scene switching with arrow keys
            WindowEvent::KeyboardInput {
                input: KeyboardInput {
                    state: ElementState::Pressed,
                    virtual_keycode: Some(VirtualKeyCode::Right),
                    ..
                },
                ..
            } => {
                self.next_scene();
                true
            }
            WindowEvent::KeyboardInput {
                input: KeyboardInput {
                    state: ElementState::Pressed,
                    virtual_keycode: Some(VirtualKeyCode::Left),
                    ..
                },
                ..
            } => {
                self.prev_scene();
                true
            }
            // Toggle animations
            WindowEvent::KeyboardInput {
                input: KeyboardInput {
                    state: ElementState::Pressed,
                    virtual_keycode: Some(VirtualKeyCode::Return),
                    ..
                },
                ..
            } => {
                self.shared.animations_enabled = !self.shared.animations_enabled;
                true
            }
            // Mouse scroll for camera
            WindowEvent::MouseWheel { delta, .. } => {
                self.shared.camera_controller.process_scroll(delta);
                true
            }
            // Mouse press for camera rotation
            WindowEvent::MouseInput {
                button: MouseButton::Left,
                state,
                ..
            } => {
                self.mouse_pressed = *state == ElementState::Pressed;
                true
            }
            // Pass to current scene
            _ => self.scenes[self.current_scene_index].input(event),
        }
    }
    
    pub fn update(&mut self) {
        self.shared.update(&self.queue);
        
        let dt = 1.0 / 60.0; // TODO: proper delta time
        self.scenes[self.current_scene_index].update(dt, &self.queue, &self.shared);
    }
    
    // Add this method for camera control
    pub fn camera_controller(&mut self) -> &mut crate::camera::controller::CameraController {
        &mut self.shared.camera_controller
    }
    
    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });
        
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.01,
                            g: 0.01,
                            b: 0.01,
                            a: 1.0,
                        }),
                        store: true,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });
            
            self.scenes[self.current_scene_index].render(&mut render_pass, &self.shared);
        }
        
        self.queue.submit(iter::once(encoder.finish()));
        output.present();
        
        Ok(())
    }
    
    // Keep this for the lib.rs call, but it's now a no-op or handled in scene creation
    pub fn update_plane_vertices(&mut self) {
        // This is now handled in WavePlaneScene::new()
    }
}