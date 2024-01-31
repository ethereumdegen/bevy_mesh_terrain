

// need to finish me !!! 
// see https://github.com/ethereumdegen/bevy_terrain/blob/main/assets/shaders/advanced.wgsl 

//see bindings in terrain_material.rs 

#import bevy_pbr::forward_io VertexOutput
 


struct ChunkMaterialUniforms {
    color_texture_expansion_factor: f32 ,
    chunk_uv: vec4<f32>,  //start_x, start_y, end_x, end_y   -- used to subselect a region from the splat texture 
    
};

@group(1) @binding(0)
var<uniform> material: ChunkMaterialUniforms;
@group(1) @binding(1)
var base_color_texture: texture_2d_array<f32>;
@group(1) @binding(2)
var base_color_sampler: sampler;


//the splat map texture has 3 channels: R, G, B
//R tells us the terrain_layer_index 0 per pixel
//G tells us the terrain_layer_index 1 per pixel
//B is 0-255 mapped to 0 to 100% telling us how much of R to render versus how much of G to render 
@group(1) @binding(3)
 var splat_map_texture: texture_2d<f32>; 
//var splat_map_texture: texture_2d_array<f32>; //these are control maps and there will be 4 
@group(1) @binding(4)
var splat_map_sampler: sampler;

//works similar to splat mask  -- we use a separate tex for this for NOW to make collision mesh building far easier (only need height map and not splat)
@group(1) @binding(5)
var alpha_mask_texture: texture_2d<f32>; 
@group(1) @binding(6)
var alpha_mask_sampler: sampler;
 


//should consider adding vertex painting to this .. need another binding of course.. performs a color shift 

@fragment
fn fragment(
    mesh: VertexOutput,
) -> @location(0) vec4<f32> {
    
   
   // let tiled_uv = material.color_texture_expansion_factor*mesh.uv;  //cannot get this binding to work !? 
    let tiled_uv = 4.0*mesh.uv;
    
    
    // seems to be working !! yay ! makes our splat texture encompass all of the chunks 
    let splat_uv = material.chunk_uv.xy + mesh.uv * (material.chunk_uv.zw - material.chunk_uv.xy);
    
    let splat_values = textureSample(splat_map_texture, splat_map_sampler, splat_uv );
    let alpha_mask_value = textureSample(alpha_mask_texture, alpha_mask_sampler, splat_uv );  //comes from height map atm but COULD come from splat map now 
    
       //comes from the  control map .. float -> integer 
    let terrain_layer_index_0 = i32( splat_values.r * 255.0 );     ///* 255.0
    let terrain_layer_index_1 = i32( splat_values.g * 255.0 );
    
    //this technique lets us use 255 total textures BUT we can only layer 2 at a time.  
    let color_from_texture_0 = textureSample(base_color_texture, base_color_sampler, tiled_uv, terrain_layer_index_0);
    let color_from_texture_1 = textureSample(base_color_texture, base_color_sampler, tiled_uv, terrain_layer_index_1);
    

    let blend_amount = splat_values.b;  //comes from B channel -- this pixel 
      
    

    let blended_color = color_from_texture_0 * (1.0 - blend_amount) +
                        color_from_texture_1 * (blend_amount)  ;

    let final_color = vec4(blended_color.rgb, alpha_mask_value.r);
      
    
    return final_color;
    
}