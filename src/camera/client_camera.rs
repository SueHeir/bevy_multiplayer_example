use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

use crate::players;

use super::{CameraAction, MainCamera};

pub fn track_owned_player(
    mut cam: Query<
        (
            &mut Transform,
            &mut OrthographicProjection,
            &ActionState<CameraAction>,
        ),
        (With<MainCamera>, Without<players::ControlledPlayer>),
    >,
    player: Query<&Transform, (Without<MainCamera>, With<players::ControlledPlayer>)>,
) {
    if let Ok(pos_p) = player.get_single() {
        let (mut pos_c, mut _cam, action) = cam.single_mut();

        if action.pressed(CameraAction::Up)
            || action.pressed(CameraAction::Down)
            || action.pressed(CameraAction::Right)
            || action.pressed(CameraAction::Left)
        {
        } else {
            pos_c.translation = pos_c.translation.lerp(
                Vec3::new(
                    pos_p.translation.x,
                    pos_p.translation.y,
                    pos_c.translation.z,
                ),
                0.03,
            );
        }
    }
}
