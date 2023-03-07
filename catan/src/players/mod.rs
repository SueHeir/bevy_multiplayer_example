use crate::{map, protocol};
use bevy::{prelude::*, sprite::Anchor};
use bevy_quinnet::shared::{
    channel::{ChannelId, ChannelType},
    ClientId,
};
use rand::prelude::*;
use seldom_state::{
    prelude::{AnyState, StateMachine, Trigger, TriggerPlugin},
    StateMachinePlugin,
};

const MOVE_SPEED: f32 = 100.0;

#[derive(Resource)]
struct PlayerTextures {
    player: Handle<Image>,
    player_col: usize,
    player_row: usize,
    player_x: f32,
    player_y: f32,
    padding_x: f32,
    padding_y: f32,
}

pub struct PlayerSpawnEvent {
    pub current_vertex: Option<Entity>,
    pub x: Option<f32>,
    pub y: Option<f32>,
    pub id: Option<u32>,
    pub client_owner_id: ClientId,
}

#[derive(Clone, Component, Reflect)]
pub struct Player {
    pub id: u32,
    pub current_vertex: Entity,
    pub current_vertex_id: u32,
    pub next_entity: Vec<Entity>,
    pub next_entity_id: Option<u32>,
    pub animation_timer: f32,
    pub roation_index: i32,
    pub client_owner_id: ClientId,
}

#[derive(Component)]
pub struct ControlledPlayer;

#[derive(Clone, Component, Reflect)]
#[component(storage = "SparseSet")]
struct Idle;

#[derive(Clone, Copy, Component, Reflect)]
#[component(storage = "SparseSet")]
struct GoToSelection {
    speed: f32,
}

#[derive(Clone, Copy, Reflect)]
struct GoToTrigger;

impl Trigger for GoToTrigger {
    type Param<'w, 's> = Query<'w, 's, &'static Player>;

    type Ok = Entity;

    type Err = i32;

    fn trigger(
        &self,
        entity: Entity,
        players: &Self::Param<'_, '_>,
    ) -> Result<bevy::prelude::Entity, i32> {
        let player = players.get(entity).unwrap();
        if player.next_entity.len() > 0 {
            return Ok(player.next_entity[0]);
        } else {
            return Err(0);
        }
    }
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let player_textures = PlayerTextures {
        player: asset_server.load("./players/obisan-Sheet.png"),
        player_col: 6,
        player_row: 16,
        player_x: 64.0,
        player_y: 64.0,

        padding_x: 0.0,
        padding_y: 0.0,
    };

    commands.insert_resource(player_textures);
}

fn server_spawn_players(
    mut commands: Commands,
    mut player_spawn: EventReader<PlayerSpawnEvent>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    player_textures: Res<PlayerTextures>,
    mut query: Query<(Entity, &mut Transform, &mut map::Vertex), With<map::VertexStart>>,
    mut total_players: Local<u32>,
    is_server: Res<protocol::IsServer>,
) {
    if is_server.0 {
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
                    })
                    .insert(
                        StateMachine::new(Idle)
                            .trans::<AnyState>(GoToTrigger, GoToSelection { speed: 100.0 }),
                    )
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
}

fn client_spawn_players(
    mut commands: Commands,
    mut player_spawn: EventReader<PlayerSpawnEvent>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    player_textures: Res<PlayerTextures>,
    mut query: Query<(Entity, &mut Transform, &mut map::Vertex), With<map::Vertex>>,
    is_server: Res<protocol::IsServer>,
    mut users: ResMut<protocol::Users>,
) {
    if !is_server.0 {
        for (i, _player) in player_spawn.iter().enumerate() {
            if !_player.current_vertex.is_some() {
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

            if let Ok((entity, pos, mut vertex)) = query.get_mut(_player.current_vertex.unwrap()) {
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
                        id: _player.id.unwrap(),
                        current_vertex: entity,
                        current_vertex_id: vertex.id,
                        next_entity: Vec::new(),
                        next_entity_id: None,
                        animation_timer: 0.0,
                        roation_index: 0,
                        client_owner_id: _player.client_owner_id,
                    })
                    .insert(
                        StateMachine::new(Idle)
                            .trans::<AnyState>(GoToTrigger, GoToSelection { speed: 100.0 }),
                    )
                    .id();
                if users.self_id == _player.client_owner_id {
                    commands.entity(_entity).insert(ControlledPlayer);
                }
            } else {
                println!("Failed to spawn player on client")
            }

            // if spawn.vertex_start {
            //     commands.entity(entity).insert(VertexStart);
            // }
        }
    }
}

fn move_players(
    mut players: Query<(Entity, &mut Transform, &mut Player), (With<Player>, Without<map::Vertex>)>,
    mut vertexes: Query<
        (
            Entity,
            &mut Transform,
            &map::EntityAdjacencies,
            &mut map::Vertex,
        ),
        (With<map::Vertex>, Without<Player>),
    >,
    time: Res<Time>,
    is_server: Res<protocol::IsServer>,
) {
    if is_server.0 {
        for (_entity, mut pos, mut player) in players.iter_mut() {
            if player.next_entity.len() > 0 {
                if let Ok((_e, vert_pos, _adj, mut vert)) = vertexes.get_mut(player.next_entity[0])
                {
                    let direction = Vec2::new(
                        vert_pos.translation.x - pos.translation.x,
                        vert_pos.translation.y - pos.translation.y,
                    )
                    .normalize();

                    let next_id = vert.id;

                    pos.translation += direction.extend(0.0) * MOVE_SPEED * time.delta_seconds();

                    let mut angle = direction.y.atan2(direction.x);
                    while angle > 360.0 / 2.0 {
                        angle -= 360.0;
                    }

                    while angle < -360.0 / 2.0 {
                        angle += 360.0;
                    }

                    if angle >= 0.0 {
                        player.roation_index = (angle % (360.0 / 8.0)).ceil() as i32;
                    }
                    if angle < 0.0 {
                        player.roation_index = (angle % (360.0 / 8.0)).floor() as i32;
                    }

                    if (direction.x > 0.0 && pos.translation.x > vert_pos.translation.x)
                        || (direction.x < 0.0 && pos.translation.x < vert_pos.translation.x)
                        || (direction.y > 0.0 && pos.translation.y > vert_pos.translation.y)
                        || (direction.y < 0.0 && pos.translation.y < vert_pos.translation.y)
                    {
                        pos.translation =
                            Vec3::new(vert_pos.translation.x, vert_pos.translation.y, 100.0);
                        vert.filled = true;
                        if let Ok((_ee, _vert_poss, _adjj, mut vertt)) =
                            vertexes.get_mut(player.current_vertex)
                        {
                            vertt.filled = false
                        }
                        player.current_vertex_id = next_id;
                        player.current_vertex = player.next_entity.remove(0);
                    }
                }
            } else {
                // if let Ok((_e, _vert_pos, adj, _vert)) = vertexes.get_mut(player.current_vertex) {
                //     let mut rng = rand::thread_rng();
                //     let y: f32 = rng.gen();

                //     let index = (y * adj.vertex_list.len() as f32) as usize;

                //     let next_vert = adj.vertex_list[index];

                //     if let Ok((_e, _vert_pos, _adj, mut vert)) = vertexes.get_mut(next_vert) {
                //         if !vert.filled {
                //             player.next_entity.push(next_vert);
                //             vert.filled = true;
                //         }
                //     }
                // }
            }
        }
    }
}

fn animate_player(
    mut gotoselections: Query<
        (Entity, &mut TextureAtlasSprite, &mut Player, &GoToSelection),
        Without<Idle>,
    >,
    mut idles: Query<(Entity, &mut TextureAtlasSprite, &mut Player, &Idle), Without<GoToSelection>>,
    time: Res<Time>,
) {
    for (_entity, mut sprite, mut player, _idle) in &mut idles {
        player.animation_timer += time.delta_seconds() * 2.0;
        if player.animation_timer > 6.0 {
            player.animation_timer = 0.0;
        }
        //{-3:"Walk_northwest",-2:"Walk_north",-1:"Walk_northeast",0:"Walk_east",1:"Walk_southeast",2:"Walk_south",3:"Walk_southwest",4:"Walk_west"}

        let mut angle = player.roation_index % 8 - 4;

        if angle < 0 {
            angle += 8;
        }

        sprite.index = 6 * angle as usize + (player.animation_timer) as usize;
    }

    for (_entity, mut sprite, mut player, _gotoselection) in &mut gotoselections {
        player.animation_timer += time.delta_seconds() * 2.0;
        if player.animation_timer > 4.0 {
            player.animation_timer = 0.0;
        }
        //{-3:"Walk_northwest",-2:"Walk_north",-1:"Walk_northeast",0:"Walk_east",1:"Walk_southeast",2:"Walk_south",3:"Walk_southwest",4:"Walk_west"}

        let mut angle = player.roation_index - 4;

        if angle < 0 {
            angle += 8;
        }

        sprite.index = 6 * 8 + 6 * angle as usize + (player.animation_timer) as usize;
    }
}

fn send_game_state(
    is_server: Res<protocol::IsServer>,
    server: ResMut<bevy_quinnet::server::Server>,
    players: Query<(Entity, &mut Transform, &mut Player), With<Player>>,
    users: Res<protocol::Users>,
    mut timer: Local<f32>,
    time: Res<Time>,
) {
    *timer += time.delta_seconds();

    if *timer < 1.0 / 35.0 {
        return;
    } else {
        *timer -= 1.0 / 35.0;

        if is_server.0 {
            let mut players_data = Vec::<protocol::Player>::new();

            for (_e, pos, player) in players.iter() {
                players_data.push(protocol::Player {
                    id: player.id,
                    x: pos.translation.x,
                    y: pos.translation.y,
                    rotation: player.roation_index,
                    current_vertex: player.current_vertex_id,
                    next_vertex: player.next_entity_id.unwrap_or(10000),
                    client_owner_id: player.client_owner_id,
                })
            }
            // println!("{:?}", users.names.keys().into_iter());

            if let Ok(_temp) = server.endpoint().send_group_message_on(
                users.names.keys().into_iter(),
                ChannelId::UnorderedReliable,
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
}

pub struct PlayersPlugin;

impl Plugin for PlayersPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system_to_stage(StartupStage::PreStartup, setup)
            // .add_startup_system_to_stage(StartupStage::PostStartup, test_spawn_player)
            .add_plugin(StateMachinePlugin)
            // .add_startup_system_to_stage(StartupStage::PostStartup, setup_entity_adjacencies)
            .add_system(server_spawn_players)
            .add_system(client_spawn_players)
            .add_event::<PlayerSpawnEvent>()
            .add_system(move_players)
            .add_system(animate_player)
            .add_plugin(TriggerPlugin::<GoToTrigger>::default())
            .add_system(send_game_state);
        // // .add_event::<VertexSelectEvent>()
        // // .add_system(vertex_click_to_spawn)
        // // .add_system(edge_spawn)
        // // .add_system(select_vertex_adjacent_vertexes)
        // .add_system(animate_mana);
    }
}
