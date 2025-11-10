use korp_engine::{misc::Morph, renderer::Canvas};

use crate::{
    commands::Command,
    ecs::{
        components::{Body, Motion},
        forge::Forge,
        sparse_set::SparseSet,
        systems::{Executor, Renderer},
    },
};

pub struct Components {
    pub bodies: SparseSet<Morph<Body>>,
    pub motions: SparseSet<Motion>,
}

pub struct Cosmos {
    pub components: Components,
    forge: Forge,
    executor: Executor,
    renderer: Renderer,
}

impl Cosmos {
    pub fn new() -> Self {
        Self {
            components: Components::new(),
            forge: Forge::new(),
            executor: Executor::new(),
            renderer: Renderer::new(),
        }
    }

    pub fn update(&mut self, commands: &[Command]) {
        self.executor
            .execute(&mut self.components, &mut self.forge, commands);
    }

    pub fn render(&self, canvas: &mut Canvas, toggle: bool) {
        self.renderer.render(&self.components, canvas, toggle);
    }
}

impl Components {
    pub fn new() -> Self {
        Self {
            bodies: SparseSet::new(u16::MAX as usize),
            motions: SparseSet::new(u16::MAX as usize),
        }
    }
}
