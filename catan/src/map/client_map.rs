use bevy::prelude::*;

use super::*;
use crate::protocol;

pub fn update_map(
    mut update_map: EventReader<protocol::ServerUpdateMapEvent>,
    mut query_vertexes: Query<(Entity, &mut Vertex), With<Vertex>>,
    // query_edges: Query<(Entity, &mut Edge), (With<Edge>, Without<Vertex>, Without<Material>)>,
    // query_materials: Query<
    //     (Entity, &mut Material),
    //     (With<Material>, Without<Edge>, Without<Vertex>),
    // >,
) {
    let update = update_map.iter().last();

    if update.is_none() {
        return;
    }

    for vertex in update.unwrap().vertexes.iter() {
        for (_e, mut vert) in query_vertexes.iter_mut() {
            if vert.id == vertex.id {
                vert.filled = vertex.filled;
                break;
            }
        }
    }
}
