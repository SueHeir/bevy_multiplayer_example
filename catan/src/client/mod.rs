use bevy::prelude::*;
mod systems;

#[derive(Component)]
pub struct ClientAbilityState(String);

pub struct ClientPlugin;

impl Plugin for ClientPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(systems::setup);
    }
}
