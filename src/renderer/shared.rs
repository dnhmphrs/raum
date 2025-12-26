use cgmath::*;
use wgpu::util::DeviceExt;

use crate::camera;

/// Resources shared across all scenes
pub struct SharedResources {
    // Camera
    pub camera: camera::Camera,
    pub camera_controller: camera::controller::CameraController,
    pub camera_uniform: camera::uniform::CameraUniform,
    pub camera_buffer: wgpu::Buffer,
    pub camera_bind_group: wgpu::BindGroup,
    pub camera_bind_group_layout: wgpu::BindGroupLayout,
    
    // Time
    pub start_time: instant::Instant,
    pub current_time: f32,
    pub time_scale: f32,
    pub time_buffer: wgpu::Buffer,
    pub time_bind_group: wgpu::BindGroup,
    pub time_bind_group_layout: wgpu::BindGroupLayout,
    
    // Animation state
    pub animations_enabled: bool,
}

impl SharedResources {
    pub fn new(device: &wgpu::Device) -> Self {
        // Camera setup
        let camera = camera::Camera {
            eye: Point3::new(15.0, 10.0, 30.0),
            target: Point3::new(0.0, 0.0, 0.0),
            up: Vector3::new(0.0, 1.0, 0.0),
            aspect: 800.0 / 600.0,
            fovy: 20.0,
            znear: 0.1,
            zfar: 200.0,
        };
        let camera_controller = camera::controller::CameraController::new(1.0, 1.0, 2.0);
        
        let mut camera_uniform = camera::uniform::CameraUniform::new();
        camera_uniform.update(&camera);
        
        let camera_buffer = camera::binding::create_camera_buffer(device, &camera_uniform);
        let camera_bind_group_layout = camera::binding::create_bind_group_layout(device);
        let camera_bind_group = camera::binding::create_bind_group(
            device,
            &camera_buffer,
            &camera_bind_group_layout,
        );
        
        // Time setup
        let time_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Time Bind Group Layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: wgpu::BufferSize::new(16), // f32 + 3 padding
                },
                count: None,
            }],
        });
        
        let time_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Time Buffer"),
            contents: bytemuck::cast_slice(&[0.0f32, 0.0, 0.0, 0.0]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        
        let time_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &time_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: time_buffer.as_entire_binding(),
            }],
            label: Some("Time Bind Group"),
        });
        
        Self {
            camera,
            camera_controller,
            camera_uniform,
            camera_buffer,
            camera_bind_group,
            camera_bind_group_layout,
            start_time: instant::Instant::now(),
            current_time: 0.0,
            time_scale: 0.05,
            time_buffer,
            time_bind_group,
            time_bind_group_layout,
            animations_enabled: true,
        }
    }
    
    pub fn update(&mut self, queue: &wgpu::Queue) {
        // Update camera
        self.camera_controller.update_camera(&mut self.camera);
        self.camera_uniform.update(&self.camera);
        queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[self.camera_uniform]),
        );
        
        // Update time
        if self.animations_enabled {
            self.current_time = self.start_time.elapsed().as_secs_f32() * self.time_scale;
        }
        queue.write_buffer(
            &self.time_buffer,
            0,
            bytemuck::cast_slice(&[self.current_time, 0.0f32, 0.0, 0.0]),
        );
    }
    
    pub fn resize(&mut self, width: u32, height: u32) {
        self.camera.update_aspect(width, height);
    }
}