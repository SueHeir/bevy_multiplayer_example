use std::{collections::HashMap, time::Duration};

use bevy::{app::ScheduleRunnerSettings, prelude::*};
use bevy_quinnet::{
    server::{
        certificate::CertificateRetrievalMode, ConnectionLostEvent, Endpoint, QuinnetServerPlugin,
        Server, ServerConfigurationData,
    },
    shared::{channel::ChannelId, ClientId},
};

use protocol::{ClientEvent, ClientMessage, ServerMessage};

mod camera;
mod map;
mod players;
mod protocol;
mod server;

fn handle_client_messages(
    mut server: ResMut<Server>,
    mut users: ResMut<protocol::Users>,
    mut player_spawn: EventWriter<players::PlayerSpawnEvent>,
    mut client_event: EventWriter<protocol::ClientEvent>,
    mut init_map: EventWriter<map::InitMapSend>,
) {
    let endpoint = server.endpoint_mut();
    for client_id in endpoint.clients() {
        while let Some(message) = endpoint.try_receive_message_from::<ClientMessage>(client_id) {
            match message {
                ClientMessage::Join { name } => {
                    if users.names.contains_key(&client_id) {
                        warn!(
                            "Received a Join from an already connected client: {}",
                            client_id
                        )
                    } else {
                        info!("{} connected", name);
                        users.names.insert(client_id, name.clone());
                        // Initialize this client with existing state
                        endpoint
                            .send_message(
                                client_id,
                                ServerMessage::InitClient {
                                    client_id: client_id,
                                    usernames: users.names.clone(),
                                },
                            )
                            .unwrap();
                        // Broadcast the connection event
                        endpoint
                            .send_group_message(
                                users.names.keys().into_iter(),
                                ServerMessage::ClientConnected {
                                    client_id: client_id,
                                    username: name,
                                },
                            )
                            .unwrap();
                        //Send Map
                        init_map.send(map::InitMapSend { client_id });
                        //Spawn Player
                        player_spawn.send(players::PlayerSpawnEvent {
                            current_vertex: None,
                            x: None,
                            y: None,
                            id: None,
                            client_owner_id: client_id,
                        });
                    }
                }
                ClientMessage::Disconnect {} => {
                    // We tell the server to disconnect this user
                    endpoint.disconnect_client(client_id).unwrap();
                    handle_disconnect(endpoint, &mut users, client_id);
                }
                ClientMessage::ChatMessage { message } => {
                    info!(
                        "Chat message | {:?}: {}",
                        users.names.get(&client_id),
                        message
                    );
                    endpoint.try_send_group_message_on(
                        users.names.keys().into_iter(),
                        ChannelId::UnorderedReliable,
                        ServerMessage::ChatMessage {
                            client_id: client_id,
                            message: message,
                        },
                    );
                }
                ClientMessage::SendEvent {
                    name,
                    map_type,
                    type_id,
                } => client_event.send(ClientEvent {
                    name,
                    map_type,
                    type_id,
                    client_id,
                }),
            }
        }
    }
}

fn handle_server_events(
    mut connection_lost_events: EventReader<ConnectionLostEvent>,
    mut server: ResMut<Server>,
    mut users: ResMut<protocol::Users>,
) {
    // The server signals us about users that lost connection
    for client in connection_lost_events.iter() {
        handle_disconnect(server.endpoint_mut(), &mut users, client.id);
    }
}

/// Shared disconnection behaviour, whether the client lost connection or asked to disconnect
fn handle_disconnect(
    endpoint: &mut Endpoint,
    users: &mut ResMut<protocol::Users>,
    client_id: ClientId,
) {
    // Remove this user
    if let Some(username) = users.names.remove(&client_id) {
        // Broadcast its deconnection

        endpoint
            .send_group_message(
                users.names.keys().into_iter(),
                ServerMessage::ClientDisconnected {
                    client_id: client_id,
                },
            )
            .unwrap();
        info!("{} disconnected", username);
    } else {
        warn!(
            "Received a Disconnect from an unknown or disconnected client: {}",
            client_id
        )
    }
}

#[derive(Resource)]
struct PlayerChannel(ChannelId);

fn start_listening(mut server: ResMut<Server>, mut commands: Commands) {
    server
        .start_endpoint(
            ServerConfigurationData::new("127.0.0.1".to_string(), 6000, "0.0.0.0".to_string()),
            CertificateRetrievalMode::GenerateSelfSigned,
        )
        .unwrap();

    let player_channel: PlayerChannel = PlayerChannel(
        server
            .endpoint_mut()
            .open_channel(bevy_quinnet::shared::channel::ChannelType::OrderedReliable)
            .unwrap(),
    );

    commands.insert_resource(player_channel);
}

fn main() {
    App::new()
        .insert_resource(protocol::IsServer(true))
        // run the server at a reduced tick rate (35 ticks per second)
        .insert_resource(ScheduleRunnerSettings::run_loop(Duration::from_secs_f32(
            1. / 30.,
        )))
        .add_plugins(
            DefaultPlugins
                .set(
                    // here we configure the main window
                    WindowPlugin {
                        window: WindowDescriptor {
                            title: "Catan Server".to_owned(),
                            width: 640.0,
                            height: 360.0,

                            ..Default::default()
                        },

                        ..Default::default()
                    },
                )
                .set(ImagePlugin::default_nearest()),
        )
        .add_startup_system(setup)
        .add_plugin(QuinnetServerPlugin::default())
        .add_plugin(map::MapPlugin)
        .add_plugin(players::PlayersPlugin)
        .add_plugin(camera::CameraPlugin)
        .add_plugin(server::ServerPlugin)
        .insert_resource(protocol::Users::default())
        .add_startup_system(start_listening)
        .add_system(handle_client_messages)
        .add_system(handle_server_events)
        .add_event::<protocol::ClientEvent>()
        // .add_system(send_game_state)
        .run();
}

fn setup(mut _commands: Commands) {

    //stuff here if needed
}
