use bevy::asset::VisitAssetDependencies;
use bevy::prelude::*;
use bevy::reflect::{TypeUuid,TypePath};
use bevy::render:: render_resource::*;

/*

This is where we set up all of our pipeline bindings 

reference: 
https://github.com/bevyengine/bevy/blob/main/assets/shaders/custom_material.wgsl





*/



#[derive( Clone, ShaderType)]
pub struct ChunkMaterialUniforms {
    pub chunk_uv: Vec4 ,  //start_x, start_y, end_x, end_y   -- used to subselect a region from the splat texture 
    pub color_texture_expansion_factor: f32 
} 




#[derive(AsBindGroup, TypeUuid, TypePath, Clone)]
#[uuid = "4acc53dd-2cfd-48ba-b659-c0e1a9bc0bdb"]
pub struct TerrainMaterial {
   
    
    #[uniform(0)]
    pub uniforms: ChunkMaterialUniforms ,
   // pub chunk_uv: Vec4,
   // pub color_texture_expansion_factor: f32  ,
    
    
    #[texture(1, dimension = "2d_array")]
    #[sampler(2)]
    pub array_texture: Option<Handle<Image>>, 
    
    #[texture(3)]
    #[sampler(4)]
    pub splat_texture: Option<Handle<Image>>, 
    
    #[texture(5)]
    #[sampler(6)]
    pub alpha_mask_texture: Option<Handle<Image>>, 
    
}

impl Material for TerrainMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/terrain.wgsl".into()
    }
    
     fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Blend
    }
}
 
 
 impl Asset for TerrainMaterial {
     
     
 }
 
 impl VisitAssetDependencies for TerrainMaterial {
     
     
 }