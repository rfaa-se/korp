use korp_math::Vec2;

use crate::{
    color::Color,
    misc::Morph,
    shapes::{Line, Rectangle, Triangle},
};

pub struct Renderer {
    surface: wgpu::Surface<'static>,
    surface_configuration: wgpu::SurfaceConfiguration,
    device: wgpu::Device,
    queue: wgpu::Queue,
    pipeline: wgpu::RenderPipeline,
    buffer: wgpu::Buffer,
    uniform: Uniform,
    pub(crate) canvas: Canvas,
}

pub struct Canvas {
    vertices: Vec<Vertex>,
    clear_color: wgpu::Color,
}

struct Uniform {
    buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    bind_group_layout: wgpu::BindGroupLayout,
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable, Default)]
struct UniformBuffer {
    view_projection: [[f32; 4]; 4],
    alpha: f32,
    _padding: [f32; 3],
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable, Default)]
struct Vertex {
    position_old: [f32; 2],
    position_new: [f32; 2],
    rotation_old: [f32; 2],
    rotation_new: [f32; 2],
    origin_old: [f32; 2],
    origin_new: [f32; 2],
    color_old: u32,
    color_new: u32,
}

impl Vertex {
    const ATTRIBUTES: [wgpu::VertexAttribute; 8] = wgpu::vertex_attr_array![
        0 => Float32x2,
        1 => Float32x2,
        2 => Float32x2,
        3 => Float32x2,
        4 => Float32x2,
        5 => Float32x2,
        6 => Uint32,
        7 => Uint32,
    ];

    fn description() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBUTES,
        }
    }
}

impl Uniform {
    fn new(device: &mut wgpu::Device) -> Self {
        let buffer = wgpu::util::DeviceExt::create_buffer_init(
            device,
            &wgpu::util::BufferInitDescriptor {
                label: Some("uniform_buffer"),
                contents: bytemuck::cast_slice(&[UniformBuffer::default()]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            },
        );

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("uniform_bind_group_layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("uniform_bind_group"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
        });

        Self {
            buffer,
            bind_group,
            bind_group_layout,
        }
    }
}

impl Renderer {
    pub(crate) async fn new(
        window: impl Into<wgpu::SurfaceTarget<'static>>,
        width: u32,
        height: u32,
    ) -> Self {
        let instance = wgpu::Instance::default();
        let surface = instance
            .create_surface(window)
            .expect("could not create surface");

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .await
            .expect("could not request adapter");

        let (mut device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("device"),
                required_features: wgpu::Features::default(),
                required_limits: wgpu::Limits::downlevel_defaults(),
                experimental_features: wgpu::ExperimentalFeatures::disabled(),
                memory_hints: wgpu::MemoryHints::default(),
                trace: wgpu::Trace::Off,
            })
            .await
            .expect("could not request device");

        let surface_capabilities = surface.get_capabilities(&adapter);
        let surface_format = surface_capabilities.formats[0];
        let surface_configuration = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width,
            height,
            present_mode: surface_capabilities.present_modes[0],
            desired_maximum_frame_latency: 2,
            alpha_mode: surface_capabilities.alpha_modes[0],
            view_formats: Vec::new(),
        };

        surface.configure(&device, &surface_configuration);

        let uniform = Uniform::new(&mut device);

        let shader_module = device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("pipeline_layout"),
            bind_group_layouts: &[&uniform.bind_group_layout],
            push_constant_ranges: &[],
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader_module,
                entry_point: Some("vs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[Vertex::description()],
            },
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module: &shader_module,
                entry_point: Some("fs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_configuration.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview: None,
            cache: None,
        });

        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("buffer"),
            size: 1024 * 10,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            canvas: Canvas::new(),
            surface,
            surface_configuration,
            device,
            queue,
            pipeline,
            buffer,
            uniform,
        }
    }

    pub(crate) fn resize(&mut self, width: u32, height: u32) {
        self.surface_configuration.width = width;
        self.surface_configuration.height = height;
        self.surface
            .configure(&self.device, &self.surface_configuration);
    }

    pub(crate) fn prepare(&mut self) {
        self.canvas.prepare();
    }

    pub(crate) fn render(&mut self, alpha: f32) {
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("encoder"),
            });

        let surface_texture = self
            .surface
            .get_current_texture()
            .expect("could not get current texture");

        let surface_view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("render_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &surface_view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(self.canvas.clear_color),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            let view_projection = Self::ortho_lh_zo(
                0.0,
                self.surface_configuration.width as f32,
                self.surface_configuration.height as f32,
                0.0,
                0.0,
                1.0,
            );

            self.queue.write_buffer(
                &self.uniform.buffer,
                0,
                bytemuck::cast_slice(&[UniformBuffer {
                    view_projection,
                    alpha,
                    _padding: [0.0; 3],
                }]),
            );

            render_pass.set_pipeline(&self.pipeline);
            render_pass.set_bind_group(0, &self.uniform.bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.buffer.slice(..));

            render_pass.draw(0..self.canvas.vertices.len() as u32, 0..1);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        surface_texture.present();
    }

    fn ortho_lh_zo(
        left: f32,
        right: f32,
        bottom: f32,
        top: f32,
        near: f32,
        far: f32,
    ) -> [[f32; 4]; 4] {
        [
            [2.0 / (right - left), 0.0, 0.0, 0.0],
            [0.0, 2.0 / (top - bottom), 0.0, 0.0],
            [0.0, 0.0, 1.0 / (far - near), 0.0],
            [
                -(right + left) / (right - left),
                -(top + bottom) / (top - bottom),
                -near / (far - near),
                1.0,
            ],
        ]
    }

    pub(crate) fn update(&mut self) {
        self.queue
            .write_buffer(&self.buffer, 0, bytemuck::cast_slice(&self.canvas.vertices));
    }
}

impl Canvas {
    pub(crate) fn new() -> Self {
        Self {
            vertices: Vec::new(),
            clear_color: wgpu::Color::BLACK,
        }
    }

    pub(crate) fn prepare(&mut self) {
        self.vertices.clear();
    }

    pub fn draw_line(
        &mut self,
        line: Morph<Line>,
        rotation: Morph<Vec2<f32>>,
        origin: Morph<Vec2<f32>>,
        color: Morph<Color>,
    ) {
        let t = |a: Vec2<f32>, b: Vec2<f32>| {
            let dir = b - a;
            let norm = dir.perp().normalize() * 0.5;
            (a + norm, b + norm, b - norm, a - norm)
        };

        let (ov0, ov1, ov2, ov3) = t(line.old.start, line.old.end);
        let (pv0, pv1, pv2, pv3) = t(line.new.start, line.new.end);

        let v0 = Vertex {
            position_old: ov0.into(),
            position_new: pv0.into(),
            rotation_old: rotation.old.into(),
            rotation_new: rotation.new.into(),
            origin_old: origin.old.into(),
            origin_new: origin.new.into(),
            color_old: color.old.into(),
            color_new: color.new.into(),
        };

        let v1 = Vertex {
            position_old: ov1.into(),
            position_new: pv1.into(),
            ..v0
        };

        let v2 = Vertex {
            position_old: ov2.into(),
            position_new: pv2.into(),
            ..v0
        };

        let v3 = Vertex {
            position_old: ov3.into(),
            position_new: pv3.into(),
            ..v0
        };

        self.vertices.push(v0);
        self.vertices.push(v1);
        self.vertices.push(v2);
        self.vertices.push(v2);
        self.vertices.push(v3);
        self.vertices.push(v0);
    }

    pub fn draw_triangle_filled(
        &mut self,
        triangle: Morph<Triangle>,
        rotation: Morph<Vec2<f32>>,
        origin: Morph<Vec2<f32>>,
        color: Morph<Color>,
    ) {
        let top = Vertex {
            position_old: triangle.old.top.into(),
            position_new: triangle.new.top.into(),
            rotation_old: rotation.old.into(),
            rotation_new: rotation.new.into(),
            origin_old: origin.old.into(),
            origin_new: origin.new.into(),
            color_old: color.old.into(),
            color_new: color.new.into(),
        };

        let left = Vertex {
            position_old: triangle.old.left.into(),
            position_new: triangle.new.left.into(),
            ..top
        };

        let right = Vertex {
            position_old: triangle.old.right.into(),
            position_new: triangle.new.right.into(),
            ..top
        };

        self.vertices.push(top);
        self.vertices.push(left);
        self.vertices.push(right);
    }

    pub fn draw_triangle_lines(
        &mut self,
        triangle: Morph<Triangle>,
        rotation: Morph<Vec2<f32>>,
        origin: Morph<Vec2<f32>>,
        color: Morph<Color>,
    ) {
        let vertices = [
            (triangle.old.top, triangle.new.top),
            (triangle.old.left, triangle.new.left),
            (triangle.old.right, triangle.new.right),
        ];

        for i in 0..vertices.len() {
            // TODO: the lines don't match up 100% with the filled version, fixable?
            // let l = Line {
            //     start: vertices[i].0
            //         + match i {
            //             0 => Vec2::new(0.5, 0.),
            //             1 => Vec2::new(0.5, -0.5), // OK
            //             _ => Vec2::new(-0.5, 0.),
            //         },
            //     end: vertices[(i + 1) % 3].0
            //         + match i {
            //             0 => Vec2::new(0.5, -0.),
            //             1 => Vec2::new(-0.5, -0.5), // OK
            //             _ => Vec2::new(-0.5, 0.),
            //         },
            // };
            self.draw_line(
                Morph {
                    old: Line {
                        start: vertices[i].0,
                        end: vertices[(i + 1) % 3].0,
                    },
                    new: Line {
                        start: vertices[i].1,
                        end: vertices[(i + 1) % 3].1,
                    },
                },
                rotation,
                origin,
                color,
            );
        }
    }

    pub fn draw_rectangle_filled(
        &mut self,
        rect: Morph<Rectangle>,
        rotation: Morph<Vec2<f32>>,
        origin: Morph<Vec2<f32>>,
        color: Morph<Color>,
    ) {
        let v1 = Vertex {
            position_old: [rect.old.x, rect.old.y],
            position_new: [rect.new.x, rect.new.y],
            rotation_old: rotation.old.into(),
            rotation_new: rotation.new.into(),
            origin_old: origin.old.into(),
            origin_new: origin.new.into(),
            color_old: color.old.into(),
            color_new: color.new.into(),
        };

        let v2 = Vertex {
            position_old: [rect.old.x + rect.old.width, rect.old.y],
            position_new: [rect.new.x + rect.new.width, rect.new.y],
            ..v1
        };

        let v3 = Vertex {
            position_old: [rect.old.x, rect.old.y + rect.old.height],
            position_new: [rect.new.x, rect.new.y + rect.new.height],
            ..v1
        };

        let v4 = Vertex {
            position_old: [rect.old.x + rect.old.width, rect.old.y + rect.old.height],
            position_new: [rect.new.x + rect.new.width, rect.new.y + rect.new.height],
            ..v1
        };

        self.vertices.push(v1);
        self.vertices.push(v2);
        self.vertices.push(v3);
        self.vertices.push(v2);
        self.vertices.push(v4);
        self.vertices.push(v3);
    }

    pub fn draw_rectangle_lines(
        &mut self,
        rect: Morph<Rectangle>,
        rotation: Morph<Vec2<f32>>,
        origin: Morph<Vec2<f32>>,
        color: Morph<Color>,
    ) {
        self.draw_line(
            Morph {
                old: Line {
                    start: Vec2::new(rect.old.x, rect.old.y + 0.5),
                    end: Vec2::new(rect.old.x + rect.old.width, rect.old.y + 0.5),
                },
                new: Line {
                    start: Vec2::new(rect.new.x, rect.new.y + 0.5),
                    end: Vec2::new(rect.new.x + rect.new.width, rect.new.y + 0.5),
                },
            },
            rotation,
            origin,
            color,
        );

        self.draw_line(
            Morph {
                old: Line {
                    start: Vec2::new(rect.old.x + rect.old.width - 0.5, rect.old.y),
                    end: Vec2::new(
                        rect.old.x + rect.old.width - 0.5,
                        rect.old.y + rect.old.height,
                    ),
                },
                new: Line {
                    start: Vec2::new(rect.new.x + rect.new.width - 0.5, rect.new.y),
                    end: Vec2::new(
                        rect.new.x + rect.new.width - 0.5,
                        rect.new.y + rect.new.height,
                    ),
                },
            },
            rotation,
            origin,
            color,
        );

        self.draw_line(
            Morph {
                old: Line {
                    start: Vec2::new(
                        rect.old.x + rect.old.width,
                        rect.old.y + rect.old.height - 0.5,
                    ),
                    end: Vec2::new(rect.old.x, rect.old.y + rect.old.height - 0.5),
                },
                new: Line {
                    start: Vec2::new(
                        rect.new.x + rect.new.width,
                        rect.new.y + rect.new.height - 0.5,
                    ),
                    end: Vec2::new(rect.new.x, rect.new.y + rect.new.height - 0.5),
                },
            },
            rotation,
            origin,
            color,
        );

        self.draw_line(
            Morph {
                old: Line {
                    start: Vec2::new(rect.old.x + 0.5, rect.old.y + rect.old.height),
                    end: Vec2::new(rect.old.x + 0.5, rect.old.y),
                },
                new: Line {
                    start: Vec2::new(rect.new.x + 0.5, rect.new.y + rect.new.height),
                    end: Vec2::new(rect.new.x + 0.5, rect.new.y),
                },
            },
            rotation,
            origin,
            color,
        );
    }
}
