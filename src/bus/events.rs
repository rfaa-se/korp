use std::net::IpAddr;

use korp_engine::CoreEvent as Core;
use korp_math::{Flint, Vec2};

use crate::{
    ecs::{
        commands::{Command, SpawnKind},
        entities::Entity,
    },
    nexus::{State, game, lobby, menu},
};

#[derive(Debug)]
pub enum Event {
    Cosmos(Cosmos),
    Network(Network),
    Core(Core),
    Nexus(Nexus),
    Internal(Internal),
}

#[derive(Debug)]
pub enum Cosmos {
    Intent(CosmosIntent),
    Event(CosmosEvent),
}

#[derive(Debug)]
pub enum CosmosIntent {
    Spawn {
        id: Option<usize>,
        kind: SpawnKind,
        centroid: Vec2<Flint>,
    },
    TrackDeath(Entity),
    TrackMovement(Entity),
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
pub enum Network {
    Intent(NetworkIntent),
    Event(NetworkEvent),
}

#[derive(Debug)]
pub enum NetworkIntent {
    Host,
    Connect(IpAddr),
    Disconnect,
    Launch,
    Commands { tick: usize, commands: Vec<Command> },
}

#[derive(Debug)]
pub enum NetworkEvent {
    Hosted {
        id: usize,
    },
    Connected {
        id: usize,
    },
    Launched,
    Disconnected {
        id: usize,
    },
    Commands {
        id: usize,
        tick: usize,
        commands: Vec<Command>,
    },
}

#[derive(Debug)]
pub enum Nexus {
    Intent(NexusIntent),
    Event(NexusEvent),
}

#[derive(Debug)]
pub enum NexusIntent {
    Transition(State),
}

#[derive(Debug)]
pub enum NexusEvent {
    Transitioned(State),
}

#[derive(Debug)]
pub enum Internal {
    Menu(menu::Action),
    Lobby(lobby::Action),
    Game(game::Action),
}

impl From<Cosmos> for Event {
    fn from(value: Cosmos) -> Self {
        Event::Cosmos(value)
    }
}

impl From<CosmosEvent> for Cosmos {
    fn from(value: CosmosEvent) -> Self {
        Cosmos::Event(value)
    }
}

impl From<CosmosEvent> for Event {
    fn from(value: CosmosEvent) -> Self {
        Cosmos::Event(value).into()
    }
}

impl From<CosmosIntent> for Cosmos {
    fn from(value: CosmosIntent) -> Self {
        Cosmos::Intent(value)
    }
}

impl From<CosmosIntent> for Event {
    fn from(value: CosmosIntent) -> Self {
        Cosmos::Intent(value).into()
    }
}

impl From<Network> for Event {
    fn from(value: Network) -> Self {
        Event::Network(value)
    }
}

impl From<NetworkIntent> for Network {
    fn from(value: NetworkIntent) -> Self {
        Network::Intent(value)
    }
}

impl From<NetworkIntent> for Event {
    fn from(value: NetworkIntent) -> Self {
        Network::Intent(value).into()
    }
}

impl From<NetworkEvent> for Event {
    fn from(value: NetworkEvent) -> Self {
        Network::Event(value).into()
    }
}

impl From<NexusEvent> for Event {
    fn from(value: NexusEvent) -> Self {
        Nexus::Event(value).into()
    }
}

impl From<NexusIntent> for Event {
    fn from(value: NexusIntent) -> Self {
        Nexus::Intent(value).into()
    }
}

impl From<Nexus> for Event {
    fn from(value: Nexus) -> Self {
        Event::Nexus(value)
    }
}

impl From<Internal> for Event {
    fn from(value: Internal) -> Self {
        Event::Internal(value)
    }
}
