use bevy::asset::LoadState;
use bevy::prelude::*;
use bevy::render::render_resource::TextureFormat;
use bevy::tasks::{Task, AsyncComputeTaskPool};

use bevy::utils::HashMap;
use futures_lite::future;

use crate::heightmap::{SubHeightMapU16, HeightMapU16, HeightMap};
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
  //  ChunkEntitySpawned(Entity)
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
   // spawned_mesh_entity: Option<Entity> ,
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
   
    pub material_handle: Option<Handle<TerrainMaterial> >
}


impl ChunkData {
    
    
    
    pub fn get_splat_texture_image(&self) -> &Option<Handle<Image>> {
        
        &self.splat_image_handle 
    }
    
    pub fn get_alpha_mask_texture_image(&self) -> &Option<Handle<Image>> {
        
        &self.alpha_mask_image_handle 
    }
    
    
}
   
 
pub type TerrainPbrBundle = MaterialMeshBundle<TerrainMaterial>;
 
 

#[derive(Component)]
pub struct MeshBuilderTask(Task<BuiltChunkMeshData>);


pub struct BuiltChunkMeshData {
     chunk_entity_id: Entity, 
  //  chunk_bounds: [[usize;2]; 2 ],
    
   // chunk_id: u32,
   // chunk_location_offset:Vec3, 
    
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


pub type ChunkCoords =  [u32; 2 ] ; 

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
  
  
  
  
  
  pub fn initialize_chunk_data( 
      mut commands: Commands,
      
        asset_server: Res<AssetServer> ,
        
      mut chunk_query : Query<( Entity, &Chunk,  &Parent  ), Without<ChunkData> > ,
      
      terrain_query: Query<(&TerrainConfig,& TerrainData)>,
  ) {
       
        for (chunk_entity, chunk,  terrain_entity  ) in chunk_query.iter_mut() { 
            
             
        let terrain_entity_id = terrain_entity.get();   
        if terrain_query.get (terrain_entity_id).is_ok() == false {continue;println!("warn terrain ent ")}
        let (terrain_config,terrain_data)  = terrain_query.get( terrain_entity_id ).unwrap();
          
            
            
            let chunk_id = 1; //chunk.chunk_id;
            
             //default_terrain/diffuse
            let height_texture_folder_path = &terrain_config .height_folder_path;
            let height_texture_path = format!("{}/{}.png" , height_texture_folder_path, chunk_id); 
            println!("height_texture_path {}",height_texture_path);
            let height_map_image_handle: Handle<Image> = asset_server.load(height_texture_path);
            
            //default_terrain/splat
            let splat_texture_folder_path = &terrain_config .splat_folder_path;
            let splat_texture_path = format!("{}/{}.png" , splat_texture_folder_path, chunk_id); 
            let splat_image_handle: Handle<Image> = asset_server.load(splat_texture_path);
            
            
            let chunk_data_component = ChunkData {
                chunk_state: ChunkState::Init,
                lod_level: 0,   // hmm might cause issues .. 
                
                height_map_image_handle : Some(height_map_image_handle) ,
                height_map_data: None,   //make this its own component ? 
                height_map_image_data_load_status: TerrainImageDataLoadStatus::NotLoaded,
                
                splat_image_handle: Some(splat_image_handle),
                alpha_mask_image_handle: None ,  //gets set later 
                material_handle: None//gets set later   
            };
               
            commands.get_entity( chunk_entity ).unwrap().insert(  chunk_data_component  );
                
        }
      
      
}
      
pub fn build_chunk_height_data(
  // mut commands: Commands,
   
  // mut terrain_query: Query<(&TerrainConfig,& TerrainData)>,
    
    asset_server: Res<AssetServer>,  
    mut images: ResMut<Assets<Image>>, 
    
   mut chunk_query : Query<( Entity, &Chunk,&mut ChunkData, &Parent  ) >,
){
    
     for (chunk_entity, chunk,mut chunk_data, terrain_entity ) in chunk_query.iter_mut() { 
         
        if chunk_data.height_map_image_data_load_status == TerrainImageDataLoadStatus::NotLoaded {
             
                
                
                let height_map_image:&Image = match  &chunk_data.height_map_image_handle {
                    Some(height_map_handle) => {
                        
                        let height_map_loaded = asset_server.get_load_state( height_map_handle )  ;
                    
                        if height_map_loaded != Some(LoadState::Loaded)  {
                            println!("height map not yet loaded");
                            continue;
                        }  
                        
                        images.get(height_map_handle).unwrap()
                    }
                    None => {continue} 
                };
                    
                    //maybe we can do this in a thread since it is quite cpu intense ? 
                let loaded_heightmap_data_result =  HeightMapU16::load_from_image( height_map_image) ;
                   
                      println!("built heightmapu16 ");  //this is working !! !
                      
                match loaded_heightmap_data_result {
                    Ok( loaded_heightmap_data ) => {
                       
                           //take out of box 
                            chunk_data.height_map_data = Some( *loaded_heightmap_data ); 
                 
                    },
                    Err(e) => {
                        
                        println!("{}",e);
                    }
                    
                }
                
                let alpha_mask_image:Image = build_alpha_mask_image( height_map_image );
                chunk_data.alpha_mask_image_handle = Some(images.add(  alpha_mask_image   ));
                   
            
               
                //we can let go of the height map image handle now that we loaded our heightmap data from it 
                //terrain_data.height_map_image_handle = None;
                chunk_data.height_map_image_data_load_status = TerrainImageDataLoadStatus::Loaded;
                 
            
        }
        
     }
}



pub fn build_alpha_mask_image( height_map_image: &Image ) -> Image {
     
    
    let width = height_map_image.size().x as usize;
    let height = height_map_image.size().y as usize;
    
    const THRESHOLD: u16 = (0.05 * 65535.0) as u16;
    
    // With the format being R16Uint, each pixel is represented by 2 bytes
    let mut modified_data = Vec::with_capacity(height * width * 2);  // 2 bytes per pixel
    
    for y in 0..height {
        for x in 0..width {
            let index = 2 * (y * width + x); // 2 because of R16Uint
            let height_value = u16::from_le_bytes([height_map_image.data[index], height_map_image.data[index + 1]]);
            
            let pixel_value:f32 = if height_value > THRESHOLD {
                1.0
            } else {
                0.0
            };
            modified_data.extend_from_slice(&pixel_value.to_le_bytes());
        }
    }

    // Assuming Image has a method from_data for creating an instance from raw data
  
    
    Image::new(
        height_map_image.texture_descriptor.size, 
        height_map_image.texture_descriptor.dimension, 
        modified_data,
        TextureFormat::R32Float)
    
   
}




//consider building a custom loader for this , not  Image 
/*
pub fn build_chunk_material( 
    mut chunk_query: Query<( &Chunk, &mut ChunkData, &Parent ) >,
       terrain_query: Query<(&TerrainConfig,& TerrainData)>,
     
    asset_server: Res<AssetServer>,  
    mut images: ResMut<Assets<Image>>  , 
    
    mut materials: ResMut<Assets<TerrainMaterial>>,
){
    
     for (  chunk,mut chunk_data, terrain_entity ) in chunk_query.iter_mut() { 
            
            if chunk_data.material_handle.is_some(){continue;} 
            
          let terrain_entity_id = terrain_entity.get();   
        if terrain_query.get(terrain_entity_id).is_ok() == false {continue;}
        let (terrain_config,terrain_data)  = terrain_query.get( terrain_entity_id ).unwrap();
                
                    
                    
            chunk_data.material_handle = Some(  materials.add(
                        TerrainMaterial {
                                uniforms: ChunkMaterialUniforms{
                                     color_texture_expansion_factor: 4.0,   //makes it look less tiley when LOWER  
                                     chunk_uv: Vec4::new( 0.0,1.0,0.0,1.0 ),
                                },
                               
                                array_texture:  terrain_data.texture_image_handle.clone(),
                                splat_texture:  chunk_data.splat_image_handle.clone(),
                                alpha_mask_texture: chunk_data.alpha_mask_image_handle.clone() 
                            }
                    ) ); 
                    
        
    }
}*/
/*

On initialization of terrain entity, the chunk entities should be spawned and they should just remain there forever !!! 
 */ 
 
 //this may lag.. 
pub fn build_chunk_meshes(
   mut commands: Commands,
   terrain_query: Query<(&TerrainConfig,& TerrainData)>,
    
   mut chunk_query : Query<( Entity, &Chunk,&mut ChunkData, &Parent, &Visibility  ) >,
){
    
     for (chunk_entity, chunk,mut chunk_data, terrain_entity , visibility) in chunk_query.iter_mut() { 
         
        if chunk_data.chunk_state == ChunkState::Init {
             
             
             
        let terrain_entity_id = terrain_entity.get();   
        if terrain_query.get (terrain_entity_id).is_ok() == false {continue;}
        let (terrain_config,terrain_data)  = terrain_query.get( terrain_entity_id ).unwrap();
                
                
             
             
     //   let height_map_image:&Image = images.get(height_map_handle).unwrap(); 
        
        
        let height_map_data =  &chunk_data.height_map_data .clone();
              
        if height_map_data.is_none() {
            println!("chunk is missing height map data .");
            continue; 
        }
        
        if  chunk_data.alpha_mask_image_handle.is_none() {
            println!("chunk is missing alpha_mask_image_handle .");
            continue; 
        }
              
        if  chunk_data.splat_image_handle.is_none() {
            println!("chunk is missing splat_image_handle .");
            continue; 
        }
              
        if visibility != Visibility::Visible {
            
            //do not do the intensive calculations to build a chunk mesh until it is 'visible' -- this speeds up initial map loading 
           continue;  
        }
             
             
             
             chunk_data.chunk_state = ChunkState::Building;
           
                       
            let thread_pool = AsyncComputeTaskPool::get();
        
       
              
                
              let chunk_rows = terrain_config.chunk_rows;
              let terrain_dimensions = terrain_config.terrain_dimensions;
              let height_scale = terrain_config.height_scale;
                
               //build the meshes !!!
          //    let chunk_coords = ChunkCoords::from_chunk_id(chunk.chunk_id.clone(), chunk_rows);
          //    let chunk_dimensions = terrain_config.get_chunk_dimensions(  );
                  
           //   let chunk_location_offset:Vec3 = chunk_coords.get_location_offset( chunk_dimensions ) ; 
               
             // let height_map_subsection_pct = chunk_coords.get_heightmap_subsection_bounds_pct(chunk_rows);
               
               let height_map_subsection_pct =  [[0.0,0.0],[1.0,1.0]]; //we use this now since each height map represents its entire chunks topology
               
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
                                    
               let chunk_id_clone = chunk.chunk_id.clone();
               
               let task = thread_pool.spawn(async move {
                    
                    
                    /*
                    
                    This could be optimized by not passing in the ENTIRE height map data cloned but only a subsection for this chunk... 
                    
                    */ 
                    
                    let sub_heightmap = SubHeightMapU16::from_heightmap_u16(
                        &height_map_data_cloned, 
                        height_map_subsection_pct
                    );
                    
                    
                    println!("build premesh 1 ");
                    let mesh = PreMesh::from_heightmap_subsection( 
                         &sub_heightmap, 
                         
                          height_scale,
                          lod_level,  
                       
                        
                        [terrain_dimensions.x, terrain_dimensions.y]
                    ).build(); 
                    
                            println!("build premesh 2 ");
                            
                     BuiltChunkMeshData {
                         chunk_entity_id: chunk_entity.clone(), 
                         //chunk_id: chunk_id_clone,
                        // chunk_bounds: [sub_heightmap.start_bound,sub_heightmap.end_bound],
                         
                        // chunk_location_offset: chunk_location_offset.clone(),
                         
                         //terrain_entity_id: terrain_entity.get().clone(),
                         mesh,
                         chunk_uv,
                         lod_level 
                     }
                });

                // Spawn new entity and add our new task as a component
                commands.spawn(MeshBuilderTask(task));
                
                //add the mesh builder component to the chunk entity 
               // commands.get_entity( chunk_entity ).unwrap().insert(  MeshBuilderTask(task)  );
                
           // }                
        } 
              
             
             
             
             
    } 
         
       
    
}
 
 
 
 

 
 
pub fn finish_chunk_build_tasks(
    mut commands: Commands,
    mut chunk_build_tasks: Query<(Entity,  &mut MeshBuilderTask)>,  //&Chunk, &mut ChunkData, &Parent,
    
    mut chunk_query: Query<(Entity, &Chunk, &mut ChunkData, &Parent)>,  //&Chunk, &mut ChunkData, &Parent,
    
    mut meshes: ResMut<Assets<Mesh>>,
     
      terrain_query: Query<(&TerrainData,&TerrainConfig)>,
    mut terrain_materials: ResMut<Assets<TerrainMaterial>>,
    
    mut chunk_events: EventWriter<ChunkEvent> ,
) {
    
    //chunk, mut chunk_data,  terrain_entity,
    
    for (entity, mut task) in &mut chunk_build_tasks {
        println!("for chunk build task 1");
        if let Some(built_chunk_mesh_data) = future::block_on(future::poll_once(&mut task.0)) {
            // Add our new PbrBundle of components to our tagged entity
                 println!("for chunk build task 2");
            let chunk_entity_id = built_chunk_mesh_data.chunk_entity_id;
            
            if chunk_query.get_mut(chunk_entity_id).is_ok() == false {continue;}
             let (chunk_entity, chunk,mut chunk_data,terrain_entity) = chunk_query.get_mut(chunk_entity_id).unwrap(); 
           
            let terrain_entity_id = terrain_entity.get();
               
            let chunk_uv = built_chunk_mesh_data.chunk_uv;
            let mesh = built_chunk_mesh_data.mesh; 
            
            let chunk_id = chunk.chunk_id;
         //   let chunk_location_offset = built_chunk_mesh_data.chunk_location_offset;
            
            //careful w this unwrap
           if terrain_query.get(terrain_entity_id).is_ok() == false {continue;}
            
            let (terrain_data,terrain_config) = terrain_query.get (terrain_entity_id).unwrap(); 
               
             
             //need to load these for sure !!! 
             println!("  finish_chunk_build_tasks  ");
             let array_texture =  terrain_data.get_array_texture_image().clone();
             
             let splat_texture =  chunk_data.get_splat_texture_image().clone();
             let alpha_mask_texture =  chunk_data.get_alpha_mask_texture_image().clone();
                                        
                                        
            // if terrain_data.chunks.get_mut( &chunk_id ).is_some() == false {continue;}      
               //careful w unwrap!!! 
            // let chunk_data = &mut terrain_data.chunks.get_mut( &chunk_id ).unwrap();
                                        
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
                
                
                
            let chunk_coords  = ChunkCoords::from_chunk_id(chunk_id, terrain_config.chunk_rows);// [ chunk_id / terrain_config.chunk_rows ,  chunk_id  % terrain_config.chunk_rows]; 
            let chunk_dimensions = terrain_config.get_chunk_dimensions();
                 
            
             let mesh_bundle = commands.spawn( 
                 TerrainPbrBundle {
                        mesh: terrain_mesh_handle,
                        material: chunk_terrain_material ,
                        transform: Transform::from_xyz( 
                            0.0,
                            0.0,
                            0.0 
                          
                            ),
                      //  visibility: Visibility::Hidden
                        ..default()
                        }  
                        
              ).id();
            
              let mut chunk_entity_commands  = commands.get_entity(chunk_entity_id).unwrap();
              chunk_entity_commands.add_child ( mesh_bundle );
            
              chunk_data.chunk_state = ChunkState::FullyBuilt; 
               
            
           commands.entity(entity).despawn();
         
        }
    }
}
 
 
 
pub fn update_chunk_visibility(
     terrain_query: Query<(&TerrainConfig,& TerrainData)>,
    
   mut chunk_query: Query<( &Chunk,&mut ChunkData, &Parent,  &GlobalTransform, &mut Visibility ) >,   //only will find chunks that have the meshmaterial on them 
    
   terrain_viewer: Query<&GlobalTransform, With<TerrainViewer>> 
){
    
    let viewer  = terrain_viewer.get_single();
        
    let viewer_location:Vec3 = match viewer {
        Ok(view) => { view.translation() },
        // FIX: probably should log a warning if there are multiple (or no) viewers, rather than just setting to the origin
        Err(_e) => Vec3::new(0.0,0.0,0.0)
    };
         
    for (chunk,mut chunk_data, parent_entity,  chunk_transform, mut chunk_visibility) in chunk_query.iter_mut() { 
        
       
         if  let Ok ((terrain_config, terrain_data)) = terrain_query.get(parent_entity.get()) {
              
                
            //  let render_distance_chunks:i32  = terrain_config.get_chunk_render_distance() as i32 ; //make based on render dist 
              let lod_level_distance:f32 = terrain_config.get_chunk_lod_distance(); 
              let lod_level_offset:u8 = terrain_config.lod_level_offset;
              
                    
 
                        //calc chunk world loc and use to calc the lod 
                        let chunk_world_location = chunk_transform.translation();
 
    
                         let distance_to_chunk:   f32    =  chunk_world_location.distance( viewer_location )   ;
        
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
               
                        let should_be_visible =   distance_to_chunk <= max_render_distance  ;
                                 
                                    
                                  
                        *chunk_visibility = match should_be_visible {
                            true =>  Visibility::Visible ,
                            false =>  Visibility::Hidden   
                        } ;         
                     
         
      
         } 
       }
}