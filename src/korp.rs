use korp_engine::{
    Core,
    color::Color,
    input::{Input, KeyCode},
    misc::Morph,
    renderer::{Camera, Renderer},
    shapes::Rectangle,
};
use korp_math::{Flint, Vec2, lerp};

use crate::{
    commands::{Command, Spawn},
    ecs::{components::traits::Renderable, cosmos::Cosmos, entities::Entity},
};

pub struct Korp {
    cosmos: Cosmos,
    key_bindings: KeyBindings,
    commands: Vec<Command>,
    actions: Vec<Action>,
    toggle: bool,
    camera: Camera,
    camera_target: Morph<Vec2<f32>>,
    player_id: Entity,
}

struct KeyBindings {
    up: KeyCode,
    down: KeyCode,
    left: KeyCode,
    right: KeyCode,
    toggle: KeyCode,
    triangle: KeyCode,
    rectangle: KeyCode,
}

enum Action {
    Toggle,
    Command(Command),
}

impl Korp {
    pub fn new() -> Self {
        Self {
            cosmos: Cosmos::new(),
            key_bindings: KeyBindings::new(),
            commands: Vec::new(),
            actions: Vec::new(),
            toggle: false,
            camera: Camera::new(1000.0, 1000.0),
            camera_target: Morph::one(Vec2::new(0.0, 0.0)),
            // TODO: need a way to track local player entity
            player_id: Entity {
                index: 0,
                generation: 0,
            },
        }
    }
}

pub struct RenderData {
    bodies: Vec<Morph<crate::ecs::components::Body<f32>>>,
    hitboxes: Vec<Morph<Rectangle<f32>>>,
    camera_target: Morph<Vec2<f32>>,
}

impl Core for Korp {
    type RenderData = RenderData;

    fn update(&mut self) -> RenderData {
        self.commands.clear();

        while let Some(action) = self.actions.pop() {
            match action {
                Action::Toggle => {
                    self.toggle = !self.toggle;
                }
                Action::Command(command) => {
                    self.commands.push(command);
                }
            }
        }

        self.cosmos.update(&self.commands);

        // make sure the camera tracks the player
        if let Some(body) = self.cosmos.components.logic.bodies.get(&self.player_id) {
            self.camera_target.old = body.old.centroid.into();
            self.camera_target.new = body.new.centroid.into();
        } else {
            // when player is dead, set the new value as the old to prevent wobbling
            // TODO: listen to when player dies and set the camera target then, once
            self.camera_target.old = self.camera_target.new;
        }

        RenderData {
            bodies: self
                .cosmos
                .components
                .render
                .bodies
                .iter()
                .map(|(_, body)| body.clone())
                .collect(),
            hitboxes: self
                .cosmos
                .components
                .render
                .hitboxes
                .iter()
                .map(|(_, x)| x.clone())
                .collect(),
            camera_target: self.camera_target,
        }
    }

    fn input(&mut self, input: &Input) {
        if input.down(&self.key_bindings.up) {
            for (entity, _) in self.cosmos.components.logic.motions.iter() {
                self.actions
                    .push(Action::Command(Command::Accelerate(*entity)));
            }
        }

        if input.down(&self.key_bindings.down) {
            for (entity, _) in self.cosmos.components.logic.motions.iter() {
                self.actions
                    .push(Action::Command(Command::Decelerate(*entity)));
            }
        }

        if input.down(&self.key_bindings.left) {
            for (entity, _) in self.cosmos.components.logic.motions.iter() {
                self.actions
                    .push(Action::Command(Command::TurnLeft(*entity)));
            }
        }

        if input.down(&self.key_bindings.right) {
            for (entity, _) in self.cosmos.components.logic.motions.iter() {
                self.actions
                    .push(Action::Command(Command::TurnRight(*entity)));
            }
        }

        if input.is_pressed(&self.key_bindings.toggle) {
            self.actions.push(Action::Toggle);
        }

        if input.is_pressed(&self.key_bindings.triangle) {
            self.actions.push(Action::Command(Command::Spawn(
                Spawn::Triangle,
                Vec2::new(
                    Flint::from_i16(input.mouse.x as i16),
                    Flint::from_i16(input.mouse.y as i16),
                ),
            )));
        }

        if input.is_pressed(&self.key_bindings.rectangle) {
            self.actions.push(Action::Command(Command::Spawn(
                Spawn::Rectangle,
                Vec2::new(
                    Flint::from_i16(input.mouse.x as i16),
                    Flint::from_i16(input.mouse.y as i16),
                ),
            )));
        }
    }

    // fn render(&mut self, renderer: &mut Renderer, alpha: f32) {
    //     self.camera.reposition(Vec2::new(
    //         lerp(self.camera_target.old.x, self.camera_target.new.x, alpha),
    //         lerp(self.camera_target.old.y, self.camera_target.new.y, alpha),
    //     ));

    //     {
    //         // render cosmos using the camera
    //         let scope = renderer.begin(&self.camera);

    //         self.cosmos.render(scope.renderer, self.toggle, alpha);
    //     }

    //     // render ui
    //     renderer.draw_rectangle_lines(
    //         Rectangle::from(800.0, 120.0, Vec2::new(400.0, 540.0)),
    //         Vec2::new(1.0, 0.0),
    //         Vec2::new(400.0, 540.0),
    //         Color::GREEN,
    //     );
    // }

    fn resize(&mut self, width: u32, height: u32) {
        self.camera.resize(width as f32, height as f32);
    }

    fn render(
        data: &RenderData,
        renderer: &mut Renderer<RenderData>,
        camera: &mut Camera,
        alpha: f32,
    ) {
        camera.reposition(Vec2::new(
            lerp(data.camera_target.old.x, data.camera_target.new.x, alpha),
            lerp(data.camera_target.old.y, data.camera_target.new.y, alpha),
        ));

        {
            // render cosmos using the camera
            let scope = renderer.begin(&camera);

            // self.cosmos.render(scope.renderer, self.toggle, alpha);
            for body in data.bodies.iter() {
                body.render(scope.renderer, false, alpha);
            }
        }

        // render ui
        renderer.draw_rectangle_lines(
            Rectangle::from(800.0, 120.0, Vec2::new(400.0, 540.0)),
            Vec2::new(1.0, 0.0),
            Vec2::new(400.0, 540.0),
            Color::GREEN,
        );
    }
    // // type RenderConfigurator = Fn(&RenderData, &mut Renderer, f32);
    // type RenderConfigurator = fn(&RenderData, &mut Renderer, f32);

    // fn prepare(&self) -> Self::RenderConfigurator {
    //     move |data, renderer, alpha| {
    //         // let scope = renderer.begin(&self.camera);
    //         for body in data.bodies.iter() {
    //             // body.render(scope.renderer, false, alpha);
    //         }

    //         for hitbox in data.hitboxes.iter() {}
    //     }
    // }

    // type RenderData = RenderData;

    // fn prepare(&self, data: &mut RenderData) {
    //     todo!()
    // }

    // fn render(&self, data: &RenderData, renderer: &mut Renderer, alpha: f32) {
    //     todo!()
    // }
}

// impl Korp {
//     fn render(&self) -> RenderData {
//         RenderData {
//             bodies: self
//                 .cosmos
//                 .components
//                 .render
//                 .bodies
//                 .iter()
//                 .map(|(_, body)| body.clone())
//                 .collect(),
//             hitboxes: self
//                 .cosmos
//                 .components
//                 .render
//                 .hitboxes
//                 .iter()
//                 .map(|(_, x)| x.clone())
//                 .collect(),
//         }
//     }
// }

impl KeyBindings {
    fn new() -> Self {
        Self {
            up: KeyCode::ArrowUp,
            down: KeyCode::ArrowDown,
            left: KeyCode::ArrowLeft,
            right: KeyCode::ArrowRight,
            toggle: KeyCode::F1,
            triangle: KeyCode::Digit1,
            rectangle: KeyCode::Digit2,
        }
    }
}
