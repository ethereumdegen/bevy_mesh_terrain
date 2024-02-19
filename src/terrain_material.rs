use bevy::asset::VisitAssetDependencies;
use bevy::prelude::*;
use bevy::reflect::{TypePath, TypeUuid};
use bevy::render::render_resource::*;

use bevy::render::render_asset::RenderAssets;

use bevy::pbr::StandardMaterialUniform;
use bevy::pbr::StandardMaterialFlags;

use bevy::pbr::MaterialExtension;



/*


This is where we set up all of our pipeline bindings

reference:
https://github.com/bevyengine/bevy/blob/main/assets/shaders/custom_material.wgsl



*/

#[derive(Clone, ShaderType,Default,Debug)]
pub struct ChunkMaterialUniforms {
    pub color_texture_expansion_factor: f32,
    pub chunk_uv: Vec4, //start_x, start_y, end_x, end_y   -- used to subselect a region from the splat texture
}
 

#[derive(Asset, AsBindGroup, TypePath, Debug, Clone,Default )]
pub struct TerrainMaterial {
  
     
    #[uniform(20)]
    pub uniforms: ChunkMaterialUniforms,
  
    
    #[texture(21, dimension = "2d_array")]
    #[sampler(22)]
    pub array_texture: Option<Handle<Image>>,

  
    #[texture(23)]
    #[sampler(24)]
    pub splat_texture: Option<Handle<Image>>,

   
    #[texture(25)]
    #[sampler(26)]
    pub alpha_mask_texture: Option<Handle<Image>>,
}



impl MaterialExtension for TerrainMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/terrain.wgsl".into()
    }

    fn deferred_fragment_shader() -> ShaderRef {
        "shaders/terrain.wgsl".into()
    }
}
 

/*
impl Material for TerrainMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/terrain.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Mask(0.1)
    }
}

impl Asset for TerrainMaterial {}

impl VisitAssetDependencies for TerrainMaterial {
    fn visit_dependencies(&self, visit: &mut impl FnMut(bevy::asset::UntypedAssetId)) {
        //what to do here ?
    }
}


 */
 
 