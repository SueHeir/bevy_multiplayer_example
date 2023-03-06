use std::{collections::HashMap, thread::sleep, time::Duration};

use bevy::{
    app::{AppExit, ScheduleRunnerSettings},
    prelude::*,
};
use bevy_quinnet::{
    client::{
        certificate::CertificateVerificationMode, connection::ConnectionConfiguration, Client,
        QuinnetClientPlugin,
    },
    shared::ClientId,
};

mod camera;
mod map;
mod players;

use protocol::{ClientMessage, ServerMessage};
mod protocol;

#[derive(Resource, Debug, Clone, Default)]
struct Users {
    self_id: ClientId,
    names: HashMap<ClientId, String>,
}

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
        ConnectionConfiguration::new(
            std::net::IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1)).to_string(),
            6000,
            std::net::IpAddr::V4(std::net::Ipv4Addr::new(0, 0, 0, 0)).to_string(),
            0,
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

fn handle_server_messages(mut users: ResMut<Users>, mut client: ResMut<Client>) {
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
        }
    }
}

fn main() {
    App::new()
        // run the server at a reduced tick rate (35 ticks per second)
        .insert_resource(ScheduleRunnerSettings::run_loop(Duration::from_secs_f32(
            1. / 35.,
        )))
        .add_plugins(
            DefaultPlugins
                .set(
                    // here we configure the main window
                    WindowPlugin {
                        window: WindowDescriptor {
                            title: "Catan Server".to_owned(),
                            width: 320.0,
                            height: 180.0,

                            ..Default::default()
                        },

                        ..Default::default()
                    },
                )
                .set(ImagePlugin::default_nearest()),
        )
        .add_startup_system(setup)
        .add_plugin(map::MapPlugin)
        .add_plugin(players::PlayersPlugin)
        .add_plugin(camera::CameraPlugin)
        .add_plugin(QuinnetClientPlugin::default())
        .insert_resource(Users::default())
        .add_startup_system(start_connection)
        .add_system(handle_server_messages)
        .add_system(on_app_exit)
        .add_system(handle_server_messages)
        .run();
}
