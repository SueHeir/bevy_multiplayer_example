use crate::{map, protocol};
use bevy::{prelude::*, sprite::Anchor};
use bevy_quinnet::shared::ClientId;

mod client_players;
mod server_players;

const MOVE_SPEED: f32 = 60.0;

pub struct PlayersPlugin;

impl Plugin for PlayersPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup.in_base_set(StartupSet::PreStartup))
            .add_event::<PlayerSpawnEvent>()
            .add_system(move_players)
            .add_system(animate_player);
    }
}

pub struct ServerPlayersPlugin;

impl Plugin for ServerPlayersPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(PlayersPlugin)
            .add_system(server_players::spawn_players)
            .add_system(server_players::send_game_state)
            .add_system(server_players::handle_client_move_player);
    }
}

pub struct ClientPlayersPlugin;

impl Plugin for ClientPlayersPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(PlayersPlugin)
            .add_system(client_players::spawn_players)
            .add_system(client_players::move_my_player)
            .add_system(client_players::update_players)
            .add_event::<protocol::ServerUpdatePlayerEvent>();
    }
}

#[derive(Resource)]
pub struct PlayerTextures {
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

#[derive(Clone, Component)]
pub struct Player {
    pub id: u32,
    pub current_vertex: Entity,
    pub current_vertex_id: u32,
    pub next_entity: Vec<Entity>,
    pub next_entity_id: Option<u32>,
    pub animation_timer: f32,
    pub roation_index: i32,
    pub client_owner_id: ClientId,
    pub state: States,
}

#[derive(Clone, PartialEq)]
pub enum States {
    Idle,
    MoveToEntity,
}

#[derive(Component)]
pub struct ControlledPlayer;

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
        if player.next_entity.len() > 0 {
            if let Ok((_e, vert_pos, _adj, mut vert)) = vertexes.get_mut(player.next_entity[0]) {
                vert.filled = true;
                player.state = States::MoveToEntity;
                let direction = Vec2::new(
                    vert_pos.translation.x - pos.translation.x,
                    vert_pos.translation.y - pos.translation.y,
                )
                .normalize();

                let next_id = vert.id;
                player.next_entity_id = Some(vert.id);

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
                    player.next_entity_id = None;
                }
            }
        } else {
            player.next_entity_id = None;
            player.state = States::Idle;
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

fn animate_player(
    mut players: Query<(Entity, &mut TextureAtlasSprite, &mut Player)>,
    time: Res<Time>,
) {
    for (_entity, mut sprite, mut player) in &mut players {
        match player.state {
            States::Idle => {
                player.animation_timer += time.delta_seconds() * 6.0;
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

            States::MoveToEntity => {
                player.animation_timer += time.delta_seconds() * 6.0;
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
    }
}
