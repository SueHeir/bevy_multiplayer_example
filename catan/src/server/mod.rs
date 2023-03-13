use bevy::prelude::*;

pub struct ServerPlugin;

mod systems;

impl Plugin for ServerPlugin {
    fn build(&self, _app: &mut App) {
        //If this is even needed
    }
}
