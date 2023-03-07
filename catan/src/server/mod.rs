use bevy::prelude::*;

pub struct ServerPlugin;

mod systems;

impl Plugin for ServerPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(systems::handle_client_move_player);
    }
}
