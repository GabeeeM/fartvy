#[cfg(not(target_arch = "wasm32"))]
use std::time::Instant;

use bevy::{
    pbr::CascadeShadowConfigBuilder,
    prelude::*,
    render::{
        mesh::{Indices, PrimitiveTopology},
        render_asset::RenderAssetUsages,
    },
};
use bevy_rapier3d::prelude::*;

#[cfg(not(target_arch = "wasm32"))]
use meshopt::{
    generate_vertex_remap, optimize_vertex_cache, optimize_vertex_fetch, remap_index_buffer,
    remap_vertex_buffer,
};

use noisy_bevy::simplex_noise_2d;

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
    #[cfg(not(target_arch = "wasm32"))]
    let start_time = Instant::now();

    // Parameters for terrain
    #[cfg(not(target_arch = "wasm32"))]
    const SIZE: f32 = 1200000.0;
    #[cfg(not(target_arch = "wasm32"))]
    const RESOLUTION: i32 = 750;
    #[cfg(not(target_arch = "wasm32"))]
    const HEIGHT_SCALE: f32 = 2300.0;
    // const ARRAY_SIZE: usize = (RESOLUTION as usize) * ((HEIGHT_SCALE as usize) / 2);

    #[cfg(target_arch = "wasm32")]
    const SIZE: f32 = 12000.0;
    #[cfg(target_arch = "wasm32")]
    const RESOLUTION: i32 = 300;
    #[cfg(target_arch = "wasm32")]
    const HEIGHT_SCALE: f32 = 100.0;

    let mut positions: Vec<[f32; 3]> = Vec::with_capacity((RESOLUTION + 1).pow(2) as usize);
    // let mut indices = Vec::with_capacity((RESOLUTION.pow(2) * 2 * 3) as usize);
    // let mut indices = [0u32; ARRAY_SIZE];
    let mut uvs = Vec::new();

    for z in 0..=RESOLUTION {
        for x in 0..=RESOLUTION {
            let px = SIZE * (x as f32 / RESOLUTION as f32 - 0.5);
            let pz = SIZE * (z as f32 / RESOLUTION as f32 - 0.5);

            let height = simplex_noise_2d(Vec2::new(px, pz)) * HEIGHT_SCALE;

            positions.push([px, height, pz]);
            uvs.push([x as f32 / RESOLUTION as f32, z as f32 / RESOLUTION as f32]);
        }
    }

    let (positions, indices) = handle_mesh(positions);

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
    // mesh.duplicate_vertices();
    mesh.compute_normals();
    mesh.generate_tangents().expect("TANGENTS FAILED OH GOD");

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
            translation: Vec3::new(500.0, 20.0, 500.0), // High in the sky
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
            maximum_distance: 50000.0, // Set your desired shadow distance here
            num_cascades: 4,           // More cascades for better shadow detail
            overlap_proportion: 0.8,
            ..default()
        }
        .into(),
        ..Default::default()
    };
    commands.spawn(ground);
    commands.spawn(sunlight);

    // dbg!(commands.spawn(cube).id());
    #[cfg(not(target_arch = "wasm32"))]
    let duration = start_time.elapsed();
    #[cfg(not(target_arch = "wasm32"))]
    println!(
        "Time taken for mesh generation and optimization: {:?}",
        duration
    );
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
                linvel: (ev.0.forward() * 50.0),
                ..Default::default()
            },
        ));
    }
}

fn handle_mesh(positions: Vec<[f32; 3]>) -> (Vec<[f32; 3]>, Vec<u32>) {
    let vertex_count = positions.len();

    // Generate triangle indices for a grid-like structure
    let mut initial_indices = Vec::new();
    let width = (vertex_count as f32).sqrt() as u32;

    // Generate grid indices
    for row in 0..width - 1 {
        for col in 0..width - 1 {
            let top_left = row * width + col;
            let top_right = top_left + 1;
            let bottom_left = (row + 1) * width + col;
            let bottom_right = bottom_left + 1;

            // First triangle
            initial_indices.push(top_left);
            initial_indices.push(bottom_left);
            initial_indices.push(top_right);

            // Second triangle
            initial_indices.push(top_right);
            initial_indices.push(bottom_left);
            initial_indices.push(bottom_right);
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        // Generate vertex remap and optimize only when not targeting wasm
        let (unique_vertex_count, remap) = generate_vertex_remap(&positions, None);

        // Remap buffers
        let mut remapped_indices = remap_index_buffer(Some(&initial_indices), vertex_count, &remap);
        let remapped_positions = remap_vertex_buffer(&positions, vertex_count, &remap);

        // Apply optimizations
        optimize_vertex_cache(&mut remapped_indices, unique_vertex_count);
        optimize_vertex_fetch(&mut remapped_indices, &remapped_positions);

        return (remapped_positions, remapped_indices);
    }

    (positions, initial_indices)
}
