use bevy::asset::VisitAssetDependencies;
use bevy::prelude::*;
use bevy::reflect::{TypePath};
use bevy::render::render_resource::*;

use bevy::render::render_asset::RenderAssets;

use bevy::pbr::StandardMaterialUniform;
use bevy::pbr::StandardMaterialFlags;


/*


This is where we set up all of our pipeline bindings

reference:
https://github.com/bevyengine/bevy/blob/main/assets/shaders/custom_material.wgsl



*/

#[derive(Clone, ShaderType)]
pub struct ChunkMaterialUniforms {
    pub color_texture_expansion_factor: f32,
    pub chunk_uv: Vec4, //start_x, start_y, end_x, end_y   -- used to subselect a region from the splat texture
}

#[derive(Clone, Default, ShaderType)]
pub struct CustomMaterialUniform {
  //  pub time: f32,
    /// Doubles as diffuse albedo for non-metallic, specular for metallic and a mix for everything
    /// in between.
    pub base_color: Vec4,
    // Use a color for user friendliness even though we technically don't use the alpha channel
    // Might be used in the future for exposure correction in HDR
    pub emissive: Vec4,
    /// Linear perceptual roughness, clamped to [0.089, 1.0] in the shader
    /// Defaults to minimum of 0.089
    pub roughness: f32,
    /// From [0.0, 1.0], dielectric to pure metallic
    pub metallic: f32,
    /// Specular intensity for non-metals on a linear scale of [0.0, 1.0]
    /// defaults to 0.5 which is mapped to 4% reflectance in the shader
    pub reflectance: f32,
    pub flags: u32,
    /// When the alpha mode mask flag is set, any base color alpha above this cutoff means fully opaque,
    /// and any below means fully transparent.
    pub alpha_cutoff: f32,
}


#[derive(AsBindGroup,  TypePath, Clone)]   
#[uniform(0, StandardMaterialUniform)]
pub struct TerrainMaterial {
  
    //pub std_material: StandardMaterial, // no binding here bc it is bound by the derive to uniform 0 !

    #[texture(1)]
    #[sampler(2)]    
    pub base_color_texture: Option<Handle<Image>>,

    #[texture(3)]
    #[sampler(4)]   
    pub emissive_texture: Option<Handle<Image>>,

    #[texture(5)]
    #[sampler(6)] 
    pub metallic_roughness_texture: Option<Handle<Image>>,

    #[texture(7)]
    #[sampler(8)] 
    pub occlusion_texture: Option<Handle<Image>>,

   
    #[uniform(20)]
    pub uniforms: ChunkMaterialUniforms,
    // pub chunk_uv: Vec4,
    // pub color_texture_expansion_factor: f32  ,
    
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


 
 

impl AsBindGroupShaderType<StandardMaterialUniform> for TerrainMaterial {
    fn as_bind_group_shader_type(&self, images: &RenderAssets<Image>) -> StandardMaterialUniform {
        let mut flags = StandardMaterialFlags::NONE;
        /*if self.base_color_texture.is_some() {
            flags |= StandardMaterialFlags::BASE_COLOR_TEXTURE;
        }
        if self.emissive_texture.is_some() {
            flags |= StandardMaterialFlags::EMISSIVE_TEXTURE;
        }
        if self.metallic_roughness_texture.is_some() {
            flags |= StandardMaterialFlags::METALLIC_ROUGHNESS_TEXTURE;
        }
        if self.occlusion_texture.is_some() {
            flags |= StandardMaterialFlags::OCCLUSION_TEXTURE;
        }
        if self.double_sided {
            flags |= StandardMaterialFlags::DOUBLE_SIDED;
        }
        if self.unlit {
            flags |= StandardMaterialFlags::UNLIT;
        }
        if self.fog_enabled {
            flags |= StandardMaterialFlags::FOG_ENABLED;
        }
        if self.depth_map.is_some() {
            flags |= StandardMaterialFlags::DEPTH_MAP;
        }
        #[cfg(feature = "pbr_transmission_textures")]
        {
            if self.specular_transmission_texture.is_some() {
                flags |= StandardMaterialFlags::SPECULAR_TRANSMISSION_TEXTURE;
            }
            if self.thickness_texture.is_some() {
                flags |= StandardMaterialFlags::THICKNESS_TEXTURE;
            }
            if self.diffuse_transmission_texture.is_some() {
                flags |= StandardMaterialFlags::DIFFUSE_TRANSMISSION_TEXTURE;
            }
        }
     
        // NOTE: 0.5 is from the glTF default - do we want this?
        let mut alpha_cutoff = 0.5;
        match self.alpha_mode {
            AlphaMode::Opaque => flags |= StandardMaterialFlags::ALPHA_MODE_OPAQUE,
            AlphaMode::Mask(c) => {
                alpha_cutoff = c;
                flags |= StandardMaterialFlags::ALPHA_MODE_MASK;
            }
            AlphaMode::Blend => flags |= StandardMaterialFlags::ALPHA_MODE_BLEND,
            AlphaMode::Premultiplied => flags |= StandardMaterialFlags::ALPHA_MODE_PREMULTIPLIED,
            AlphaMode::Add => flags |= StandardMaterialFlags::ALPHA_MODE_ADD,
            AlphaMode::Multiply => flags |= StandardMaterialFlags::ALPHA_MODE_MULTIPLY,
        };

        if self.attenuation_distance.is_finite() {
            flags |= StandardMaterialFlags::ATTENUATION_ENABLED;
        }
*/

       /* let has_normal_map = self.normal_map_texture.is_some();
        if has_normal_map {
            let normal_map_id = self.normal_map_texture.as_ref().map(|h| h.id()).unwrap();
            if let Some(texture) = images.get(normal_map_id) {
                match texture.texture_format {
                    // All 2-component unorm formats
                    TextureFormat::Rg8Unorm
                    | TextureFormat::Rg16Unorm
                    | TextureFormat::Bc5RgUnorm
                    | TextureFormat::EacRg11Unorm => {
                        flags |= StandardMaterialFlags::TWO_COMPONENT_NORMAL_MAP;
                    }
                    _ => {}
                }
            }
            if self.flip_normal_map_y {
                flags |= StandardMaterialFlags::FLIP_NORMAL_MAP_Y;
            }
        }*/

        flags |= StandardMaterialFlags::ALPHA_MODE_MASK;
        flags |= StandardMaterialFlags::FOG_ENABLED;
        flags |= StandardMaterialFlags::DEPTH_MAP;

        flags |= StandardMaterialFlags::FLIP_NORMAL_MAP_Y;

        StandardMaterialUniform {
            flags: flags.bits() ,
             roughness: 0.9,

            // From [0.0, 1.0], dielectric to pure metallic
             metallic: 0.0,

            // Specular intensity for non-metals on a linear scale of [0.0, 1.0]
            // defaults to 0.5 which is mapped to 4% reflectance in the shader
             reflectance: 0.0,
            ..default()
        }
    }
}