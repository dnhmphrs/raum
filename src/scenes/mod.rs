use winit::event::WindowEvent;
use crate::renderer::shared::SharedResources;

pub mod wave_plane;
pub mod raymarch_volume;

pub trait Scene {
    fn update(&mut self, dt: f32, queue: &wgpu::Queue, shared: &SharedResources);
    
    fn render<'a>(
        &'a self,
        render_pass: &mut wgpu::RenderPass<'a>,
        shared: &'a SharedResources,
    );
    
    fn input(&mut self, event: &WindowEvent) -> bool;
    
    fn resize(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        config: &wgpu::SurfaceConfiguration,
    );
    
    fn name(&self) -> &'static str;
}