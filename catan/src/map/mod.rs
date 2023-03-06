use bevy::prelude::*;
use bevy_interact_2d::{Group, Interactable, InteractionState};
use serde::Deserialize;
use serde_json;

pub const MAPCLICKABLE: u8 = 0;
pub const VERTEX: u8 = 1;
pub const EDGE: u8 = 2;
pub const MATERIAL: u8 = 3;

pub const BLUE: u8 = 0;
pub const YELLOW: u8 = 1;
pub const RED: u8 = 2;
pub const GREEN: u8 = 3;
pub const PURPLE: u8 = 4;
pub const ORANGE: u8 = 5;

#[derive(Component)]
struct MapClickable {
    selected: bool,
    map_type: u8,
    mana_type: u8,
    animation_timer: f32,
}

struct MapObjectSpawnEvent {
    map_type: u8,
    map_type_id: u32,
    x: f32,
    y: f32,
    roation: f32,
    edge_list: Vec<u32>,
    vertex_list: Vec<u32>,
    material_list: Vec<u32>,
    material_type: Option<u8>,
    vertex_start: bool,
}

#[derive(Component)]
struct Adjacencies {
    edge_list: Vec<u32>,
    vertex_list: Vec<u32>,
    material_list: Vec<u32>,
}
#[derive(Component)]
pub struct EntityAdjacencies {
    pub edge_list: Vec<Entity>,
    pub vertex_list: Vec<Entity>,
    pub material_list: Vec<Entity>,
}

#[derive(Component)]
pub struct Edge(u32);

#[derive(Component)]
pub struct Vertex {
    id: u32,
    pub filled: bool,
}

#[derive(Component)]
pub struct VertexStart;

#[derive(Component)]
pub struct Material(u32);

#[derive(Resource)]
struct MapTextures {
    vertex: Handle<Image>,
    edge: Handle<Image>,
    mana: Handle<Image>,
    vertex_col: usize,
    vertex_row: usize,
    vertex_x: f32,
    vertex_y: f32,
    edge_col: usize,
    edge_row: usize,
    edge_x: f32,
    edge_y: f32,
    mana_col: usize,
    mana_row: usize,
    mana_x: f32,
    mana_y: f32,
    padding_x: f32,
    padding_y: f32,
}

#[derive(Debug, Deserialize, Clone)]
struct MapInitData {
    map_start_vertexes: Vec<u32>,
    vertex_positions: Vec<MapInitVertexPositions>,
    vertex_connections: Vec<MapInitVertexConnections>,
    mana_points: Vec<MapInitMana>,
    mana_connections: Vec<MapInitManaConnections>,
}

#[derive(Debug, Deserialize, Clone, Copy)]
struct MapInitVertexPositions {
    x: f32,
    y: f32,
}

#[derive(Debug, Deserialize, Clone, Copy)]
struct MapInitVertexConnections {
    a: i32,
    b: i32,
}

#[derive(Debug, Deserialize, Clone)]
struct MapInitMana {
    x: f32,
    y: f32,
    color: String,
}

#[derive(Debug, Deserialize, Clone, Copy)]
struct MapInitManaConnections {
    a: i32,
    b: i32,
}

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system_to_stage(StartupStage::PreStartup, setup)
            .add_startup_system_to_stage(StartupStage::Startup, spawn_map_object_system)
            .add_startup_system_to_stage(StartupStage::PostStartup, setup_entity_adjacencies)
            .add_system(click_map_object)
            .add_event::<MapObjectSpawnEvent>()
            // .add_event::<VertexSelectEvent>()
            // .add_system(vertex_click_to_spawn)
            // .add_system(edge_spawn)
            // .add_system(select_vertex_adjacent_vertexes)
            .add_system(animate_mana);
    }
}
fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut map_generator: EventWriter<MapObjectSpawnEvent>,
) {
    let map_textures = MapTextures {
        vertex: asset_server.load("circle.png"),
        edge: asset_server.load("line.png"),
        mana: asset_server.load("mana_sheet.png"),
        vertex_col: 3,
        vertex_row: 1,
        vertex_x: 32.0,
        vertex_y: 32.0,
        edge_col: 3,
        edge_row: 1,
        edge_x: 32.0,
        edge_y: 2.0,
        mana_col: 6,
        mana_row: 12,
        mana_x: 32.0,
        mana_y: 32.0,
        padding_x: 1.0,
        padding_y: 0.0,
    };

    commands.insert_resource(map_textures);

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

fn animate_mana(
    mut query: Query<(Entity, &mut TextureAtlasSprite, &mut MapClickable), With<Material>>,
    time: Res<Time>,
) {
    for (_entity, mut sprite, mut clickable) in query.iter_mut() {
        clickable.animation_timer += time.delta_seconds() * 1.0;
        if clickable.animation_timer > 6.0 {
            clickable.animation_timer = 0.0;
        }

        sprite.index = 6 * clickable.mana_type as usize
            + 6 * 6 * clickable.selected as usize
            + (clickable.animation_timer) as usize;
    }
}

fn click_map_object(
    interaction_state: Res<InteractionState>,
    mouse_button_input: Res<Input<MouseButton>>,
    mut query: Query<
        (
            Entity,
            &mut TextureAtlasSprite,
            &mut MapClickable,
            &EntityAdjacencies,
        ),
        With<MapClickable>,
    >,
    keys: Res<Input<KeyCode>>,
    // mut vertex_test: EventWriter<VertexSelectEvent>,
) {
    let mut double_selected_entity: Option<Entity> = None;

    for (entity, mut sprite, mut clickable, _adj) in query.iter_mut() {
        if interaction_state
            .get_group(Group(MAPCLICKABLE))
            .iter()
            .find(|(e, _)| *e == entity)
            .is_some()
        {
            if mouse_button_input.just_released(MouseButton::Left) {
                // info!("Clicked.");
                if clickable.selected == true {
                    double_selected_entity = Some(entity.clone());
                }

                if clickable.map_type != MATERIAL {
                    sprite.index = 2;
                }
                clickable.selected = true;
            } else if clickable.selected != true {
                if sprite.index == 0 {
                    // info!("Hover.");
                }
                if clickable.map_type != MATERIAL {
                    sprite.index = 1;
                }
            }
        } else {
            let mut double_select = false;

            for key in keys.get_pressed() {
                match key {
                    KeyCode::LShift => {
                        double_select = true;
                    }
                    _ => {}
                }
            }

            if mouse_button_input.just_released(MouseButton::Left) && !double_select {
                // info!("Deselected.");
                clickable.selected = false;
            }
            if clickable.selected != true {
                if sprite.index == 1 {
                    // info!("Stop Hover.");
                }
                if clickable.map_type != MATERIAL {
                    sprite.index = 0;
                }
            }
        }
    }
    if double_selected_entity.is_some() {
        let mut adjacent = Vec::new();
        if let Ok((_entity, mut _sprite, mut _clickable, adj)) =
            query.get_mut(double_selected_entity.unwrap())
        {
            for vert in adj.vertex_list.iter() {
                adjacent.push(*vert)
            }
            for edge in adj.edge_list.iter() {
                adjacent.push(*edge)
            }
            for material in adj.material_list.iter() {
                adjacent.push(*material)
            }
        }

        for adjac in adjacent.iter() {
            if let Ok((_entity, mut sprite, mut clickable, _adj)) = query.get_mut(*adjac) {
                clickable.selected = true;
                if clickable.map_type != MATERIAL {
                    sprite.index = 2;
                }
            }
        }
    } else {
    }
}

// fn select_vertex_adjacent_vertexes(
//     mut query: Query<(Entity, &mut TextureAtlasSprite, &mut MapClickable, &Vertex), With<Vertex>>,
//     mut vertex_test: EventReader<VertexSelectEvent>,
// ) {
//     for selected in vertex_test.iter() {
//         info!("vertex_test");
//         println!("{:?}", selected.entites);

//         for (entity, mut sprite, mut clickable, vertex) in query.iter_mut() {
//             if selected.entites.contains(&vertex.0) {
//                 clickable.selected = true;
//                 sprite.index = 2;
//             }
//         }
//     }
// }

fn spawn_map_object_system(
    mut spawn_data: EventReader<MapObjectSpawnEvent>,
    mut commands: Commands,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    map_textures: Res<MapTextures>,
) {
    for spawn in spawn_data.iter() {
        if spawn.map_type == VERTEX {
            let texture_atlas = TextureAtlas::from_grid(
                map_textures.vertex.clone(),
                Vec2::new(map_textures.vertex_x, map_textures.vertex_y),
                map_textures.vertex_col,
                map_textures.vertex_row,
                Some(Vec2::new(map_textures.padding_x, map_textures.padding_y)),
                None,
            );
            let texture_atlas_handle = texture_atlases.add(texture_atlas);

            // info!("Spawn map object system");
            // println!("{} {}", spawn.x, spawn.y);
            {
                let entity = commands
                    .spawn(SpriteSheetBundle {
                        texture_atlas: texture_atlas_handle,
                        transform: Transform::from_xyz(spawn.x, spawn.y, 10.0),

                        ..default()
                    })
                    .insert(Interactable {
                        groups: vec![Group(MAPCLICKABLE)],
                        bounding_box: (Vec2::new(-16., -16.), Vec2::new(16., 16.)),
                        ..Default::default()
                    })
                    .insert(Vertex {
                        id: spawn.map_type_id,
                        filled: false,
                    })
                    .insert(Adjacencies {
                        vertex_list: spawn.vertex_list.clone(),
                        edge_list: spawn.edge_list.clone(),
                        material_list: spawn.material_list.clone(),
                    })
                    .insert(EntityAdjacencies {
                        vertex_list: Vec::new(),
                        edge_list: Vec::new(),
                        material_list: Vec::new(),
                    })
                    .insert(MapClickable {
                        selected: false,
                        map_type: VERTEX,
                        animation_timer: 0.0,
                        mana_type: 0,
                    })
                    .id();

                if spawn.vertex_start {
                    commands.entity(entity).insert(VertexStart);
                }
            }
        }
        if spawn.map_type == EDGE {
            let texture_atlas = TextureAtlas::from_grid(
                map_textures.edge.clone(),
                Vec2::new(map_textures.edge_x, map_textures.edge_y),
                map_textures.edge_col,
                map_textures.edge_row,
                Some(Vec2::new(map_textures.padding_x, map_textures.padding_y)),
                None,
            );
            let texture_atlas_handle = texture_atlases.add(texture_atlas);

            // info!("Spawn map object system");
            // println!("{} {}", spawn.x, spawn.y);
            commands
                .spawn(SpriteSheetBundle {
                    texture_atlas: texture_atlas_handle,
                    transform: Transform {
                        translation: Vec3 {
                            x: spawn.x,
                            y: spawn.y,
                            z: 10.0,
                        },
                        rotation: Quat::from_rotation_z(spawn.roation),
                        ..Default::default()
                    },

                    ..default()
                })
                .insert(Interactable {
                    groups: vec![Group(MAPCLICKABLE)],
                    bounding_box: (Vec2::new(-16., -16.), Vec2::new(16., 16.)),
                })
                .insert(Adjacencies {
                    vertex_list: spawn.vertex_list.clone(),
                    edge_list: spawn.edge_list.clone(),
                    material_list: spawn.material_list.clone(),
                })
                .insert(EntityAdjacencies {
                    vertex_list: Vec::new(),
                    edge_list: Vec::new(),
                    material_list: Vec::new(),
                })
                .insert(Edge(spawn.map_type_id))
                .insert(MapClickable {
                    selected: false,
                    map_type: EDGE,
                    animation_timer: 0.0,
                    mana_type: 0,
                });
        }
        if spawn.map_type == MATERIAL {
            let texture_atlas = TextureAtlas::from_grid(
                map_textures.mana.clone(),
                Vec2::new(map_textures.mana_x, map_textures.mana_y),
                map_textures.mana_col,
                map_textures.mana_row,
                Some(Vec2::new(0.0, 0.0)),
                None,
            );
            let texture_atlas_handle = texture_atlases.add(texture_atlas);

            // info!("Spawn map object system");
            // println!("{} {}", spawn.x, spawn.y);
            commands
                .spawn(SpriteSheetBundle {
                    texture_atlas: texture_atlas_handle,
                    transform: Transform::from_xyz(spawn.x, spawn.y, 10.0),

                    ..default()
                })
                .insert(Interactable {
                    groups: vec![Group(MAPCLICKABLE)],
                    bounding_box: (Vec2::new(-16., -16.), Vec2::new(16., 16.)),
                    ..Default::default()
                })
                .insert(Material(spawn.map_type_id))
                .insert(Adjacencies {
                    vertex_list: spawn.vertex_list.clone(),
                    edge_list: spawn.edge_list.clone(),
                    material_list: spawn.material_list.clone(),
                })
                .insert(EntityAdjacencies {
                    vertex_list: Vec::new(),
                    edge_list: Vec::new(),
                    material_list: Vec::new(),
                })
                .insert(MapClickable {
                    selected: false,
                    map_type: spawn.map_type,
                    animation_timer: 0.0,
                    mana_type: spawn.material_type.unwrap(),
                });
        }
    }
}

fn setup_entity_adjacencies(
    mut vertexes: Query<
        (Entity, &mut EntityAdjacencies, &Adjacencies, &Vertex),
        (With<Vertex>, Without<Edge>, Without<Material>),
    >,
    mut edges: Query<
        (Entity, &mut EntityAdjacencies, &Adjacencies, &Edge),
        (With<Edge>, Without<Vertex>, Without<Material>),
    >,
    mut material: Query<
        (Entity, &mut EntityAdjacencies, &Adjacencies, &Material),
        (With<Material>, Without<Vertex>, Without<Edge>),
    >,
) {
    let mut combinations = vertexes.iter_combinations_mut();
    while let Some(
        [(entity1, mut entity_adjacencies1, adjacencies1, vertex1), (entity2, mut entity_adjacencies2, adjacencies2, vertex2)],
    ) = combinations.fetch_next()
    {
        if adjacencies1.vertex_list.contains(&vertex2.id) {
            entity_adjacencies1.vertex_list.push(entity2);
        }
        if adjacencies2.vertex_list.contains(&vertex1.id) {
            entity_adjacencies2.vertex_list.push(entity1);
        }
    }

    for (entity, mut entity_adjacencies, adjacencies, _vertex) in vertexes.iter_mut() {
        for (edge_entity, mut edge_entity_adj, _edge_adj, edge) in edges.iter_mut() {
            if adjacencies.edge_list.contains(&edge.0) {
                entity_adjacencies.edge_list.push(edge_entity);
                edge_entity_adj.vertex_list.push(entity);
            }
        }
    }

    for (entity, mut entity_adjacencies, adjacencies, _vertex) in vertexes.iter_mut() {
        for (material_entity, mut material_entity_adj, _material_adj, material) in
            material.iter_mut()
        {
            if adjacencies.material_list.contains(&material.0) {
                entity_adjacencies.material_list.push(material_entity);
                material_entity_adj.vertex_list.push(entity);
            }
        }
    }
}

// fn vertex_click_to_spawn(
//     mut spawn_data: EventWriter<MapObjectSpawnEvent>,
//     mouse_button_input: Res<Input<MouseButton>>,
//     keys: Res<Input<KeyCode>>,
//     world_pos: Res<bevy_mouse_tracking_plugin::MousePosWorld>,
// ) {
//     for key in keys.get_pressed() {
//         match key {
//             KeyCode::V => {
//                 if mouse_button_input.just_released(MouseButton::Left) {
//                     let map_object_spawn = MapObjectSpawnEvent {
//                         map_type: VERTEX,
//                         x: world_pos.x,
//                         y: world_pos.y,
//                         roation: 0.0,
//                         entity_1: None,
//                         entity_2: None,
//                         material_type: None,
//                     };
//                     spawn_data.send(map_object_spawn);
//                 }
//             }
//             KeyCode::M => {
//                 if mouse_button_input.just_released(MouseButton::Left) {
//                     let map_object_spawn = MapObjectSpawnEvent {
//                         map_type: MATERIAL,
//                         x: world_pos.x,
//                         y: world_pos.y,
//                         roation: 0.0,
//                         entity_1: None,
//                         entity_2: None,
//                         material_type: Some(0),
//                     };
//                     spawn_data.send(map_object_spawn);
//                 }
//             }
//             _ => {}
//         }
//     }
// }

// fn edge_spawn(
//     mut spawn_data: EventWriter<MapObjectSpawnEvent>,
//     keys: Res<Input<KeyCode>>,
//     mut query: Query<(Entity, &mut Transform, &mut MapClickable, &mut Adjacencies), With<Vertex>>,
// ) {
//     let mut selected_count = 0;

//     let mut pos1 = Vec3::new(0.0, 0.0, 0.0);
//     let mut pos2 = Vec3::new(0.0, 0.0, 0.0);
//     let mut entity1 = None;
//     let mut entity2 = None;
//     for (entity, transform, clickable, _adjacencies) in query.iter() {
//         if clickable.selected {
//             selected_count += 1;
//             if selected_count == 1 {
//                 pos1 = transform.translation;
//                 entity1 = Some(entity);
//             }
//             if selected_count == 2 {
//                 pos2 = transform.translation;
//                 entity2 = Some(entity);
//                 if _adjacencies.vertex_list.contains(&entity) {
//                     return;
//                 }
//             }
//         }
//     }

//     if selected_count != 2 {
//         return;
//     };

//     for key in keys.get_just_released() {
//         match key {
//             KeyCode::E => {
//                 let edge_pos = (pos1 + pos2) / 2.0;
//                 let dif = pos2 - pos1;

//                 let map_object_spawn = MapObjectSpawnEvent {
//                     map_type: EDGE,
//                     x: edge_pos.x,
//                     y: edge_pos.y,
//                     roation: (dif.y).atan2(dif.x),
//                     entity_1: entity1,
//                     entity_2: entity2,
//                     material_type: None,
//                 };
//                 selected_count = 0;
//                 for (_entity, _transform, clickable, mut adjacencies) in query.iter_mut() {
//                     if clickable.selected {
//                         selected_count += 1;
//                         if selected_count == 1 {
//                             adjacencies.vertex_list.push(entity2.unwrap());
//                         }
//                         if selected_count == 2 {
//                             adjacencies.vertex_list.push(entity1.unwrap());
//                         }
//                     }
//                 }
//                 spawn_data.send(map_object_spawn);
//             }
//             _ => {}
//         }
//     }
// }
