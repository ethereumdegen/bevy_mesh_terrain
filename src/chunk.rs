

use bevy::{   prelude::*, asset::LoadState};

use crate::{terrain::{TerrainConfig, TerrainViewer, TerrainData}, pre_mesh::PreMesh, terrain_material::{TerrainMaterial, ChunkMaterialUniforms}};




#[derive(Component,Default)]
pub struct Chunk {
    chunk_id: u32 
    
}

 
   
 
pub struct ChunkData {
    is_active: bool,
    
    needs_rebuild: bool, 
    
    spawned_mesh_entity: Option<Entity> ,
    
    chunk_state: ChunkState ,
    
    lod_level: u8 //lod level of 0 is maximum quality.  1 is less, 2 is much less, 3 is very decimated 
}

#[derive(Eq,PartialEq)]
enum ChunkState{
    FULLY_BUILT,
    BUILDING,   
    PENDING 
}


pub trait ChunkCoordinates {
    fn get_chunk_index(&self, chunk_rows: u32) -> u32; 
    fn from_location( location: Vec3 ,  terrain_origin: Vec3 , terrain_dimensions: Vec2 , chunk_rows: u32 ) -> Option<UVec2> ;
    fn to_location(&self, terrain_origin: Vec3, terrain_dimensions: Vec2, chunk_rows: u32) -> Option<Vec3> ;
    
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
    
    //returns the middle of the chunk 
    fn to_location(&self, terrain_origin: Vec3, terrain_dimensions: Vec2, chunk_rows: u32) -> Option<Vec3> {
    // Ensure chunk coordinates are within bounds
    if self.x < chunk_rows && self.y < chunk_rows {
        // Calculate the dimensions of a single chunk
        let chunk_dim_x = terrain_dimensions.x / chunk_rows as f32;
        let chunk_dim_z = terrain_dimensions.y / chunk_rows as f32;

        // Calculate world location for the bottom-left corner of the chunk
        let world_x = terrain_origin.x + self.x as f32 * chunk_dim_x + chunk_dim_x/2.0;
        let world_z = terrain_origin.z + self.y as f32 * chunk_dim_z + chunk_dim_z/2.0;
        
        

        return Some(Vec3::new(world_x, terrain_origin.y, world_z));
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

  
pub fn destroy_terrain_chunks(
    mut commands: Commands, 
    mut chunk_query: Query<(Entity, &mut Chunk, &Parent) > ,
    
    mut terrain_query : Query<(&mut TerrainData,&TerrainConfig, &Transform)> ,
     terrain_viewer: Query<&Transform, With<TerrainViewer>> 
){
     let viewer  = terrain_viewer.get_single();
        
    let viewer_location:Vec3 = match viewer {
        Ok(view) => { view.translation },
        Err(e) => Vec3::new(0.0,0.0,0.0)
    };
        
    
    for (chunk_entity_id, mut chunk_data, parent_terrain_entity) in chunk_query.iter_mut() { 
        
        let chunk_id = chunk_data.chunk_id; 
        
        let mut needs_despawn = None;
        
        if let Ok( (mut terrain_data, terrain_config, terrain_transform ) ) = terrain_query.get_mut( parent_terrain_entity.get() ){
            
            let chunk_rows = terrain_config.chunk_rows;
            let terrain_dimensions= terrain_config.terrain_dimensions;
            let terrain_origin = terrain_transform.translation;
            let max_render_distance = terrain_config.get_max_render_distance();
            
            //if too far away ...
            let chunk_coords = ChunkCoords::from_chunk_id(chunk_id, chunk_rows);
             
             let chunk_world_location = chunk_coords.to_location(   
                            terrain_origin, 
                            terrain_dimensions, 
                            chunk_rows 
                        );
    
             let distance_to_chunk: Option<f32>  = match chunk_world_location {
                             Some( location ) => {
                                 Some( location.distance( viewer_location ) ) 
                             },
                             None => None
                             
            } ; 
            
           
            
            let should_despawn = match distance_to_chunk {
                Some( dist ) => dist > max_render_distance,
                None => true 
            };
            
            if should_despawn {
                 needs_despawn = Some(chunk_entity_id);
                 terrain_data.chunks.remove(  &chunk_id ); 
            }
            
        }else{
            
            //despawn it for sure 
            needs_despawn = Some(chunk_entity_id);
            
        }
        
        
        if needs_despawn.is_some() {
           
            commands.entity( needs_despawn.unwrap() ).despawn();    
            
        }
        
    }
        
    
}



/*
Dont need this to run each frame... 
*/
pub fn activate_terrain_chunks(
   mut  commands:  Commands, 
   mut terrain_query: Query<(&TerrainConfig,&mut TerrainData,&Transform)>,
    
   terrain_viewer: Query<&Transform, With<TerrainViewer>> 
){
    
    let viewer  = terrain_viewer.get_single();
        
    let viewer_location:Vec3 = match viewer {
        Ok(view) => { view.translation },
        Err(e) => Vec3::new(0.0,0.0,0.0)
    };
        
    for (terrain_config,mut terrain_data,terrain_transform) in terrain_query.iter_mut() { 
        
        let terrain_origin = terrain_transform.translation;
        
        let terrain_dimensions = terrain_config.terrain_dimensions; 
        
      
        
        let chunk_rows = terrain_config.chunk_rows; 
        
        let chunk_coords_opt: Option<ChunkCoords> = ChunkCoords::from_location( viewer_location , terrain_origin, terrain_dimensions, chunk_rows);  
        
        //these are the chunk coords of the viewer - the center 
        if let Some(chunk_coords_at_viewer) = chunk_coords_opt {
             
                
              let render_distance_chunks:u32 = terrain_config.get_chunk_render_distance()  ; //make based on render dist 
              let lod_level_distance:f32 = terrain_config.get_chunk_lod_distance(); 
        
                // loop through the potential chunks that are around the client to maybe activate them 
                for x_offset in  -1*render_distance_chunks as i32..render_distance_chunks as i32 {
                    for z_offset in  -1*render_distance_chunks as i32..render_distance_chunks as i32 {
                        
                        let chunk_coords_x = chunk_coords_at_viewer.x as i32 + x_offset ;
                        let chunk_coords_z = chunk_coords_at_viewer.y as i32 + z_offset ;
                        
                        let chunk_coords = ChunkCoords::new( chunk_coords_x as u32, chunk_coords_z as u32  );
                        
                        //calc chunk world loc and use to calc the lod 
                        let chunk_world_location = chunk_coords.to_location(   
                            terrain_origin, 
                            terrain_dimensions, 
                            chunk_rows 
                        );
    
                         let distance_to_chunk: Option<f32>  = match chunk_world_location {
                             Some( location ) => {
                                 Some( location.distance( viewer_location ) ) 
                             },
                             None => None
                             
                         } ;
        
                        let lod_level: u8 = match distance_to_chunk {
                            Some(dist) => {
                                 if dist > lod_level_distance*2.0 {  2 }
                                 else if dist > lod_level_distance  {  1 }
                                 else {
                                 0 
                                 }
                            },
                            None => 2 
                        }; 
                        //  ----- 
                        
                        if  chunk_coords_x >= 0 && chunk_coords_x < chunk_rows as i32
                            && chunk_coords_z >=0 && chunk_coords_z < chunk_rows as i32  {
                                //then this is a valid coordinate location 
                                activate_chunk_at_coords( 
                                    &mut commands,
                                    chunk_coords,  
                                    &mut terrain_data,
                                    &terrain_config,
                                    lod_level
                                );
                                                                                                
                            }
                        
                    }
                    
                }
        
        
        
        }
        
      
        
  }
}


pub fn activate_chunk_at_coords( 
    mut commands: &mut Commands, 
    chunk_coords: ChunkCoords,
    mut terrain_data: &mut TerrainData,
    terrain_config: &TerrainConfig,
    lod_level: u8
) {
    
    let chunk_rows = terrain_config.chunk_rows;
    
    let chunk_index: u32 = chunk_coords.get_chunk_index( chunk_rows  );
    
    let chunk_exists = terrain_data.chunks.contains_key( &chunk_index );
    
    let mut need_to_spawn = false;
    let mut need_to_despawn: Option<Entity>  = None; 
    
    
    if chunk_exists { 
         
       let chunk_data = terrain_data.chunks.get(&chunk_index).unwrap(); 
         
        // do not rebuild chunks until they are already fully built 
        if chunk_data.chunk_state ==  ChunkState::FULLY_BUILT {
            
            let existing_lod = chunk_data.lod_level; 
            if lod_level != existing_lod && chunk_data.spawned_mesh_entity.is_some(){
                need_to_despawn = chunk_data.spawned_mesh_entity;           
                need_to_spawn = true;
                
                
            }  
        }  
        
    }else { 
        need_to_spawn = true;    
    }
        
        
    if let Some(entity) = need_to_despawn { 
         commands.entity(entity).despawn();   
         terrain_data.chunks.remove(  &chunk_index ); 
    }
        
    if need_to_spawn {
        
         terrain_data.chunks.insert(  
            chunk_index  , 
            ChunkData {
               is_active: true, //useless for now 
               chunk_state: ChunkState::PENDING,
               spawned_mesh_entity: None,
               needs_rebuild: true ,  //useless for now 
               lod_level 
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
              
        let array_texture =  terrain_data.get_array_texture_image().clone();
        let splat_texture =  terrain_data.get_splat_texture_image().clone();
              
        let terrain_material_handle = terrain_material_handle_option.as_ref().unwrap();
         
 
        
        let terrain_data_chunks = &mut terrain_data.chunks; 
        for (chunk_id , chunk_data) in terrain_data_chunks.iter_mut(){
            
            if chunk_data.chunk_state == ChunkState::PENDING {
                chunk_data.chunk_state = ChunkState::BUILDING;
                
              let chunk_rows = terrain_config.chunk_rows;
              let terrain_dimensions = terrain_config.terrain_dimensions;
                
               //build the meshes !!!
              let chunk_coords = ChunkCoords::from_chunk_id(chunk_id.clone(), chunk_rows);
              let chunk_dimensions = terrain_config.get_chunk_dimensions(  );
                  
              let chunk_location_offset:Vec3 = chunk_coords.get_location_offset( chunk_dimensions ) ; 
               
              let height_map_subsection_pct = chunk_coords.get_heightmap_subsection_bounds_pct(chunk_rows);
               //sample me and build triangle data !! 
              let height_map_data =  height_map_data.as_ref().unwrap();
              
             
              
              let lod_level = chunk_data.lod_level;
            
              let mesh = PreMesh::from_heightmap_subsection( 
                  height_map_data, 
                  lod_level, 
                  height_map_subsection_pct
              ).build();   //need to add chunk coords.. 
              
              
              
          
            let chunk_uv = Vec4::new( //tell the shader how to use the splat map for this chunk  
                                    height_map_subsection_pct[0][0],
                                    height_map_subsection_pct[0][1],
                                    height_map_subsection_pct[1][0],
                                    height_map_subsection_pct[1][1] );
                                    
            let chunk_terrain_material:Handle<TerrainMaterial>  =  terrain_materials.add(
                    TerrainMaterial {
                               
                               
                                  uniforms: ChunkMaterialUniforms{
                                     color_texture_expansion_factor: 16.0,
                                     chunk_uv 
                                },
                                
                                array_texture: array_texture.clone(),
                                splat_texture : splat_texture.clone()
                            }
                
            )  ;
         
              let terrain_mesh_handle = meshes.add( mesh );
               
              let sample_material_handle = materials.add(Color::rgb(0.3, 0.5, 0.3).into());
             
              let child_mesh =  commands.spawn(
                     TerrainPbrBundle {
                        mesh: terrain_mesh_handle,
                        material: chunk_terrain_material ,
                        transform: Transform::from_xyz( chunk_location_offset.x,chunk_location_offset.y,chunk_location_offset.z ) ,
                        ..default()
                        } 
                    ).insert(  
                        Chunk {
                            chunk_id: chunk_id.clone()
                        } 
                    ) 
                    .id() ; 
              
            
              let mut terrain_entity_commands  = commands.get_entity(terrain_entity).unwrap();
              terrain_entity_commands.add_child(    child_mesh  );
            
               chunk_data.chunk_state = ChunkState::FULLY_BUILT;
               chunk_data.spawned_mesh_entity = Some( child_mesh  ) ;
       
                //update chunk data state from loading to completed -- NEVER delete the component while its loading 
                
            }                
        }
        
        
    }
    
}