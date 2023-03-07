use std::collections::HashMap;

use bevy::prelude::Entity;
use bevy_quinnet::shared::ClientId;
use serde::{Deserialize, Serialize};

use crate::map;

pub struct ClientEvent {
    pub name: String,
    pub map_type: u8,
    pub type_id: u32,
    pub client_id: ClientId,
}

#[derive(bevy::prelude::Resource, Debug, Clone, Default)]
pub struct Users {
    pub self_id: ClientId,
    pub names: HashMap<ClientId, String>,
}

#[derive(bevy::prelude::Resource, Debug, Clone, Default)]
pub struct IsServer(pub bool);

// Messages from clients
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClientMessage {
    Join {
        name: String,
    },
    Disconnect {},
    ChatMessage {
        message: String,
    },
    SendEvent {
        name: String,
        map_type: u8,
        type_id: u32,
    },
}

// Messages from the server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServerMessage {
    ClientConnected {
        client_id: ClientId,
        username: String,
    },
    ClientDisconnected {
        client_id: ClientId,
    },
    ChatMessage {
        client_id: ClientId,
        message: String,
    },
    InitClient {
        client_id: ClientId,
        usernames: HashMap<ClientId, String>,
    },
    InitMap {
        vertexes: Vec<Vertex>,
        edges: Vec<Edge>,
        materials: Vec<Material>,
    },
    UpdatePlayers {
        players: Vec<Player>,
    },
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vertex {
    pub id: u32,
    pub adjacentices: map::Adjacencies,
    pub x: f32,
    pub y: f32,
    pub is_start_vertex: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    pub id: u32,
    pub x: f32,
    pub y: f32,
    pub rotation: i32,
    pub current_vertex: u32,
    pub next_vertex: u32,
    pub client_owner_id: ClientId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Edge {
    pub id: u32,
    pub adjacentices: map::Adjacencies,
    pub x: f32,
    pub y: f32,
    pub rotation: f32,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Material {
    pub id: u32,
    pub adjacentices: map::Adjacencies,
    pub x: f32,
    pub y: f32,
    pub material_type: u8,
}

// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct Adjacencies {
//     pub edge_list: Vec<u32>,
//     pub vertex_list: Vec<u32>,
//     pub material_list: Vec<u32>,
// }
