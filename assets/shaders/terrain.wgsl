

// need to finish me !!! 
// see https://github.com/ethereumdegen/bevy_terrain/blob/main/assets/shaders/advanced.wgsl 

//see bindings in terrain_material.rs 

#import bevy_pbr::mesh_vertex_output MeshVertexOutput

struct ChunkMaterialUniforms {
    chunk_uv: vec4<f32>,  //start_x, start_y, end_x, end_y   -- used to subselect a region from the splat texture 
    color_texture_expansion_factor: f32 
};

@group(1) @binding(0)
var<uniform> material: ChunkMaterialUniforms;
@group(1) @binding(1)
var base_color_texture: texture_2d_array<f32>;
@group(1) @binding(2)
var base_color_sampler: sampler;

@group(1) @binding(3)
var splat_map_texture: texture_2d<f32>; //each chunk will need its own  ! 
@group(1) @binding(4)
var splat_map_sampler: sampler;

//use a U16 bit texture for splat mapping where each pixel acts like a bitfield..?    


@fragment
fn fragment(
    mesh: MeshVertexOutput,
) -> @location(0) vec4<f32> {
    
   
    let tiled_uv = material.color_texture_expansion_factor*mesh.uv;
    
    // seems to be working !! yay ! makes our splat texture encompass all of the chunks 
    let splat_uv = material.chunk_uv.xy + mesh.uv * (material.chunk_uv.zw - material.chunk_uv.xy);
    
    let splat_values = textureSample(splat_map_texture, splat_map_sampler, splat_uv );

    let color_from_texture_0 = textureSample(base_color_texture, base_color_sampler, tiled_uv, 0);
    let color_from_texture_1 = textureSample(base_color_texture, base_color_sampler, tiled_uv, 1);
    let color_from_texture_2 = textureSample(base_color_texture, base_color_sampler, tiled_uv, 2);
    let color_from_texture_3 = textureSample(base_color_texture, base_color_sampler, tiled_uv, 3);

    let final_color = color_from_texture_0 * splat_values.r +
                      color_from_texture_1 * splat_values.g +
                      color_from_texture_2 * splat_values.b +
                      color_from_texture_3 * splat_values.a;

    return final_color;
    
    
}