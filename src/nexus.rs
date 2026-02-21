use korp_engine::{input::Input, renderer::Renderer};

use crate::{
    bus::{
        Bus,
        events::{self, Event, NexusEvent, NexusIntent},
    },
    nexus::{game::Game, lobby::Lobby, menu::Menu},
};

mod game;
mod lobby;
mod menu;

pub struct Nexus {
    state: State,
    actions: Vec<Action>,
}

#[derive(Debug, Copy, Clone)]
pub enum NexusState {
    Menu,
    Lobby { id: usize, host: bool },
    Game { id: usize },
}

enum Action {
    Transition(NexusState),
}

enum State {
    Menu(Menu),
    Lobby(Lobby),
    Game(Game),
}

impl Nexus {
    pub fn new() -> Self {
        Self {
            state: State::Menu(Menu::new()),
            actions: Vec::new(),
        }
    }

    pub fn update(&mut self, bus: &mut Bus) {
        self.action(bus);

        match &mut self.state {
            State::Menu(menu) => menu.update(bus),
            State::Lobby(lobby) => lobby.update(bus),
            State::Game(game) => game.update(bus),
        }
    }

    pub fn event(&mut self, event: &Event) {
        match &mut self.state {
            State::Menu(menu) => menu.event(event),
            State::Lobby(lobby) => lobby.event(event),
            State::Game(game) => game.event(event),
        }

        let Event::Nexus(events::Nexus::Intent(event)) = event else {
            return;
        };

        match event {
            NexusIntent::Transition(state) => {
                self.actions.push(Action::Transition(*state));
            }
        }
    }

    pub fn render(&mut self, renderer: &mut Renderer, alpha: f32) {
        match &mut self.state {
            State::Menu(menu) => menu.render(renderer, alpha),
            State::Lobby(lobby) => lobby.render(renderer, alpha),
            State::Game(game) => game.render(renderer, alpha),
        }
    }

    pub fn input(&mut self, input: &Input) {
        match &mut self.state {
            State::Menu(menu) => menu.input(input),
            State::Lobby(lobby) => lobby.input(input),
            State::Game(game) => game.input(input),
        }
    }

    fn action(&mut self, bus: &mut Bus) {
        while let Some(action) = self.actions.pop() {
            match action {
                Action::Transition(state) => {
                    let new = match state {
                        NexusState::Menu => State::Menu(Menu::new()),
                        NexusState::Lobby { id, host } => State::Lobby(Lobby::new(id, host)),
                        NexusState::Game { id } => State::Game(Game::new(id)),
                    };

                    self.state = new;
                    bus.send(NexusEvent::Transitioned(state));
                }
            }
        }
    }
}
