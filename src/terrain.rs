use bevy::prelude::*;
use bevy::asset::LoadState;
use bevy::render::render_resource::{SamplerDescriptor, AddressMode, FilterMode, TextureFormat, TextureDescriptor};
use bevy::render::texture::{ImageSampler, ImageSamplerDescriptor, ImageAddressMode, ImageFilterMode};
use bevy::utils::HashMap;

use crate::terrain_material::{TerrainMaterial, ChunkMaterialUniforms};
use crate::chunk::{ChunkData, Chunk};
use crate::heightmap::{HeightMap,HeightMapU16};

use crate::terrain_config::TerrainConfig;


/*


Chunks should be more persistent

each chunk should have its own heightmap and splat map !!!  these are their own files too. 



*/


//attach me to camera 
#[derive(Component,Default)]
pub struct TerrainViewer {
    
}
 



#[derive(Default,PartialEq, Eq)]
pub enum TerrainImageDataLoadStatus { //us this for texture image and splat image and alpha mask .. ? 
    #[default]
    NotLoaded,
    Loaded,
    NeedsReload    
}

#[derive(Default,PartialEq, Eq)]
pub enum TerrainDataStatus { //us this for texture image and splat image and alpha mask .. ? 
    #[default]
    NotLoaded,
    Loaded
       
}



#[derive(Component,Default)]
pub struct TerrainData {
       
    pub chunks: HashMap<u32,ChunkData>,  //why is this necessary  ?? 
    
    pub terrain_data_status: TerrainDataStatus,
  
    
    //could be a massive image like 4k 
   // pub height_map_image_handle: Option<Handle<Image>>, 
   // pub height_map_image_data_load_status: TerrainImageDataLoadStatus,
    
    //need to add asset handles here for the heightmap image and texture image !!! 
    
     
   // pub height_map_data: Option<HeightMapU16>,
    
    
    
    texture_image_handle: Option<Handle<Image>>,
    texture_image_sections: u32, 
    texture_image_finalized: bool,  //need this for now bc of the weird way we have to load an array texture w polling and stuff... GET RID of me ???replace w enum ? 
    
   // splat_image_handle: Option<Handle<Image>>,
    
    alpha_mask_image_handle: Option<Handle<Image>>, //built from the height map 
   
    pub terrain_material_handle: Option<Handle<TerrainMaterial> >
}
 
 impl TerrainData{
     
     pub fn new( ) -> Self{
         
         let terrain_data = TerrainData::default();
         
         
         //spawn the chunks as default lil entities 
         
         
         
         
         terrain_data
     }
}
 
 pub fn initialize_terrain(  
      mut commands: Commands,
    mut terrain_query: Query<(Entity, &mut TerrainData, &TerrainConfig) >,
   
    
){ 
    
    for (terrain_entity,mut terrain_data, terrain_config) in terrain_query.iter_mut() {
        
        
        if terrain_data.terrain_data_status == TerrainDataStatus::NotLoaded {
            
            let max_chunks = terrain_config.chunk_rows *  terrain_config.chunk_rows ;
            
            for chunk_id in 0 .. max_chunks {
                let chunk_entity =  commands.spawn(
                    Chunk::new(chunk_id)
                ).id();
                
                
                let mut terrain_entity_commands  = commands.get_entity(terrain_entity).unwrap();
             
                terrain_entity_commands.add_child(chunk_entity);
                
            }
            
                   
            
            
            terrain_data.terrain_data_status = TerrainDataStatus::Loaded
        }
        
    }
        
}
 
 /*
impl TerrainData{
    
    pub fn add_height_map_image( mut self, handle: Handle<Image> ) -> Self {
        self.height_map_image_handle = Some(handle.clone()); //strong clone 
        self 
    }
    
    pub fn add_array_texture_image(mut self, handle: Handle<Image>, sections: u32  )-> Self{
        self.texture_image_handle = Some(handle.clone()); //strong clone 
        self.texture_image_sections = sections; 
        self 
    }
    
    pub fn add_splat_texture_image(mut self, handle: Handle<Image>   )-> Self{
        self.splat_image_handle = Some(handle.clone()); //strong clone 
        self 
       
    }
    
    
    pub fn get_array_texture_image(&self) -> &Option<Handle<Image>> {
        
        &self.texture_image_handle 
    }
    
    pub fn get_splat_texture_image(&self) -> &Option<Handle<Image>> {
        
        &self.splat_image_handle 
    }
    
    pub fn get_alpha_mask_texture_image(&self) -> &Option<Handle<Image>> {
        
        &self.alpha_mask_image_handle 
    }
    
}
 */
 
 
  
 
 /*
 
 //finalizes loading of height map by looking for image handle and applying it to the height map data 
pub fn load_height_map_data_from_image(  
    
    mut terrain_query: Query<&mut TerrainData, With<TerrainConfig>>,
    asset_server: Res<AssetServer>,  
    mut images: ResMut<Assets<Image>>, 
    
){ 
    
    for mut terrain_data in terrain_query.iter_mut() {
        
        
      //  let height_map_data_is_some = terrain_data.height_map_data.is_some(); 
         
         if terrain_data.height_map_image_data_load_status != TerrainImageDataLoadStatus::Loaded {
         
         //try to load the height map data from the height_map_image_handle 
       //  if !height_map_data_is_some {
                
                //try to get the loaded height map image from its handle via the asset server - must exist and be loaded 
                let height_map_image:&Image = match &terrain_data.height_map_image_handle {
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
                   
                      println!("built heightmapu16 ");
                      
                match loaded_heightmap_data_result {
                    Ok( loaded_heightmap_data ) => {
                       
                           //take out of box 
                            terrain_data.height_map_data = Some( *loaded_heightmap_data ); 
                 
                    },
                    Err(e) => {
                        
                        println!("{}",e);
                    }
                    
                }
                
                let alpha_mask_image:Image = build_alpha_mask_image( height_map_image );
                terrain_data.alpha_mask_image_handle = Some(images.add(  alpha_mask_image   ));
                   
            
               
                //we can let go of the height map image handle now that we loaded our heightmap data from it 
                //terrain_data.height_map_image_handle = None;
                terrain_data.height_map_image_data_load_status = TerrainImageDataLoadStatus::Loaded;
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
pub fn load_terrain_texture_from_image( 
    mut terrain_query: Query<&mut TerrainData, With<TerrainConfig>>,
    asset_server: Res<AssetServer>,  
    mut images: ResMut<Assets<Image>>  , 
    
    mut materials: ResMut<Assets<TerrainMaterial>>,
){
       for mut terrain_data in terrain_query.iter_mut() {
  
           let texture_image_finalized  = terrain_data.texture_image_finalized; 
         
         //try to load the height map data from the height_map_image_handle 
            if !texture_image_finalized {
                 
                let texture_image:&mut Image = match &terrain_data.texture_image_handle {
                    Some(texture_image_handle) => {
                        
                        let texture_image_loaded = asset_server.get_load_state( texture_image_handle )  ;
                    
                        if texture_image_loaded != Some(LoadState::Loaded)  {
                            println!("terrain texture not yet loaded");
                            continue;
                        }  
                        
                        images.get_mut(texture_image_handle).unwrap()
                    }
                    None => {continue} 
                };
                
                
                   //https://github.com/bevyengine/bevy/pull/10254
                   texture_image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
                        label: None,
                        address_mode_u: ImageAddressMode::Repeat,
                        address_mode_v: ImageAddressMode::Repeat,
                        address_mode_w: ImageAddressMode::Repeat,
                        mag_filter: ImageFilterMode::Linear,
                        min_filter: ImageFilterMode::Linear,
                        mipmap_filter: ImageFilterMode::Linear,
                        ..default()
                    });
                
                    // Create a new array texture asset from the loaded texture.
                    let array_layers = terrain_data.texture_image_sections;
                    
                    if  array_layers > 1 {
                         texture_image.reinterpret_stacked_2d_as_array(array_layers);
                    }
                   
                   terrain_data. texture_image_finalized = true; 
                   
                   
                   terrain_data.terrain_material_handle = Some(  materials.add(
                        TerrainMaterial {
                                uniforms: ChunkMaterialUniforms{
                                     color_texture_expansion_factor: 4.0,   //makes it look less tiley when LOWER  
                                     chunk_uv: Vec4::new( 0.0,1.0,0.0,1.0 ),
                                },
                               
                                array_texture:  terrain_data.texture_image_handle.clone(),
                                splat_texture:  terrain_data.splat_image_handle.clone(),
                                alpha_mask_texture: terrain_data.alpha_mask_image_handle.clone() 
                            }
                    ) ); 
                    
                    
                    
                 
                
            }
       }   
}

*/