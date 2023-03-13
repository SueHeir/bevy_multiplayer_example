use bevy::prelude::*;

use super::*;

pub fn setup(mut map_generator: EventWriter<MapObjectSpawnEvent>) {
    let map_file_name = "./assets/levels/level_3.json";

    let map_file = {
        // Load the first file into a string.
        let text = std::fs::read_to_string(&map_file_name).unwrap();

        // Parse the string into a dynamically-typed JSON structure.
        serde_json::from_str::<MapInitData>(&text).unwrap()
    };

    let vertex_positions = map_file.vertex_positions.clone();
    let vertex_connections = map_file.vertex_connections.clone();
    let mana_connections = map_file.mana_connections.clone();

    for i in 0..map_file.vertex_positions.len() {
        let mut edge_list = Vec::new();
        let mut vertex_list = Vec::new();
        let mut material_list = Vec::new();

        for j in 0..mana_connections.len() {
            if mana_connections[j].b == i as i32 {
                material_list.push(mana_connections[j].a as u32);
            }
        }

        for j in 0..vertex_connections.len() {
            if vertex_connections[j].a == i as i32 {
                vertex_list.push(vertex_connections[j].b as u32);
                edge_list.push(j as u32);
            }
            if vertex_connections[j].b == i as i32 {
                vertex_list.push(vertex_connections[j].a as u32);
                edge_list.push(j as u32);
            }
        }

        let map_spawn = MapObjectSpawnEvent {
            map_type: VERTEX,
            map_type_id: i as u32,
            x: vertex_positions[i].x,
            y: vertex_positions[i].y,
            roation: 0.0,
            material_type: None,
            edge_list,
            vertex_list,
            material_list,
            vertex_start: map_file.map_start_vertexes.contains(&(i as u32)),
        };
        map_generator.send(map_spawn);
    }

    for i in 0..map_file.vertex_connections.len() {
        let edge_list = Vec::new();
        let mut vertex_list = Vec::new();
        let material_list = Vec::new();

        vertex_list.push(vertex_connections[i].a as u32);
        vertex_list.push(vertex_connections[i].b as u32);

        let a = vertex_positions[vertex_connections[i].a as usize];
        let b = vertex_positions[vertex_connections[i].b as usize];

        let pos = (Vec2::new(a.x, a.y) + Vec2::new(b.x, b.y)) / 2.0;

        let map_spawn = MapObjectSpawnEvent {
            map_type: EDGE,
            map_type_id: i as u32,
            x: pos.x,
            y: pos.y,
            roation: (a.y - b.y).atan2(a.x - b.x),

            material_type: None,
            edge_list,
            vertex_list,
            material_list,
            vertex_start: false,
        };
        map_generator.send(map_spawn);
    }

    for (i, mana_positions) in map_file.mana_points.iter().enumerate() {
        let mut color = 0;

        let edge_list = Vec::new();
        let vertex_list = Vec::new();
        let mut material_list = Vec::new();

        for j in 0..mana_connections.len() {
            if mana_connections[j].a == i as i32 {
                material_list.push(mana_connections[j].b as u32);
            }
        }

        match mana_positions.color.as_str() {
            "blue" => color = BLUE,
            "yellow" => color = YELLOW,
            "red" => color = RED,
            "green" => color = GREEN,
            "purple" => color = PURPLE,
            "orange" => color = ORANGE,
            _ => {}
        }

        let map_spawn = MapObjectSpawnEvent {
            map_type: MATERIAL,
            map_type_id: i as u32,
            x: mana_positions.x,
            y: mana_positions.y,
            roation: 0.0,
            edge_list,
            vertex_list,
            material_list,
            material_type: Some(color),
            vertex_start: false,
        };
        map_generator.send(map_spawn);
    }
}

pub struct InitMapSend {
    pub client_id: u64,
}

pub fn handle_init_map_send(
    mut init_map_event: EventReader<InitMapSend>,
    vertexes: Query<
        (Entity, &Transform, &Adjacencies, &Vertex),
        (With<Vertex>, Without<Edge>, Without<super::Material>),
    >,
    edges: Query<
        (Entity, &Transform, &Adjacencies, &Edge),
        (With<Edge>, Without<Vertex>, Without<super::Material>),
    >,
    materials: Query<
        (
            Entity,
            &Transform,
            &Adjacencies,
            &super::Material,
            &MapClickable,
        ),
        (With<super::Material>, Without<Vertex>, Without<Edge>),
    >,
    mut server: ResMut<Server>,
) {
    if init_map_event.len() > 0 {
        let mut vertexes_data = Vec::<protocol::Vertex>::new();
        let mut edges_data = Vec::<protocol::Edge>::new();
        let mut materials_data = Vec::<protocol::Material>::new();

        for (_e, pos, adj, vert) in vertexes.iter() {
            vertexes_data.push(protocol::Vertex {
                id: vert.id,
                adjacentices: adj.clone(),
                x: pos.translation.x,
                y: pos.translation.y,
                is_start_vertex: vert.is_start,
            })
        }

        for (_e, pos, adj, edge) in edges.iter() {
            edges_data.push(protocol::Edge {
                id: edge.id,
                adjacentices: adj.clone(),
                x: pos.translation.x,
                y: pos.translation.y,
                rotation: edge.roation,
            })
        }

        for (_e, pos, adj, mat, click) in materials.iter() {
            materials_data.push(protocol::Material {
                id: mat.0,
                adjacentices: adj.clone(),
                x: pos.translation.x,
                y: pos.translation.y,
                material_type: click.mana_type,
            })
        }

        for init_map in init_map_event.iter() {
            if let Ok(_result) = server.endpoint_mut().send_message(
                init_map.client_id,
                protocol::ServerMessage::InitMap {
                    vertexes: vertexes_data.clone(),
                    edges: edges_data.clone(),
                    materials: materials_data.clone(),
                },
            ) {
                info!("Sent map to Client");
            } else {
                info!("Failed to send map to Client");
            }
        }
    }
}

pub fn update_map(
    query_vertexes: Query<(Entity, &mut Vertex), With<Vertex>>,
    // query_edges: Query<(Entity, &mut Edge), (With<Edge>, Without<Vertex>, Without<Material>)>,
    // query_materials: Query<
    //     (Entity, &mut Material),
    //     (With<Material>, Without<Edge>, Without<Vertex>),
    // >,
    server: ResMut<bevy_quinnet::server::Server>,
    users: Res<protocol::Users>,
    mut timer: Local<f32>,
    time: Res<Time>,
) {
    *timer += time.delta_seconds();

    if *timer < 1.0 / 35.0 {
        return;
    } else {
        *timer -= 1.0 / 35.0;

        let mut vertexes: Vec<protocol::VertexUpdate> = Vec::new();
        let mut _edges: Vec<protocol::EdgeUpdate> = Vec::new();
        let mut _materials: Vec<protocol::MaterialUpdate> = Vec::new();

        for (_e, vertex) in query_vertexes.iter() {
            vertexes.push(protocol::VertexUpdate {
                id: vertex.id,
                filled: vertex.filled,
            })
        }

        if let Ok(_temp) = server.endpoint().send_group_message(
            users.names.keys().into_iter(),
            //ChannelId::Unreliable,
            protocol::ServerMessage::UpdateMap {
                vertexes,
                edges: _edges,
                materials: _materials,
            },
        ) {
            // info!("Sent Players")
        } else {
            info!("Failed to Update Map")
        }
    }
}
