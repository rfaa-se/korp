use korp_engine::{
    Core,
    color::Color,
    input::{Input, KeyCode},
    misc::Morph,
    renderer::Canvas,
};
use korp_math::{Flint, Vec2};

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
        let speed = 20;
        let rotation_speed: Flint = 12.into();

        for body in self.bodies.iter_mut() {
            body.old = body.new;

            if self.up {
                body.new.centroid += body.new.rotation * speed;
            }

            if self.down {}

            if self.left {
                body.new.rotation = body.new.rotation.rotated(-rotation_speed);
            }

            if self.right {
                body.new.rotation = body.new.rotation.rotated(rotation_speed);
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

        let rotation = Vec2 {
            x: Flint::ZERO,
            y: Flint::NEG_ONE,
        };

        if input.is_pressed(&KeyCode::Space) {
            let body = Body {
                centroid: Vec2::new((input.mouse.x as i16).into(), (input.mouse.y as i16).into()),
                rotation,
                shape: Shape::Triangle(Triangle {
                    top: Vec2::new(0.into(), (-50 as i16).into()),
                    left: Vec2::new((-30 as i16).into(), 25.into()),
                    right: Vec2::new(30.into(), 25.into()),
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
                centroid: Vec2::new((input.mouse.x as i16).into(), (input.mouse.y as i16).into()),
                rotation,
                shape: Shape::Rectangle(Rectangle {
                    width: 60.into(),
                    height: 40.into(),
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
