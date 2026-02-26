use korp_engine::{input::Input, renderer::Renderer};

use crate::{
    bus::{
        Bus,
        events::{self, Event, NexusEvent, NexusIntent},
    },
    nexus::{game::Game, lobby::Lobby, menu::Menu},
};

pub mod game;
pub mod lobby;
pub mod menu;

pub struct Nexus {
    context: Context,
    actions: Vec<Action>,
}

#[derive(Debug, Clone)]
pub enum State {
    Menu,
    Lobby {
        id: usize,
        host: bool,
    },
    Game {
        id: usize,
        ids: Vec<usize>,
        seed: u64,
    },
}

enum Action {
    Transition(State),
}

enum Context {
    Menu(Menu),
    Lobby(Lobby),
    Game(Game),
}

impl Nexus {
    pub fn new() -> Self {
        Self {
            context: Context::Menu(Menu::new()),
            actions: Vec::new(),
        }
    }

    pub fn update(&mut self, bus: &mut Bus) {
        self.action(bus);

        match &mut self.context {
            Context::Menu(menu) => menu.update(bus),
            Context::Lobby(lobby) => lobby.update(bus),
            Context::Game(game) => game.update(bus),
        }
    }

    pub fn event(&mut self, event: &Event) {
        match &mut self.context {
            Context::Menu(menu) => menu.event(event),
            Context::Lobby(lobby) => lobby.event(event),
            Context::Game(game) => game.event(event),
        }

        let Event::Nexus(events::Nexus::Intent(event)) = event else {
            return;
        };

        match event {
            NexusIntent::Transition(state) => {
                self.actions.push(Action::Transition((*state).clone()));
            }
        }
    }

    pub fn render(&mut self, renderer: &mut Renderer, alpha: f32) {
        match &mut self.context {
            Context::Menu(menu) => menu.render(renderer, alpha),
            Context::Lobby(lobby) => lobby.render(renderer, alpha),
            Context::Game(game) => game.render(renderer, alpha),
        }
    }

    pub fn input(&mut self, input: &Input) {
        match &mut self.context {
            Context::Menu(menu) => menu.input(input),
            Context::Lobby(lobby) => lobby.input(input),
            Context::Game(game) => game.input(input),
        }
    }

    fn action(&mut self, bus: &mut Bus) {
        while let Some(action) = self.actions.pop() {
            match action {
                Action::Transition(state) => {
                    let context = match state.clone() {
                        State::Menu => Context::Menu(Menu::new()),
                        State::Lobby { id, host } => Context::Lobby(Lobby::new(id, host)),
                        State::Game { id, ids, seed } => Context::Game(Game::new(id, ids, seed)),
                    };

                    self.context = context;
                    bus.send(NexusEvent::Transitioned(state));
                }
            }
        }
    }
}
