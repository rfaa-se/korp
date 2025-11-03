use korp_engine::{
    Core,
    color::Color,
    input::{Input, KeyCode},
    misc::Morph,
    renderer::Canvas,
};
use korp_math::Vec2;

use crate::components::{Body, Rectangle, Shape, Triangle, traits::Drawable};

pub struct Korp {
    bodies: Vec<Morph<Body>>,
    up: bool,
    down: bool,
    left: bool,
    right: bool,
    toggle: bool,
    key_bindings: KeyBindings,
}

struct KeyBindings {
    up: KeyCode,
    down: KeyCode,
    left: KeyCode,
    right: KeyCode,
}

impl Korp {
    pub fn new() -> Self {
        Self {
            bodies: Vec::new(),
            up: false,
            down: false,
            left: false,
            right: false,
            toggle: false,
            key_bindings: KeyBindings::new(),
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

        if input.is_pressed(&KeyCode::F1) {
            self.toggle = !self.toggle;
        }

        let rotation = Vec2 { x: 0.0, y: -1.0 };

        if input.is_pressed(&KeyCode::Space) {
            let body = Body {
                centroid: input.mouse,
                rotation,
                shape: Shape::Triangle(Triangle {
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

        if input.is_pressed(&KeyCode::AltLeft) {
            let body = Body {
                centroid: input.mouse,
                rotation,
                shape: Shape::Rectangle(Rectangle {
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
            body.draw(canvas, self.toggle);
        }
    }
}

impl KeyBindings {
    fn new() -> Self {
        Self {
            up: KeyCode::ArrowUp,
            down: KeyCode::ArrowDown,
            left: KeyCode::ArrowLeft,
            right: KeyCode::ArrowRight,
        }
    }
}
