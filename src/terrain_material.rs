use bevy::{ 
    prelude::*,
    reflect::{TypeUuid,TypePath},
    render:: render_resource::*  ,
};

#[derive(AsBindGroup, TypeUuid, TypePath, Clone)]
#[uuid = "4acc53dd-2cfd-48ba-b659-c0e1a9bc0bdb"]
pub struct TerrainMaterial {
    //#[texture(0, dimension = "2d_array")]
   // #[sampler(1)]
   // pub array_texture: Handle<Image>,
    
    #[uniform(0)]
    pub color: Color,
    #[texture(1, dimension = "2d_array")]
    #[sampler(2)]
    pub array_texture: Option<Handle<Image>>,
   // alpha_mode: AlphaMode,
    
    
}

impl Material for TerrainMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/terrain.wgsl".into()
    }
}

/*
 #[uniform(0)]
    color: Color,
    #[texture(1)]
    #[sampler(2)]
    color_texture: Option<Handle<Image>>,
    alpha_mode: AlphaMode
    
    
    */ 