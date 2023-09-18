use bevy::{ 
    prelude::*,
    reflect::{TypeUuid,TypePath},
    render:: render_resource::*  ,
};

/*

This is where we set up all of our pipeline bindings 

reference: 
https://github.com/bevyengine/bevy/blob/main/assets/shaders/custom_material.wgsl



*/
#[derive(AsBindGroup, TypeUuid, TypePath, Clone)]
#[uuid = "4acc53dd-2cfd-48ba-b659-c0e1a9bc0bdb"]
pub struct TerrainMaterial {
   
    
    //#[uniform(0)]
    //pub color: Color,
    
    
    #[texture(1, dimension = "2d_array")]
    #[sampler(2)]
    pub array_texture: Option<Handle<Image>>, 
    
    #[texture(3)]
    #[sampler(4)]
    pub splat_texture: Option<Handle<Image>>, 
    
}

impl Material for TerrainMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/terrain.wgsl".into()
    }
}
 