use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};

use korp_math::Vec2;

use crate::renderer::{RawRenderer, Renderer};
use crate::{input::Input, renderer::Camera};

pub trait Core {
    type RenderData: Send + Sync + 'static;
    // type RenderConfigurator: Fn(&Self::RenderData, &mut Renderer, f32) + Send + Sync;
    // type Job: FnMut(Arc<Self::RenderData>) + Send + Sync;

    fn update(&mut self) -> Self::RenderData;
    fn input(&mut self, input: &Input);
    // fn prepare(&self, data: &mut Self::RenderData);
    // fn prepare(&self) -> Self::RenderConfigurator;
    // fn render(&mut self, renderer: &mut Renderer, alpha: f32);
    // fn init(&mut self);
    // fn exit(&mut self);
    fn resize(&mut self, width: u32, height: u32);
    // fn data(&self) -> Self::RenderData;
    // fn r(&self) -> Self::Job;
    fn render(
        data: &Self::RenderData,
        renderer: &mut Renderer<Self::RenderData>,
        camera: &mut Camera,
        alpha: f32,
    );
    // fn work(&mut self) -> Self::RenderData;
}

pub struct Engine<T: Core> {
    state: State<T::RenderData>,
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
    // buffer: [Arc<T::RenderData>; 2],
    // active: Arc<AtomicBool>,
}

enum State<T: Send + Sync + 'static> {
    Uninitialized,
    Initialized {
        renderer: RawRenderer<T>,
        // window must be last, otherwise segfault at exit
        window: Arc<winit::window::Window>,
    },
}

impl<T: Core> Engine<T> {
    const ONE: Duration = Duration::from_secs(1);

    pub fn new(tps: u8, core: T, title: &str) -> Self {
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
            // buffer: [Arc::new(buffer[0]), Arc::new(buffer[1])],
            // active: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn run(&mut self) {
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
            let renderer = pollster::block_on(async {
                RawRenderer::new(window.clone(), width, height, T::render).await
            });

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
                let (w, h) = (width.max(1), height.max(1));

                renderer.resize(w, h);

                self.core.resize(w, h);
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

                    core.input(input);
                    let data = core.update();
                    renderer.update(data);
                    input.update();

                    // let current = self.active.load(Ordering::Relaxed);
                    // let idx = if current { 0 } else { 1 };
                    // let data = Arc::get_mut(&mut self.buffer[idx]).unwrap();
                    // core.prepare(data);
                    // self.active.store(!current, Ordering::Release);
                }

                let alpha = self.accumulator.as_secs_f32() / self.timestep.as_secs_f32();

                renderer.render(alpha);

                // let mut renderer = renderer.begin();
                // ren(todo!(), &mut renderer, alpha);
                // core.render(&mut renderer, alpha);
                // let active = self.active.clone();
                // let mut a = self.buffer[0].clone();
                // let mut b = self.buffer[1].clone();
                // let mut buf = [a, b];
                // std::thread::spawn(move || {
                //     T::render(&mut data, &mut renderer, alpha);
                // });

                // std::thread::spawn(move || {
                //     loop {
                //         let idx = if active.load(Ordering::Acquire) { 1 } else { 0 };
                //         let abc = buf[idx];
                //         // T::render(&self.core, &*d, &mut renderer, alpha);
                //     }
                // });

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
