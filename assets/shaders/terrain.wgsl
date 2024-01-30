

// need to finish me !!! 
// see https://github.com/ethereumdegen/bevy_terrain/blob/main/assets/shaders/advanced.wgsl 

//see bindings in terrain_material.rs 

#import bevy_pbr::forward_io VertexOutput
 


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

//works similar to splat mask 
@group(1) @binding(5)
var alpha_mask_texture: texture_2d<f32>; 
@group(1) @binding(6)
var alpha_mask_sampler: sampler;
 


@fragment
fn fragment(
    mesh: VertexOutput,
) -> @location(0) vec4<f32> {
    
   
    let tiled_uv = material.color_texture_expansion_factor*mesh.uv;
    
    // seems to be working !! yay ! makes our splat texture encompass all of the chunks 
    let splat_uv = material.chunk_uv.xy + mesh.uv * (material.chunk_uv.zw - material.chunk_uv.xy);
    
    let splat_values = textureSample(splat_map_texture, splat_map_sampler, splat_uv );
    let alpha_mask_value = textureSample(alpha_mask_texture, alpha_mask_sampler, splat_uv );

    let color_from_base_texture = textureSample(base_color_texture, base_color_sampler, tiled_uv, 0);
    let color_from_texture_0 = textureSample(base_color_texture, base_color_sampler, tiled_uv, 1);
    let color_from_texture_1 = textureSample(base_color_texture, base_color_sampler, tiled_uv, 2);
    let color_from_texture_2 = textureSample(base_color_texture, base_color_sampler, tiled_uv, 3);
    let color_from_texture_3 = textureSample(base_color_texture, base_color_sampler, tiled_uv, 4);

 
    
    let splat_max =  max ( max(splat_values.r , splat_values.g ) , max ( splat_values.b , splat_values.a) );
    let base_blend_factor = 1.0 - splat_max;

    let blended_color = color_from_base_texture * base_blend_factor +
                        color_from_texture_0 * splat_values.r +
                        color_from_texture_1 * splat_values.g +
                        color_from_texture_2 * splat_values.b +
                        color_from_texture_3 * splat_values.a;

    let final_color = vec4(blended_color.rgb, alpha_mask_value.r);
      
    
    return final_color;
    
}