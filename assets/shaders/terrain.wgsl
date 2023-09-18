

// need to finish me !!! 
// see https://github.com/ethereumdegen/bevy_terrain/blob/main/assets/shaders/advanced.wgsl 

//see bindings in terrain_material.rs 

#import bevy_pbr::mesh_vertex_output MeshVertexOutput

struct ChunkMaterial {
    chunk_uv: vec4<f32>,  //start_x, start_y, end_x, end_y 
};

@group(1) @binding(0)
var<uniform> material: ChunkMaterial;
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
    
    // fix me up  -- seems to be ALMOST working right 
    let splat_uv = material.chunk_uv.xy + mesh.uv * (material.chunk_uv.zw - material.chunk_uv.xy);
    
    let splat_values = textureSample(splat_map_texture, splat_map_sampler, splat_uv );

    let color_from_texture_0 = textureSample(base_color_texture, base_color_sampler, mesh.uv, 0);
    let color_from_texture_1 = textureSample(base_color_texture, base_color_sampler, mesh.uv, 1);
    let color_from_texture_2 = textureSample(base_color_texture, base_color_sampler, mesh.uv, 2);
    let color_from_texture_3 = textureSample(base_color_texture, base_color_sampler, mesh.uv, 3);

    let final_color = color_from_texture_0 * splat_values.r +
                      color_from_texture_1 * splat_values.g +
                      color_from_texture_2 * splat_values.b +
                      color_from_texture_3 * splat_values.a;

    return final_color;
    
    
}