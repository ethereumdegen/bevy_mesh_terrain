 

use bevy::prelude::*;
use bevy::input::mouse::MouseMotion;
use bevy_mesh_terrain::{TerrainMeshPlugin, terrain::{TerrainConfig, TerrainData, TerrainViewer}};





fn main() {
    App::new()
        .add_plugins(DefaultPlugins) 
         
        .add_plugins( TerrainMeshPlugin::default() )
          
        .add_systems(Startup, setup) 
        
        .add_systems(Update, update_camera_look ) 
        .add_systems(Update, update_camera_move ) 
        
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
     
    
    
   
    commands.spawn(VisibilityBundle::default() ) 
    .insert( TransformBundle::default() )
    .insert(
        TerrainConfig::default()
        .set_render_distance( 1500.0 )
        )
    .insert(
        TerrainData::default()
        .add_height_map_image(   height_map  ) 
        .add_array_texture_image(array_texture, 8) 
        .add_splat_texture_image( splat_texture )
    ); 
    
     
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
    
   
}


 
 
fn update_camera_look(
    mut event_reader:   EventReader<MouseMotion>  ,
    mouse_input:  Res< Input<MouseButton> > ,
    mut query: Query<(&mut Transform, &Camera3d)>,
    
    
){
    const MOUSE_SENSITIVITY: f32 = 2.0;
     
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


fn update_camera_move(
   
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<(&mut Transform, &Camera3d)>,
    
    
){
      const MOVE_SPEED: f32 = 10.0; // You can adjust this value as needed
     
     
     
  
    // Apply to each camera with the CameraTag
    for (mut transform, _) in query.iter_mut() {
       
      
           // Move the camera forward if W is pressed
        if keyboard_input.pressed(KeyCode::W) {
            let forward = transform.forward();
            transform.translation += forward * MOVE_SPEED;
        }
         
          if keyboard_input.pressed(KeyCode::S) {
            let forward = transform.forward() ;
            transform.translation -= forward * MOVE_SPEED;
        }
         
        
       
    }
    
}
