use bevy::prelude::*;

use super::*;
use crate::protocol;

pub fn update_map(
    mut update_map: EventReader<protocol::ServerUpdateMapEvent>,
    mut query_vertexes: Query<(Entity, &mut Vertex), With<Vertex>>,
    vertex_lookup: Res<VertexClientServerLookup>,
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
        if let Some(v) = vertex_lookup.0.get(&vertex.id) {
            if let Ok((_e, mut vert)) = query_vertexes.get_mut(*v) {
                vert.filled = vertex.filled;
                continue;
            } else {
                info!("failed to query vertex from a vertex lookup")
            }
        } else {
            info!("Vertex Lookup Failed")
        }
    }
}
