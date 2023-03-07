use bevy::prelude::*;
use bevy_quinnet::client::Client;

use crate::players::{self, ControlledPlayer, Player};

use crate::map::{self, Edge, Material, Vertex};
use crate::protocol;

pub(crate) fn handle_client_move_player(
    mut client_event: EventReader<protocol::ClientEvent>,
    mut players: Query<&mut players::Player, Without<Vertex>>,
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
        if event.name != "MOVE".to_string() {
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
                    player.next_entity.push(vert.0);
                    break;
                }
            }
        }
    }
}
