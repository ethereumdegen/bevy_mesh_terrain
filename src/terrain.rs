use bevy::prelude::*;
use bevy::asset::LoadState;
use bevy::render::render_resource::{SamplerDescriptor, AddressMode, FilterMode, TextureFormat};
use bevy::render::texture::ImageSampler;
use bevy::utils::HashMap;

use crate::terrain_material::{TerrainMaterial, ChunkMaterialUniforms};
use crate::chunk::ChunkData;
use crate::heightmap::{HeightMap,HeightMapU16};

//attach me to camera 
#[derive(Component,Default)]
pub struct TerrainViewer {
    
}
 


#[derive(Component)]
pub struct TerrainConfig { 
    pub terrain_dimensions: Vec2,  
     
    pub chunk_rows: u32,
    
    pub render_distance: f32, 
    pub lod_distance: f32 ,
    
    pub lod_level_offset: u8 , 

    pub height_scale: f32,

    
    pub attach_collision_data: bool 
}

impl Default for TerrainConfig {
    fn default() -> Self {
        Self {
           // chunk_width: 64.0 ,
            terrain_dimensions: Vec2::new(1024.0,1024.0), //this should match the heightmap dimensions... consider removing this var or changing how it fundamentally works . 
            chunk_rows: 16 ,   //making this too high produces too many materials which causes lag.  Too low and we cant LOD properly . 16 seems good . 
            render_distance: 2000.0, 
            lod_distance: 1000.0 ,

            lod_level_offset: 0, 
            
             height_scale: 0.004,  //for building the mesh 
            
            attach_collision_data: true 
        }
    }
}

impl TerrainConfig {
    
     pub fn set_render_distance(mut self, distance: f32 ) -> Self {
         
         self.render_distance = distance;
         self 
     }
     
     pub fn set_lod_distance(mut self, distance: f32 ) -> Self {
         
         self.lod_distance = distance;
         self 
     }
    
     pub fn get_chunk_dimensions(&self ) -> Vec2 {
        let chunk_dimension_x = self.terrain_dimensions.x / self.chunk_rows as f32;
        let chunk_dimension_z = self.terrain_dimensions.y / self.chunk_rows as f32;
         
        
        Vec2::new(chunk_dimension_x, chunk_dimension_z) 
        
    }  
    
    pub fn get_max_render_distance(&self) -> f32{
        return self.render_distance  ; 
    }
    
    pub fn get_chunk_render_distance(&self) -> u32{
        return self.render_distance as u32 / self.chunk_rows; 
    }
     
     
     pub fn get_chunk_lod_distance(&self) -> f32{
        return self.lod_distance  ; 
    }
}

#[derive(Component,Default)]
pub struct TerrainData {
      //  pub terrain_origin: Vec3 // should be a component of an entity 
    //chunk_index -> chunk data 
    //chunk index is   chunk_col * 64  + chunk_row   IF chunk_rows is 64 
    //this only tracks loaded and active chunks and these are all entities 
    pub chunks: HashMap<u32,ChunkData>, 
  
    
    //could be a massive image like 4k 
    height_map_image_handle: Option<Handle<Image>>, 
    
    //need to add asset handles here for the heightmap image and texture image !!! 
    
     
    pub height_map_data: Option<HeightMapU16>,
    
    
    
    texture_image_handle: Option<Handle<Image>>,
    texture_image_sections: u32, 
    texture_image_finalized: bool,  //need this for now bc of the weird way we have to load an array texture w polling and stuff...
    
    splat_image_handle: Option<Handle<Image>>,
    
    alpha_mask_image_handle: Option<Handle<Image>>, //built from the height map 
   
    pub terrain_material_handle: Option<Handle<TerrainMaterial> >
}
 
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
 
 //finalizes loading of height map by looking for image handle and applying it to the height map data 
pub fn load_height_map_data_from_image(  
    
    mut terrain_query: Query<(Entity, &TerrainConfig,&mut TerrainData)>, 
    asset_server: Res<AssetServer>,  
    mut images: ResMut<Assets<Image>>, 
    
){ 
    
    for (terrain_entity, terrain_config, mut terrain_data) in terrain_query.iter_mut() { 
        
        
        let height_map_data_is_some = terrain_data.height_map_data.is_some(); 
         
         //try to load the height map data from the height_map_image_handle 
         if !height_map_data_is_some {
                
                //try to get the loaded height map image from its handle via the asset server - must exist and be loaded 
                let height_map_image:&Image = match &terrain_data.height_map_image_handle {
                    Some(height_map_handle) => {
                        
                        let height_map_loaded = asset_server.get_load_state( height_map_handle )  ;
                    
                        if height_map_loaded != LoadState::Loaded  {
                            println!("height map not yet loaded");
                            continue;
                        }  
                        
                        images.get(height_map_handle).unwrap()
                    }
                    None => {continue} 
                };
                
                let loaded_heightmap_data_result =  HeightMapU16::load_from_image( height_map_image) ;
                   
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
                terrain_data.height_map_image_handle = None;
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
    mut terrain_query: Query<(Entity, &TerrainConfig,&mut TerrainData)>, 
    asset_server: Res<AssetServer>,  
    mut images: ResMut<Assets<Image>>  , 
    
    mut materials: ResMut<Assets<TerrainMaterial>>,
){
       for (terrain_entity, terrain_config, mut terrain_data) in terrain_query.iter_mut() { 
  
           let texture_image_finalized  = terrain_data.texture_image_finalized; 
         
         //try to load the height map data from the height_map_image_handle 
            if !texture_image_finalized {
                 
                let mut texture_image:&mut Image = match &terrain_data.texture_image_handle {
                    Some(texture_image_handle) => {
                        
                        let texture_image_loaded = asset_server.get_load_state( texture_image_handle )  ;
                    
                        if texture_image_loaded != LoadState::Loaded  {
                            println!("terrain texture not yet loaded");
                            continue;
                        }  
                        
                        images.get_mut(texture_image_handle).unwrap()
                    }
                    None => {continue} 
                };
                
                
                   texture_image.sampler_descriptor = ImageSampler::Descriptor(SamplerDescriptor {
                        label: None,
                        address_mode_u: AddressMode::Repeat,
                        address_mode_v: AddressMode::Repeat,
                        address_mode_w: AddressMode::Repeat,
                        mag_filter: FilterMode::Linear,
                        min_filter: FilterMode::Linear,
                        mipmap_filter: FilterMode::Linear,
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
                                     color_texture_expansion_factor: 16.0,
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