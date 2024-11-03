use bevy::{
    ecs::query,
    pbr::CascadeShadowConfigBuilder,
    prelude::*,
    render::{
        mesh::{Indices, PrimitiveTopology},
        render_asset::RenderAssetUsages,
    },
};
use bevy_rapier3d::prelude::*;
use noisy_bevy::simplex_noise_2d;
use rand::Rng;

use crate::camera::FlyCameraMarker;

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_world)
            .add_event::<Shoot>()
            .add_systems(Update, shoot);
    }
}

#[derive(Event)]
pub struct Shoot(pub Transform);

fn spawn_world(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Parameters for terrain
    let size = 6000.0;
    let resolution = 25;
    let noise_scale = 0.1;
    let height_scale = 300.0;

    // Create a plane mesh manually with the grid pattern
    let mut positions = Vec::new();
    let mut indices = Vec::new();
    let mut uvs = Vec::new();

    // Generate vertices
    for z in 0..=resolution {
        for x in 0..=resolution {
            let px = size * (x as f32 / resolution as f32 - 0.5);
            let pz = size * (z as f32 / resolution as f32 - 0.5);

            let height =
                simplex_noise_2d(Vec2::new(px * noise_scale, pz * noise_scale)) * height_scale;

            positions.push([px, height, pz]);
            uvs.push([x as f32 / resolution as f32, z as f32 / resolution as f32]);
        }
    }

    // Generate indices
    for z in 0..resolution {
        for x in 0..resolution {
            let tl = z * (resolution + 1) + x;
            let tr = tl + 1;
            let bl = (z + 1) * (resolution + 1) + x;
            let br = bl + 1;

            indices.extend_from_slice(&[
                tl as u32, bl as u32, tr as u32, tr as u32, bl as u32, br as u32,
            ]);
        }
    }

    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );

    let collider_vertices: Vec<Vec3> = positions
        .iter()
        .map(|&[x, y, z]| Vec3::new(x, y, z))
        .collect();

    // Convert indices to [u32; 3] triangles for Rapier
    let collider_indices: Vec<[u32; 3]> = indices
        .chunks(3)
        .map(|chunk| [chunk[0], chunk[1], chunk[2]])
        .collect();

    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U32(indices));

    // Compute normals
    mesh.duplicate_vertices();
    mesh.compute_flat_normals();

    // Material with better terrain settings
    let material = StandardMaterial {
        base_color: Color::srgb(0.5, 0.5, 0.5),
        perceptual_roughness: 0.9,
        metallic: 0.0,
        reflectance: 0.2,
        ..default()
    };

    let ground = (
        PbrBundle {
            mesh: meshes.add(mesh),
            material: materials.add(material),
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            ..Default::default()
        },
        Collider::trimesh(collider_vertices, collider_indices),
        RigidBody::Fixed,
    );

    let sunlight = DirectionalLightBundle {
        transform: Transform {
            translation: Vec3::new(0.0, 1000.0, 0.0), // High in the sky
            rotation: Quat::from_euler(EulerRot::XYZ, -std::f32::consts::FRAC_PI_4, 0.0, 0.0), // Angle the light
            ..Default::default()
        },
        directional_light: DirectionalLight {
            color: Color::srgb(1.0, 0.9, 0.7), // Warm, sun-like color
            illuminance: 1500.0,               // Intensity for a sun-like light
            shadows_enabled: true,
            shadow_depth_bias: 0.002, // Lower shadow depth bias to reduce artifacts
            shadow_normal_bias: 0.1,
            ..Default::default()
        },
        cascade_shadow_config: CascadeShadowConfigBuilder {
            first_cascade_far_bound: 40.0,
            minimum_distance: 10.0,
            maximum_distance: 10000.0, // Set your desired shadow distance here
            num_cascades: 4,           // More cascades for better shadow detail
            overlap_proportion: 0.8,
            ..default()
        }
        .into(),
        ..Default::default()
    };

    // let cube = PbrBundle {
    //     mesh: meshes.add(Cuboid {
    //         half_size: Vec3 {
    //             x: 15.0,
    //             y: 15.0,
    //             z: 15.0,
    //         },
    //     }),
    //     material: materials.add(Color::srgb(0.0, 0.0, 1.0)),
    //     transform: Transform::from_xyz(25.0, 15.0, 25.0),
    //     ..Default::default()
    // };

    commands.spawn(ground);
    commands.spawn(sunlight);

    // dbg!(commands.spawn(cube).id());
}

fn shoot(
    mut ev_shoot: EventReader<Shoot>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for ev in ev_shoot.read() {
        commands.spawn((
            PbrBundle {
                mesh: meshes.add(Sphere { radius: 2.0 }),
                material: materials.add(Color::WHITE),
                transform: ev.0,
                ..Default::default()
            },
            Collider::ball(2.0),
            RigidBody::Dynamic,
            Velocity {
                linvel: (ev.0.forward() * 50.0).into(),
                ..Default::default()
            },
        ));
    }
}
