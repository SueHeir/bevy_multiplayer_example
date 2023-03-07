use bevy::prelude::*;
use bevy_quinnet::client::Client;

use crate::players::{self, ControlledPlayer, Player};

use crate::map::{self, Edge, Material, Vertex};
use crate::protocol;

pub fn setup(mut commands: Commands) {
    commands.spawn(super::ClientAbilityState("MOVE".to_string()));
}

pub fn move_my_player(
    client: ResMut<Client>,
    query_state: Query<&super::ClientAbilityState>,
    my_player: Query<&players::Player, With<players::ControlledPlayer>>,
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
    if state.0 == "MOVE".to_string() {
        if let Ok(player) = my_player.get_single() {
            if let Ok((_vertex, adj, _click)) = vertexes.get(player.current_vertex) {
                let adj_verts = adj.vertex_list.clone();

                for adj_vert in adj_verts {
                    if let Ok((vertex, _adj, mut click)) = vertexes.get_mut(adj_vert) {
                        if click.selected && !vertex.filled {
                            click.selected = false;
                            let temp = client.connection().send_message(
                                protocol::ClientMessage::SendEvent {
                                    name: "MOVE".to_string(),
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
