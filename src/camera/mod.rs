use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;
use bevy_easings::EasingsPlugin;
use bevy_interact_2d::InteractionPlugin;
use bevy_interact_2d::{Group, InteractionSource};
use leafwing_input_manager::prelude::*;

use crate::map;

mod client_camera;
pub struct CameraPlugin;

static ZOOM_SPEED: f32 = 0.05;
static MIN_ZOOM: f32 = 0.35;
static MAX_ZOOM: f32 = 1.0;
static MOVE_SPEED: f32 = 200.0;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(EasingsPlugin)
            .add_plugin(InteractionPlugin)
            .add_plugin(InputManagerPlugin::<CameraAction>::default())
            .add_startup_system(setup)
            .add_system(zoom_system)
            .add_system(move_system);
    }
}

pub struct ServerCameraPlugin;

impl Plugin for ServerCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(CameraPlugin);
    }
}

pub struct ClientCameraPlugin;

impl Plugin for ClientCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(CameraPlugin)
            .add_system(client_camera::track_owned_player);
    }
}

// This is the list of "things in the game I want to be able to do based on input"
#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug)]
pub enum CameraAction {
    Up,
    Down,
    Left,
    Right,
}
#[derive(Component)]
pub struct MainCamera;

fn setup(mut commands: Commands) {
    commands
        .spawn(Camera2dBundle {
            // transform: Transform {
            //     scale: Vec3 {
            //         x: 0.5,
            //         y: 0.5,
            //         z: 0.5,
            //     },
            //     ..Default::default()
            // },
            ..Default::default()
        })
        .insert(InteractionSource {
            groups: vec![
                Group(map::MAPCLICKABLE),
                Group(map::VERTEX),
                Group(map::EDGE),
                Group(map::MATERIAL),
            ],
            ..Default::default()
        })
        // .add_world_tracking()
        // .add_mouse_tracking()
        .insert(InputManagerBundle::<CameraAction> {
            // Stores "which actions are currently pressed"
            action_state: ActionState::default(),
            // Describes how to convert from player inputs into those actions
            input_map: InputMap::new([
                (KeyCode::W, CameraAction::Up),
                (KeyCode::S, CameraAction::Down),
                (KeyCode::A, CameraAction::Left),
                (KeyCode::D, CameraAction::Right),
            ]),
        })
        .insert(MainCamera);
}

fn zoom_system(
    mut whl: EventReader<MouseWheel>,
    mut cam: Query<(&mut Transform, &mut OrthographicProjection), With<MainCamera>>,
) {
    let delta_zoom: f32 = whl.iter().map(|e| e.y).sum();
    if delta_zoom == 0. {
        return;
    }

    let (mut _pos, mut cam) = cam.single_mut();

    cam.scale -= ZOOM_SPEED * delta_zoom * cam.scale;
    cam.scale = cam.scale.clamp(MIN_ZOOM, MAX_ZOOM);
}

fn move_system(
    mut cam: Query<(&mut Transform, &mut OrthographicProjection), With<MainCamera>>,
    camera_actions: Query<&ActionState<CameraAction>, With<MainCamera>>,
    time: Res<Time>,
) {
    let action_state = camera_actions.single();

    let mut x_axis = 0;
    let mut y_axis = 0;
    if action_state.pressed(CameraAction::Up) {
        y_axis += 1;
    }
    if action_state.pressed(CameraAction::Down) {
        y_axis -= 1;
    }
    if action_state.pressed(CameraAction::Right) {
        x_axis += 1;
    }
    if action_state.pressed(CameraAction::Left) {
        x_axis -= 1;
    }

    let (mut pos, mut _cam) = cam.single_mut();

    pos.translation += Vec2::new(
        MOVE_SPEED * x_axis as f32 * time.delta_seconds(),
        MOVE_SPEED * y_axis as f32 * time.delta_seconds(),
    )
    .extend(0.0);
}
