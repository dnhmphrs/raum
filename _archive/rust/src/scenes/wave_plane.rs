use wgpu::util::DeviceExt;
use winit::event::WindowEvent;
use rand::Rng;

use super::Scene;
use crate::renderer::shared::SharedResources;

// Keep your existing vertex types here
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
#[allow(dead_code)]
struct PlaneVertex {
    position: [f32; 3],
    displacement: f32,
}

impl PlaneVertex {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<PlaneVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32,
                },
            ],
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
#[allow(dead_code)]
struct LineVertex {
    position: [f32; 3],
}

impl LineVertex {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<LineVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[wgpu::VertexAttribute {
                offset: 0,
                shader_location: 0,
                format: wgpu::VertexFormat::Float32x3,
            }],
        }
    }
}

const GRID_SIZE: usize = 100;

pub struct WavePlaneScene {
    // Plane
    plane_pipeline: wgpu::RenderPipeline,
    plane_vertex_buffer: wgpu::Buffer,
    plane_index_buffer: wgpu::Buffer,
    plane_num_indices: u32,
    
    // Cube wireframe
    line_pipeline: wgpu::RenderPipeline,
    line_vertex_buffer: wgpu::Buffer,
    line_index_buffer: wgpu::Buffer,
    line_num_indices: u32,
    
    // Wells
    well_pipeline: wgpu::RenderPipeline,
    well_vertex_buffer: wgpu::Buffer,
    well_index_buffer: wgpu::Buffer,
    well_num_indices: u32,
}

impl WavePlaneScene {
    pub fn new(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        shared: &SharedResources,
    ) -> Self {
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Wave Plane Pipeline Layout"),
            bind_group_layouts: &[
                &shared.camera_bind_group_layout,
                &shared.time_bind_group_layout,
            ],
            push_constant_ranges: &[],
        });
        
        // Generate plane geometry
        let plane_vertices = Self::generate_plane_vertices();
        let plane_indices = Self::generate_plane_indices();
        
        let plane_vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Plane Vertex Buffer"),
            contents: bytemuck::cast_slice(&plane_vertices),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });
        
        let plane_index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Plane Index Buffer"),
            contents: bytemuck::cast_slice(&plane_indices),
            usage: wgpu::BufferUsages::INDEX,
        });
        
        // Plane shader and pipeline
        let plane_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Plane Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/plane_shader.wgsl").into()),
        });
        
        let plane_pipeline = Self::create_triangle_pipeline(
            device,
            config,
            &pipeline_layout,
            &plane_shader,
            PlaneVertex::desc(),
            "Plane Pipeline",
        );
        
        // Cube wireframe
        let cube_edges = Self::cube_edges();
        let cube_indices = Self::cube_edge_indices();
        
        let line_vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Line Vertex Buffer"),
            contents: bytemuck::cast_slice(&cube_edges),
            usage: wgpu::BufferUsages::VERTEX,
        });
        
        let line_index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Line Index Buffer"),
            contents: bytemuck::cast_slice(&cube_indices),
            usage: wgpu::BufferUsages::INDEX,
        });
        
        let line_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Line Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/line_shader.wgsl").into()),
        });
        
        let line_pipeline = Self::create_line_pipeline(
            device,
            config,
            &pipeline_layout,
            &line_shader,
            "Line Pipeline",
        );
        
        // Wells
        let (well_vertices, well_indices) = Self::generate_wells();
        
        let well_vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Well Vertex Buffer"),
            contents: bytemuck::cast_slice(&well_vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        
        let well_index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Well Index Buffer"),
            contents: bytemuck::cast_slice(&well_indices),
            usage: wgpu::BufferUsages::INDEX,
        });
        
        let well_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Well Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/well_shader.wgsl").into()),
        });
        
        let well_pipeline = Self::create_line_pipeline_blended(
            device,
            config,
            &pipeline_layout,
            &well_shader,
            "Well Pipeline",
        );
        
        Self {
            plane_pipeline,
            plane_vertex_buffer,
            plane_index_buffer,
            plane_num_indices: plane_indices.len() as u32,
            line_pipeline,
            line_vertex_buffer,
            line_index_buffer,
            line_num_indices: cube_indices.len() as u32,
            well_pipeline,
            well_vertex_buffer,
            well_index_buffer,
            well_num_indices: well_indices.len() as u32,
        }
    }
    
    fn generate_plane_vertices() -> Vec<PlaneVertex> {
        let mut vertices = Vec::new();
        let mut rng = rand::thread_rng();
        
        for x in 0..GRID_SIZE {
            for z in 0..GRID_SIZE {
                let x_pos = (x as f32 / (GRID_SIZE - 1) as f32) * 10.0 - 5.0;
                let z_pos = (z as f32 / (GRID_SIZE - 1) as f32) * 10.0 - 5.0;
                
                let distance = (x_pos * x_pos + z_pos * z_pos + rng.gen_range(0.0..0.05)).sqrt();
                let wave_freq = 0.05 + rng.gen_range(0.0..0.01);
                let wave_freq_2 = 0.5 + rng.gen_range(0.0..0.01);
                let displacement = 3.0
                    * (x_pos * wave_freq - wave_freq_2 * distance).sin()
                    * (z_pos * wave_freq - wave_freq_2 * distance).cos();
                
                vertices.push(PlaneVertex {
                    position: [x_pos, 0.0, z_pos],
                    displacement,
                });
            }
        }
        vertices
    }
    
    fn generate_plane_indices() -> Vec<u16> {
        let mut indices = Vec::new();
        for x in 0..GRID_SIZE - 1 {
            for z in 0..GRID_SIZE - 1 {
                let start = x * GRID_SIZE + z;
                indices.extend(&[
                    start as u16,
                    (start + GRID_SIZE) as u16,
                    (start + 1) as u16,
                    (start + 1) as u16,
                    (start + GRID_SIZE) as u16,
                    (start + GRID_SIZE + 1) as u16,
                ]);
            }
        }
        indices
    }
    
    fn cube_edges() -> Vec<LineVertex> {
        vec![
            LineVertex { position: [-5.0, -5.0, -5.0] },
            LineVertex { position: [5.0, -5.0, -5.0] },
            LineVertex { position: [-5.0, 5.0, -5.0] },
            LineVertex { position: [5.0, 5.0, -5.0] },
            LineVertex { position: [-5.0, -5.0, 5.0] },
            LineVertex { position: [5.0, -5.0, 5.0] },
            LineVertex { position: [-5.0, 5.0, 5.0] },
            LineVertex { position: [5.0, 5.0, 5.0] },
        ]
    }
    
    fn cube_edge_indices() -> Vec<u16> {
        vec![
            0, 1, 1, 3, 3, 2, 2, 0, // front
            4, 5, 5, 7, 7, 6, 6, 4, // back
            0, 4, 1, 5, 2, 6, 3, 7, // connecting
        ]
    }
    
    fn generate_wells() -> (Vec<LineVertex>, Vec<u16>) {
        let num_lines = 10;
        let mut vertices = Vec::new();
        
        for i in 0..num_lines {
            let x = i as f32 - (num_lines as f32 / 2.0) + 0.5;
            vertices.push(LineVertex { position: [x, -5.0, 0.0] });
            vertices.push(LineVertex { position: [x, 5.0, 0.0] });
        }
        
        let indices: Vec<u16> = (0..(num_lines * 2) as u16).collect();
        (vertices, indices)
    }
    
    fn create_triangle_pipeline(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        layout: &wgpu::PipelineLayout,
        shader: &wgpu::ShaderModule,
        vertex_layout: wgpu::VertexBufferLayout<'static>,
        label: &str,
    ) -> wgpu::RenderPipeline {
        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some(label),
            layout: Some(layout),
            vertex: wgpu::VertexState {
                module: shader,
                entry_point: "vs_main",
                buffers: &[vertex_layout],
            },
            fragment: Some(wgpu::FragmentState {
                module: shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                cull_mode: None,
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
        })
    }
    
    fn create_line_pipeline(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        layout: &wgpu::PipelineLayout,
        shader: &wgpu::ShaderModule,
        label: &str,
    ) -> wgpu::RenderPipeline {
        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some(label),
            layout: Some(layout),
            vertex: wgpu::VertexState {
                module: shader,
                entry_point: "vs_main",
                buffers: &[LineVertex::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::LineList,
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
        })
    }
    
    fn create_line_pipeline_blended(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        layout: &wgpu::PipelineLayout,
        shader: &wgpu::ShaderModule,
        label: &str,
    ) -> wgpu::RenderPipeline {
        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some(label),
            layout: Some(layout),
            vertex: wgpu::VertexState {
                module: shader,
                entry_point: "vs_main",
                buffers: &[LineVertex::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::SrcAlpha,
                            dst_factor: wgpu::BlendFactor::One,
                            operation: wgpu::BlendOperation::Add,
                        },
                        alpha: wgpu::BlendComponent::REPLACE,
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::LineList,
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
        })
    }
}

impl Scene for WavePlaneScene {
    fn update(&mut self, _dt: f32, _queue: &wgpu::Queue, _shared: &SharedResources) {
        // Scene-specific updates if needed
    }
    
    fn render<'a>(
        &'a self,
        render_pass: &mut wgpu::RenderPass<'a>,
        shared: &'a SharedResources,
    ) {
        // Plane
        render_pass.set_pipeline(&self.plane_pipeline);
        render_pass.set_bind_group(0, &shared.camera_bind_group, &[]);
        render_pass.set_bind_group(1, &shared.time_bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.plane_vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.plane_index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.draw_indexed(0..self.plane_num_indices, 0, 0..1);
        
        // Cube wireframe
        render_pass.set_pipeline(&self.line_pipeline);
        render_pass.set_bind_group(0, &shared.camera_bind_group, &[]);
        render_pass.set_bind_group(1, &shared.time_bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.line_vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.line_index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.draw_indexed(0..self.line_num_indices, 0, 0..1);
        
        // Wells
        render_pass.set_pipeline(&self.well_pipeline);
        render_pass.set_bind_group(0, &shared.camera_bind_group, &[]);
        render_pass.set_bind_group(1, &shared.time_bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.well_vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.well_index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.draw_indexed(0..self.well_num_indices, 0, 0..1);
    }
    
    fn input(&mut self, _event: &WindowEvent) -> bool {
        false
    }
    
    fn resize(
        &mut self,
        _device: &wgpu::Device,
        _queue: &wgpu::Queue,
        _config: &wgpu::SurfaceConfiguration,
    ) {
        // No scene-specific resize needed
    }
    
    fn name(&self) -> &'static str {
        "Wave Plane"
    }
}