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

impl TerrainConfig {
    
     fn get_chunk_dimensions(&self ) -> Vec2 {
        let chunk_dimension_x = self.terrain_dimensions.x / self.chunk_rows as f32;
        let chunk_dimension_z = self.terrain_dimensions.y / self.chunk_rows as f32;
         
        
        Vec2::new(chunk_dimension_x  , chunk_dimension_z  ) 
        
    }  
    
    
}

#[derive(Component,Default)]
struct TerrainData {
      //  pub terrain_origin: Vec3 // should be a component of an entity 
    //chunk_index -> chunk data 
    //chunk index is   chunk_col * 64  + chunk_row   IF chunk_rows is 64 
    pub chunks: HashMap<u32,ChunkData>, 
    
    //need to add asset handles here for the heightmap image and texture image !!! 
    

    
    
}
 
struct ChunkData {
    is_active: bool,
    has_spawned_mesh: bool, 
    
    chunk_state: ChunkState
    
}

enum ChunkState{
    FULLY_BUILT,
    BUILDING,   
    PENDING
}


trait ChunkCoordinates {
    fn get_chunk_index(&self, chunk_rows: u32) -> u32; 
    fn from_location( location: Vec3 ,  terrain_origin: Vec3 , terrain_dimensions: Vec2 , chunk_rows: u32 ) -> Option<UVec2> ;

    
    
    fn from_chunk_id(chunk_id:u32,chunk_rows:u32) -> Self;
    fn get_location_offset(&self,  chunk_dimensions: Vec2 ) -> Vec3; 
}


type ChunkCoords = UVec2 ; 

impl ChunkCoordinates for  ChunkCoords {
    
    
     //chunk index is   chunk_col * 64  + chunk_row   IF chunk_rows is 64 
    fn get_chunk_index(&self, chunk_rows: u32) -> u32 {
        
        return self.y * chunk_rows + self.x as u32; 
        
    }
    
    
    fn from_chunk_id(chunk_id:u32, chunk_rows: u32) -> Self { 
        let coords_y = chunk_id / chunk_rows;
        let coords_x = chunk_id % chunk_rows;
        
        UVec2::new(coords_x,coords_y)
    }
      
      
    
    
    fn get_location_offset(&self,  chunk_dimensions: Vec2 ) -> Vec3 { 
         
        Vec3::new(chunk_dimensions.x * self.x as f32,0.0,chunk_dimensions.y * self.y as f32) 
        
    }  
        
   fn from_location(from_location: Vec3, terrain_origin: Vec3, terrain_dimensions: Vec2, chunk_rows: u32) -> Option<UVec2> {
        let location_delta = from_location - terrain_origin;

        //let terrain_min = terrain_origin;
        //let terrain_max = terrain_origin + Vec3::new(terrain_dimensions.x, 0.0, terrain_dimensions.y);

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
        .add_systems(Update, build_active_terrain_chunks.after( update_terrain_chunks ))
        
        
        .run();
}

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // plane
    commands.spawn(VisibilityBundle::default() ) 
    .insert( TransformBundle::default() )
    .insert(TerrainConfig::default())
    .insert(TerrainData::default()) 
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
        transform: Transform::from_xyz(20.0, 12.5, 20.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    })
    .insert(TerrainViewer::default());
    
    println!("completed setup");
}



fn update_terrain_chunks(
    mut terrain_query: Query<(&TerrainConfig,&mut TerrainData,&Transform)>,
    
    terrain_viewer: Query<&Transform, With<TerrainViewer>>
    
    
){
    
    let viewer:&Transform = terrain_viewer.single();
        
    for (terrain_config,mut terrain_data,terrain_transform) in terrain_query.iter_mut() { 
        
        let terrain_origin = terrain_transform.translation;
        
        let terrain_dimensions = terrain_config.terrain_dimensions; 
        
        let viewer_location:Vec3 = viewer.translation; 
        
        let chunk_rows = terrain_config.chunk_rows; 
        
        let chunk_coords_opt: Option<ChunkCoords> = ChunkCoords::from_location( viewer_location , terrain_origin, terrain_dimensions, chunk_rows);  
        
        //these are the chunk coords of the viewer - the center 
        if let Some(chunk_coords_at_viewer) = chunk_coords_opt {
            
              //let chunk_index: u32 = chunk_coords_at_viewer.get_chunk_index( chunk_rows  );
                
              let render_distance_chunks = 2 as i32; //make based on render dist 
                
        
                // loop through the potential chunks that are around the client to maybe activate them 
                for x_offset in  -render_distance_chunks..render_distance_chunks {
                    for z_offset in  -render_distance_chunks..render_distance_chunks {
                        
                        let chunk_coords_x = chunk_coords_at_viewer.x as i32 + x_offset ;
                        let chunk_coords_z = chunk_coords_at_viewer.y as i32 + z_offset ;
                        
                        if  chunk_coords_x >= 0 && chunk_coords_x < chunk_rows as i32
                            && chunk_coords_z >=0 && chunk_coords_z < chunk_rows as i32  {
                                //then this is a valid coordinate location 
                                activate_chunk_at_coords( 
                                    ChunkCoords::new( chunk_coords_x as u32, chunk_coords_z as u32  ),  
                                    &mut terrain_data ,
                                    &terrain_config
                                );
                                                                                                
                            }
                        //let chunk_index = get_chunk_index_for_location( viewer_location );  
                    }
                    
                }
        
        
        
        }
        
      
        
  }
}

fn activate_chunk_at_coords( 
    chunk_coords: ChunkCoords,
    mut terrain_data: &mut TerrainData,
    terrain_config: &TerrainConfig
) {
    
    let chunk_rows = terrain_config.chunk_rows;
    
    let chunk_index: u32 = chunk_coords.get_chunk_index( chunk_rows  );
    
    let chunk_exists = terrain_data.chunks.contains_key( &chunk_index );
    
    if chunk_exists {
        
    }else {
        
        //we flag the chunk so that in a SEPARATE update loop, it will have meshes generated 
        println!("activating chunk! ");
        terrain_data.chunks.insert(  
            chunk_index  , 
            ChunkData {
               is_active: true,
               chunk_state: ChunkState::PENDING,
               has_spawned_mesh: false 
            });
        
    }
        
    
    
}
 
 

//loop through all of the active chunks that exist to maybe deactive them if client is too far away 



fn build_active_terrain_chunks(
    mut commands: Commands, 
    mut terrain_query: Query<(Entity, &TerrainConfig,&mut TerrainData)>,
    
    //terrain_viewer: Query<&Transform, With<TerrainViewer>>
    
    //assets -- temp 
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
){
    
    
    for (terrain_entity, terrain_config, mut terrain_data) in terrain_query.iter_mut() { 
        
        for (chunk_id , chunk_data) in terrain_data.chunks.iter_mut(){
            
            if !chunk_data.has_spawned_mesh {
                chunk_data.has_spawned_mesh = true;
                
                let chunk_rows = terrain_config.chunk_rows;
                let terrain_dimensions = terrain_config.terrain_dimensions;
                
               //build the meshes !!!
              let chunk_coords = ChunkCoords::from_chunk_id(chunk_id.clone(), chunk_rows);
              let chunk_dimensions = terrain_config.get_chunk_dimensions(  );
                  
              let chunk_location_offset:Vec3 = chunk_coords.get_location_offset( chunk_dimensions ) ; 
              
          
              
              let child_mesh =  commands.spawn(
                     PbrBundle {
                        mesh: meshes.add(shape::Plane::from_size( chunk_dimensions.x ).into()),
                        material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
                        transform: Transform::from_xyz( chunk_location_offset.x,chunk_location_offset.y,chunk_location_offset.z ) ,
                        ..default()
                        } 
                    ).id() ; 
              
              println!("adding plane mesh to chunk ");
              let mut terrain_entity_commands  = commands.get_entity(terrain_entity).unwrap();
              terrain_entity_commands.add_child(    child_mesh  );
                
                
            }                
        }
        
        
    }
    
}