use bevy::pbr::wireframe::WireframeConfig;
use bevy::render::RenderPlugin;
use bevy::render::settings::WgpuSettings;
use bevy::render::settings::WgpuFeatures;
use bevy::render::settings::RenderCreation;
use bevy::pbr::wireframe::WireframePlugin;
use bevy::input::mouse::MouseMotion;
use bevy::prelude::*;
use bevy::color::palettes;

use bevy_mesh_terrain::{
    terrain::{TerrainData, TerrainViewer},
    terrain_config::TerrainConfig,
    TerrainMeshPlugin,
};

#[derive(Resource)]
pub struct TextureLoaderResource {}

fn main() {
    App::new()
       .add_plugins((
            DefaultPlugins.set(RenderPlugin {
                render_creation: RenderCreation::Automatic(WgpuSettings {
                    // WARN this is a native only feature. It will not work with webgl or webgpu
                    features: WgpuFeatures::POLYGON_MODE_LINE,
                    ..default()
                }),
                ..default()
            }),
            // You need to add this plugin to enable wireframe rendering
            WireframePlugin,
        ))
        .add_plugins(TerrainMeshPlugin::default())
        
        .insert_resource(WireframeConfig {
            // The global wireframe config enables drawing of wireframes on every mesh,
            // except those with `NoWireframe`. Meshes with `Wireframe` will always have a wireframe,
            // regardless of the global configuration.
            global: false,
            // Controls the default color of all wireframes. Used as the default color for global wireframes.
            // Can be changed per mesh using the `WireframeColor` component.
            default_color: Color::WHITE.into(),
        })


        .add_systems(Startup, setup)
        .add_systems(Update, update_camera_look)
        .add_systems(Update, update_camera_move)
        .add_systems(Update, update_directional_light_position)
        .run();
}

/// set up a simple 3D scene
fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn(SpatialBundle::default())
        .insert(TerrainConfig::load_from_file("assets/default_terrain/terrain_config.ron").unwrap())
        .insert(TerrainData::new());

    // light
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            //shadow_depth_bias: 0.5,
            //shadow_normal_bias: 0.5,
            illuminance: 700.0,  
            color: Color::Srgba(palettes::css::ANTIQUE_WHITE),

            ..default()
        },
        transform: Transform::from_xyz(4.0, 6.0, 4.0),
        ..default()
    });
    // light

    commands.insert_resource(AmbientLight {
        color: Color::Srgba(palettes::css::ANTIQUE_WHITE),
        brightness: 122.12,
    });

    // camera
    commands
        .spawn(Camera3dBundle {
            transform: Transform::from_xyz(20.0, 162.5, 20.0)
                .looking_at(Vec3::new(900.0, 0.0, 900.0), Vec3::Y),
            ..default()
        })
        .insert(TerrainViewer::default())
       // .insert(ShadowFilteringMethod::Jimenez14)
       ;
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


fn update_directional_light_position(
    mut query: Query<&mut Transform, With<DirectionalLight>>,
   
    time: Res<Time>,
) {

    let current_time = time.elapsed();


 //   let delta_time = time.delta_seconds();
    
    let SECONDS_IN_A_CYCLE = 20.0;

    let angle = (current_time.as_millis() as f32 / (SECONDS_IN_A_CYCLE* 1000.0) ) * std::f32::consts::PI * 2.0; // Convert time to radians

    let radius = 20.0; // Adjust the radius of the sun's orbit
    let x = angle.cos() * radius;
    let y = angle.sin() * radius + 10.0; // Adjust the height of the sun
    let z = 0.0;

    for mut transform in query.iter_mut() {

        transform.translation = Vec3::new(x, y, z);
        transform.look_at(Vec3::ZERO, Vec3::Y);
    }
}