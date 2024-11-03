use bevy::prelude::*;

mod camera;
mod world;

use camera::FlyCamPlugin;
use world::WorldPlugin;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, FlyCamPlugin, WorldPlugin))
        .run();
}
