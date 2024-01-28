use std::iter;

// use cgmath::prelude::*;
use cgmath::*;
// use rand::Rng;
use std::time::Instant;
use wgpu::util::DeviceExt;
use winit::{dpi::PhysicalSize, event::*, window::Window};

use crate::app::window;
use crate::camera;
use crate::texture;

use camera::controller::CameraController;

// --------------------------------------
// read signal
// --------------------------------------

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct SignalData {
    time: f32,
    slow_wave: f32,
    delta_wave: f32,
    theta_wave: f32,
    alpha_wave: f32,
    beta_wave: f32,
    gamma_wave: f32,
    swr: f32,
    ied: f32,
    composite_signal: f32,
}

use csv;
use std::error::Error;
use std::fs::File;

fn read_signals_from_csv(file_path: &str) -> Result<Vec<SignalData>, Box<dyn Error>> {
    let mut rdr = csv::Reader::from_reader(File::open(file_path)?);
    let mut signals = Vec::new();

    for result in rdr.records() {
        let record = result?;
        let signal_data = SignalData {
            time: record[0].parse()?,
            slow_wave: record[1].parse()?,
            delta_wave: record[2].parse()?,
            theta_wave: record[3].parse()?,
            alpha_wave: record[4].parse()?,
            beta_wave: record[5].parse()?,
            gamma_wave: record[6].parse()?,
            swr: record[7].parse()?,
            ied: record[8].parse()?,
            composite_signal: record[9].parse()?,
        };
        signals.push(signal_data);
    }

    Ok(signals)
}

// --------------------------------------
// time
// --------------------------------------

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct TimeUniform {
    time: f32,
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
// signals
// --------------------------------------

const NUM_LINES: usize = 9;
const VERTICES_PER_LINE: usize = 2; // just 2

fn generate_line_vertices() -> Vec<LineVertex> {
    let mut vertices = Vec::new();
    for line_index in 0..NUM_LINES {
        let z_position = line_index as f32 - (NUM_LINES as f32) * 0.5 + 0.5; // Spread lines along the x-axis
        for vertex_index in 0..VERTICES_PER_LINE {
            let x_position = vertex_index as f32 * 1000.0 - 500.0; // Vertical position
            vertices.push(LineVertex {
                position: [x_position, 0.0, z_position], // All lines are along the z-axis
            });
        }
    }
    vertices
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct SignalInstance {
    position: [f32; 3],
    signal_value: f32,
}

fn create_signal_instances(signals: &Vec<SignalData>) -> Vec<SignalInstance> {
    signals
        .iter()
        .enumerate()
        .map(|(index, signal)| {
            let signal_value = match index {
                0 => signal.slow_wave,
                1 => signal.delta_wave,
                2 => signal.theta_wave,
                3 => signal.alpha_wave,
                4 => signal.beta_wave,
                5 => signal.gamma_wave,
                6 => signal.swr,
                7 => signal.ied,
                8 => signal.composite_signal,
                _ => 0.0,
            };
            SignalInstance {
                position: [0.0, index as f32, 0.0],
                signal_value,
            }
        })
        .collect()
}

impl SignalInstance {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<SignalInstance>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as u64,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32,
                },
            ],
        }
    }
}

pub struct Renderer {
    window: Window,
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    pub size: PhysicalSize<u32>,
    camera: camera::Camera,
    pub camera_controller: CameraController,
    camera_uniform: camera::uniform::CameraUniform,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    depth_texture: crate::texture::Texture,
    // time
    start_time: Instant,
    current_time: f32,
    time_scale: f32,
    time_buffer: wgpu::Buffer,
    time_bind_group: wgpu::BindGroup,
    // cube
    line_render_pipeline: wgpu::RenderPipeline,
    line_vertex_buffer: wgpu::Buffer,
    line_index_buffer: wgpu::Buffer,
    num_line_indices: u32,
    // signals
    signals: Vec<SignalData>,
    signals_buffer: wgpu::Buffer,
    signal_render_pipeline: wgpu::RenderPipeline,
    signal_vertex_buffer: wgpu::Buffer,
    signal_index_buffer: wgpu::Buffer,
    signal_num_line_indices: u32,
    signal_instances: Vec<SignalInstance>,
    signal_instance_buffer: wgpu::Buffer,
    signals_bind_group: wgpu::BindGroup,
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
            eye: Point3::new(30.0, 15.0, 30.0),
            target: Point3::new(0.0, 0.0, 0.0),
            up: Vector3::new(0.0, 1.0, 0.0),
            aspect: 800.0 / 600.0,
            fovy: 20.0,
            znear: 0.1,
            zfar: 1000.0,
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

        // depth & render layout

        let depth_texture =
            texture::Texture::create_depth_texture(&device, &config, "depth_texture");

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&camera_bind_group_layout, &time_bind_group_layout],
                push_constant_ranges: &[],
            });

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
        // signals
        // -----------------------------------------

        let signal_vertices = generate_line_vertices();

        let signal_vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Line Vertex Buffer"),
            contents: bytemuck::cast_slice(&signal_vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let signal_indices: Vec<u16> = (0..(NUM_LINES * VERTICES_PER_LINE) as u16).collect();
        let signal_num_line_indices = signal_indices.len() as u32;

        let signal_index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Line Index Buffer"),
            contents: bytemuck::cast_slice(&signal_indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        let signal_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/signal_shader.wgsl").into()),
        });

        let signals = read_signals_from_csv("data/signals.csv").unwrap();
        let signal_instances = create_signal_instances(&signals);

        let signal_instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Instance Buffer"),
            contents: bytemuck::cast_slice(&signal_instances),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let signals = read_signals_from_csv("data/signals.csv").unwrap();

        let signals_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Signals Buffer"),
            contents: bytemuck::cast_slice(&signals),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });

        let signals_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Signals Bind Group Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(
                            std::mem::size_of::<SignalData>() as u64
                        ),
                    },
                    count: None,
                }],
            });

        let signals_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &signals_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: signals_buffer.as_entire_binding(),
            }],
            label: Some("Signals Bind Group"),
        });

        let signal_render_pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Render Pipeline"),
                layout: Some(&render_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &signal_shader,
                    entry_point: "vs_main",
                    buffers: &[LineVertex::desc(), SignalInstance::desc()],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &signal_shader,
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
            camera,
            camera_controller,
            camera_buffer,
            camera_bind_group,
            camera_uniform,
            depth_texture,
            // Time
            start_time,
            current_time: 0.0,
            time_scale: 0.05,
            time_buffer,
            time_bind_group,
            // cube
            line_render_pipeline,
            line_vertex_buffer,
            line_index_buffer,
            num_line_indices,
            // signal
            signals,
            signals_buffer,
            signal_render_pipeline,
            signal_vertex_buffer,
            signal_index_buffer,
            signal_num_line_indices,
            signal_instances,
            signal_instance_buffer,
            signals_bind_group,
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
                // cube
                render_pass.set_pipeline(&self.line_render_pipeline); // Use the line rendering pipeline
                render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
                render_pass.set_bind_group(1, &self.time_bind_group, &[]); // Use the correct index for the bind group

                render_pass.set_vertex_buffer(0, self.line_vertex_buffer.slice(..));
                render_pass
                    .set_index_buffer(self.line_index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                render_pass.draw_indexed(0..self.num_line_indices, 0, 0..1);

                // signals
                render_pass.set_pipeline(&self.signal_render_pipeline);
                render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
                render_pass.set_bind_group(1, &self.time_bind_group, &[]);
                render_pass.set_bind_group(2, &self.signals_bind_group, &[]); // Set the signals bind group

                render_pass.set_vertex_buffer(0, self.signal_vertex_buffer.slice(..));
                render_pass.set_vertex_buffer(1, self.signal_instance_buffer.slice(..));
                render_pass.set_index_buffer(
                    self.signal_index_buffer.slice(..),
                    wgpu::IndexFormat::Uint16,
                );
                render_pass.draw_indexed(
                    0..self.signal_num_line_indices,
                    0,
                    0..self.signal_instances.len() as u32,
                );

                // render_pass
                //     .set_index_buffer(self.signal_index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                // render_pass.draw_indexed(0..self.signal_num_line_indices, 0, 0..1);
            } else {
                ()
            }
        }

        self.queue.submit(iter::once(encoder.finish()));
        output.present();

        Ok(())
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

        self.queue
            .write_buffer(&self.signals_buffer, 0, bytemuck::cast_slice(&self.signals));

        self.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[self.camera_uniform]),
        );
    }
}
