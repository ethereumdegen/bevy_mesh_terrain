use bevy::prelude::*;
use bevy::tasks::{Task, AsyncComputeTaskPool};

use bevy::utils::HashMap;
use futures_lite::future;

use crate::heightmap::{SubHeightMapU16, HeightMapU16};
use crate::pre_mesh::PreMesh;
use crate::terrain::{ TerrainViewer, TerrainData, TerrainImageDataLoadStatus};
use crate::terrain_config::TerrainConfig;
use crate::terrain_material::{TerrainMaterial, ChunkMaterialUniforms};



#[derive(Default,Eq,PartialEq)]
enum ChunkState{
    #[default]
    Init,
    
    Building, 
    FullyBuilt,
}


#[derive(Event )]
pub enum ChunkEvent {
    ChunkEntitySpawned(Entity)
} 


#[derive(Component,Default)]
pub struct Chunk {
    pub chunk_id: u32, //same as chunk index   
  //  pub chunk_bounds: [[usize;2]; 2 ],
   // pub chunk_state: ChunkState,
   // pub lod_level: Option<u8>
    
} 

impl Chunk {
    
    pub fn new (chunk_id:u32  ) -> Self {
        
        Self {
            chunk_id,
           // chunk_bounds,
          //  chunk_state: ChunkState::Init,
            
          //  lod_level: 
            
        }
        
        
    }
}


#[derive(Component)]
pub struct ChunkData {
    spawned_mesh_entity: Option<Entity> ,
    chunk_state: ChunkState ,
    lod_level: u8, 
  
    
    //could be a massive image like 4k 
    pub height_map_image_handle: Option<Handle<Image>>, 
    pub height_map_image_data_load_status: TerrainImageDataLoadStatus,
    
    //need to add asset handles here for the heightmap image and texture image !!! 
     
     
    pub height_map_data: Option<HeightMapU16>,
   
    
 //   texture_image_handle: Option<Handle<Image>>,
 //   texture_image_sections: u32, 
 //   texture_image_finalized: bool,  //need this for now bc of the weird way we have to load an array texture w polling and stuff... GET RID of me ???replace w enum ? 
    
    splat_image_handle: Option<Handle<Image>>,
    
    alpha_mask_image_handle: Option<Handle<Image>>, //built from the height map 
   
    pub terrain_material_handle: Option<Handle<TerrainMaterial> >
}


   
 
 

#[derive(Component)]
pub struct MeshBuilderTask(Task<BuiltChunkMeshData>);


pub struct BuiltChunkMeshData {
    terrain_entity_id: Entity, 
    chunk_bounds: [[usize;2]; 2 ],
    
    chunk_id: u32,
    chunk_location_offset:Vec3, 
    
    mesh:Mesh,
    chunk_uv: Vec4,
    
     lod_level: u8  
    
}
 
 


pub trait ChunkCoordinates {
    
    fn x(&self) -> u32 ;
    fn y(&self) -> u32 ;
    
    fn get_chunk_index(&self, chunk_rows: u32) -> u32; 


    fn from_location( location: Vec3 ,  terrain_origin: Vec3 , terrain_dimensions: Vec2 , chunk_rows: u32 ) -> Option<UVec2> ;
    fn to_location(&self, terrain_origin: Vec3, terrain_dimensions: Vec2, chunk_rows: u32) -> Option<Vec3> ;
    
    fn from_chunk_id(chunk_id:u32,chunk_rows:u32) -> Self;
    fn get_location_offset(&self,  chunk_dimensions: Vec2 ) -> Vec3; 
    
    fn get_heightmap_subsection_bounds_pct(&self, chunk_rows:u32 ) -> [ [f32 ; 2]  ;2 ] ; 
}


type ChunkCoords =  [u32; 2 ] ; 

impl ChunkCoordinates for  ChunkCoords {
    
     fn x(&self) -> u32 {
        self[0]
    }
     fn y(&self) -> u32 {
        self[1]
    }
    
     //chunk index is   chunk_col * 64  + chunk_row   IF chunk_rows is 64 
    fn get_chunk_index(&self, chunk_rows: u32) -> u32 {
        
        return self.y() * chunk_rows + self.x() as u32; 
        
    }
    
    
    fn from_chunk_id(chunk_id:u32, chunk_rows: u32) -> Self { 
        let coords_y = chunk_id / chunk_rows;
        let coords_x = chunk_id % chunk_rows;
        
        [coords_x,coords_y]
    }
      
      
    
    
    fn get_location_offset(&self,  chunk_dimensions: Vec2 ) -> Vec3 { 
         
        Vec3::new(chunk_dimensions.x * self.x() as f32,0.0,chunk_dimensions.y * self.y() as f32) 
        
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
    if self.x() < chunk_rows && self.y() < chunk_rows {
        // Calculate the dimensions of a single chunk
        let chunk_dim_x = terrain_dimensions.x / chunk_rows as f32;
        let chunk_dim_z = terrain_dimensions.y / chunk_rows as f32;

        // Calculate world location for the bottom-left corner of the chunk
        let world_x = terrain_origin.x + self.x() as f32 * chunk_dim_x + chunk_dim_x/2.0;
        let world_z = terrain_origin.z + self.y() as f32 * chunk_dim_z + chunk_dim_z/2.0;
        
        

        return Some(Vec3::new(world_x, terrain_origin.y, world_z));
    }

    None
    }
    
     fn get_heightmap_subsection_bounds_pct(
         &self,
         chunk_rows: u32
         
         ) -> [ [f32 ; 2]  ;2 ] {
        let chunk_x = self.x();
        let chunk_y = self.y();
        
        let pct_per_row = 1.0 / chunk_rows as f32;  
        
        return [
            [ chunk_x as f32 * pct_per_row , chunk_y as f32 * pct_per_row ],  //start corner x and y 
            [(chunk_x+1) as f32 * pct_per_row , (chunk_y+1) as f32 * pct_per_row]    //end corner x and y 
        ]
    }
    
    
}

  
  
  
fn calculate_chunk_coords( from_location: Vec3, terrain_origin: Vec3, terrain_dimensions: Vec2, chunk_rows: u32  ) -> [ i32 ;2] {
     
        let location_delta = from_location - terrain_origin;

        
        let chunk_x = (location_delta.x / terrain_dimensions.x * chunk_rows as f32) as i32;
        let chunk_z = (location_delta.z / terrain_dimensions.y * chunk_rows as f32) as i32;

        return  [chunk_x, chunk_z] ; 
    
}
  
  
  
/*

On initialization of terrain entity, the chunk entities should be spawned and they should just remain there forever !!! 
 */ 
 
 //this may lag.. 
pub fn build_chunk_meshes(
    commands: Commands,
   mut terrain_query: Query<(&TerrainConfig,& TerrainData)>,
    
   mut chunk_query : Query<( &Chunk,&mut ChunkData, &Parent,  &GlobalTransform, &Visibility ) >,
){
    
     for (chunk,mut chunk_data, parent_entity,  chunk_transform, mut chunk_visibility) in chunk_query.iter_mut() { 
         
         if chunk_data.chunk_state == ChunkState::Init {
             
             
         
        let Ok((terrain_config,terrain_data)) = terrain_query.get( parent_entity.get() );
             
             
             
             
         let height_map_image:&Image = images.get(height_map_handle).unwrap(); 
        let height_map_data =  &terrain_data.height_map_data .clone();
              
        if height_map_data.is_none() {
            continue; 
        }
              
        let terrain_material_handle_option = &terrain_data.terrain_material_handle.clone() ; 
              
        if terrain_material_handle_option.is_none() {
            println!("no terrain material yet.. ");
            continue; 
        }
              
      //  let array_texture =  terrain_data.get_array_texture_image().clone();
      //  let splat_texture =  terrain_data.get_splat_texture_image().clone();
                       
            let thread_pool = AsyncComputeTaskPool::get();
        
       
              chunk_data.chunk_state = ChunkState::Building;
           
                
              let chunk_rows = terrain_config.chunk_rows;
              let terrain_dimensions = terrain_config.terrain_dimensions;
              let height_scale = terrain_config.height_scale;
                
               //build the meshes !!!
              let chunk_coords = ChunkCoords::from_chunk_id(chunk_id.clone(), chunk_rows);
              let chunk_dimensions = terrain_config.get_chunk_dimensions(  );
                  
              let chunk_location_offset:Vec3 = chunk_coords.get_location_offset( chunk_dimensions ) ; 
               
              let height_map_subsection_pct = chunk_coords.get_heightmap_subsection_bounds_pct(chunk_rows);
               //sample me and build triangle data !! 
               
               // might use lots of RAM ? idk ..
               //maybe we subsection first and THEN build the mesh!  oh well... anyways 
              let height_map_data_cloned =  height_map_data.as_ref().unwrap().clone();
           
              
              let lod_level = chunk_data.lod_level;
              
               
               
              let chunk_uv = Vec4::new( //tell the shader how to use the splat map for this chunk  
                                    height_map_subsection_pct[0][0],
                                    height_map_subsection_pct[0][1],
                                    height_map_subsection_pct[1][0],
                                    height_map_subsection_pct[1][1] );
                                    
               let chunk_id_clone = chunk_id.clone();
               
               let task = thread_pool.spawn(async move {
                    
                    
                    /*
                    
                    This could be optimized by not passing in the ENTIRE height map data cloned but only a subsection for this chunk... 
                    
                    */ 
                    
                    let sub_heightmap = SubHeightMapU16::from_heightmap_u16(
                        &height_map_data_cloned,
                        height_map_subsection_pct
                    );
                    
                    
                    
                    let mesh = PreMesh::from_heightmap_subsection( 
                         &sub_heightmap, 
                         
                          height_scale,
                          lod_level,  
                       
                        
                        [terrain_dimensions.x, terrain_dimensions.y]
                    ).build(); 
                    
                     BuiltChunkMeshData {
                         chunk_id: chunk_id_clone,
                         chunk_bounds: [sub_heightmap.start_bound,sub_heightmap.end_bound],
                         
                         chunk_location_offset: chunk_location_offset.clone(),
                         
                         terrain_entity_id: terrain_entity.clone(),
                         mesh,
                         chunk_uv,
                         lod_level 
                     }
                });

                // Spawn new entity and add our new task as a component
                commands.spawn(MeshBuilderTask(task));
                
           // }                
        } 
             
             
             
              
             
             
             
             
             
         } 
         
       
    
}
 
 
 
 
 
 
 
pub fn finish_chunk_build_tasks(
    mut commands: Commands,
    mut chunk_build_tasks: Query<(Entity, &mut MeshBuilderTask)>,
    
    mut meshes: ResMut<Assets<Mesh>>,
    mut terrain_query: Query<&mut TerrainData,With<TerrainConfig>>,
    mut terrain_materials: ResMut<Assets<TerrainMaterial>>,
    
     mut chunk_events: EventWriter<ChunkEvent> ,
) {
    
    
    
    for (entity, mut task) in &mut chunk_build_tasks {
        if let Some(built_chunk_mesh_data) = future::block_on(future::poll_once(&mut task.0)) {
            // Add our new PbrBundle of components to our tagged entity
          
           
            let terrain_entity_id = built_chunk_mesh_data.terrain_entity_id;
               
            let chunk_uv = built_chunk_mesh_data.chunk_uv;
            let mesh = built_chunk_mesh_data.mesh; 
            
            let chunk_id = built_chunk_mesh_data.chunk_id;
            let chunk_location_offset = built_chunk_mesh_data.chunk_location_offset;
            
            //careful w this unwrap
            if terrain_query.get_mut(terrain_entity_id).is_ok() == false {continue;}
            
            let mut terrain_data = terrain_query.get_mut(terrain_entity_id).unwrap(); 
               
             
             
             let array_texture =  terrain_data.get_array_texture_image().clone();
             let splat_texture =  terrain_data.get_splat_texture_image().clone();
             let alpha_mask_texture =  terrain_data.get_alpha_mask_texture_image().clone();
                                        
                                        
             if terrain_data.chunks.get_mut( &chunk_id ).is_some() == false {continue;}      
               //careful w unwrap!!! 
             let chunk_data = &mut terrain_data.chunks.get_mut( &chunk_id ).unwrap();
                                        
            let chunk_terrain_material:Handle<TerrainMaterial>  =  terrain_materials.add(
                    TerrainMaterial {
                               
                               
                                uniforms: ChunkMaterialUniforms{
                                     color_texture_expansion_factor: 16.0,  //why wont this apply to shader properly ? 
                                     chunk_uv 
                                },
                                
                                array_texture: array_texture.clone(),
                                splat_texture : splat_texture.clone(),
                                alpha_mask_texture: alpha_mask_texture.clone() 
                            }
                
            )  ;
         
              let terrain_mesh_handle = meshes.add( mesh );
                
             
              let child_mesh =  commands.spawn(
                     TerrainPbrBundle {
                        mesh: terrain_mesh_handle,
                        material: chunk_terrain_material ,
                        transform: Transform::from_xyz( 
                            0.0,
                            0.0,
                            0.0 
                            ) ,
                        ..default()
                        } 
                    )
                    
                    .id() ; 
                    
             
            
              let mut terrain_entity_commands  = commands.get_entity(terrain_entity_id).unwrap();
              terrain_entity_commands.add_child(    child_mesh  );
            
               chunk_data.chunk_state = ChunkState::FullyBuilt; 
               
               //need to do this in a safer way!!! 
             /*  if let Some(old_mesh_entity) =  chunk_data.spawned_mesh_entity { 
                        commands.entity(old_mesh_entity).insert(
                            NeedToDespawnChunk{}
                        );     
               };*/
               
              chunk_data.spawned_mesh_entity = Some( child_mesh  ) ;
              chunk_events.send(  ChunkEvent::ChunkEntitySpawned( child_mesh ) );
           

            // Task is complete, so remove task component from entity
            commands.entity(entity).remove::<MeshBuilderTask>();

         
        }
    }
}
 
 
 
pub fn update_chunk_visibility(
   mut terrain_query: Query<(&TerrainConfig,& TerrainData)>,
    
   mut chunk_query : Query<( &Chunk,&mut ChunkData, &Parent,  &GlobalTransform, &Visibility ) >,
    
   terrain_viewer: Query<&GlobalTransform, With<TerrainViewer>> 
){
    
    let viewer  = terrain_viewer.get_single();
        
    let viewer_location:Vec3 = match viewer {
        Ok(view) => { view.translation() },
        // FIX: probably should log a warning if there are multiple (or no) viewers, rather than just setting to the origin
        Err(_e) => Vec3::new(0.0,0.0,0.0)
    };
        
    for (chunk,mut chunk_data, parent_entity,  chunk_transform, mut chunk_visibility) in chunk_query.iter_mut() { 
        
       // let terrain_origin = terrain_transform.translation();
        
       // let terrain_dimensions = terrain_config.terrain_dimensions; 
        
      
        
       // let chunk_rows = terrain_config.chunk_rows; 
        
        
      /*  let chunk_coords_signed = calculate_chunk_coords ( 
            viewer_location , 
            terrain_origin, 
            terrain_dimensions, 
            chunk_rows
          );
        
           */
     
       
         if  let Ok ((terrain_config, terrain_data)) = terrain_query.get(parent_entity.get()) {
              
                
              let render_distance_chunks:i32  = terrain_config.get_chunk_render_distance() as i32 ; //make based on render dist 
              let lod_level_distance:f32 = terrain_config.get_chunk_lod_distance(); 
              let lod_level_offset:u8 = terrain_config.lod_level_offset;
              
        
                // loop through the potential chunks that are around the client to maybe activate them 
              //  for x_offset in  -1*render_distance_chunks ..render_distance_chunks  {
               //     for z_offset in  -1*render_distance_chunks  ..render_distance_chunks   {
                        
                      
 

                         
 
                        //calc chunk world loc and use to calc the lod 
                        let chunk_world_location = chunk_transform;
 
    
                         let distance_to_chunk:   f32    =  chunk_world_location.translation().distance( viewer_location )   ;
        
                        let lod_level: u8 = match distance_to_chunk {
                             dist  => {
                                 if dist > lod_level_distance*2.0 {  2 }
                                 else if dist > lod_level_distance  {  1 }
                                 else {
                                 0 
                                 }
                            },  
                        } + lod_level_offset; 
                      
                        chunk_data.lod_level = lod_level;
                        
                           let max_render_distance = terrain_config.get_max_render_distance() ;  
               
                            let should_be_visible = match distance_to_chunk {
                                dist  => dist <= max_render_distance 
                            };
                                 
                                  
                           chunk_visibility = match should_be_visible {
                            true => &Visibility::Visible ,
                            false => &Visibility::Hidden   
                           } ;         
                          println!(" set chunk vis   {:?}",  chunk_visibility ) ;
                        
                         
         
      
         } 
       }
}