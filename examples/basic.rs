//! A simple 3D scene with light shining over a cube sitting on a plane.

use bevy::{prelude::*, utils::HashMap};


//attach me to camera 
#[derive(Component,Default)]
struct TerrainViewer {
    
}


#[derive(Component)]
struct TerrainConfig {
    
    pub terrain_dimensions: Vec2,
  //  pub chunk_width: f32,
    pub chunk_rows: u32,
    
    pub render_distance: f32, 
}

impl Default for TerrainConfig {
    fn default() -> Self {
        Self {
           // chunk_width: 64.0 ,
            terrain_dimensions: Vec2::new(1024.0,1024.0),
            chunk_rows: 64 ,
            render_distance: 400.0, 
        }
    }
}


#[derive(Component,Default)]
struct TerrainData {
    
    //chunk_index -> chunk data 
    //chunk index is   chunk_col * 64  + chunk_row   IF chunk_rows is 64 
    pub chunks: HashMap<u32,ChunkData>, 
    
  //  pub terrain_origin: Vec3 // should be a component of an entity 
    
    
}
 
struct ChunkData {
    
    chunk_state: ChunkState
    
}

enum ChunkState{
    ACTIVE,
    LOADING,
    INACTIVE
    
}


trait ChunkCoordinates {
    fn get_chunk_index(&self, chunk_rows: u32) -> u32; 
    fn from_location( location: Vec3 ,  terrain_origin: Vec3 , terrain_dimensions: Vec2 , chunk_rows: u32 ) -> Option<UVec2> ;
}


type ChunkCoords = UVec2 ; 

impl ChunkCoordinates for  ChunkCoords {
    
    
     //chunk index is   chunk_col * 64  + chunk_row   IF chunk_rows is 64 
    fn get_chunk_index(&self, chunk_rows: u32) -> u32 {
        
        return self.y * chunk_rows + self.x as u32; 
        
    }
    
        
   fn from_location(from_location: Vec3, terrain_origin: Vec3, terrain_dimensions: Vec2, chunk_rows: u32) -> Option<UVec2> {
        let location_delta = from_location - terrain_origin;

        let terrain_min = terrain_origin;
        let terrain_max = terrain_origin + Vec3::new(terrain_dimensions.x, 0.0, terrain_dimensions.y);

        // Check if from_location is within the terrain bounds
        if location_delta.x >= 0.0 && location_delta.x <= terrain_dimensions.x && 
           location_delta.z >= 0.0 && location_delta.z <= terrain_dimensions.y {

            // Calculate the chunk's x and z coordinates
            let chunk_x = (location_delta.x / terrain_dimensions.x * chunk_rows as f32) as u32;
            let chunk_z = (location_delta.z / terrain_dimensions.y * chunk_rows as f32) as u32;

            return Some(UVec2::new(chunk_x, chunk_z));
        }

        None
    }
}

  









fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
     
       
         
        .add_systems(Startup, setup)
        
        
        .add_systems(Update, update_terrain_chunks)
        
        
        .run();
}

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // plane
    commands.spawn(PbrBundle {
        mesh: meshes.add(shape::Plane::from_size(5.0).into()),
        material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
        ..default()
    })
    .insert(TerrainConfig::default())
    .insert(TerrainData::default()) 
    ;
    
    
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
        transform: Transform::from_xyz(200.0, 12.5, 200.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    }).insert(TerrainViewer::default());
}



fn update_terrain_chunks(
    terrain_query: Query<(&TerrainConfig,&TerrainData,&Transform)>,
    
    terrain_viewer: Query<&Transform, With<TerrainViewer>>
    
    
){
    
        let viewer:&Transform = terrain_viewer.single();
        
    for (terrain_config,terrain_data,terrain_transform) in terrain_query.iter() { 
        
        let terrain_origin = terrain_transform.translation;
        
        let terrain_dimensions = terrain_config.terrain_dimensions;
    
        
        let viewer_location:Vec3 = viewer.translation; 
        
        let chunk_rows = terrain_config.chunk_rows; 
        
        let chunk_coords_opt: Option<ChunkCoords> = ChunkCoords::from_location( viewer_location , terrain_origin, terrain_dimensions, chunk_rows);  
        
        
        if let Some(chunk_coords) = chunk_coords_opt {
            
            
            
              let chunk_index: u32 = chunk_coords.get_chunk_index( chunk_rows  );
                
        
                // loop through the potential chunks that are around the client to maybe activate them 
                for x_index in  -2..2 {
                    for z_index in  -2..2 {
                        
                        
                        //let chunk_index = get_chunk_index_for_location( viewer_location );  
                        
                    
                    }
                    
                }
        
        
        
        }
        
      
        
  }
}


 

//loop through all of the active chunks that exist to maybe deactive them if client is too far away 