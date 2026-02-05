mod camera;
mod uniform;
mod vertex;

use std::{
    sync::Arc,
    time::{Duration, Instant},
};

pub use camera::*;
use korp_math::Vec2;

use crate::{
    color::Color,
    renderer::{
        uniform::{Uniform, UniformBuffer},
        vertex::Vertex,
    },
    shapes::{Line, Rectangle, Triangle},
};

pub struct ThreadRenderer<T: Send + Sync + 'static> {
    tx: std::sync::mpsc::Sender<RenderAction<T>>,
    handle: Option<std::thread::JoinHandle<()>>,
}

struct RawRenderer {
    vertices: Vec<Vertex>,
    vertices_max: usize,
    batches: Vec<RenderBatch>,
    camera: Camera,
    view_projections: Vec<[[f32; 4]; 4]>,
    view_projection_default: [[f32; 4]; 4],
    view_projection_stride: u32,
    view_projections_max: usize,
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

struct RenderThread<T: Send + Sync + 'static> {
    window: Arc<winit::window::Window>,
    surface: wgpu::Surface<'static>,
    rx: std::sync::mpsc::Receiver<RenderAction<T>>,
    surface_configuration: wgpu::SurfaceConfiguration,
    device: wgpu::Device,
    queue: wgpu::Queue,
    shader: wgpu::ShaderModule,
    pipeline: wgpu::RenderPipeline,
    buffer: wgpu::Buffer,
    uniform: Uniform,
    clear_color: wgpu::Color,
    callback: fn(&mut T, &mut Renderer, f32),
    data: Option<T>,
    renderer: RawRenderer,
    timestep: Duration,
    last_submit: Instant,
    last_render: Instant,
    timer: Duration,
    fps: u32,
}

struct RenderBatch {
    start: u32,
    end: u32,
    view_projection_idx: u32,
}

// struct Frame<T: Send + Sync> {
//     actions: Vec<FrameAction<T>>,
// }

// enum FrameAction<T: Send + Sync> {
//     Set { data: T },

// }

enum RenderAction<T: Send + Sync> {
    Submit { data: T },
    // Command(RenderCommand),
    Resize { width: u32, height: u32 },
    Die,
    // Begin,
    // End,
    // BeginScope { view_projection: [[f32; 4]; 4] },
    // EndScope,
    // Render { alpha: f32 },
}

enum RenderCommand {
    DrawLine {
        line: Line<f32>,
        rotation: Vec2<f32>,
        origin: Vec2<f32>,
        color: Color,
    },
    DrawTriangleFilled {
        triangle: Triangle<f32>,
        rotation: Vec2<f32>,
        origin: Vec2<f32>,
        color: Color,
    },
    DrawTriangleLines {
        triangle: Triangle<f32>,
        rotation: Vec2<f32>,
        origin: Vec2<f32>,
        color: Color,
    },
    DrawRectangleFilled {
        rectangle: Rectangle<f32>,
        rotation: Vec2<f32>,
        origin: Vec2<f32>,
        color: Color,
    },
    DrawRectangleLines {
        rectangle: Rectangle<f32>,
        rotation: Vec2<f32>,
        origin: Vec2<f32>,
        color: Color,
    },
}

impl<T: Send + Sync> Drop for ThreadRenderer<T> {
    fn drop(&mut self) {
        self.tx.send(RenderAction::Die);

        if let Some(handle) = self.handle.take() {
            handle.join();
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
    }
}

impl RawRenderer {
    fn new(
        vertices_max: usize,
        view_projection_stride: u32,
        view_projections_max: usize,
        width: f32,
        height: f32,
    ) -> Self {
        let mut camera = Camera::new(width as f32, height as f32);
        camera.reposition(Vec2::new(width as f32 / 2.0, height as f32 / 2.0));

        let view_projection_default = camera.view_projection();

        Self {
            vertices: Vec::new(),
            vertices_max,
            batches: Vec::new(),
            camera,
            view_projections: Vec::new(),
            view_projection_default,
            view_projection_stride,
            view_projections_max,
        }
    }

    fn begin(&mut self) -> Renderer<'_> {
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

impl<T: Send + Sync> RenderThread<T> {
    const ONE: Duration = Duration::from_secs(1);

    async fn new(
        window: Arc<winit::window::Window>,
        surface: wgpu::Surface<'static>,
        adapter: &wgpu::Adapter,
        surface_configuration: wgpu::SurfaceConfiguration,
        rx: std::sync::mpsc::Receiver<RenderAction<T>>,
        width: u32,
        height: u32,
        callback: fn(&mut T, &mut Renderer, f32),
        timestep: Duration,
    ) -> Option<Self> {
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

        surface.configure(&device, &surface_configuration);

        let view_projections_max = 1;
        let uniform = Uniform::new(&mut device, view_projections_max);

        let shader = device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));

        let pipeline =
            Self::create_pipeline(&mut device, &uniform, &shader, &surface_configuration);

        let vertices_max = 8;
        let buffer = Self::create_buffer(&mut device, vertices_max);

        let view_projection_stride = device.limits().min_uniform_buffer_offset_alignment;

        let renderer = RawRenderer::new(
            vertices_max,
            view_projection_stride,
            view_projections_max,
            width as f32,
            height as f32,
        );

        Some(Self {
            window,
            surface,
            rx,
            surface_configuration,
            device,
            queue,
            pipeline,
            shader,
            buffer,
            uniform,
            clear_color: wgpu::Color::BLACK,
            callback,
            data: None,
            renderer,
            timestep,
            last_submit: Instant::now(),
            last_render: Instant::now(),
            timer: Duration::ZERO,
            fps: 0,
        })
    }

    fn run(&mut self) {
        let mut running = true;

        while running {
            // let mut latest = None;

            // while let Ok(action) = self.rx.try_recv() {
            //     latest = Some(action);
            // }

            // if let Some(action) = latest {
            //     match action {
            //         RenderAction::Die => running = false,
            //         RenderAction::Resize { width, height } => {
            //             self.resize(width, height);
            //         }
            //         RenderAction::Submit { data } => {
            //             self.data = Some(data);
            //         } // RenderAction::Render { alpha } => {
            //           //     self.render(alpha);
            //           // }
            //     }
            // }

            while let Ok(action) = self.rx.try_recv() {
                match action {
                    RenderAction::Die => running = false,
                    RenderAction::Resize { width, height } => {
                        self.resize(width, height);
                    }
                    RenderAction::Submit { data } => {
                        self.data = Some(data);
                        self.last_submit = Instant::now();
                    } // RenderAction::Render { alpha } => {
                      //     self.render(alpha);
                      // }
                }
            }

            self.render();
        }
    }

    fn resize(&mut self, width: u32, height: u32) {
        self.surface_configuration.width = width;
        self.surface_configuration.height = height;

        self.surface
            .configure(&self.device, &self.surface_configuration);

        self.renderer.camera.resize(width as f32, height as f32);
        self.renderer
            .camera
            .reposition(Vec2::new(width as f32 / 2.0, height as f32 / 2.0));

        self.renderer.view_projection_default = self.renderer.camera.view_projection();
    }

    fn render(&mut self) {
        let now = Instant::now();
        // println!("{:?} {:?}", now, self.last_render);
        let delta = now - self.last_submit;
        let alpha = delta.as_secs_f32() / self.timestep.as_secs_f32();

        let delta = now - self.last_render;

        self.timer += delta;
        self.fps += 1;
        self.last_render = now;

        if self.timer >= Self::ONE {
            println!("fps {}", self.fps);

            self.fps = 0;
            self.timer -= Self::ONE;
        }

        if let Some(data) = &mut self.data {
            let mut renderer = self.renderer.begin();
            (self.callback)(data, &mut renderer, alpha);
        }

        if self.renderer.view_projections.len() > self.renderer.view_projections_max {
            // recreate uniform and pipeline to ensure we can support
            // the required amount of view projections
            while self.renderer.view_projections_max < self.renderer.view_projections.len() {
                self.renderer.view_projections_max *= 2;
            }

            self.uniform = Uniform::new(&mut self.device, self.renderer.view_projections_max);
            self.pipeline = Self::create_pipeline(
                &mut self.device,
                &self.uniform,
                &self.shader,
                &self.surface_configuration,
            );
        }

        if self.renderer.vertices.len() > self.renderer.vertices_max {
            // recreate vertex buffer to ensure we can support
            // the required amount of vertices
            while self.renderer.vertices_max < self.renderer.vertices.len() {
                self.renderer.vertices_max *= 2;
            }

            self.buffer = Self::create_buffer(&mut self.device, self.renderer.vertices_max);
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

            self.queue.write_buffer(
                &self.buffer,
                0,
                bytemuck::cast_slice(&self.renderer.vertices),
            );

            for (i, view_projection) in self.renderer.view_projections.iter().enumerate() {
                let offset = i as u32 * self.renderer.view_projection_stride;

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

            for batch in self.renderer.batches.iter() {
                render_pass.set_bind_group(
                    0,
                    &self.uniform.bind_group,
                    &[batch.view_projection_idx * self.renderer.view_projection_stride],
                );

                render_pass.draw(batch.start..batch.end, 0..1);
            }

            self.renderer.batches.clear();
        }

        self.window.pre_present_notify();
        self.queue.submit(std::iter::once(encoder.finish()));
        surface_texture.present();

        self.window.request_redraw();
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

impl<T: Send + Sync + 'static> ThreadRenderer<T> {
    pub(crate) async fn new(
        // window: impl Into<wgpu::SurfaceTarget<'static>>,
        window: Arc<winit::window::Window>,
        width: u32,
        height: u32,
        tps: u8,
        callback: fn(&mut T, &mut Renderer, f32),
    ) -> Self {
        let instance = wgpu::Instance::default();
        let surface = instance
            .create_surface(window.clone())
            .expect("could not create surface");

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .await
            .expect("could not request adapter");

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

        let (tx, rx) = std::sync::mpsc::channel::<RenderAction<T>>();

        let mut thread = RenderThread::<T>::new(
            window,
            surface,
            &adapter,
            surface_configuration,
            rx,
            width,
            height,
            callback,
            Duration::from_secs_f32(1.0 / tps as f32),
        )
        .await
        .expect("could not create render thread");

        let handle = std::thread::spawn(move || thread.run());

        Self {
            tx,
            handle: Some(handle),
        }
    }

    pub(crate) fn resize(&mut self, width: u32, height: u32) {
        self.tx.send(RenderAction::Resize { width, height });
    }

    pub(crate) fn submit(&mut self, data: T) {
        self.tx.send(RenderAction::Submit { data });
    }

    // pub(crate) fn render(&self, alpha: f32) {
    //     self.tx.send(RenderAction::Render { alpha });
    // }
}
