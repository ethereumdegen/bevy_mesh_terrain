//! A simple 3D scene with light shining over a cube sitting on a plane.

use bevy::{prelude::*, utils::HashMap, input::mouse::MouseMotion};
use bevy_mesh_terrain::{TerrainMeshPlugin, terrain::{TerrainConfig, TerrainData, TerrainViewer}};





fn main() {
    App::new()
        .add_plugins(DefaultPlugins) 
         
        .add_plugins( TerrainMeshPlugin::default() )
          
        .add_systems(Startup, setup) 
        
        .add_systems(Update, update_camera_look ) 
       
        
        .run();
}

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    
   
    asset_server: Res<AssetServer> 
) {
    let array_texture: Handle<Image> = asset_server.load("terrain/textures/array_texture.png");
    let height_map: Handle<Image> = asset_server.load("terrain/source/height.png");
   
    let splat_texture: Handle<Image> = asset_server.load("terrain/textures/splat_texture.png");
     
    let mut terrain_data = TerrainData::default();
    terrain_data.add_height_map_image(   height_map  ) ;
    terrain_data.add_array_texture_image(array_texture, 4) ;
    terrain_data.add_splat_texture_image( splat_texture ); 
    
   
    commands.spawn(VisibilityBundle::default() ) 
    .insert( TransformBundle::default() )
    .insert(TerrainConfig::default())
    .insert(terrain_data) 
    ;
     
    
     
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight::default(),
        ..default()
    });
    // light
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 800.0, 4.0),
        ..default()
    });
    // camera
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(20.0, 162.5, 20.0).looking_at(Vec3::new(900.0,0.0,900.0), Vec3::Y),
        ..default()
    })
    .insert(TerrainViewer::default());
    
    println!("completed setup");
}


 
 
fn update_camera_look(
    mut event_reader:   EventReader<MouseMotion>  ,
    mouse_input:  Res< Input<MouseButton> > ,
    mut query: Query<(&mut Transform, &Camera3d)>,
    
    
){
    let MOUSE_SENSITIVITY = 2.0;
     
     if !mouse_input.pressed(MouseButton::Left) {
        return;
    }
    
      
      // Accumulate mouse delta
    let mut delta: Vec2 = Vec2::ZERO;
    for event in event_reader.iter() {
        delta += event.delta;
    }

    // Apply to each camera with the CameraTag
    for (mut transform, _) in query.iter_mut() {
       // let rotation = transform.rotation;
      
        let (mut yaw, mut pitch, _roll) = transform.rotation.to_euler(EulerRot::YXZ);
       
        yaw -= delta.x / 180.0   * MOUSE_SENSITIVITY  ;
        pitch -= delta.y / 180.0   * MOUSE_SENSITIVITY;
        pitch = pitch .clamp(-std::f32::consts::PI / 2.0, std::f32::consts::PI / 2.0) ;
   
        transform.rotation = Quat::from_euler(EulerRot::YXZ, yaw, pitch, 0.0);
       
    }
    
}