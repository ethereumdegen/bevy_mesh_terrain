use bevy::prelude::*;

use bevy::reflect::TypePath;
use bevy::render::render_resource::*;


#[derive(Debug, Clone, Default, ShaderType)]
struct ShowPrepassSettings {
    show_depth: u32,
    show_normals: u32,
    show_motion_vectors: u32,
    padding_1: u32,
    padding_2: u32,
}

// This shader simply loads the prepass texture and outputs it directly
#[derive(Asset, AsBindGroup, TypePath, Debug, Clone)]
pub struct ShowPrepassOutputMaterial {
    #[uniform(0)]
    settings: ShowPrepassSettings,
}


impl Default for ShowPrepassOutputMaterial{
    fn default() -> Self {
        Self {

           settings: ShowPrepassSettings { 
                    show_depth: 1,  
                    ..default()
                    }
        }
    }
}



impl Material for ShowPrepassOutputMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/show_prepass.wgsl".into()
    }

    // This needs to be transparent in order to show the scene behind the mesh
    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Blend
    }
}
