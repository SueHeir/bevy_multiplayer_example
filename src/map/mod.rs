use crate::protocol;
use bevy::prelude::*;
use bevy_interact_2d::{Group, Interactable, InteractionState};
use bevy_quinnet::server::Server;
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;
mod client_map;
pub(crate) mod server_map;

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

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup.in_base_set(StartupSet::PreStartup))
            .add_system(spawn_map_object_system.pipe(setup_entity_adjacencies))
            .add_system(click_map_object)
            .add_event::<MapObjectSpawnEvent>()
            .add_system(animate_map_objects);
    }
}

pub struct ClientMapPlugin;

impl Plugin for ClientMapPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(MapPlugin)
            .add_system(client_map::update_map)
            .add_event::<protocol::ServerUpdateMapEvent>();
    }
}

pub struct ServerMapPlugin;

impl Plugin for ServerMapPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(MapPlugin)
            .add_startup_system(server_map::setup)
            .add_event::<server_map::InitMapSend>()
            .add_system(server_map::handle_init_map_send)
            .add_system(server_map::update_map);
    }
}

#[derive(Component)]
pub struct MapClickable {
    pub selected: bool,
    hover: bool,
    _map_type: u8,
    mana_type: u8,
    animation_timer: f32,
}

pub struct MapObjectSpawnEvent {
    pub map_type: u8,
    pub map_type_id: u32,
    pub x: f32,
    pub y: f32,
    pub roation: f32,
    pub edge_list: Vec<u32>,
    pub vertex_list: Vec<u32>,
    pub material_list: Vec<u32>,
    pub material_type: Option<u8>,
    pub vertex_start: bool,
}

#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct Adjacencies {
    pub edge_list: Vec<u32>,
    pub vertex_list: Vec<u32>,
    pub material_list: Vec<u32>,
}
#[derive(Component)]
pub struct EntityAdjacencies {
    pub edge_list: Vec<Entity>,
    pub vertex_list: Vec<Entity>,
    pub material_list: Vec<Entity>,
}

#[derive(Component)]
pub struct Edge {
    id: u32,
    roation: f32,
}

#[derive(Component)]
pub struct Vertex {
    pub id: u32,
    pub filled: bool,
    pub is_start: bool,
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
#[derive(Resource)]
pub struct VertexClientServerLookup(pub HashMap<u32, Entity>);
#[derive(Resource)]
pub struct EdgeClientServerLookup(pub HashMap<u32, Entity>);
#[derive(Resource)]
pub struct MaterialClientServerLookup(pub HashMap<u32, Entity>);

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

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
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
    commands.insert_resource(VertexClientServerLookup(HashMap::new()));
    commands.insert_resource(EdgeClientServerLookup(HashMap::new()));
    commands.insert_resource(MaterialClientServerLookup(HashMap::new()));
}

fn animate_map_objects(
    mut query_material: Query<
        (Entity, &mut TextureAtlasSprite, &mut MapClickable),
        (With<Material>, Without<Edge>, Without<Vertex>),
    >,
    mut query_vertex: Query<
        (Entity, &mut TextureAtlasSprite, &mut MapClickable, &Vertex),
        (With<Vertex>, Without<Edge>, Without<Material>),
    >,
    mut query_edge: Query<
        (Entity, &mut TextureAtlasSprite, &mut MapClickable, &Edge),
        (With<Edge>, Without<Material>, Without<Material>),
    >,
    time: Res<Time>,
) {
    for (_entity, mut sprite, clickable, vert) in query_vertex.iter_mut() {
        if clickable.selected {
            sprite.index = 2
        } else if clickable.hover {
            sprite.index = 1
        } else {
            sprite.index = 0
        }

        //For Debug purposes
        if vert.filled {
            sprite.index = 1
        }
    }
    for (_entity, mut sprite, clickable, _edge) in query_edge.iter_mut() {
        if clickable.selected {
            sprite.index = 2
        } else if clickable.hover {
            sprite.index = 1
        } else {
            sprite.index = 0
        }
    }
    for (_entity, mut sprite, mut clickable) in query_material.iter_mut() {
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
    mut query: Query<(Entity, &TextureAtlasSprite, &mut MapClickable), With<MapClickable>>,
) {
    for (entity, sprite, mut clickable) in query.iter_mut() {
        if interaction_state
            .get_group(Group(MAPCLICKABLE))
            .iter()
            .find(|(e, _)| *e == entity)
            .is_some()
        {
            if mouse_button_input.just_released(MouseButton::Left) {
                // info!("Clicked.");
                clickable.selected = true;
            } else if clickable.selected != true {
                if sprite.index == 0 {
                    // info!("Hover.");
                    clickable.hover = true;
                }
            }
        } else {
            if mouse_button_input.just_released(MouseButton::Left) {
                // info!("Deselected.");
                clickable.selected = false;
            }
            if clickable.selected != true {
                if sprite.index == 1 {
                    clickable.hover = false;
                }
            }
        }
    }
}

fn spawn_map_object_system(
    mut spawn_data: EventReader<MapObjectSpawnEvent>,
    mut commands: Commands,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    map_textures: Res<MapTextures>,
    mut vertex_lookup: ResMut<VertexClientServerLookup>,
    mut edge_lookup: ResMut<EdgeClientServerLookup>,
    mut material_lookup: ResMut<MaterialClientServerLookup>,
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
                        is_start: spawn.vertex_start,
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
                        hover: false,
                        _map_type: VERTEX,
                        animation_timer: 0.0,
                        mana_type: 0,
                    })
                    .id();

                if spawn.vertex_start {
                    commands.entity(entity).insert(VertexStart);
                }

                vertex_lookup.0.insert(spawn.map_type_id, entity);
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
            let entity = commands
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
                .insert(Edge {
                    id: spawn.map_type_id,
                    roation: spawn.roation,
                })
                .insert(MapClickable {
                    selected: false,
                    hover: false,
                    _map_type: EDGE,
                    animation_timer: 0.0,
                    mana_type: 0,
                })
                .id();

            edge_lookup.0.insert(spawn.map_type_id, entity);
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
                    hover: false,
                    _map_type: spawn.map_type,
                    animation_timer: 0.0,
                    mana_type: spawn.material_type.unwrap(),
                })
                .id();

            material_lookup.0.insert(spawn.map_type_id, entity);
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
            if adjacencies.edge_list.contains(&edge.id) {
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
