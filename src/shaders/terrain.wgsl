 
//see bindings in terrain_material.rs 
 
 //https://github.com/nicopap/bevy_mod_paramap/blob/main/src/parallax_map.wgsl



 #import bevy_pbr::{
    forward_io::{VertexOutput, FragmentOutput},
      mesh_view_bindings::view,

      pbr_bindings,
    
    pbr_fragment::pbr_input_from_standard_material,
      pbr_functions::{alpha_discard,calculate_tbn_mikktspace,apply_pbr_lighting, main_pass_post_lighting_processing,
      prepare_world_normal,
      apply_normal_mapping,
      calculate_view

      },
    // we can optionally modify the lit color before post-processing is applied
    pbr_types::{STANDARD_MATERIAL_FLAGS_DOUBLE_SIDED_BIT,STANDARD_MATERIAL_FLAGS_UNLIT_BIT},
}

#import bevy_core_pipeline::tonemapping::tone_mapping
  
struct StandardMaterial {
    time: f32,
    base_color: vec4<f32>,
    emissive: vec4<f32>,
    perceptual_roughness: f32,
    metallic: f32,
    reflectance: f32,
    // 'flags' is a bit field indicating various options. u32 is 32 bits so we have up to 32 options.
    flags: u32,
    alpha_cutoff: f32,
};


struct ChunkMaterialUniforms {
    color_texture_expansion_factor: f32 ,
    chunk_uv: vec4<f32>,  //start_x, start_y, end_x, end_y   -- used to subselect a region from the splat texture 
    
};


struct ToolPreviewUniforms { 
    tool_coordinates: vec2<f32>,
    tool_radius: f32,
    tool_color: vec3<f32>    
};

//https://github.com/DGriffin91/bevy_mod_standard_material/blob/main/assets/shaders/pbr.wgsl


@group(1) @binding(1)
var base_color_texture1: texture_2d<f32>;
@group(1) @binding(2)
var base_color_sampler1: sampler;
 

@group(1) @binding(3)
var emissive_texture: texture_2d<f32>;
@group(1) @binding(4)
var emissive_sampler: sampler;

@group(1) @binding(5)
var metallic_roughness_texture: texture_2d<f32>;
@group(1) @binding(6)
var metallic_roughness_sampler: sampler;

@group(1) @binding(7)
var occlusion_texture: texture_2d<f32>;
@group(1) @binding(8)
var occlusion_sampler: sampler;


@group(2) @binding(20)
var<uniform> chunk_uniforms: ChunkMaterialUniforms;

@group(2) @binding(21)
var<uniform> tool_preview_uniforms: ToolPreviewUniforms;

@group(2) @binding(22)
var base_color_texture: texture_2d_array<f32>;
@group(2) @binding(23)
var base_color_sampler: sampler;

@group(2) @binding(24)
var normal_texture: texture_2d_array<f32>;
@group(2) @binding(25)
var normal_sampler: sampler;





// see hypersplat.rs 
@group(2) @binding(26)
var splat_index_map_texture: texture_2d<u32>; 
 @group(2) @binding(27)
var splat_index_map_sampler: sampler;

@group(2) @binding(28)
var splat_strength_map_texture: texture_2d<u32>; 
 @group(2) @binding(29)
var splat_strength_map_sampler: sampler;



// we use a separate tex for this for NOW to make collision mesh building far easier (only need height map and not splat)
@group(2) @binding(30)
var height_map_texture: texture_2d<u32>; 
@group(2) @binding(31)
var height_map_sampler: sampler;
  
 

//should consider adding vertex painting to this .. need another binding of course.. performs a color shift 
// this could be used for baked shadows !! THIS IS PROB HOW TIRISFALL GLADES WORKS 
@group(2) @binding(32)
var vertex_color_tint_texture: texture_2d<f32>; 
@group(2) @binding(33)
var vertex_color_tint_sampler: sampler;



@fragment
fn fragment(
    mesh: VertexOutput,
    
     
    @builtin(front_facing) is_front: bool,
) -> @location(0) vec4<f32> {
    
   
    //let tiled_uv = chunk_uniforms.color_texture_expansion_factor*mesh.uv;  //cannot get this binding to work !? 
    let tiled_uv =  chunk_uniforms.color_texture_expansion_factor*mesh.uv;
    
    
    // seems to be working !! yay ! makes our splat texture encompass all of the chunks 
    let splat_uv = chunk_uniforms.chunk_uv.xy + mesh.uv * (chunk_uniforms.chunk_uv.zw - chunk_uniforms.chunk_uv.xy);
    
    //let splat_values = textureSample(splat_map_texture, splat_map_sampler, splat_uv );
        





    let height_map_texture_dimensions = textureDimensions(height_map_texture);
   
    let height_map_sample_coord  = vec2<i32>(
        i32(splat_uv.x * f32(height_map_texture_dimensions.x)),
        i32(splat_uv.y * f32(height_map_texture_dimensions.y))
    );

          
    var height_map_value: u32  = textureLoad(height_map_texture,   vec2<i32>(  height_map_sample_coord  ) , 0 ).r;







    let splat_map_texture_dimensions = textureDimensions(splat_index_map_texture);
   
    let splat_map_sample_coord  = vec2<i32>(
        i32(splat_uv.x * f32(splat_map_texture_dimensions.x)),
        i32(splat_uv.y * f32(splat_map_texture_dimensions.y))
    );
 

     let splat_index_values_at_pixel :vec4<u32> = textureLoad(splat_index_map_texture,   vec2<i32>(  splat_map_sample_coord  ) , 0 ).rgba;

     let splat_strength_values_at_pixel :vec4<u32> = textureLoad(splat_strength_map_texture,   vec2<i32>(  splat_map_sample_coord  ) , 0 ).rgba;
     //let splat_strength_values_at_pixel :vec4<f32> = textureSample(splat_strength_map_texture, splat_strength_map_sampler, splat_uv ).rgba;






    //let alpha_mask_value = textureSample(alpha_mask_texture, alpha_mask_sampler, splat_uv );  //comes from height map atm but COULD come from splat map now 
    
       //comes from the  control map .. float -> integer 
    //let terrain_layer_index_0 = i32( splat_values.r * 255.0 );     ///* 255.0
    //let terrain_layer_index_1 = i32( splat_values.g * 255.0 );

 

    // Initialize an array to store the splat strength values and the index values
    var splat_strength_array: array<u32, 4> = array<u32, 4>(splat_strength_values_at_pixel.x  , splat_strength_values_at_pixel.y, splat_strength_values_at_pixel.z , splat_strength_values_at_pixel.w );
    var splat_index_array: array<u32, 4> = array<u32, 4>(splat_index_values_at_pixel.x, splat_index_values_at_pixel.y, splat_index_values_at_pixel.z, splat_index_values_at_pixel.w);


  

      // Initialize texture_layers_used and blended color
    

    var blended_color: vec4<f32> = vec4<f32>(0.0);
    var blended_normal: vec4<f32> = vec4<f32>(0.0);


    // Loop through each layer (max 4 layers)
    for (var i: u32 = 0u; i < 4u; i = i + 1u) {

        //if there is only a base layer, it is always full strength. This allows for better blends so the base layer can be low strength (1).
       

        var splat_strength = splat_strength_array[i];
        /*if (texture_layers_used == 1u) {
            splat_strength = 255u;
        } else {
            splat_strength = splat_strength_array[i];
        }*/


        if (splat_strength > 0 ) {   
           // texture_layers_used += 1u;
            let splat_strength_float =  f32(splat_strength) / 255.0 ;

            // Look up the terrain layer index and sample the corresponding texture
            let terrain_layer_index =  i32(splat_index_array[i]);
            let color_from_diffuse = textureSample(base_color_texture, base_color_sampler, tiled_uv, terrain_layer_index);
            let color_from_normal = textureSample(normal_texture, normal_sampler, tiled_uv, terrain_layer_index);
            
            if i == 0u {
                blended_color = color_from_diffuse;
                blended_normal = color_from_normal;
            }else {
              
                blended_color = mix( blended_color, color_from_diffuse, splat_strength_float  );
                blended_normal = mix( blended_normal, color_from_normal, splat_strength_float  );
            }
            // Accumulate the blended color based on splat strength
            //blended_color += color_from_diffuse * splat_strength_float;
            //blended_normal += color_from_normal * splat_strength_float;
        }
    }

    // Normalize by dividing by the number of active layers if there are any
    //this is too jarring ! 
    /*if (texture_layers_used > 1u) {
        blended_color /= f32(texture_layers_used);
        blended_normal /= f32(texture_layers_used);
    }*/

    

    blended_normal = normalize(blended_normal); 
                    
   let blended_normal_vec3 = vec3<f32>( blended_normal.r, blended_normal.g, blended_normal.b );         
   
  // generate a PbrInput struct from the StandardMaterial bindings
  //remove this fn to make things faster as it duplicates work in gpu .. 
    var pbr_input = pbr_input_from_standard_material(mesh, is_front);
      
    
 
    //hack the material (StandardMaterialUniform)  so the color is from the terrain splat 
    pbr_input.material.base_color =  blended_color;

    //test for now 
   // pbr_input.material.base_color   = vec4(blended_normal_vec3.r,blended_normal_vec3.g,blended_normal_vec3.b,1.0);
    
    let double_sided = (pbr_input.material.flags & STANDARD_MATERIAL_FLAGS_DOUBLE_SIDED_BIT) != 0u;



     
     
    pbr_input.world_position = mesh.world_position ;
    pbr_input.world_normal =  prepare_world_normal(
        mesh.world_normal,
        double_sided,
        is_front,
    );

// https://github.com/bevyengine/bevy/blob/main/assets/shaders/array_texture.wgsl 
    
    let tangent = normalize( blended_normal_vec3 );

    //we mix the normal with our sample so shadows are affected by the normal map ! 
    let normal_mixed = mix( normalize( mesh.world_normal ) , normalize( tangent ) , 0.7 );

   /*  let TBN = calculate_tbn_mikktspace(normalize( normal_mixed ), vec4(tangent,1.0 )  ) ;  //for anistropy ??

 

    let Nt = textureSampleBias(pbr_bindings::normal_map_texture, pbr_bindings::normal_map_sampler, mesh.uv, view.mip_bias).rgb;

    

    pbr_input.N =  apply_normal_mapping(
        pbr_input.material.flags,
       
       TBN, 
        
        double_sided,
        is_front,
        
        Nt
    );*/

    pbr_input.N  = normal_mixed;


    pbr_input.V =  calculate_view(mesh.world_position, pbr_input.is_orthographic);


    var pbr_out: FragmentOutput;
 
    
    // apply lighting
    pbr_out.color = apply_pbr_lighting(pbr_input);
    // we can optionally modify the lit color before post-processing is applied
    // out.color = out.color;
    // apply in-shader post processing (fog, alpha-premultiply, and also tonemapping, debanding if the camera is non-hdr)
    // note this does not include fullscreen postprocessing effects like bloom.
    pbr_out.color = main_pass_post_lighting_processing(pbr_input, pbr_out.color);

    pbr_out.color=  tone_mapping(pbr_out.color, view.color_grading);

    // -----

   // let shadowFactor = calculate_shadow_factor(frag_lightSpacePos);


   
    let vertex_world_psn = mesh.world_position.xz; // Assuming the vertex position is in world space

    let tool_coordinates = tool_preview_uniforms.tool_coordinates;
    let tool_radius = tool_preview_uniforms.tool_radius;
    let color_from_tool = tool_preview_uniforms.tool_color;

    let distance = length(vertex_world_psn - tool_coordinates);

    let within_tool_radius = f32(distance <= tool_radius);

    let final_color = mix(
        vec4(pbr_out.color.rgb, 1.0),
        vec4(pbr_out.color.rgb * color_from_tool, 1.0),
        within_tool_radius
    );
          

      // Implement alpha masking
    if (height_map_value < 8) { // Use your threshold value here
        discard;
    }
    
    return final_color;
    
}
 



 //mod the UV using parallax 
  // https://github.com/nicopap/bevy_mod_paramap/blob/main/src/parallax_map.wgsl

 //later ? 