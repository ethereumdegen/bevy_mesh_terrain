
use bevy::prelude::*;



fn main() { 


}

/*


pub fn initialize_chunk_data(
    mut commands: Commands,

    asset_server: Res<AssetServer>,

    mut chunk_query: Query<(Entity, &Chunk, &Parent), Without<ChunkData>>,

    terrain_query: Query<(&TerrainConfig, &TerrainData)>,
) {
    for (chunk_entity, chunk, terrain_entity) in chunk_query.iter_mut() {
        let terrain_entity_id = terrain_entity.get();
        if terrain_query.get(terrain_entity_id).is_ok() == false {
            continue;
        }
        let (terrain_config, terrain_data) = terrain_query.get(terrain_entity_id).unwrap();

        let chunk_id = chunk.chunk_id;
        let file_name = format!("{}.png", chunk_id);

        //default_terrain/diffuse
        let height_texture_path = terrain_config.height_folder_path.join(&file_name);
        println!("loading from {}", height_texture_path.display());

        let height_map_image_handle: Handle<Image> = asset_server.load(height_texture_path);

        //default_terrain/splat
        let splat_texture_path = terrain_config.splat_folder_path.join(&file_name);
        println!("loading from {}", splat_texture_path.display());

        let splat_image_handle: Handle<Image> = asset_server.load(splat_texture_path);


       	//...
    }
}



*/