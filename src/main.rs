use std::{
    collections::HashSet,
    sync::Arc,
    time::{Duration, Instant},
};

fn main() {
    println!("Hello, korp!");

    Engine::new(12, Korp::new(), "korp").run();
}

trait Core {
    fn update(&mut self);
    fn input(&mut self, input: &Input);
    fn render(&mut self, canvas: &mut Canvas);
}

struct Canvas {
    vertices: Vec<Vertex>,
    clear_color: wgpu::Color,
}

struct Engine<T: Core> {
    state: State,
    last_render: Instant,
    elapsed: Duration,
    accumulator: Duration,
    timer: Duration,
    timestep: Duration,
    input: Input,
    core: T,
    fps: u32,
    tps: u32,
    title: String,
}

struct Renderer {
    surface: wgpu::Surface<'static>,
    surface_configuration: wgpu::SurfaceConfiguration,
    device: wgpu::Device,
    queue: wgpu::Queue,
    pipeline: wgpu::RenderPipeline,
    buffer: wgpu::Buffer,
    uniform: Uniform,
    canvas: Canvas,
}

struct Input {
    keyboard: Morph<HashSet<winit::keyboard::KeyCode>>,
    keyboard_down: HashSet<winit::keyboard::KeyCode>,
    mouse: Vec2<f32>,
}

struct InputEvent {}

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
    flags_data: u32,
}

#[derive(Copy, Clone)]
struct Vec2<T> {
    x: T,
    y: T,
}

struct Morph<T> {
    old: T,
    new: T,
}

#[derive(Copy, Clone)]
struct Triangle {
    top: Vec2<f32>,
    left: Vec2<f32>,
    right: Vec2<f32>,
}

#[derive(Copy, Clone)]
struct Rectangle {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}

#[derive(Copy, Clone)]
struct Color {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

enum State {
    Uninitialized,
    Initialized {
        renderer: Renderer,
        // window must be last, otherwise segfault at exit
        window: Arc<winit::window::Window>,
    },
}

impl Vertex {
    const ATTRIBUTES: [wgpu::VertexAttribute; 9] = wgpu::vertex_attr_array![
        0 => Float32x2,
        1 => Float32x2,
        2 => Float32x2,
        3 => Float32x2,
        4 => Float32x2,
        5 => Float32x2,
        6 => Uint32,
        7 => Uint32,
        8 => Uint32,
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

impl<T: Core> Engine<T> {
    const ONE: Duration = Duration::from_secs(1);

    fn new(tps: u8, core: T, title: &str) -> Self {
        Self {
            state: State::Uninitialized,
            last_render: Instant::now(),
            elapsed: Duration::ZERO,
            accumulator: Duration::ZERO,
            timer: Duration::ZERO,
            timestep: Duration::from_secs_f32(1.0 / tps as f32),
            input: Input::new(),
            core,
            fps: 0,
            tps: 0,
            title: title.to_owned(),
        }
    }

    fn run(&mut self) {
        let event_loop = match winit::event_loop::EventLoop::new() {
            Ok(v) => v,
            Err(e) => panic!("could not create event loop: {}", e),
        };

        match event_loop.run_app(self) {
            Ok(_) => (),
            Err(e) => panic!("could not run app: {}", e),
        }
    }
}

impl<T: Core> winit::application::ApplicationHandler for Engine<T> {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let State::Uninitialized = self.state else {
            return;
        };

        event_loop.set_control_flow(winit::event_loop::ControlFlow::Poll);

        let window_attributes =
            winit::window::Window::default_attributes().with_title(self.title.clone());

        if let Ok(window) = event_loop.create_window(window_attributes) {
            let window = Arc::new(window);
            let inner_size = window.inner_size();
            let (width, height) = (inner_size.width, inner_size.height);
            let renderer =
                pollster::block_on(async { Renderer::new(window.clone(), width, height).await });

            self.state = State::Initialized { renderer, window };
        }
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        let State::Initialized { renderer, window } = &mut self.state else {
            return;
        };

        match event {
            winit::event::WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            winit::event::WindowEvent::CursorMoved { position, .. } => {
                self.input.mouse = Vec2::new(position.x as f32, position.y as f32);
            }
            winit::event::WindowEvent::KeyboardInput {
                event:
                    winit::event::KeyEvent {
                        physical_key: winit::keyboard::PhysicalKey::Code(key_code),
                        repeat: false,
                        state,
                        ..
                    },
                ..
            } => {
                match state {
                    winit::event::ElementState::Pressed => {
                        self.input.keyboard.new.insert(key_code);
                        self.input.keyboard_down.insert(key_code);
                    }
                    winit::event::ElementState::Released => {
                        self.input.keyboard.new.remove(&key_code);
                    }
                }

                if key_code == winit::keyboard::KeyCode::Escape {
                    event_loop.exit();
                }
            }
            winit::event::WindowEvent::Resized(winit::dpi::PhysicalSize { width, height }) => {
                renderer.resize(width.max(1), height.max(1));
            }
            winit::event::WindowEvent::RedrawRequested => {
                let now = Instant::now();
                let delta = now - self.last_render;
                let core = &mut self.core;
                let input = &mut self.input;

                self.last_render = now;
                self.elapsed += delta;
                self.accumulator += delta;
                self.timer += delta;

                while self.accumulator >= self.timestep {
                    self.accumulator -= self.timestep;
                    self.tps += 1;

                    renderer.prepare();

                    core.input(input);
                    core.update();
                    core.render(&mut renderer.canvas);

                    input.update();
                    renderer.update();
                }

                let alpha = self.accumulator.as_secs_f32() / self.timestep.as_secs_f32();

                renderer.render(alpha);
                self.fps += 1;

                if self.timer >= Self::ONE {
                    println!(
                        "tps {} | fps {} | elapsed {}",
                        self.tps,
                        self.fps,
                        self.elapsed.as_secs_f32()
                    );

                    self.tps = 0;
                    self.fps = 0;
                    self.timer -= Self::ONE;
                }
            }
            _ => (),
        }

        window.request_redraw();
    }
}

impl Renderer {
    async fn new(window: impl Into<wgpu::SurfaceTarget<'static>>, width: u32, height: u32) -> Self {
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
            size: 1024,
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

    fn resize(&mut self, width: u32, height: u32) {
        self.surface_configuration.width = width;
        self.surface_configuration.height = height;
        self.surface
            .configure(&self.device, &self.surface_configuration);
    }

    fn prepare(&mut self) {
        self.canvas.prepare();
    }

    fn render(&mut self, alpha: f32) {
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
                    _padding: [0.0, 0.0, 0.0],
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

    fn update(&mut self) {
        self.queue
            .write_buffer(&self.buffer, 0, bytemuck::cast_slice(&self.canvas.vertices));
    }
}

impl Input {
    fn new() -> Self {
        Self {
            keyboard: Morph::new(HashSet::new(), HashSet::new()),
            keyboard_down: HashSet::new(),
            mouse: Vec2::new(0.0, 0.0),
        }
    }

    fn update(&mut self) {
        self.keyboard.old.clear();
        self.keyboard.old.extend(&self.keyboard.new);
        self.keyboard_down.clear();
    }

    fn is_pressed(&self, key: &winit::keyboard::KeyCode) -> bool {
        self.keyboard.new.contains(key) && !self.keyboard.old.contains(key)
    }

    fn is_down(&self, key: &winit::keyboard::KeyCode) -> bool {
        self.keyboard.new.contains(key)
    }

    fn is_released(&self, key: &winit::keyboard::KeyCode) -> bool {
        !self.keyboard.new.contains(key)
    }

    fn was_down(&self, key: &winit::keyboard::KeyCode) -> bool {
        self.keyboard_down.contains(key)
    }

    fn down(&self, key: &winit::keyboard::KeyCode) -> bool {
        self.was_down(key) || self.is_down(key)
    }
}

mod flags {
    pub const NONE: u32 = 0 << 0;
    pub const WIREFRAME: u32 = 1 << 0;

    pub mod wireframe {
        pub const NONE: u32 = 0 << 16;
        pub const TOP: u32 = 1 << 16;
        pub const LEFT: u32 = 2 << 16;
        pub const RIGHT: u32 = 3 << 16;
    }
}

impl Canvas {
    fn new() -> Self {
        Self {
            vertices: Vec::new(),
            clear_color: wgpu::Color::BLACK,
        }
    }

    fn prepare(&mut self) {
        self.vertices.clear();
    }

    fn draw_triangle_filled(
        &mut self,
        triangle: Morph<Triangle>,
        rotation: Morph<Vec2<f32>>,
        origin: Morph<Vec2<f32>>,
        color: Morph<Color>,
    ) {
        self.draw_triangle(triangle, rotation, origin, color, flags::NONE);
    }

    fn draw_triangle_lines(
        &mut self,
        triangle: Morph<Triangle>,
        rotation: Morph<Vec2<f32>>,
        origin: Morph<Vec2<f32>>,
        color: Morph<Color>,
    ) {
        self.draw_triangle(triangle, rotation, origin, color, flags::WIREFRAME);
    }

    fn draw_triangle(
        &mut self,
        triangle: Morph<Triangle>,
        rotation: Morph<Vec2<f32>>,
        origin: Morph<Vec2<f32>>,
        color: Morph<Color>,
        flags: u32,
    ) {
        let g = |o, n, d| Vertex {
            position_old: o,
            position_new: n,
            rotation_old: rotation.old.into(),
            rotation_new: rotation.new.into(),
            origin_old: origin.old.into(),
            origin_new: origin.new.into(),
            color_old: color.old.into(),
            color_new: color.new.into(),
            flags_data: flags | d,
        };

        self.vertices.push(g(
            triangle.old.top.into(),
            triangle.new.top.into(),
            flags::wireframe::TOP,
        ));

        self.vertices.push(g(
            triangle.old.left.into(),
            triangle.new.left.into(),
            flags::wireframe::LEFT,
        ));

        self.vertices.push(g(
            triangle.old.right.into(),
            triangle.new.right.into(),
            flags::wireframe::RIGHT,
        ));
    }

    fn draw_rectangle_filled(
        &mut self,
        rect: Morph<Rectangle>,
        rotation: Morph<Vec2<f32>>,
        origin: Morph<Vec2<f32>>,
        color: Morph<Color>,
    ) {
        self.draw_rectangle(rect, rotation, origin, color, flags::NONE);
    }

    fn draw_rectangle_lines(
        &mut self,
        rect: Morph<Rectangle>,
        rotation: Morph<Vec2<f32>>,
        origin: Morph<Vec2<f32>>,
        color: Morph<Color>,
    ) {
        self.draw_rectangle(rect, rotation, origin, color, flags::WIREFRAME);
    }

    fn draw_rectangle(
        &mut self,
        rect: Morph<Rectangle>,
        rotation: Morph<Vec2<f32>>,
        origin: Morph<Vec2<f32>>,
        color: Morph<Color>,
        flags: u32,
    ) {
        let g = |o, n, d| Vertex {
            position_old: o,
            position_new: n,
            rotation_old: rotation.old.into(),
            rotation_new: rotation.new.into(),
            origin_old: origin.old.into(),
            origin_new: origin.new.into(),
            color_old: color.old.into(),
            color_new: color.new.into(),
            flags_data: flags | d,
        };

        self.vertices.push(g(
            Vec2::new(rect.old.x, rect.old.y).into(),
            Vec2::new(rect.new.x, rect.new.y).into(),
            flags::wireframe::TOP,
        ));

        self.vertices.push(g(
            Vec2::new(rect.old.x + rect.old.width, rect.old.y).into(),
            Vec2::new(rect.new.x + rect.new.width, rect.new.y).into(),
            flags::wireframe::LEFT,
        ));

        self.vertices.push(g(
            Vec2::new(rect.old.x, rect.old.y + rect.old.height).into(),
            Vec2::new(rect.new.x, rect.new.y + rect.old.height).into(),
            flags::wireframe::RIGHT,
        ));

        self.vertices.push(g(
            Vec2::new(rect.old.x + rect.old.width, rect.old.y).into(),
            Vec2::new(rect.new.x + rect.new.width, rect.new.y).into(),
            flags::wireframe::TOP,
        ));

        self.vertices.push(g(
            Vec2::new(rect.old.x + rect.old.width, rect.old.y + rect.old.height).into(),
            Vec2::new(rect.new.x + rect.new.width, rect.new.y + rect.new.height).into(),
            flags::wireframe::LEFT,
        ));

        self.vertices.push(g(
            Vec2::new(rect.old.x, rect.old.y + rect.old.height).into(),
            Vec2::new(rect.new.x, rect.new.y + rect.old.height).into(),
            flags::wireframe::RIGHT,
        ));
    }
}

impl From<Color> for u32 {
    fn from(value: Color) -> Self {
        (value.r as u32) << 24
            | (value.g as u32) << 16
            | (value.b as u32) << 8
            | (value.a as u32) << 0
    }
}

impl<T> From<Vec2<T>> for [T; 2] {
    fn from(value: Vec2<T>) -> Self {
        [value.x, value.y]
    }
}

impl<T> Vec2<T> {
    fn new(x: T, y: T) -> Self {
        Self { x, y }
    }
}

impl<T> Morph<T> {
    fn new(old: T, new: T) -> Self {
        Self { old, new }
    }
}

impl Color {
    // const BLACK: Color = Color::new(0, 0, 0, 255);
    // const WHITE: Color = Color::new(255, 255, 255, 255);
    // const RED: Color = Color::new(255, 0, 0, 255);
    const GREEN: Color = Color::new(0, 255, 0, 255);
    // const BLUE: Color = Color::new(0, 0, 255, 255);

    const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }
}

impl Triangle {
    fn from(top: Vec2<f32>, left: Vec2<f32>, right: Vec2<f32>, centroid: Vec2<f32>) -> Triangle {
        Triangle {
            top: centroid + top,
            left: centroid + left,
            right: centroid + right,
        }
    }
}

impl Rectangle {
    fn from(width: f32, height: f32, centroid: Vec2<f32>) -> Rectangle {
        Rectangle {
            x: centroid.x - width * 0.5,
            y: centroid.y - height * 0.5,
            width,
            height,
        }
    }
}

// ----------------------------

struct Korp {
    bodies: Vec<Morph<Body>>,
    up: bool,
    down: bool,
    left: bool,
    right: bool,
    key_bindings: KeyBindings,
}

struct KeyBindings {
    up: winit::keyboard::KeyCode,
    down: winit::keyboard::KeyCode,
    left: winit::keyboard::KeyCode,
    right: winit::keyboard::KeyCode,
}

#[derive(Copy, Clone)]
struct Body {
    centroid: Vec2<f32>,
    rotation: Vec2<f32>,
    shape: Shape,
    color: Color,
}

#[derive(Copy, Clone)]
enum Shape {
    Triangle(TriShape),
    Rectangle(RectShape),
}

#[derive(Copy, Clone)]
struct TriShape {
    top: Vec2<f32>,
    left: Vec2<f32>,
    right: Vec2<f32>,
}

#[derive(Copy, Clone)]
struct RectShape {
    width: f32,
    height: f32,
}

trait Drawable {
    fn draw(&self, canvas: &mut Canvas);
}

impl KeyBindings {
    fn new() -> Self {
        Self {
            up: winit::keyboard::KeyCode::ArrowUp,
            down: winit::keyboard::KeyCode::ArrowDown,
            left: winit::keyboard::KeyCode::ArrowLeft,
            right: winit::keyboard::KeyCode::ArrowRight,
        }
    }
}

impl Core for Korp {
    fn update(&mut self) {
        let r = |p: Vec2<f32>, a: f32| {
            let rad = a.to_radians();
            let (sin, cos) = rad.sin_cos();
            Vec2::new(p.x * cos - p.y * sin, p.x * sin + p.y * cos)
        };

        for body in self.bodies.iter_mut() {
            body.old = body.new;

            if self.up {
                body.new.centroid += body.new.rotation * 20.0;
            }

            if self.down {}

            if self.left {
                body.new.rotation = r(body.new.rotation, -12.0);
            }

            if self.right {
                body.new.rotation = r(body.new.rotation, 12.0);
            }
        }
    }

    fn input(&mut self, input: &Input) {
        self.up = input.down(&self.key_bindings.up);
        self.down = input.down(&self.key_bindings.down);
        self.left = input.down(&self.key_bindings.left);
        self.right = input.down(&self.key_bindings.right);

        let rotation = Vec2 { x: 1.0, y: 0.0 };

        if input.is_pressed(&winit::keyboard::KeyCode::Space) {
            let body = Body {
                centroid: input.mouse,
                rotation,
                shape: Shape::Triangle(TriShape {
                    top: Vec2::new(0.0, -50.0),
                    left: Vec2::new(-30.0, 25.0),
                    right: Vec2::new(30.0, 25.0),
                }),
                color: Color::GREEN,
            };

            self.bodies.push(Morph {
                old: body,
                new: body,
            });
        }

        if input.is_pressed(&winit::keyboard::KeyCode::AltLeft) {
            let body = Body {
                centroid: input.mouse,
                rotation,
                shape: Shape::Rectangle(RectShape {
                    width: 60.0,
                    height: 40.0,
                }),
                color: Color::GREEN,
            };

            self.bodies.push(Morph {
                old: body,
                new: body,
            });
        }
    }

    fn render(&mut self, canvas: &mut Canvas) {
        for body in self.bodies.iter() {
            body.draw(canvas);
        }
    }
}

impl Drawable for Morph<Body> {
    fn draw(&self, canvas: &mut Canvas) {
        let rotation = Morph {
            old: self.old.rotation,
            new: self.new.rotation,
        };

        let centroid = Morph {
            old: self.old.centroid,
            new: self.new.centroid,
        };

        let color = Morph {
            old: self.old.color,
            new: self.new.color,
        };

        match (self.old.shape, self.new.shape) {
            (Shape::Triangle(old), Shape::Triangle(new)) => {
                canvas.draw_triangle_lines(
                    Morph {
                        old: Triangle::from(old.top, old.left, old.right, self.old.centroid),
                        new: Triangle::from(new.top, new.left, new.right, self.new.centroid),
                    },
                    rotation,
                    centroid,
                    color,
                );
            }
            (Shape::Rectangle(old), Shape::Rectangle(new)) => {
                canvas.draw_rectangle_lines(
                    Morph {
                        old: Rectangle::from(old.width, old.height, self.old.centroid),
                        new: Rectangle::from(new.width, new.height, self.new.centroid),
                    },
                    rotation,
                    centroid,
                    color,
                );
            }
            // TODO: can't morph between different shapes, draw old or new?
            _ => (),
        }
    }
}

impl Korp {
    fn new() -> Self {
        Self {
            bodies: Vec::new(),
            up: false,
            down: false,
            left: false,
            right: false,
            key_bindings: KeyBindings::new(),
        }
    }
}

impl std::ops::Add for Vec2<f32> {
    type Output = Vec2<f32>;

    fn add(self, rhs: Self) -> Self::Output {
        Vec2::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl std::ops::AddAssign for Vec2<f32> {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl std::ops::Mul<f32> for Vec2<f32> {
    type Output = Vec2<f32>;

    fn mul(self, rhs: f32) -> Self::Output {
        Vec2::new(self.x * rhs, self.y * rhs)
    }
}
