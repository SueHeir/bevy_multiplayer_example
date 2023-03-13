use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    thread::sleep,
    time::Duration,
};

use bevy::{
    app::{AppExit, ScheduleRunnerSettings},
    prelude::*,
};
use bevy_quinnet::client::{
    certificate::CertificateVerificationMode, connection::ConnectionConfiguration, Client,
    QuinnetClientPlugin,
};

mod camera;
mod client;
mod map;
mod players;

use players::PlayerSpawnEvent;
use protocol::{ClientMessage, ServerMessage};
mod protocol;

fn setup(mut _commands: Commands) {

    //stuff here if needed
}

pub fn on_app_exit(app_exit_events: EventReader<AppExit>, client: Res<Client>) {
    if !app_exit_events.is_empty() {
        client
            .connection()
            .send_message(ClientMessage::Disconnect {})
            .unwrap();
        // TODO Clean: event to let the async client send his last messages.
        sleep(Duration::from_secs_f32(0.1));
    }
}

fn start_connection(mut client: ResMut<Client>) {
    match client.open_connection(
        ConnectionConfiguration::from_addrs(
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 6000),
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 0),
        ),
        CertificateVerificationMode::SkipVerification,
    ) {
        Ok(info) => {
            println!("started connection {} with ChannelId: {:?}", info.0, info.1);
        }

        Err(e) => {
            println!("failed to connect, Error {:?}", e);
        }
    }
    // When trully connected, you will receive a ConnectionEvent
}

fn join_game(mut client: ResMut<Client>) {
    let x = rand::random::<u32>();
    if let Ok(_temp) = client.connection_mut().send_message(ClientMessage::Join {
        name: "Test".to_owned() + &x.to_string(),
    }) {
    } else {
        println!("Failed to Join Game");
    }
}

fn handle_server_messages(
    mut users: ResMut<protocol::Users>,
    mut client: ResMut<Client>,
    mut init_map: EventWriter<map::MapObjectSpawnEvent>,
    mut init_player: EventWriter<players::PlayerSpawnEvent>,
    mut update_map: EventWriter<protocol::ServerUpdateMapEvent>,
    mut players_query: Query<
        (Entity, &mut Transform, &mut players::Player),
        (With<players::Player>, Without<map::Vertex>),
    >,
    vertex_query: Query<(Entity, &mut map::Vertex), (With<map::Vertex>, Without<players::Player>)>,

    mut map_is_init: Local<bool>,
    mut commands: Commands,
) {
    while let Some(message) = client
        .connection_mut()
        .try_receive_message::<ServerMessage>()
    {
        match message {
            ServerMessage::ClientConnected {
                client_id,
                username,
            } => {
                info!("{} joined", username);
                users.names.insert(client_id, username);
            }
            ServerMessage::ClientDisconnected { client_id } => {
                if let Some(username) = users.names.remove(&client_id) {
                    println!("{} left", username);
                } else {
                    warn!("ClientDisconnected for an unknown client_id: {}", client_id)
                }
            }
            ServerMessage::ChatMessage { client_id, message } => {
                if let Some(username) = users.names.get(&client_id) {
                    if client_id != users.self_id {
                        println!("{}: {}", username, message);
                    }
                } else {
                    warn!("Chat message from an unknown client_id: {}", client_id)
                }
            }
            ServerMessage::InitClient {
                client_id,
                usernames,
            } => {
                users.self_id = client_id;
                users.names = usernames;
            }

            ServerMessage::InitMap {
                vertexes,
                edges,
                materials,
            } => {
                for vert in vertexes.iter() {
                    init_map.send(map::MapObjectSpawnEvent {
                        map_type: map::VERTEX,
                        map_type_id: vert.id,
                        x: vert.x,
                        y: vert.y,
                        roation: 0.0,
                        edge_list: vert.adjacentices.edge_list.clone(),
                        vertex_list: vert.adjacentices.vertex_list.clone(),
                        material_list: vert.adjacentices.material_list.clone(),
                        material_type: None,
                        vertex_start: vert.is_start_vertex,
                    })
                }
                for edge in edges.iter() {
                    init_map.send(map::MapObjectSpawnEvent {
                        map_type: map::EDGE,
                        map_type_id: edge.id,
                        x: edge.x,
                        y: edge.y,
                        roation: edge.rotation,
                        edge_list: edge.adjacentices.edge_list.clone(),
                        vertex_list: edge.adjacentices.vertex_list.clone(),
                        material_list: edge.adjacentices.material_list.clone(),
                        material_type: None,
                        vertex_start: false,
                    })
                }
                for material in materials.iter() {
                    init_map.send(map::MapObjectSpawnEvent {
                        map_type: map::MATERIAL,
                        map_type_id: material.id,
                        x: material.x,
                        y: material.y,
                        roation: 0.0,
                        edge_list: material.adjacentices.edge_list.clone(),
                        vertex_list: material.adjacentices.vertex_list.clone(),
                        material_list: material.adjacentices.material_list.clone(),
                        material_type: Some(material.material_type),
                        vertex_start: false,
                    })
                }
                *map_is_init = true;
            }
            ServerMessage::UpdateMap {
                vertexes,
                edges,
                materials,
            } => update_map.send(protocol::ServerUpdateMapEvent {
                vertexes,
                edges,
                materials,
            }),
            ServerMessage::UpdatePlayers { players } => {
                if *map_is_init {
                    for play in players.iter() {
                        let mut vert = None;
                        let mut next_vert = None;

                        for (e, vertex) in vertex_query.iter() {
                            if vertex.id == play.current_vertex {
                                vert = Some(e);
                            }
                            if play.next_vertex.is_some() {
                                if vertex.id == play.next_vertex.unwrap() {
                                    next_vert = Some(e);
                                }
                            }
                        }

                        if vert.is_some() {
                            let mut player_found = false;
                            for (_e, mut pos, mut player) in players_query.iter_mut() {
                                if play.id == player.id {
                                    player_found = true;

                                    // pos.translation =
                                    //     pos.translation.lerp(Vec3::new(play.x, play.y, 100.0), 1.0);
                                    // if player.state = players::States::Idle
                                    // println!("{:?}", play.current_vertex);
                                    // println!("{:?}", play.next_vertex);
                                    player.current_vertex = vert.unwrap();
                                    player.roation_index = play.rotation;
                                    if next_vert.is_some() {
                                        if !player.next_entity.contains(&next_vert.unwrap()) {
                                            player.next_entity.push(next_vert.unwrap());
                                        }

                                        //println!("is_some")
                                    } else {
                                        player.next_entity.clear();
                                        player.next_entity_id = None;
                                        pos.translation.x = play.x;
                                        pos.translation.y = play.y;
                                        //println!("else")
                                    }
                                    player.next_entity_id = play.next_vertex;
                                }
                            }

                            if !player_found {
                                init_player.send(PlayerSpawnEvent {
                                    current_vertex: vert,
                                    x: Some(play.x),
                                    y: Some(play.y),
                                    id: Some(play.id),
                                    client_owner_id: play.client_owner_id,
                                });
                                info!("Created New Player");
                            }
                        } else {
                            info!("Can't find vertex to spawn player")
                        }
                    }
                    for (e, _pos, player) in players_query.iter() {
                        let mut player_found = false;
                        for play in players.iter() {
                            if play.id == player.id {
                                player_found = true;
                            }
                        }

                        if !player_found {
                            commands.entity(e).despawn();
                            info!("Deleted Player")
                        }
                    }
                }
            }
        }
    }
}

fn main() {
    App::new()
        .insert_resource(protocol::IsServer(false))
        // run the server at a reduced tick rate (35 ticks per second)
        .insert_resource(ScheduleRunnerSettings::run_loop(Duration::from_secs_f32(
            1. / 30.,
        )))
        .add_plugins(
            DefaultPlugins
                .set(
                    // here we configure the main window
                    WindowPlugin {
                        // window: WindowDescriptor {
                        //     title: "Catan Client".to_owned(),
                        //     width: 640.0,
                        //     height: 360.0,

                        //     ..Default::default()
                        // },
                        ..Default::default()
                    },
                )
                .set(ImagePlugin::default_nearest()),
        )
        .add_startup_system(setup)
        .add_plugin(map::ClientMapPlugin)
        .add_plugin(players::ClientPlayersPlugin)
        .add_plugin(camera::ClientCameraPlugin)
        .add_plugin(client::ClientPlugin)
        .add_plugin(QuinnetClientPlugin::default())
        .insert_resource(protocol::Users::default())
        .add_startup_system(start_connection.in_base_set(StartupSet::PreStartup))
        .add_startup_system(join_game.in_base_set(StartupSet::Startup))
        .add_system(handle_server_messages)
        .add_system(on_app_exit)
        .add_system(handle_server_messages)
        .run();
}
