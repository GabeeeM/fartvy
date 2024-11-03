use bevy::{ecs::query, prelude::*};
use noisy_bevy::simplex_noise_2d;

use crate::camera::FlyCameraMarker;

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (spawn_world));
    }
}

fn spawn_world(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let ground = PbrBundle {
        mesh: meshes.add(Plane3d {
            half_size: Vec2 { x: 150.0, y: 150.0 },
            ..Default::default()
        }),
        material: materials.add(Color::srgb(1.0, 0.0, 0.0)),
        transform: Transform::from_xyz(0.0, 0.0, 0.0),
        ..Default::default()
    };

    let cube = PbrBundle {
        mesh: meshes.add(Cuboid {
            half_size: Vec3 {
                x: 15.0,
                y: 15.0,
                z: 15.0,
            },
        }),
        material: materials.add(Color::srgb(0.0, 0.0, 1.0)),
        transform: Transform::from_xyz(25.0, 7.5, 25.0),
        ..Default::default()
    };

    commands.spawn(ground);
    commands.spawn(cube);
}
