

use bevy::{   prelude::*, asset::LoadState};

use crate::{terrain::{TerrainConfig, TerrainViewer, TerrainData}, pre_mesh::PreMesh, terrain_material::TerrainMaterial};




#[derive(Component,Default)]
pub struct Chunk {
    chunk_id: u32 
    
}

 
   
 
pub struct ChunkData {
    is_active: bool,
    has_spawned_mesh: bool,  
    needs_rebuild: bool, 
    
    chunk_state: ChunkState 
}

enum ChunkState{
    FULLY_BUILT,
    BUILDING,   
    PENDING
}


pub trait ChunkCoordinates {
    fn get_chunk_index(&self, chunk_rows: u32) -> u32; 
    fn from_location( location: Vec3 ,  terrain_origin: Vec3 , terrain_dimensions: Vec2 , chunk_rows: u32 ) -> Option<UVec2> ;
 
    
    fn from_chunk_id(chunk_id:u32,chunk_rows:u32) -> Self;
    fn get_location_offset(&self,  chunk_dimensions: Vec2 ) -> Vec3; 
    
    fn get_heightmap_subsection_bounds_pct(&self, chunk_rows:u32 ) -> [ [f32 ; 2]  ;2 ] ; 
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
    
     fn get_heightmap_subsection_bounds_pct(
         &self,
         chunk_rows: u32
         
         ) -> [ [f32 ; 2]  ;2 ] {
        let chunk_x = self.x;
        let chunk_y = self.y;
        
        let pct_per_row = 1.0 / chunk_rows as f32;  
        
        return [
            [ chunk_x as f32 * pct_per_row , chunk_y as f32 * pct_per_row ],  //start corner x and y 
            [(chunk_x+1) as f32 * pct_per_row , (chunk_y+1) as f32 * pct_per_row]    //end corner x and y 
        ]
    }
    
    
}

  





pub fn update_terrain_chunks(
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
             
                
              let render_distance_chunks:u32 = terrain_config.get_chunk_render_distance()  ; //make based on render dist 
            
        
                // loop through the potential chunks that are around the client to maybe activate them 
                for x_offset in  -1*render_distance_chunks as i32..render_distance_chunks as i32 {
                    for z_offset in  -1*render_distance_chunks as i32..render_distance_chunks as i32 {
                        
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
                        
                    }
                    
                }
        
        
        
        }
        
      
        
  }
}


pub fn activate_chunk_at_coords( 
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
               has_spawned_mesh: false ,
               needs_rebuild: true 
            });
        
    }
        
    
    
}



pub type TerrainPbrBundle = MaterialMeshBundle<TerrainMaterial>;
 

 // if height_map_data is ever edited, remember that the chunks which render those datapoints need to be flagged as needing re-render !
             
pub fn build_active_terrain_chunks(
    mut commands: Commands, 
    mut terrain_query: Query<(Entity, &TerrainConfig,&mut TerrainData)>,
    
    //terrain_viewer: Query<&Transform, With<TerrainViewer>>
    asset_server: Res<AssetServer>,
    
    //assets -- temp 
    images: Res<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    
    mut terrain_materials: ResMut<Assets<TerrainMaterial>>,
    
){
    
    
    for (terrain_entity, terrain_config, mut terrain_data) in terrain_query.iter_mut() { 
        
        /* let height_map_handle = &terrain_data.height_map_image_handle;
         let height_map_loaded = asset_server.get_load_state( height_map_handle )  ;
              
         if height_map_loaded != LoadState::Loaded  {
            println!("height map not yet loaded");
            continue;
          }
              
        let height_map_image:&Image = images.get(height_map_handle).unwrap();*/
        let height_map_data = &terrain_data.height_map_data.clone();
              
        if height_map_data.is_none() {
            continue; 
        }
              
        let terrain_material_handle_option = &terrain_data.terrain_material_handle.clone() ; 
              
        if terrain_material_handle_option.is_none() {
            println!("no terrain material yet.. ");
            continue; 
        }
              
        let terrain_material_handle = terrain_material_handle_option.as_ref().unwrap();
              
        for (chunk_id , chunk_data) in terrain_data.chunks.iter_mut(){
            
            if !chunk_data.has_spawned_mesh {
                chunk_data.has_spawned_mesh = true;
                
              let chunk_rows = terrain_config.chunk_rows;
              let terrain_dimensions = terrain_config.terrain_dimensions;
                
               //build the meshes !!!
              let chunk_coords = ChunkCoords::from_chunk_id(chunk_id.clone(), chunk_rows);
              let chunk_dimensions = terrain_config.get_chunk_dimensions(  );
                  
              let chunk_location_offset:Vec3 = chunk_coords.get_location_offset( chunk_dimensions ) ; 
               
              let height_map_subsection_pct = chunk_coords.get_heightmap_subsection_bounds_pct(chunk_rows);
               //sample me and build triangle data !! 
              let height_map_data =  height_map_data.as_ref().unwrap();
              
              println!("subsection pct {:?}", height_map_subsection_pct);
              
               
            
              let pre_mesh = PreMesh::from_heightmap_subsection( height_map_data, height_map_subsection_pct  );   //need to add chunk coords.. 
              
              let mesh = pre_mesh.build();
              
            //  let sample_mesh:Mesh = shape::Plane::from_size( chunk_dimensions.x ).into();
         
              let terrain_mesh_handle = meshes.add( mesh );
               
              let sample_material_handle = materials.add(Color::rgb(0.3, 0.5, 0.3).into());
             
              let child_mesh =  commands.spawn(
                     TerrainPbrBundle {
                        mesh: terrain_mesh_handle,
                        material: terrain_material_handle.clone(),
                        transform: Transform::from_xyz( chunk_location_offset.x,chunk_location_offset.y,chunk_location_offset.z ) ,
                        ..default()
                        } 
                    ).insert(  
                        Chunk {
                            chunk_id: chunk_id.clone()
                        } 
                    )
                    
                    
                    
                    .id() ; 
              
              println!("adding plane mesh to chunk ");
              let mut terrain_entity_commands  = commands.get_entity(terrain_entity).unwrap();
              terrain_entity_commands.add_child(    child_mesh  );
                
                
            }                
        }
        
        
    }
    
}