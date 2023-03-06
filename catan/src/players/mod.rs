use crate::map;
use bevy::{prelude::*, sprite::Anchor};
use rand::prelude::*;

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

struct PlayerSpawnEvent {
    x: f32,
    y: f32,
    vertex_spawn: Entity,
}

#[derive(Component)]
struct Player {
    current_vertex: Entity,
    next_vertexes: Vec<Entity>,
}

pub struct PlayersPlugin;

impl Plugin for PlayersPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system_to_stage(StartupStage::PreStartup, setup)
            .add_startup_system_to_stage(StartupStage::PostStartup, test_spawn_player)
            // .add_startup_system_to_stage(StartupStage::PostStartup, setup_entity_adjacencies)
            .add_system(spawn_players)
            .add_event::<PlayerSpawnEvent>()
            .add_system(move_players);
        // // .add_event::<VertexSelectEvent>()
        // // .add_system(vertex_click_to_spawn)
        // // .add_system(edge_spawn)
        // // .add_system(select_vertex_adjacent_vertexes)
        // .add_system(animate_mana);
    }
}
fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let player_textures = PlayerTextures {
        player: asset_server.load("./players/obisan-Sheet.png"),
        player_col: 6,
        player_row: 8,
        player_x: 64.0,
        player_y: 64.0,

        padding_x: 0.0,
        padding_y: 0.0,
    };

    commands.insert_resource(player_textures);
}

fn test_spawn_player(
    mut player_spawn: EventWriter<PlayerSpawnEvent>,
    query: Query<(Entity, &mut Transform, &map::Vertex), With<map::VertexStart>>,
) {
    let mut start_spot: Option<Entity> = None;
    let mut x = 0.0;
    let mut y = 0.0;
    for (entity, pos, vertex) in query.iter() {
        if !vertex.filled {
            start_spot = Some(entity);
            x = pos.translation.x;
            y = pos.translation.y;
        }
    }

    if start_spot.is_some() {
        let player = PlayerSpawnEvent {
            x,
            y,
            vertex_spawn: start_spot.unwrap(),
        };

        player_spawn.send(player);
    } else {
        panic!("No place to have player join")
    }
}

fn spawn_players(
    mut commands: Commands,
    mut player_spawn: EventReader<PlayerSpawnEvent>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    player_textures: Res<PlayerTextures>,
    mut query: Query<(Entity, &mut map::Vertex), With<map::VertexStart>>,
) {
    for player in player_spawn.iter() {
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

        if let Ok((entity, mut vertex)) = query.get_mut(player.vertex_spawn) {
            vertex.filled = true;

            let _entity = commands
                .spawn(SpriteSheetBundle {
                    texture_atlas: texture_atlas_handle,
                    transform: Transform::from_xyz(player.x, player.y, 100.0),
                    sprite: TextureAtlasSprite {
                        anchor: Anchor::Custom(Vec2 { x: 0.0, y: -0.3 }),
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .insert(Player {
                    current_vertex: entity,
                    next_vertexes: Vec::new(),
                })
                .id();
        }

        // if spawn.vertex_start {
        //     commands.entity(entity).insert(VertexStart);
        // }
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
) {
    for (_entity, mut pos, mut player) in players.iter_mut() {
        if player.next_vertexes.len() > 0 {
            if let Ok((_e, vert_pos, _adj, mut vert)) = vertexes.get_mut(player.next_vertexes[0]) {
                let direction = Vec2::new(
                    vert_pos.translation.x - pos.translation.x,
                    vert_pos.translation.y - pos.translation.y,
                )
                .normalize();

                pos.translation += direction.extend(0.0) * MOVE_SPEED * time.delta_seconds();

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

                    player.current_vertex = player.next_vertexes.remove(0);
                }
            }
        } else {
            if let Ok((_e, _vert_pos, adj, _vert)) = vertexes.get_mut(player.current_vertex) {
                let mut rng = rand::thread_rng();
                let y: f32 = rng.gen();

                let index = (y * adj.vertex_list.len() as f32) as usize;

                let next_vert = adj.vertex_list[index];

                if let Ok((_e, _vert_pos, _adj, vert)) = vertexes.get_mut(next_vert) {
                    if !vert.filled {
                        player.next_vertexes.push(next_vert);
                    }
                }
            }
        }
    }
}
