use bytemuck;
use wgpu::util::DeviceExt;
use wgpu::{self, Device};

pub fn create_camera_buffer(
    device: &Device,
    camera_uniform: &super::uniform::CameraUniform,
) -> wgpu::Buffer {
    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Camera Buffer"),
        contents: bytemuck::cast_slice(&[*camera_uniform]),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    })
}

pub fn create_bind_group_layout(device: &Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }],
        label: Some("camera_bind_group_layout"),
    })
}

pub fn create_bind_group(
    device: &Device,
    camera_buffer: &wgpu::Buffer,
    bind_group_layout: &wgpu::BindGroupLayout,
) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        layout: bind_group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: camera_buffer.as_entire_binding(),
        }],
        label: Some("camera_bind_group"),
    })
}
