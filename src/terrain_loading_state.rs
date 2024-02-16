use bevy::prelude::*;

use crate::chunk::TerrainChunkMesh;
 
 
                
#[derive(States, Default, Clone, Eq, PartialEq, Debug, Hash)]
pub enum TerrainLoadingState {
  
     
    #[default]
    Initialized,

    Loading,
    
    Complete,
}



 /*
pub(crate) fn update_loading_state (
    mut commands:Commands,
    meshes: Res <Assets<Mesh>>,

    terrain_mesh_query: Query<   Entity , (With<TerrainChunkMesh>, Without<ColliderData>) >,

   // terrain_viewer: Query<&GlobalTransform, With<TerrainViewer>>,
    mut terrain_loading_state: ResMut<NextState<TerrainLoadingState>>,
)   {
    
    let collision_needs_building = terrain_mesh_query.iter().collect::<Vec<Entity>>().is_empty();
    
    if collision_needs_building {
        terrain_loading_state.set (TerrainLoadingState::Loading);
    }else{
          terrain_loading_state.set(TerrainLoadingState::Complete);
    }
    
}*/