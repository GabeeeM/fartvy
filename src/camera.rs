use bevy::{
    input::{keyboard::Key, mouse::MouseMotion},
    prelude::*,
    window::{CursorGrabMode, PrimaryWindow},
};

/// Settings for mouse sensitivity and movement speed
#[derive(Resource)]
pub struct FlyCamSettings {
    pub sensitivity: f32,
    pub move_speed: f32,
    pub y_lock: bool,
}

impl Default for FlyCamSettings {
    fn default() -> Self {
        Self {
            sensitivity: 0.08,
            move_speed: 50.0,
            y_lock: false,
        }
    }
}

/// Allows customizing the different movement keybinds
#[derive(Resource)]
pub struct FlyCamKeybinds {
    pub move_forward: KeyCode,
    pub move_back: KeyCode,
    pub move_left: KeyCode,
    pub move_right: KeyCode,
    pub move_up: KeyCode,
    pub move_down: KeyCode,
    pub toggle_cursor: KeyCode,
}

impl Default for FlyCamKeybinds {
    fn default() -> Self {
        Self {
            move_forward: KeyCode::KeyW,
            move_back: KeyCode::KeyS,
            move_left: KeyCode::KeyA,
            move_right: KeyCode::KeyD,
            move_up: KeyCode::Space,
            move_down: KeyCode::ShiftLeft,
            toggle_cursor: KeyCode::Escape,
        }
    }
}

/// Marker for querying flycams
#[derive(Component)]
pub struct FlyCameraMarker;

/// This plugin will add all the nessesary resources
/// and systems for a first-person flycam
pub struct FlyCamPlugin;
impl Plugin for FlyCamPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FlyCamSettings>();
        app.init_resource::<FlyCamKeybinds>();
        app.add_systems(Startup, lock_mouse);
        app.add_systems(Startup, setup_fly_cam);
        app.add_systems(Update, look_fly_cam);
        app.add_systems(Update, move_fly_cam);
    }
}

// spawns the flycam
fn setup_fly_cam(mut cmd: Commands) {
    cmd.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(0.0, 3.0, 3.0).looking_at(Vec3::ZERO, Vec3::Y),
            projection: Projection::Perspective(PerspectiveProjection {
                fov: 90.0,
                ..Default::default()
            }),
            ..default()
        },
        FlyCameraMarker,
    ))
    .with_children(|parent| {
        parent.spawn(SpotLightBundle {
            spot_light: SpotLight {
                shadows_enabled: true,
                ..default()
            },
            ..default()
        });
    });
}

// locks/hides the mouse on startup
fn lock_mouse(mut query: Query<&mut Window, With<PrimaryWindow>>) {
    let mut window = query.single_mut();
    window.cursor.grab_mode = CursorGrabMode::Locked;
    window.cursor.visible = false;
    window.present_mode = bevy::window::PresentMode::AutoNoVsync;
}

// rotates the flycam with the mouse
fn look_fly_cam(
    settings: Res<FlyCamSettings>,
    mut mouse_motion: EventReader<MouseMotion>,
    mut query: Query<&mut Transform, With<FlyCameraMarker>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    let windowe = window_query.single();

    if windowe.cursor.grab_mode == CursorGrabMode::Locked {
        for mut transform in &mut query {
            for motion in mouse_motion.read() {
                let (mut yaw, mut pitch, _) = transform.rotation.to_euler(EulerRot::YXZ);
                pitch -= (motion.delta.y * settings.sensitivity).to_radians();
                yaw -= (motion.delta.x * settings.sensitivity).to_radians();

                pitch = pitch.clamp(f32::to_radians(-89.0), f32::to_radians(89.0));

                transform.rotation =
                    Quat::from_axis_angle(Vec3::Y, yaw) * Quat::from_axis_angle(Vec3::X, pitch);
            }
        }
    }
}

// move the flycam with the set keybinds
fn move_fly_cam(
    time: Res<Time>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    settings: Res<FlyCamSettings>,
    keybinds: Res<FlyCamKeybinds>,
    mut query: Query<&mut Transform, With<FlyCameraMarker>>,
    mut window_query: Query<&mut Window, With<PrimaryWindow>>,
) {
    for mut transform in &mut query {
        let mut delta = Vec3::ZERO;

        let mut forward = -transform.local_z().as_vec3();
        if settings.y_lock {
            forward.y = 0.0;
        }

        let right = transform.local_x().as_vec3();
        if keyboard_input.pressed(keybinds.move_forward) {
            delta += forward;
        }
        if keyboard_input.pressed(keybinds.move_back) {
            delta -= forward;
        }
        if keyboard_input.pressed(keybinds.move_right) {
            delta += right;
        }
        if keyboard_input.pressed(keybinds.move_left) {
            delta -= right;
        }

        if settings.y_lock {
            delta = delta.normalize_or_zero();
        }

        if keyboard_input.pressed(keybinds.move_up) {
            delta.y += 1.0;
        }
        if keyboard_input.pressed(keybinds.move_down) {
            delta.y -= 1.0;
        }

        if !settings.y_lock {
            delta = delta.normalize_or_zero();
        }

        if keyboard_input.just_pressed(keybinds.toggle_cursor) {
            let mut windowe = window_query
                .get_single_mut()
                .expect("COULD NOT GRAB WINDOW");

            if windowe.cursor.grab_mode == CursorGrabMode::None {
                windowe.cursor.grab_mode = CursorGrabMode::Locked;
                windowe.cursor.visible = false;
            } else {
                windowe.cursor.grab_mode = CursorGrabMode::None;
                windowe.cursor.visible = true;
            }
        }

        transform.translation += delta * settings.move_speed * time.delta_seconds();
    }
}
