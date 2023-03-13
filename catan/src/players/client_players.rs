use super::*;

use bevy_quinnet::client::Client;
pub fn client_spawn_players(
    mut commands: Commands,
    mut player_spawn: EventReader<PlayerSpawnEvent>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    player_textures: Res<PlayerTextures>,
    mut query: Query<(Entity, &mut Transform, &mut map::Vertex), With<map::Vertex>>,
    users: ResMut<protocol::Users>,
) {
    for (_i, player) in player_spawn.iter().enumerate() {
        if !player.current_vertex.is_some() {
            info!("current vertex should be filled for clients");
            return;
        }

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

        if let Ok((entity, pos, mut vertex)) = query.get_mut(player.current_vertex.unwrap()) {
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
                    id: player.id.unwrap(),
                    current_vertex: entity,
                    current_vertex_id: vertex.id,
                    next_entity: Vec::new(),
                    next_entity_id: None,
                    animation_timer: 0.0,
                    roation_index: 0,
                    client_owner_id: player.client_owner_id,
                    state: super::States::Idle,
                })
                // .insert(
                //     StateMachine::new(Idle)
                //         .trans::<AnyState>(GoToTrigger, GoToSelection { speed: 65.0 })
                //         .trans::<AnyState>(NotTrigger(GoToTrigger), Idle),
                // )
                .id();
            if users.self_id == player.client_owner_id {
                commands.entity(_entity).insert(ControlledPlayer);
            }
        } else {
            println!("Failed to spawn player on client")
        }
    }
}

pub fn move_my_player(
    client: ResMut<Client>,
    query_state: Query<&protocol::CurrentClientEventTrigger>,
    my_player: Query<&Player, With<ControlledPlayer>>,
    mut vertexes: Query<
        (
            &map::Vertex,
            &map::EntityAdjacencies,
            &mut map::MapClickable,
        ),
        With<map::Vertex>,
    >,
) {
    let state = query_state.single();
    if state.0 == protocol::ClientEvents::MOVE {
        if let Ok(player) = my_player.get_single() {
            if player.state == super::States::MoveToEntity {
                return;
            }

            if let Ok((_vertex, adj, _click)) = vertexes.get(player.current_vertex) {
                let adj_verts = adj.vertex_list.clone();

                for adj_vert in adj_verts {
                    if let Ok((vertex, _adj, mut click)) = vertexes.get_mut(adj_vert) {
                        if click.selected && !vertex.filled {
                            click.selected = false;
                            let temp = client.connection().send_message(
                                protocol::ClientMessage::SendEvent {
                                    name: protocol::ClientEvents::MOVE,
                                    map_type: map::VERTEX,
                                    type_id: vertex.id,
                                },
                            );
                            if temp.is_err() {
                                println!("Error with sending Move Event")
                            }
                        }
                    }
                }
            }
        }
    }
}
