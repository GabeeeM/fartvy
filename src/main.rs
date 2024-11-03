use bevy::prelude::*;

mod camera;
mod world;

use bevy_rapier3d::plugin::{NoUserData, RapierPhysicsPlugin};
use camera::FlyCamPlugin;
use iyes_perf_ui::entries::PerfUiBundle;
use world::WorldPlugin;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            FlyCamPlugin,
            WorldPlugin,
            RapierPhysicsPlugin::<NoUserData>::default(),
            // RapierDebugRenderPlugin {
            //     ..Default::default()
            // },
        ))
        .insert_resource(ClearColor(Color::srgb(0.53, 0.81, 0.92))) // Sky blue color
        .run();
}
