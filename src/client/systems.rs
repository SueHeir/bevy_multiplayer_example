use bevy::prelude::*;

use crate::protocol;

pub fn setup(mut commands: Commands) {
    commands.spawn(protocol::CurrentClientEventTrigger(
        protocol::ClientEvents::MOVE,
    ));
}
