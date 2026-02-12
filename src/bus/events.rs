use std::net::IpAddr;

use korp_math::{Flint, Vec2};

use crate::{
    commands::{Command, SpawnKind},
    ecs::entities::Entity,
};

#[derive(Debug)]
pub enum Event {
    Cosmos(Cosmos),
    Kernel(Kernel),
    Network(Network),
}

#[derive(Debug)]
pub enum Cosmos {
    Request(CosmosRequest),
    Event(CosmosEvent),
}

#[derive(Debug)]
pub enum CosmosRequest {
    Commands(Vec<Command>),
    Spawn {
        kind: SpawnKind,
        position: Vec2<Flint>,
    },
    TrackDeath(Entity),
    TrackMovement(Entity),
}

#[derive(Debug)]
pub enum CosmosEvent {
    Spawned(Entity),
    Died(Entity),
    TrackedDeath(Entity),
    TrackedMovement {
        entity: Entity,
        centroid: Vec2<Flint>,
    },
}

#[derive(Debug)]
pub enum Kernel {
    Event(KernelEvent),
}

#[derive(Debug)]
pub enum KernelEvent {
    Init,
    Exit,
    Resized { width: f32, height: f32 },
}

#[derive(Debug)]
pub enum Network {
    Request(NetworkRequest),
    Response(NetworkResponse),
    Event(NetworkEvent),
}

#[derive(Debug)]
pub enum NetworkRequest {
    Connect(IpAddr),
}

#[derive(Debug)]
pub enum NetworkResponse {
    Connected,
}

#[derive(Debug)]
pub enum NetworkEvent {
    Connected(IpAddr),
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

impl From<CosmosRequest> for Cosmos {
    fn from(value: CosmosRequest) -> Self {
        Cosmos::Request(value)
    }
}

impl From<CosmosRequest> for Event {
    fn from(value: CosmosRequest) -> Self {
        Cosmos::Request(value).into()
    }
}

impl From<Kernel> for Event {
    fn from(value: Kernel) -> Self {
        Event::Kernel(value)
    }
}

impl From<KernelEvent> for Event {
    fn from(value: KernelEvent) -> Self {
        Kernel::Event(value).into()
    }
}
