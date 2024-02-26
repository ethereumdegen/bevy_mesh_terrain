use bevy::input::mouse::MouseMotion;
use bevy::{pbr::ShadowFilteringMethod, prelude::*};
use bevy_mesh_terrain::{
    terrain::{TerrainData, TerrainViewer},
    terrain_config::TerrainConfig,
    TerrainMeshPlugin,
};

#[derive(Resource)]
pub struct TextureLoaderResource {}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(TerrainMeshPlugin::default())
        .add_systems(Startup, setup)
        .add_systems(Update, update_camera_look)
        .add_systems(Update, update_camera_move)
        .run();
}

/// set up a simple 3D scene
fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn(SpatialBundle::default())
        .insert(TerrainConfig::load_from_file("assets/default_terrain/terrain_config.ron").unwrap())
        .insert(TerrainData::new());

    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            shadow_depth_bias: 0.5,
            shadow_normal_bias: 0.5,

            color: Color::WHITE,
            ..default()
        },

        ..default()
    });
    // light
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            shadow_depth_bias: 0.5,
            shadow_normal_bias: 0.5,

            color: Color::WHITE,
            ..default()
        },

        ..default()
    });
    // light
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,

            shadow_depth_bias: 0.5,
            shadow_normal_bias: 0.5,

            color: Color::WHITE,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 800.0, 4.0),
        ..default()
    });

    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 0.12,
    });

    // camera
    commands
        .spawn(Camera3dBundle {
            transform: Transform::from_xyz(20.0, 162.5, 20.0)
                .looking_at(Vec3::new(900.0, 0.0, 900.0), Vec3::Y),
            ..default()
        })
        .insert(TerrainViewer::default())
        .insert(ShadowFilteringMethod::Jimenez14);
}

fn update_camera_look(
    mut event_reader: EventReader<MouseMotion>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    mut query: Query<(&mut Transform, &Camera3d)>,
) {
    const MOUSE_SENSITIVITY: f32 = 2.0;

   

    // Accumulate mouse delta
    let mut delta: Vec2 = Vec2::ZERO;
    for event in event_reader.read() {
        delta += event.delta;
    }

    if !mouse_input.pressed(MouseButton::Left) {
        return;
    }

    // Apply to each camera with the CameraTag
    for (mut transform, _) in query.iter_mut() {
        // let rotation = transform.rotation;

        let (mut yaw, mut pitch, _roll) = transform.rotation.to_euler(EulerRot::YXZ);

        yaw -= delta.x / 180.0 * MOUSE_SENSITIVITY;
        pitch -= delta.y / 180.0 * MOUSE_SENSITIVITY;
        pitch = pitch.clamp(-std::f32::consts::PI / 2.0, std::f32::consts::PI / 2.0);

        transform.rotation = Quat::from_euler(EulerRot::YXZ, yaw, pitch, 0.0);
    }
}

fn update_camera_move(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut Transform, &Camera3d)>,
) {
    const MOVE_SPEED: f32 = 10.0; // You can adjust this value as needed

    // Apply to each camera with the CameraTag
    for (mut transform, _) in query.iter_mut() {
        // Move the camera forward if W is pressed
        if keyboard_input.pressed(KeyCode::KeyW) {
            let forward = transform.forward();
            transform.translation += forward * MOVE_SPEED;
        }

        if keyboard_input.pressed(KeyCode::KeyS) {
            let forward = transform.forward();
            transform.translation -= forward * MOVE_SPEED;
        }
    }
}
