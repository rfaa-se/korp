use std::net::IpAddr;

use korp_engine::CoreEvent;
use korp_math::{Flint, Vec2};

use crate::{
    ecs::{
        commands::{Command, SpawnKind},
        entities::Entity,
        tracker::Track,
    },
    network,
    nexus::{self, game, lobby, menu},
};

#[derive(Debug)]
pub enum Event {
    Cosmos(IntentEvent<CosmosIntent, CosmosEvent>),
    Network(IntentEvent<NetworkIntent, NetworkEvent>),
    Core(CoreEvent),
    Nexus(IntentEvent<NexusIntent, NexusEvent>),
    Menu(MenuEvent),
    Lobby(LobbyEvent),
    Game(GameEvent),
}

#[derive(Debug)]
pub enum IntentEvent<TIntent, TEvent> {
    Intent(TIntent),
    Event(TEvent),
}

#[derive(Debug)]
pub enum CosmosIntent {
    Command(Command),
    Track(Track),
}

#[derive(Debug)]
pub enum CosmosEvent {
    Spawned {
        id: Option<usize>,
        entity: Entity,
    },
    Died(Entity),
    TrackedDeath(Entity),
    TrackedMovement {
        entity: Entity,
        centroid: Vec2<Flint>,
    },
}

#[derive(Debug)]
pub enum NetworkIntent {
    Host,
    Connect(IpAddr),
    Disconnect,
    Launch,
    Commands { tick: usize, commands: Vec<Command> },
    Pause,
    Resume,
}

#[derive(Debug)]
pub enum NetworkEvent {
    Action(network::Action),
    Hosted {
        id: usize,
    },
    Connected {
        id: usize,
    },
    Launched {
        seed: u64,
        delay: usize,
    },
    Disconnected {
        id: usize,
    },
    Commands {
        id: usize,
        tick: usize,
        commands: Vec<Command>,
    },
    Paused,
    Resumed,
}

#[derive(Debug)]
pub enum NexusIntent {
    Transition(nexus::State),
}

#[derive(Debug)]
pub enum NexusEvent {
    Action(nexus::Action),
    Transitioned(nexus::State),
}

#[derive(Debug)]
pub enum MenuEvent {
    Action(menu::Action),
    Transitioned(menu::State),
}

#[derive(Debug)]
pub enum LobbyEvent {
    Action(lobby::Action),
    Transitioned(lobby::State),
}

#[derive(Debug)]
pub enum GameEvent {
    Action(game::Action),
    Transitioned(game::State),
    Toggled(bool),
}

impl From<CosmosEvent> for Event {
    fn from(value: CosmosEvent) -> Self {
        Event::Cosmos(IntentEvent::Event(value))
    }
}

impl From<CosmosIntent> for Event {
    fn from(value: CosmosIntent) -> Self {
        Event::Cosmos(IntentEvent::Intent(value))
    }
}

impl From<NetworkIntent> for Event {
    fn from(value: NetworkIntent) -> Self {
        Event::Network(IntentEvent::Intent(value))
    }
}

impl From<NetworkEvent> for Event {
    fn from(value: NetworkEvent) -> Self {
        Event::Network(IntentEvent::Event(value))
    }
}

impl From<NexusEvent> for Event {
    fn from(value: NexusEvent) -> Self {
        Event::Nexus(IntentEvent::Event(value))
    }
}

impl From<NexusIntent> for Event {
    fn from(value: NexusIntent) -> Self {
        Event::Nexus(IntentEvent::Intent(value))
    }
}

impl From<MenuEvent> for Event {
    fn from(value: MenuEvent) -> Self {
        Event::Menu(value)
    }
}

impl From<LobbyEvent> for Event {
    fn from(value: LobbyEvent) -> Self {
        Event::Lobby(value)
    }
}

impl From<GameEvent> for Event {
    fn from(value: GameEvent) -> Self {
        Event::Game(value)
    }
}
