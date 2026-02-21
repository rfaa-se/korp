use std::net::{IpAddr, Ipv4Addr};

use korp_engine::{
    Core,
    color::Color,
    input::Input,
    misc::Morph,
    renderer::{Camera, Renderer},
    shapes::Rectangle,
};
use korp_math::{Flint, Vec2, lerp};

use crate::{
    bus::{
        Bus,
        events::{self, CosmosEvent, CosmosIntent, Event, NetworkIntent},
    },
    constellation::Constellation,
    ecs::{
        commands::{Command, SpawnKind},
        cosmos::Cosmos,
        entities::Entity,
    },
    keybindings::KeyBindings,
    network::Network,
    nexus::Nexus,
};

pub struct Korp {
    bus: Bus,
    nexus: Nexus,
    network: Network,
}

pub struct Kernel {
    key_bindings: KeyBindings,
    commands: Vec<Command>,
    actions: Vec<Action>,
    toggle: bool,
    camera: Camera,
    camera_target: Morph<Vec2<f32>>,
    player_id: Option<Entity>,
    others: Vec<Entity>,
    launched: bool,
}

enum Action {
    Toggle,
    Command(Command),
    Init,
    PlayerDead,
    Connect,
}

impl Kernel {
    pub fn new() -> Self {
        Self {
            key_bindings: KeyBindings::new(),
            commands: Vec::new(),
            actions: Vec::new(),
            toggle: false,
            camera: Camera::new(1000.0, 1000.0),
            camera_target: Morph::one(Vec2::new(0.0, 0.0)),
            player_id: None,
            others: Vec::new(),
            launched: false,
        }
    }

    pub fn update(&mut self, bus: &mut Bus) {
        self.actions(bus);
        bus.send(CosmosIntent::PlayerCommands(std::mem::take(
            &mut self.commands,
        )));
    }

    pub fn event(&mut self, event: &Event) {
        if let Event::Cosmos(events::Cosmos::Event(CosmosEvent::TrackedMovement {
            entity,
            centroid,
        })) = event
        {
            if let Some(player_id) = self.player_id {
                // make sure the camera tracks the player
                if *entity == player_id {
                    self.camera_target.old = self.camera_target.new;
                    self.camera_target.new = (*centroid).into();
                }

                return;
            }
        }

        if let Event::Cosmos(events::Cosmos::Event(CosmosEvent::TrackedDeath(entity))) = event {
            if let Some(player_id) = self.player_id {
                // when player is dead, set the new value as the old to prevent wobbling
                if *entity == player_id {
                    self.camera_target.old = self.camera_target.new;

                    self.actions.push(Action::PlayerDead);
                }
            }

            return;
        }

        if let Event::Cosmos(events::Cosmos::Event(CosmosEvent::Spawned(entity))) = event {
            if let Some(player_id) = self.player_id {
                if *entity != player_id {
                    self.others.push(*entity);
                }
            }

            return;
        }

        if let Event::Cosmos(events::Cosmos::Event(CosmosEvent::Died(entity))) = event {
            self.others.retain(|x| x != entity);

            return;
        }

        if let Event::Network(events::Network::Response(NetworkResponse::Hosted)) = event {
            self.actions.push(Action::Connect);

            return;
        }

        if let Event::Network(events::Network::Response(NetworkResponse::Connected { ip, id })) =
            event
        {
            self.launched = true;

            return;
        }

        let Event::Nexus(events::Nexus::Event(event)) = event else {
            return;
        };

        match event {
            NexusEvent::Resized { width, height } => {
                self.camera.resize(*width, *height);
            }
            NexusEvent::Init => {
                self.actions.push(Action::Init);
            }
            NexusEvent::Exit => {}
        }
    }

    pub fn input(&mut self, input: &Input) {
        if input.down(&self.key_bindings.up) {
            if let Some(player_id) = self.player_id {
                self.actions
                    .push(Action::Command(Command::Accelerate(player_id)));
            }

            for entity in self.others.iter() {
                self.actions
                    .push(Action::Command(Command::Accelerate(*entity)));
            }
        }

        if input.down(&self.key_bindings.down) {
            if let Some(player_id) = self.player_id {
                self.actions
                    .push(Action::Command(Command::Decelerate(player_id)));
            }

            for entity in self.others.iter() {
                self.actions
                    .push(Action::Command(Command::Decelerate(*entity)));
            }
        }

        if input.down(&self.key_bindings.left) {
            if let Some(player_id) = self.player_id {
                self.actions
                    .push(Action::Command(Command::TurnLeft(player_id)));
            }

            for entity in self.others.iter() {
                self.actions
                    .push(Action::Command(Command::TurnLeft(*entity)));
            }
        }

        if input.down(&self.key_bindings.right) {
            if let Some(player_id) = self.player_id {
                self.actions
                    .push(Action::Command(Command::TurnRight(player_id)));
            }

            for entity in self.others.iter() {
                self.actions
                    .push(Action::Command(Command::TurnRight(*entity)));
            }
        }

        if input.is_pressed(&self.key_bindings.toggle) {
            self.actions.push(Action::Toggle);
        }

        if input.is_pressed(&self.key_bindings.triangle) {
            self.actions.push(Action::Command(Command::Spawn {
                id: None,
                kind: SpawnKind::Triangle,
                centroid: Vec2::new(
                    Flint::from_i16(input.mouse.x as i16),
                    Flint::from_i16(input.mouse.y as i16),
                ),
            }));
        }

        if input.is_pressed(&self.key_bindings.rectangle) {
            self.actions.push(Action::Command(Command::Spawn {
                id: None,
                kind: SpawnKind::Rectangle,
                centroid: Vec2::new(
                    Flint::from_i16(input.mouse.x as i16),
                    Flint::from_i16(input.mouse.y as i16),
                ),
            }));
        }
    }

    pub fn render(&mut self, cosmos: &Cosmos, renderer: &mut Renderer, alpha: f32) {
        {
            self.camera.reposition(Vec2::new(
                lerp(self.camera_target.old.x, self.camera_target.new.x, alpha),
                lerp(self.camera_target.old.y, self.camera_target.new.y, alpha),
            ));

            // render cosmos using the camera
            let scope = renderer.begin(&self.camera);
            cosmos.render(scope.renderer, self.toggle, alpha);
        }

        // render ui
        renderer.draw_rectangle_lines(
            Rectangle::from(800.0, 120.0, Vec2::new(400.0, 540.0)),
            Vec2::new(1.0, 0.0),
            Vec2::new(400.0, 540.0),
            Color::GREEN,
        );
    }

    fn actions(&mut self, bus: &mut Bus) {
        while let Some(action) = self.actions.pop() {
            match action {
                Action::Toggle => {
                    self.toggle = !self.toggle;
                }
                Action::Command(command) => {
                    self.commands.push(command);
                }
                Action::Init => {
                    bus.send(NetworkIntent::Host);
                }
                Action::PlayerDead => {
                    // use the next spawn as the player
                    self.player_id.generation += 1;

                    bus.send(CosmosIntent::TrackMovement(self.player_id));
                    bus.send(CosmosIntent::TrackDeath(self.player_id));
                }
                Action::Connect => {
                    bus.send(NetworkIntent::Connect(IpAddr::V4(Ipv4Addr::LOCALHOST)));
                }
            }
        }
    }
}

impl Korp {
    pub fn new() -> Self {
        Self {
            bus: Bus::new(),
            nexus: Nexus::new(),
            network: Network::new(),
        }
    }
}

impl Core for Korp {
    fn update(&mut self) {
        self.bus.update(&mut self.nexus, &mut self.network);
        self.network.update(&mut self.bus);
        self.nexus.update(&mut self.bus);
    }

    fn input(&mut self, input: &Input) {
        self.nexus.input(input);
    }

    fn render(&mut self, renderer: &mut Renderer, alpha: f32) {
        self.nexus.render(renderer, alpha);
    }

    fn event(&mut self, event: &korp_engine::CoreEvent) {
        self.bus.send(Event::Core(*event));
    }
}
