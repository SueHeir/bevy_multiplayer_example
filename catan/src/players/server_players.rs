use bevy::prelude::*;

use super::*;
use map;
pub fn server_spawn_players(
    mut commands: Commands,
    mut player_spawn: EventReader<PlayerSpawnEvent>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    player_textures: Res<PlayerTextures>,
    mut query: Query<(Entity, &mut Transform, &mut map::Vertex), With<map::VertexStart>>,
    mut total_players: Local<u32>,
) {
    let mut start_spot: Vec<Entity> = Vec::new();
    let mut x: Vec<f32> = Vec::new();
    let mut y: Vec<f32> = Vec::new();
    for (entity, pos, vertex) in query.iter() {
        if !vertex.filled && vertex.is_start {
            start_spot.push(entity);
            x.push(pos.translation.x);
            y.push(pos.translation.y);
        }
    }
    if start_spot.len() < player_spawn.len() {
        panic!("Not enough room for players to join server");
    }
    for (i, _player) in player_spawn.iter().enumerate() {
        let texture_atlas = TextureAtlas::from_grid(
            player_textures.player.clone(),
            Vec2::new(player_textures.player_x, player_textures.player_y),
            player_textures.player_col,
            player_textures.player_row,
            Some(Vec2::new(
                player_textures.padding_x,
                player_textures.padding_y,
            )),
            None,
        );
        let texture_atlas_handle = texture_atlases.add(texture_atlas);

        println!("{:?}", start_spot[i]);
        if let Ok((entity, pos, mut vertex)) = query.get_mut(start_spot[i]) {
            vertex.filled = true;

            let _entity = commands
                .spawn(SpriteSheetBundle {
                    texture_atlas: texture_atlas_handle,
                    transform: Transform::from_xyz(pos.translation.x, pos.translation.y, 100.0),
                    sprite: TextureAtlasSprite {
                        anchor: Anchor::Custom(Vec2 { x: 0.0, y: -0.3 }),
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .insert(Player {
                    id: *total_players,
                    current_vertex: entity,
                    current_vertex_id: vertex.id,
                    next_entity: Vec::new(),
                    next_entity_id: None,
                    animation_timer: 0.0,
                    roation_index: 0,
                    client_owner_id: _player.client_owner_id,
                    state: super::States::Idle,
                })
                // .insert(
                //     StateMachine::new(Idle)
                //         .trans::<AnyState>(GoToTrigger, GoToSelection { speed: 65.0 })
                //         .trans::<AnyState>(NotTrigger(GoToTrigger), Idle),
                // )
                .id();

            *total_players += 1;
        } else {
            println!("Failed to spawn player on server")
        }

        // if spawn.vertex_start {
        //     commands.entity(entity).insert(VertexStart);
        // }
    }
}

pub fn send_game_state(
    // mut client_event: EventReader<protocol::ClientEvent>,
    server: ResMut<bevy_quinnet::server::Server>,
    players: Query<(Entity, &mut Transform, &mut Player), With<Player>>,
    users: Res<protocol::Users>,
    mut timer: Local<f32>,
    time: Res<Time>,
) {
    *timer += time.delta_seconds();

    if *timer < 1.0 / 20.0 {
        return;
    } else {
        *timer -= 1.0 / 20.0;

        let mut players_data = Vec::<protocol::Player>::new();

        for (_e, pos, player) in players.iter() {
            players_data.push(protocol::Player {
                id: player.id,
                x: pos.translation.x,
                y: pos.translation.y,
                rotation: player.roation_index,
                current_vertex: player.current_vertex_id,
                next_vertex: player.next_entity_id,
                client_owner_id: player.client_owner_id,
            })
        }
        // println!("{:?}", users.names.keys().into_iter());

        if let Ok(_temp) = server.endpoint().send_group_message(
            users.names.keys().into_iter(),
            //ChannelId::Unreliable,
            protocol::ServerMessage::UpdatePlayers {
                players: players_data.clone(),
            },
        ) {
            // info!("Sent Players")
        } else {
            info!("Failed to Update Players")
        }
    }
}

pub(crate) fn handle_client_move_player(
    mut client_event: EventReader<protocol::ClientEvent>,
    mut players: Query<&mut Player, Without<map::Vertex>>,
    vertexes: Query<
        (
            Entity,
            &map::Vertex,
            &map::EntityAdjacencies,
            &map::MapClickable,
        ),
        (With<map::Vertex>, Without<Player>),
    >,
) {
    for event in client_event.iter() {
        if event.name != protocol::ClientEvents::MOVE {
            continue;
        }

        let mut target_vert = None;
        for (e, vert, _ent_adj, _click) in vertexes.iter() {
            if vert.id == event.type_id {
                target_vert = Some(e);
                break;
            }
        }

        if target_vert.is_none() {
            continue;
        }

        for mut player in players.iter_mut() {
            if player.client_owner_id == event.client_id {
                let vert = vertexes.get(target_vert.unwrap()).unwrap();

                if vert.2.vertex_list.contains(&player.current_vertex) && !vert.1.filled {
                    if player.state == super::States::Idle {
                        player.next_entity.push(vert.0);
                        break;
                    }
                }
            }
        }
    }
}
