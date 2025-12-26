use wgpu::util::DeviceExt;
use winit::event::WindowEvent;

use super::Scene;
use crate::renderer::shared::SharedResources;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
#[allow(dead_code)]
struct FullscreenVertex {
    position: [f32; 2],
}

impl FullscreenVertex {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<FullscreenVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[wgpu::VertexAttribute {
                offset: 0,
                shader_location: 0,
                format: wgpu::VertexFormat::Float32x2,
            }],
        }
    }
}

// Fullscreen triangle (more efficient than quad)
const FULLSCREEN_VERTICES: &[FullscreenVertex] = &[
    FullscreenVertex { position: [-1.0, -1.0] },
    FullscreenVertex { position: [3.0, -1.0] },
    FullscreenVertex { position: [-1.0, 3.0] },
];

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct RaymarchUniforms {
    resolution: [f32; 2],
    _padding: [f32; 2],
}

pub struct RaymarchVolumeScene {
    pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    uniforms_buffer: wgpu::Buffer,
    uniforms_bind_group: wgpu::BindGroup,
    resolution: [f32; 2],
}

impl RaymarchVolumeScene {
    pub fn new(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        shared: &SharedResources,
    ) -> Self {
        // Uniforms for resolution
        let uniforms = RaymarchUniforms {
            resolution: [config.width as f32, config.height as f32],
            _padding: [0.0, 0.0],
        };

        let uniforms_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Raymarch Uniforms Buffer"),
            contents: bytemuck::cast_slice(&[uniforms]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let uniforms_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Raymarch Uniforms Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let uniforms_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Raymarch Uniforms Bind Group"),
            layout: &uniforms_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniforms_buffer.as_entire_binding(),
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Raymarch Pipeline Layout"),
            bind_group_layouts: &[
                &shared.camera_bind_group_layout,
                &shared.time_bind_group_layout,
                &uniforms_bind_group_layout,
            ],
            push_constant_ranges: &[],
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Raymarch Shader"),
            source: wgpu::ShaderSource::Wgsl(
                include_str!("../shaders/raymarch_shader.wgsl").into(),
            ),
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Raymarch Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[FullscreenVertex::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: crate::texture::Texture::DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Fullscreen Vertex Buffer"),
            contents: bytemuck::cast_slice(FULLSCREEN_VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        Self {
            pipeline,
            vertex_buffer,
            uniforms_buffer,
            uniforms_bind_group,
            resolution: [config.width as f32, config.height as f32],
        }
    }
}

impl Scene for RaymarchVolumeScene {
    fn update(&mut self, _dt: f32, _queue: &wgpu::Queue, _shared: &SharedResources) {
        // Nothing to update per-frame beyond what shared handles
    }

    fn render<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>, shared: &'a SharedResources) {
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &shared.camera_bind_group, &[]);
        render_pass.set_bind_group(1, &shared.time_bind_group, &[]);
        render_pass.set_bind_group(2, &self.uniforms_bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.draw(0..3, 0..1);
    }

    fn input(&mut self, _event: &WindowEvent) -> bool {
        false
    }

    fn resize(
        &mut self,
        _device: &wgpu::Device,
        queue: &wgpu::Queue,
        config: &wgpu::SurfaceConfiguration,
    ) {
        self.resolution = [config.width as f32, config.height as f32];
        let uniforms = RaymarchUniforms {
            resolution: self.resolution,
            _padding: [0.0, 0.0],
        };
        queue.write_buffer(&self.uniforms_buffer, 0, bytemuck::cast_slice(&[uniforms]));
    }

    fn name(&self) -> &'static str {
        "Raymarch Volume"
    }
}