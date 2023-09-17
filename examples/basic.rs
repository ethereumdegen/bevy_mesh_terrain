//! A simple 3D scene with light shining over a cube sitting on a plane.

use bevy::{prelude::*, utils::HashMap};
use bevy_mesh_terrain::{TerrainMeshPlugin, terrain::{TerrainConfig, TerrainData, TerrainViewer}};





fn main() {
    App::new()
        .add_plugins(DefaultPlugins) 
         
         .add_plugins( TerrainMeshPlugin::default() )
          
        .add_systems(Startup, setup) 
        
      
       
        
        .run();
}

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    
   
    asset_server: Res<AssetServer> 
) {
    
    let height_map = asset_server.load("terrain/source/height.png");
     
    let mut terrain_data = TerrainData::default();
    terrain_data.add_height_map_image(   height_map  ) ;
    
    // plane
    commands.spawn(VisibilityBundle::default() ) 
    .insert( TransformBundle::default() )
    .insert(TerrainConfig::default())
    .insert(terrain_data) 
    ;
    
  /*   commands.spawn(  PbrBundle {
        mesh: meshes.add(shape::Plane::from_size(5.0).into()),
        material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
        ..default()
    } ) 
    
    ;*/
     
    
    // cube
   
    
    //see https://github.com/clynamen/bevy_terrain/blob/0.0.1/src/main.rs
    
     
    
    // light
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });
    // camera
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(20.0, 62.5, 20.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    })
    .insert(TerrainViewer::default());
    
    println!("completed setup");
}


 
 

//loop through all of the active chunks that exist to maybe deactive them if client is too far away 


