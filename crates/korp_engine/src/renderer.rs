mod camera;

pub use camera::*;
use korp_math::Vec2;

use crate::{
    color::Color,
    shapes::{Line, Rectangle, Triangle},
};

pub struct RawRenderer {
    surface: wgpu::Surface<'static>,
    surface_configuration: wgpu::SurfaceConfiguration,
    device: wgpu::Device,
    queue: wgpu::Queue,
    shader: wgpu::ShaderModule,
    pipeline: wgpu::RenderPipeline,
    buffer: wgpu::Buffer,
    uniform: Uniform,
    batches: Vec<RenderBatch>,
    vertices: Vec<Vertex>,
    vertices_max: usize,
    clear_color: wgpu::Color,
    camera: Camera,
    view_projection_default: [[f32; 4]; 4],
    view_projection_stride: u32,
    view_projections_max: usize,
    view_projections: Vec<[[f32; 4]; 4]>,
}

pub struct Renderer<'a> {
    raw: &'a mut RawRenderer,
}

pub struct RendererScope<'a, 'b>
where
    'b: 'a,
{
    pub renderer: &'a mut Renderer<'b>,
    pub camera: &'a Camera,
}

struct RenderBatch {
    start: u32,
    end: u32,
    view_projection_idx: u32,
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
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable, Default)]
struct Vertex {
    position: [f32; 2],
    rotation: [f32; 2],
    origin: [f32; 2],
    color: u32,
}

impl Vertex {
    const ATTRIBUTES: [wgpu::VertexAttribute; 4] = wgpu::vertex_attr_array![
        0 => Float32x2,
        1 => Float32x2,
        2 => Float32x2,
        3 => Uint32,
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
    fn new(device: &mut wgpu::Device, view_projections_max: usize) -> Self {
        let stride = device.limits().min_uniform_buffer_offset_alignment as u64;
        let uniform_buffer_size = std::mem::size_of::<UniformBuffer>() as u64;
        let size =
            (((uniform_buffer_size + stride - 1) / stride) * stride) * view_projections_max as u64;

        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("uniform_buffer"),
            size,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("uniform_bind_group_layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: true,
                    min_binding_size: std::num::NonZeroU64::new(uniform_buffer_size),
                },
                count: None,
            }],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("uniform_bind_group"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &buffer,
                    offset: 0,
                    size: std::num::NonZeroU64::new(uniform_buffer_size),
                }),
            }],
        });

        Self {
            buffer,
            bind_group,
            bind_group_layout,
        }
    }
}

impl Drop for RendererScope<'_, '_> {
    fn drop(&mut self) {
        let idx = self.renderer.raw.batches.len() - 1;
        let len = self.renderer.raw.vertices.len() as u32;

        // end the current batch
        self.renderer.raw.batches[idx].end = len;

        // restore the previous batch
        self.renderer.raw.batches.push(RenderBatch {
            start: len,
            end: 0,
            view_projection_idx: self.renderer.raw.batches[idx - 1].view_projection_idx,
        });
    }
}

impl Drop for Renderer<'_> {
    fn drop(&mut self) {
        let idx = self.raw.batches.len() - 1;
        let len = self.raw.vertices.len() as u32;

        // end the current batch
        self.raw.batches[idx].end = len;

        // remove empty batches
        self.raw.batches.retain(|x| x.start != x.end);

        self.raw.render();
    }
}

impl<'a, 'b> Renderer<'b> {
    pub fn begin(&'a mut self, camera: &'a Camera) -> RendererScope<'a, 'b> {
        let idx = self.raw.batches.len() - 1;
        let len = self.raw.vertices.len() as u32;

        // end the current batch
        self.raw.batches[idx].end = len;

        // try to reuse view projection if it exists
        let view_projection = camera.view_projection();
        let vp_idx = self
            .raw
            .view_projections
            .iter()
            .position(|x| *x == view_projection)
            .unwrap_or_else(|| {
                self.raw.view_projections.push(view_projection);
                self.raw.view_projections.len() - 1
            });

        // create new batch
        self.raw.batches.push(RenderBatch {
            start: len,
            end: 0,
            view_projection_idx: vp_idx as u32,
        });

        RendererScope {
            renderer: self,
            camera,
        }
    }

    pub fn draw_line(
        &mut self,
        line: Line<f32>,
        rotation: Vec2<f32>,
        origin: Vec2<f32>,
        color: Color,
    ) {
        // TODO: when multithreaded, move this into a command to send?

        let corners = |a: Vec2<f32>, b: Vec2<f32>| {
            let dir = b - a;
            let norm = dir.perp().normalized() * 0.5;
            (a + norm, b + norm, b - norm, a - norm)
        };

        let (c0, c1, c2, c3) = corners(line.start, line.end);

        let v0 = Vertex {
            position: c0.into(),
            rotation: rotation.into(),
            origin: origin.into(),
            color: color.into(),
        };

        let v1 = Vertex {
            position: c1.into(),
            ..v0
        };

        let v2 = Vertex {
            position: c2.into(),
            ..v0
        };

        let v3 = Vertex {
            position: c3.into(),
            ..v0
        };

        self.raw.vertices.push(v0);
        self.raw.vertices.push(v1);
        self.raw.vertices.push(v2);
        self.raw.vertices.push(v2);
        self.raw.vertices.push(v3);
        self.raw.vertices.push(v0);
    }

    pub fn draw_triangle_filled(
        &mut self,
        triangle: Triangle<f32>,
        rotation: Vec2<f32>,
        origin: Vec2<f32>,
        color: Color,
    ) {
        let top = Vertex {
            position: triangle.top.into(),
            rotation: rotation.into(),
            origin: origin.into(),
            color: color.into(),
        };

        let left = Vertex {
            position: triangle.left.into(),
            ..top
        };

        let right = Vertex {
            position: triangle.right.into(),
            ..top
        };

        self.raw.vertices.push(top);
        self.raw.vertices.push(left);
        self.raw.vertices.push(right);
    }

    pub fn draw_triangle_lines(
        &mut self,
        triangle: Triangle<f32>,
        rotation: Vec2<f32>,
        origin: Vec2<f32>,
        color: Color,
    ) {
        let corners = [triangle.top, triangle.left, triangle.right];

        // TOOD: the lines don't match up 100% with the filled version
        for i in 0..corners.len() {
            self.draw_line(
                Line {
                    start: corners[i],
                    end: corners[(i + 1) % 3],
                },
                rotation,
                origin,
                color,
            );
        }
    }

    pub fn draw_rectangle_filled(
        &mut self,
        rectangle: Rectangle<f32>,
        rotation: Vec2<f32>,
        origin: Vec2<f32>,
        color: Color,
    ) {
        let v0 = Vertex {
            position: [rectangle.x, rectangle.y],
            rotation: rotation.into(),
            origin: origin.into(),
            color: color.into(),
        };

        let v1 = Vertex {
            position: [rectangle.x + rectangle.width, rectangle.y],
            ..v0
        };

        let v2 = Vertex {
            position: [rectangle.x, rectangle.y + rectangle.height],
            ..v0
        };

        let v3 = Vertex {
            position: [
                rectangle.x + rectangle.width,
                rectangle.y + rectangle.height,
            ],
            ..v0
        };

        self.raw.vertices.push(v0);
        self.raw.vertices.push(v1);
        self.raw.vertices.push(v2);
        self.raw.vertices.push(v1);
        self.raw.vertices.push(v3);
        self.raw.vertices.push(v2);
    }

    pub fn draw_rectangle_lines(
        &mut self,
        rectangle: Rectangle<f32>,
        rotation: Vec2<f32>,
        origin: Vec2<f32>,
        color: Color,
    ) {
        self.draw_line(
            Line {
                start: Vec2::new(rectangle.x, rectangle.y + 0.5),
                end: Vec2::new(rectangle.x + rectangle.width, rectangle.y + 0.5),
            },
            rotation,
            origin,
            color,
        );

        self.draw_line(
            Line {
                start: Vec2::new(rectangle.x + rectangle.width - 0.5, rectangle.y),
                end: Vec2::new(
                    rectangle.x + rectangle.width - 0.5,
                    rectangle.y + rectangle.height,
                ),
            },
            rotation,
            origin,
            color,
        );

        self.draw_line(
            Line {
                start: Vec2::new(
                    rectangle.x + rectangle.width,
                    rectangle.y + rectangle.height - 0.5,
                ),
                end: Vec2::new(rectangle.x, rectangle.y + rectangle.height - 0.5),
            },
            rotation,
            origin,
            color,
        );

        self.draw_line(
            Line {
                start: Vec2::new(rectangle.x + 0.5, rectangle.y + rectangle.height),
                end: Vec2::new(rectangle.x + 0.5, rectangle.y),
            },
            rotation,
            origin,
            color,
        );
    }
}

impl RawRenderer {
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

        let view_projections_max = 1;
        let uniform = Uniform::new(&mut device, view_projections_max);

        let shader = device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));

        let pipeline =
            Self::create_pipeline(&mut device, &uniform, &shader, &surface_configuration);

        let vertices_max = 8;
        let buffer = Self::create_buffer(&mut device, vertices_max);

        let mut camera = Camera::new(width as f32, height as f32);
        camera.set_position(Vec2::new(width as f32 / 2.0, height as f32 / 2.0));

        let view_projection_default = camera.view_projection();

        let view_projection_stride = device.limits().min_uniform_buffer_offset_alignment;

        Self {
            surface,
            surface_configuration,
            device,
            queue,
            pipeline,
            shader,
            buffer,
            uniform,
            vertices: Vec::new(),
            vertices_max,
            clear_color: wgpu::Color::BLACK,
            batches: Vec::new(),
            camera,
            view_projection_default,
            view_projection_stride,
            view_projections_max,
            view_projections: Vec::new(),
        }
    }

    pub(crate) fn resize(&mut self, width: u32, height: u32) {
        self.surface_configuration.width = width;
        self.surface_configuration.height = height;
        self.surface
            .configure(&self.device, &self.surface_configuration);
        self.camera.resize(width as f32, height as f32);
        self.camera
            .set_position(Vec2::new(width as f32 / 2.0, height as f32 / 2.0));
        self.view_projection_default = self.camera.view_projection();
    }

    pub(crate) fn begin<'a>(&mut self) -> Renderer<'_> {
        self.vertices.clear();
        self.view_projections.clear();

        self.view_projections.push(self.view_projection_default);

        self.batches.push(RenderBatch {
            start: 0,
            end: 0,
            view_projection_idx: (self.view_projections.len() - 1) as u32,
        });

        Renderer { raw: self }
    }

    fn render(&mut self) {
        if self.view_projections.len() > self.view_projections_max {
            // recreate uniform and pipeline to ensure we can support
            // the required amount of view projections
            while self.view_projections_max < self.view_projections.len() {
                self.view_projections_max *= 2;
            }

            self.uniform = Uniform::new(&mut self.device, self.view_projections_max);
            self.pipeline = Self::create_pipeline(
                &mut self.device,
                &self.uniform,
                &self.shader,
                &self.surface_configuration,
            );
        }

        if self.vertices.len() > self.vertices_max {
            // recreate vertex buffer to ensure we can support
            // the required amount of vertices
            while self.vertices_max < self.vertices.len() {
                self.vertices_max *= 2;
            }

            self.buffer = Self::create_buffer(&mut self.device, self.vertices_max);
        }

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
                        load: wgpu::LoadOp::Clear(self.clear_color),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });

            self.queue
                .write_buffer(&self.buffer, 0, bytemuck::cast_slice(&self.vertices));

            for (i, view_projection) in self.view_projections.iter().enumerate() {
                let offset = i as u32 * self.view_projection_stride;

                self.queue.write_buffer(
                    &self.uniform.buffer,
                    offset as u64,
                    bytemuck::cast_slice(&[UniformBuffer {
                        view_projection: *view_projection,
                    }]),
                );
            }

            // TODO: read more about viewport
            // render_pass.set_viewport(x, y, w, h, min_depth, max_depth);

            render_pass.set_pipeline(&self.pipeline);
            render_pass.set_vertex_buffer(0, self.buffer.slice(..));

            for batch in self.batches.iter() {
                render_pass.set_bind_group(
                    0,
                    &self.uniform.bind_group,
                    &[batch.view_projection_idx * self.view_projection_stride],
                );

                render_pass.draw(batch.start..batch.end, 0..1);
            }

            self.batches.clear();
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        surface_texture.present();
    }

    fn create_pipeline(
        device: &mut wgpu::Device,
        uniform: &Uniform,
        shader: &wgpu::ShaderModule,
        surface_configuration: &wgpu::SurfaceConfiguration,
    ) -> wgpu::RenderPipeline {
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("pipeline_layout"),
            bind_group_layouts: &[&uniform.bind_group_layout],
            immediate_size: 0,
        });

        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[Vertex::description()],
            },
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_configuration.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview_mask: None,
            cache: None,
        })
    }

    fn create_buffer(device: &mut wgpu::Device, vertices_max: usize) -> wgpu::Buffer {
        device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("buffer"),
            size: vertices_max as u64 * std::mem::size_of::<Vertex>() as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        })
    }
}
