use bevy::asset::VisitAssetDependencies;
use bevy::prelude::*;
use bevy::reflect::TypePath;
use bevy::render::render_resource::*;

use bevy::render::render_asset::RenderAssets;

use bevy::pbr::StandardMaterialFlags;
use bevy::pbr::StandardMaterialUniform;

use bevy::pbr::MaterialExtension;

pub const TERRAIN_SHADER_HANDLE: Handle<Shader> = Handle::weak_from_u128(5433284082028047579);

 

#[derive(Clone, ShaderType, Default, Debug)]
pub struct ChunkMaterialUniforms {
    pub color_texture_expansion_factor: f32,
    pub chunk_uv: Vec4, //start_x, start_y, end_x, end_y   -- used to subselect a region from the splat texture
}

#[derive(Clone, ShaderType, Default, Debug)]
pub struct ToolPreviewUniforms {
    pub tool_coordinates: Vec2,
    pub tool_radius: f32,
    pub tool_color: Vec3,
}

#[derive(Asset, AsBindGroup, TypePath, Clone, Debug, Default)]
pub struct TerrainMaterial {
    #[uniform(20)]
    pub chunk_uniforms: ChunkMaterialUniforms,

    #[uniform(21)]
    pub tool_preview_uniforms: ToolPreviewUniforms,

    #[texture(22, dimension = "2d_array")]
    #[sampler(23)]
    pub diffuse_texture: Option<Handle<Image>>,

    #[texture(24, dimension = "2d_array")]
    #[sampler(25)]
    pub normal_texture: Option<Handle<Image>>,

    #[texture(26, dimension = "2d_array" )]
    #[sampler(27  )]
    pub blend_height_texture: Option<Handle<Image>>,




    #[texture(30, dimension = "2d",sample_type = "u_int")] //rgba8uint
    #[sampler(31 , sampler_type = "non_filtering")]
    pub splat_index_map_texture: Option<Handle<Image>>,

    
    #[texture(32, dimension = "2d",sample_type = "u_int")]
    #[sampler(33 , sampler_type = "non_filtering")]
    pub splat_strength_map_texture: Option<Handle<Image>>,

 


    #[texture(34, dimension = "2d",sample_type = "u_int")]  //rgba8uint
    #[sampler(35 , sampler_type = "non_filtering")]
    pub height_map_texture: Option<Handle<Image>>,

    // not used ? 
    #[texture(36)]
    #[sampler(37)]
    pub vertex_color_tint_texture: Option<Handle<Image>>,


    #[texture(38)]
    #[sampler(39)]
    pub hsv_noise_texture: Option<Handle<Image>>,


}

impl MaterialExtension for TerrainMaterial {
    fn fragment_shader() -> ShaderRef {
        ShaderRef::Handle(TERRAIN_SHADER_HANDLE)
    }

    fn deferred_fragment_shader() -> ShaderRef {
        ShaderRef::Handle(TERRAIN_SHADER_HANDLE)
    }
}


 