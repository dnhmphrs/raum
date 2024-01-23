use std::iter;

// use cgmath::prelude::*;
use cgmath::*;
use rand::Rng;
use std::time::Instant;
use wgpu::util::DeviceExt;
use winit::{dpi::PhysicalSize, event::*, window::Window};

use crate::app::window;
use crate::camera;
use crate::texture;

// use geometry::instance::{Instance, InstanceRaw, INSTANCE_DISPLACEMENT, NUM_INSTANCES_PER_ROW};
// use geometry::vertex::{Vertex, INDICES, VERTICES};

use camera::controller::CameraController;

// --------------------------------------
// time
// --------------------------------------

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct TimeUniform {
    time: f32,
}

// --------------------------------------
// main cube
// --------------------------------------

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    position: [f32; 3],
}

impl Vertex {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }
}

pub const VERTICES: &[Vertex] = &[
    Vertex {
        position: [-5.0, -5.0, -5.0],
    },
    Vertex {
        position: [-5.0, -5.0, 5.0],
    },
    Vertex {
        position: [-5.0, 5.0, -5.0],
    },
    Vertex {
        position: [-5.0, 5.0, 5.0],
    },
    Vertex {
        position: [5.0, -5.0, -5.0],
    },
    Vertex {
        position: [5.0, -5.0, 5.0],
    },
    Vertex {
        position: [5.0, 5.0, -5.0],
    },
    Vertex {
        position: [5.0, 5.0, 5.0],
    },
];

pub const INDICES: &[u16] = &[
    0, 1, 2, // front face
    2, 1, 3, 4, 5, 6, // back face
    6, 5, 7, 0, 1, 4, // bottom face
    4, 1, 5, 2, 3, 6, // top face
    6, 3, 7, 0, 2, 4, // left face
    4, 2, 6, 1, 3, 5, // right face
    5, 3, 7,
];

// --------------------------------------
// plane
// --------------------------------------

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct PlaneVertex {
    position: [f32; 3],
    displacement: f32, // Add displacement attribute
}

// Update the vertex descriptor to include the displacement attribute
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

const GRID_SIZE: usize = 100; // Grid size (e.g., 10x10)

fn generate_grid_vertices() -> Vec<PlaneVertex> {
    let mut vertices = Vec::new();
    for x in 0..GRID_SIZE {
        for z in 0..GRID_SIZE {
            let x_pos = (x as f32 / (GRID_SIZE - 1) as f32) * 10.0 - 5.0; // Map to range [-5.0, 5.0]
            let z_pos = (z as f32 / (GRID_SIZE - 1) as f32) * 10.0 - 5.0; // Map to range [-5.0, 5.0]

            vertices.push(PlaneVertex {
                position: [x_pos, 0.0, z_pos],
                displacement: 0.0, // Initial displacement
            });
        }
    }
    vertices
}

fn generate_grid_indices() -> Vec<u16> {
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

lazy_static::lazy_static! {
    static ref PLANE_VERTICES: Vec<PlaneVertex> = generate_grid_vertices();
    static ref PLANE_INDICES: Vec<u16> = generate_grid_indices();
}

// --------------------------------------
// lines
// --------------------------------------

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct LineVertex {
    position: [f32; 3],
}

impl LineVertex {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<LineVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                // Add more attributes here if your LineVertex has more data
            ],
        }
    }
}

// --------------------------------------
// wireframe cube
// --------------------------------------

const CUBE_EDGES: &[LineVertex] = &[
    LineVertex {
        position: [-5.0, -5.0, -5.0],
    }, // 0: Bottom-front-left
    LineVertex {
        position: [5.0, -5.0, -5.0],
    }, // 1: Bottom-front-right
    LineVertex {
        position: [-5.0, 5.0, -5.0],
    }, // 2: Top-front-left
    LineVertex {
        position: [5.0, 5.0, -5.0],
    }, // 3: Top-front-right
    LineVertex {
        position: [-5.0, -5.0, 5.0],
    }, // 4: Bottom-back-left
    LineVertex {
        position: [5.0, -5.0, 5.0],
    }, // 5: Bottom-back-right
    LineVertex {
        position: [-5.0, 5.0, 5.0],
    }, // 6: Top-back-left
    LineVertex {
        position: [5.0, 5.0, 5.0],
    }, // 7: Top-back-right
];

const CUBE_EDGE_INDICES: &[u16] = &[
    0, 1, // Front bottom edge
    1, 3, // Front right edge
    3, 2, // Front top edge
    2, 0, // Front left edge
    4, 5, // Back bottom edge
    5, 7, // Back right edge
    7, 6, // Back top edge
    6, 4, // Back left edge
    0, 4, // Bottom left vertical edge
    1, 5, // Bottom right vertical edge
    2, 6, // Top left vertical edge
    3, 7, // Top right vertical edge
];

// --------------------------------------
// wells
// --------------------------------------

const NUM_LINES: usize = 10;
const VERTICES_PER_LINE: usize = 2; // just 2

fn generate_line_vertices() -> Vec<LineVertex> {
    let mut vertices = Vec::new();
    for line_index in 0..NUM_LINES {
        let x_position = line_index as f32 - (NUM_LINES as f32 / 2.0) + 0.5; // Spread lines along the x-axis
        for vertex_index in 0..VERTICES_PER_LINE {
            let y_position = vertex_index as f32 * 10.0 - 5.0; // Vertical position
            vertices.push(LineVertex {
                position: [x_position, y_position, 0.0], // All lines are along the z-axis
            });
        }
    }
    vertices
}

pub struct Renderer {
    window: Window,
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    pub size: PhysicalSize<u32>,
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    num_indices: u32,
    camera: camera::Camera,
    pub camera_controller: CameraController,
    camera_uniform: camera::uniform::CameraUniform,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    // instances: Vec<Instance>,
    // instance_buffer: wgpu::Buffer,
    depth_texture: crate::texture::Texture,
    // time
    start_time: Instant,
    current_time: f32,
    time_scale: f32,
    time_buffer: wgpu::Buffer,
    time_bind_group: wgpu::BindGroup,
    // plane
    plane_render_pipeline: wgpu::RenderPipeline,
    plane_vertex_buffer: wgpu::Buffer,
    plane_index_buffer: wgpu::Buffer,
    plane_num_indices: u32,
    // cube
    line_render_pipeline: wgpu::RenderPipeline,
    line_vertex_buffer: wgpu::Buffer,
    line_index_buffer: wgpu::Buffer,
    num_line_indices: u32,
    // wells
    well_render_pipeline: wgpu::RenderPipeline,
    well_vertex_buffer: wgpu::Buffer,
    well_index_buffer: wgpu::Buffer,
    well_num_line_indices: u32,
    // interactions
    pub mouse_pressed: bool,
    render_plane: bool,
    animations: bool,
    pub mouse_released: bool,
}

impl Renderer {
    pub async fn new(window: Window) -> Self {
        // size, instance
        let size = window.inner_size();
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            dx12_shader_compiler: Default::default(),
        });

        // surface, device, and queue
        let (surface, config) = window::create_surface_and_config(&instance, &window).await;
        let (device, queue) = window::create_device_and_queue(&instance, &surface).await;

        // camera
        let camera = camera::Camera {
            eye: Point3::new(30.0, 20.0, 30.0),
            target: Point3::new(0.0, 0.0, 0.0),
            up: Vector3::new(0.0, 1.0, 0.0),
            aspect: 800.0 / 600.0,
            fovy: 20.0,
            znear: 0.1,
            zfar: 200.0,
        };
        let camera_controller = CameraController::new(1.0, 1.0, 2.0);

        // time uniform
        let time_uniform = TimeUniform { time: 0.0 };
        let time_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Time Buffer"),
            contents: bytemuck::cast_slice(&[time_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        // camera uniform, buffer, and bind group
        let mut camera_uniform = camera::uniform::CameraUniform::new();
        camera_uniform.update(&camera);

        let camera_buffer = camera::binding::create_camera_buffer(&device, &camera_uniform);
        let camera_bind_group_layout = camera::binding::create_bind_group_layout(&device);
        let camera_bind_group =
            camera::binding::create_bind_group(&device, &camera_buffer, &camera_bind_group_layout);

        // time
        let start_time = Instant::now();

        let time_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Time Bind Group Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(
                            std::mem::size_of::<TimeUniform>() as _
                        ),
                    },
                    count: None,
                }],
            });

        let time_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &time_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: time_buffer.as_entire_binding(),
            }],
            label: Some("Time Bind Group"),
        });

        // let texture_bind_group_layout =
        //     device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        //         entries: &[wgpu::BindGroupLayoutEntry {
        //             binding: 0,
        //             visibility: wgpu::ShaderStages::FRAGMENT,
        //             ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
        //             count: None,
        //         }],
        //         label: Some("texture_bind_group_layout"),
        //     });

        // let diffuse_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        //     layout: &texture_bind_group_layout,
        //     entries: &[
        //         wgpu::BindGroupEntry {
        //             binding: 0,
        //             resource: wgpu::BindingResource::TextureView(&diffuse_texture.view),
        //         },
        //         wgpu::BindGroupEntry {
        //             binding: 1,
        //             resource: wgpu::BindingResource::Sampler(&diffuse_texture.sampler),
        //         },
        //     ],
        //     label: Some("diffuse_bind_group"),
        // });

        // let instances = (0..NUM_INSTANCES_PER_ROW)
        //     .flat_map(|z| {
        //         (0..NUM_INSTANCES_PER_ROW).flat_map(move |x| {
        //             (0..NUM_INSTANCES_PER_ROW).map(move |y| {
        //                 let position = cgmath::Vector3 {
        //                     x: x as f32,
        //                     y: y as f32,
        //                     z: z as f32,
        //                 } - INSTANCE_DISPLACEMENT;

        //                 let rotation = if position.is_zero() {
        //                     // this is needed so an object at (0, 0, 0) won't get scaled to zero
        //                     // as Quaternions can effect scale if they're not created correctly
        //                     cgmath::Quaternion::from_axis_angle(
        //                         cgmath::Vector3::unit_z(),
        //                         cgmath::Deg(0.0),
        //                     )
        //                 } else {
        //                     cgmath::Quaternion::from_axis_angle(
        //                         position.normalize(),
        //                         cgmath::Deg(0.0),
        //                     )
        //                 };

        //                 Instance { position, rotation }
        //             })
        //         })
        //     })
        //     .collect::<Vec<_>>();

        // let instance_data = instances.iter().map(Instance::to_raw).collect::<Vec<_>>();
        // let instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        //     label: Some("Instance Buffer"),
        //     contents: bytemuck::cast_slice(&instance_data),
        //     usage: wgpu::BufferUsages::VERTEX,
        // });

        let cube_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/cube_shader.wgsl").into()),
        });

        let depth_texture =
            texture::Texture::create_depth_texture(&device, &config, "depth_texture");

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&camera_bind_group_layout, &time_bind_group_layout],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &cube_shader,
                entry_point: "vs_main",
                buffers: &[Vertex::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &cube_shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent::REPLACE,
                        alpha: wgpu::BlendComponent::REPLACE,
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                // Setting this to anything other than Fill requires Features::POLYGON_MODE_LINE
                // or Features::POLYGON_MODE_POINT
                polygon_mode: wgpu::PolygonMode::Fill,
                // Requires Features::DEPTH_CLIP_CONTROL
                unclipped_depth: false,
                // Requires Features::CONSERVATIVE_RASTERIZATION
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: texture::Texture::DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            // If the pipeline will be used with a multiview render pass, this
            // indicates how many array layers the attachments will have.
            multiview: None,
        });

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });
        let num_indices = INDICES.len() as u32;

        // -----------------------------------------
        // plane
        // -----------------------------------------

        let plane_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/plane_shader.wgsl").into()),
        });

        let plane_render_pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Render Pipeline"),
                layout: Some(&render_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &plane_shader,
                    entry_point: "vs_main",
                    buffers: &[PlaneVertex::desc()],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &plane_shader,
                    entry_point: "fs_main",
                    targets: &[Some(wgpu::ColorTargetState {
                        format: config.format,
                        blend: Some(wgpu::BlendState {
                            color: wgpu::BlendComponent::REPLACE,
                            alpha: wgpu::BlendComponent::REPLACE,
                        }),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: None,
                    // Setting this to anything other than Fill requires Features::POLYGON_MODE_LINE
                    // or Features::POLYGON_MODE_POINT
                    polygon_mode: wgpu::PolygonMode::Fill,
                    // Requires Features::DEPTH_CLIP_CONTROL
                    unclipped_depth: false,
                    // Requires Features::CONSERVATIVE_RASTERIZATION
                    conservative: false,
                },
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: texture::Texture::DEPTH_FORMAT,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::Less,
                    stencil: wgpu::StencilState::default(),
                    bias: wgpu::DepthBiasState::default(),
                }),
                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                // If the pipeline will be used with a multiview render pass, this
                // indicates how many array layers the attachments will have.
                multiview: None,
            });

        let plane_vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&PLANE_VERTICES[..]), // should be just 'VERTICES2' without lazystatic
            usage: wgpu::BufferUsages::VERTEX,
        });
        let plane_index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(&PLANE_INDICES[..]), // should be just 'INDICES2' without lazystatic
            usage: wgpu::BufferUsages::INDEX,
        });
        let plane_num_indices = PLANE_INDICES.len() as u32;

        // -----------------------------------------
        // cube
        // -----------------------------------------

        let line_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/line_shader.wgsl").into()),
        });

        let line_vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Line Vertex Buffer"),
            contents: bytemuck::cast_slice(CUBE_EDGES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let line_index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Line Index Buffer"),
            contents: bytemuck::cast_slice(CUBE_EDGE_INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });

        let num_line_indices = CUBE_EDGE_INDICES.len() as u32;

        let line_render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &line_shader,
                entry_point: "vs_main",
                buffers: &[LineVertex::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &line_shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent::REPLACE,
                        alpha: wgpu::BlendComponent::REPLACE,
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::LineList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                // Setting this to anything other than Fill requires Features::POLYGON_MODE_LINE
                // or Features::POLYGON_MODE_POINT
                polygon_mode: wgpu::PolygonMode::Fill,
                // Requires Features::DEPTH_CLIP_CONTROL
                unclipped_depth: false,
                // Requires Features::CONSERVATIVE_RASTERIZATION
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: texture::Texture::DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            // If the pipeline will be used with a multiview render pass, this
            // indicates how many array layers the attachments will have.
            multiview: None,
        });

        // -----------------------------------------
        // wells
        // -----------------------------------------

        let well_vertices = generate_line_vertices();

        let well_vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Line Vertex Buffer"),
            contents: bytemuck::cast_slice(&well_vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let well_indices: Vec<u16> = (0..(NUM_LINES * VERTICES_PER_LINE) as u16).collect();
        let well_num_line_indices = well_indices.len() as u32;

        let well_index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Line Index Buffer"),
            contents: bytemuck::cast_slice(&well_indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        let well_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/well_shader.wgsl").into()),
        });

        let well_render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &well_shader,
                entry_point: "vs_main",
                buffers: &[LineVertex::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &well_shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::SrcAlpha,
                            dst_factor: wgpu::BlendFactor::One,
                            operation: wgpu::BlendOperation::Add,
                        },
                        alpha: wgpu::BlendComponent::REPLACE, // Or another suitable operation for alpha
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::LineList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                // Setting this to anything other than Fill requires Features::POLYGON_MODE_LINE
                // or Features::POLYGON_MODE_POINT
                polygon_mode: wgpu::PolygonMode::Fill,
                // Requires Features::DEPTH_CLIP_CONTROL
                unclipped_depth: false,
                // Requires Features::CONSERVATIVE_RASTERIZATION
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: texture::Texture::DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            // If the pipeline will be used with a multiview render pass, this
            // indicates how many array layers the attachments will have.
            multiview: None,
        });

        Self {
            window,
            surface,
            device,
            // last_render_time,
            queue,
            config,
            size,
            render_pipeline,
            vertex_buffer,
            index_buffer,
            num_indices,
            // diffuse_texture,
            // diffuse_bind_group,
            camera,
            camera_controller,
            camera_buffer,
            camera_bind_group,
            camera_uniform,
            // instances,
            // instance_buffer,
            depth_texture,
            // Time
            start_time,
            current_time: 0.0,
            time_scale: 0.05,
            time_buffer,
            time_bind_group,
            // plane
            plane_render_pipeline,
            plane_vertex_buffer,
            plane_index_buffer,
            plane_num_indices,
            // cube
            line_render_pipeline,
            line_vertex_buffer,
            line_index_buffer,
            num_line_indices,
            // well
            well_render_pipeline,
            well_vertex_buffer,
            well_index_buffer,
            well_num_line_indices,
            // interactions
            mouse_pressed: false,
            animations: true,
            render_plane: true,
            mouse_released: false,
        }
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
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
                            r: 0.015,
                            g: 0.015,
                            b: 0.015,
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

            if self.render_plane {
                // plane
                render_pass.set_pipeline(&self.plane_render_pipeline);
                render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
                render_pass.set_bind_group(1, &self.time_bind_group, &[]); // Use the correct index for the bind group

                render_pass.set_vertex_buffer(0, self.plane_vertex_buffer.slice(..));
                render_pass
                    .set_index_buffer(self.plane_index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                render_pass.draw_indexed(0..self.plane_num_indices, 0, 0..1);
                // cube
                render_pass.set_pipeline(&self.line_render_pipeline); // Use the line rendering pipeline
                render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
                render_pass.set_bind_group(1, &self.time_bind_group, &[]); // Use the correct index for the bind group

                render_pass.set_vertex_buffer(0, self.line_vertex_buffer.slice(..));
                render_pass
                    .set_index_buffer(self.line_index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                render_pass.draw_indexed(0..self.num_line_indices, 0, 0..1);
                // wells
                render_pass.set_pipeline(&self.well_render_pipeline); // Use the line rendering pipeline
                render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
                render_pass.set_bind_group(1, &self.time_bind_group, &[]); // Use the correct index for the bind group

                render_pass.set_vertex_buffer(0, self.well_vertex_buffer.slice(..));
                render_pass
                    .set_index_buffer(self.well_index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                render_pass.draw_indexed(0..self.well_num_line_indices, 0, 0..1);
            } else {
                render_pass.set_pipeline(&self.render_pipeline);
                render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
                render_pass.set_bind_group(1, &self.time_bind_group, &[]); // Use the correct index for the bind group

                render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
                render_pass
                    .set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                render_pass.draw_indexed(0..self.num_indices, 0, 0..1);
            }
        }

        self.queue.submit(iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    pub fn update_plane_vertices(&mut self) {
        let peak_height = 3.0; // Height of the peak

        let mut new_vertices = Vec::new();
        for vertex in PLANE_VERTICES.iter() {
            // Calculate the distance from the center of the plane
            let distance = ((vertex.position[0] * vertex.position[0])
                + (vertex.position[2] * vertex.position[2])
                + Rng::gen_range(&mut rand::thread_rng(), 0.0..0.05))
            .sqrt();

            let wave_frequency = 0.05 + Rng::gen_range(&mut rand::thread_rng(), 0.0..0.01);
            let wave_frequency_2 = 0.5 + Rng::gen_range(&mut rand::thread_rng(), 0.0..0.01);
            // Calculate the displacement using a Gaussian-like function
            let displacement = peak_height
                * (vertex.position[0] * wave_frequency - wave_frequency_2 * distance).sin()
                * (vertex.position[2] * wave_frequency - wave_frequency_2 * distance).cos();

            new_vertices.push(PlaneVertex {
                position: vertex.position,
                displacement,
            });
        }

        self.plane_vertex_buffer =
            self.device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Vertex Buffer 2"),
                    contents: bytemuck::cast_slice(&new_vertices),
                    usage: wgpu::BufferUsages::VERTEX,
                });
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.camera.update_aspect(new_size.width, new_size.height);
            self.surface.configure(&self.device, &self.config);
            self.depth_texture =
                texture::Texture::create_depth_texture(&self.device, &self.config, "depth_texture");
        }
    }

    pub fn input(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        state,
                        virtual_keycode: Some(VirtualKeyCode::Space),
                        ..
                    },
                ..
            } => {
                if *state == ElementState::Pressed {
                    self.render_plane = !self.render_plane;
                    // self.update_plane_vertices();
                }
                true
            }
            WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        state,
                        virtual_keycode: Some(VirtualKeyCode::Return),
                        ..
                    },
                ..
            } => {
                if *state == ElementState::Pressed {
                    self.animations = !self.animations;
                    // self.update_plane_vertices();
                }
                true
            }
            // WindowEvent::KeyboardInput {
            //     input:
            //         KeyboardInput {
            //             virtual_keycode: Some(key),
            //             state,
            //             ..
            //         },
            //     ..
            // } => self.camera_controller.process_keyboard(*key, *state),
            WindowEvent::MouseWheel { delta, .. } => {
                self.camera_controller.process_scroll(delta);
                true
            }
            WindowEvent::MouseInput {
                button: MouseButton::Left,
                state,
                ..
            } => {
                self.mouse_pressed = *state == ElementState::Pressed;
                true
            }
            _ => false,
        }
    }

    pub fn update(&mut self) {
        self.camera_controller.update_camera(&mut self.camera);
        self.camera_uniform.update(&self.camera);

        if self.animations {
            let elapsed = self.start_time.elapsed();
            let scaled_time = elapsed.as_secs_f32() * self.time_scale; // Scale the time
            self.current_time = scaled_time;
        }

        self.queue.write_buffer(
            &self.time_buffer,
            0,
            bytemuck::cast_slice(&[TimeUniform {
                time: self.current_time,
            }]),
        );

        self.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[self.camera_uniform]),
        );
    }
}
