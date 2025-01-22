
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

@group(2) @binding(26)
var blend_height_texture: texture_2d_array<f32>;
@group(2) @binding(27)
var blend_height_sampler: sampler;





// see hypersplat.rs 
@group(2) @binding(30)
var splat_index_map_texture: texture_2d<u32>; 
 @group(2) @binding(31)
var splat_index_map_sampler: sampler;

@group(2) @binding(32)
var splat_strength_map_texture: texture_2d<f32>; 
 @group(2) @binding(33)
var splat_strength_map_sampler: sampler;



// we use a separate tex for this for NOW to make collision mesh building far easier (only need height map and not splat)
@group(2) @binding(34)
var height_map_texture: texture_2d<u32>; 
@group(2) @binding(35)
var height_map_sampler: sampler;
  
 

//should consider adding vertex painting to this .. need another binding of course.. performs a color shift 
// this could be used for baked shadows !! THIS IS PROB HOW TIRISFALL GLADES WORKS 
@group(2) @binding(36)
var vertex_color_tint_texture: texture_2d<f32>; 
@group(2) @binding(37)
var vertex_color_tint_sampler: sampler;



@group(2) @binding(38)
var hsv_noise_texture: texture_2d<f32>; 
@group(2) @binding(39)
var hsv_noise_sampler: sampler;


const BLEND_HEIGHT_OVERRIDE_THRESHOLD:f32 = 0.8;



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





    let hsv_noise_sample = textureSample(hsv_noise_texture, hsv_noise_sampler, splat_uv  ) ;




    // ----- sample splat ----

    let splat_map_texture_dimensions = textureDimensions(splat_index_map_texture);

    let noise_distortion_amount = 2.0;
    

     //we do this to 'sloppily' sample to break up the pixelation 
    let splat_uv_noise_1 =   vec2<f32>( hsv_noise_sample .r  , hsv_noise_sample .g ) * noise_distortion_amount * 0.5  ;

    let splat_map_sample_coord_float_1 = vec2<f32>(
      splat_uv.x * f32(splat_map_texture_dimensions.x)  + splat_uv_noise_1.x - noise_distortion_amount,
      splat_uv.y * f32(splat_map_texture_dimensions.y)  + splat_uv_noise_1.y - noise_distortion_amount 
    );
    

       let clamped_splat_map_sample_coord_float_1 = clamp(
        splat_map_sample_coord_float_1,
        vec2<f32>(0.0, 0.0),
        vec2<f32>(
            f32(splat_map_texture_dimensions.x - 1), 
            f32(splat_map_texture_dimensions.y - 1)
        )
    );

    let splat_map_sample_coord  = vec2<i32>(
        i32(  clamped_splat_map_sample_coord_float_1.x    ),
        i32(  clamped_splat_map_sample_coord_float_1.y   ),
    );





     let splat_index_values_at_pixel :vec4<u32> = textureLoad(splat_index_map_texture,   vec2<i32>(  splat_map_sample_coord  ) , 0 ).rgba;


     //we do this to 'sloppily' sample to break up the pixelation AGAIN and mix :D for even more slop
    let splat_uv_noise_2 =   vec2<f32>( hsv_noise_sample .b  , hsv_noise_sample .a ) * noise_distortion_amount * 0.5  ;


    let splat_map_sample_coord_float_2 = vec2<f32>(
      splat_uv.x * f32(splat_map_texture_dimensions.x)  + splat_uv_noise_2.x - noise_distortion_amount,
      splat_uv.y * f32(splat_map_texture_dimensions.y)  + splat_uv_noise_2.y - noise_distortion_amount 
    );

    let clamped_splat_map_sample_coord_float_2 = clamp(
        splat_map_sample_coord_float_2,
        vec2<f32>(0.0, 0.0),
        vec2<f32>(
            f32(splat_map_texture_dimensions.x - 1), 
            f32(splat_map_texture_dimensions.y - 1)
        )
    );

    let splat_map_sample_coord_distorted  = vec2<i32>(
        i32(  clamped_splat_map_sample_coord_float_2.x    ),
        i32(  clamped_splat_map_sample_coord_float_2.y   ),
    );
 


      let splat_index_values_at_pixel_distorted :vec4<u32> = textureLoad(splat_index_map_texture,   vec2<i32>(  splat_map_sample_coord_distorted  ) , 0 ).rgba;

    

     let splat_strength_values_at_pixel :vec4<f32> = textureSample(splat_strength_map_texture, splat_strength_map_sampler, splat_uv ).rgba;


     // --------- 



    //let alpha_mask_value = textureSample(alpha_mask_texture, alpha_mask_sampler, splat_uv );  //comes from height map atm but COULD come from splat map now 
    
       //comes from the  control map .. float -> integer 
    //let terrain_layer_index_0 = i32( splat_values.r * 255.0 );     ///* 255.0
    //let terrain_layer_index_1 = i32( splat_values.g * 255.0 );

 

    // Initialize an array to store the splat strength values and the index values
    var splat_strength_array: array<f32, 4> = array<f32, 4>(splat_strength_values_at_pixel.x  , splat_strength_values_at_pixel.y, splat_strength_values_at_pixel.z , splat_strength_values_at_pixel.w );

    var splat_index_array: array<u32, 4> = array<u32, 4>(splat_index_values_at_pixel.x, splat_index_values_at_pixel.y, splat_index_values_at_pixel.z, splat_index_values_at_pixel.w);

    var splat_index_array_distorted: array<u32, 4> = array<u32, 4>(splat_index_values_at_pixel_distorted.x, splat_index_values_at_pixel_distorted.y, splat_index_values_at_pixel_distorted.z, splat_index_values_at_pixel_distorted.w);


  

      // Initialize texture_layers_used and blended color
    

    var blended_color: vec4<f32> = vec4<f32>(0.0);
    var blended_normal: vec4<f32> = vec4<f32>(0.0);


      
    //is this right ? 
  
    var highest_drawn_pixel_height = 0.0;



    

      let hsv_noise_amount = hsv_noise_sample.r;


    // Loop through each layer (max 4 layers)
    for (var i: u32 = 0u; i < 4u; i = i + 1u) {

        let terrain_layer_index =  i32(splat_index_array[i]);
        let terrain_layer_index_distorted = i32(splat_index_array_distorted[i]);  //if this is different than the original, we can blend !!! 


        //if there is only a base layer, it is always full strength. This allows for better blends so the base layer can be low strength (1). 


        
        var splat_strength = splat_strength_array[i];




        let blend_height_strength_f = textureSample(blend_height_texture, blend_height_sampler, tiled_uv, terrain_layer_index). r ;

         //helps us blend per-pixel based on a blend height map 
     //  let blend_height_strength :u32 = textureLoad(blend_height_texture,   vec2<i32>(  blend_height_sample_coord  ) , 0 , terrain_layer_index ).r; 

      



        
        if (splat_strength < 0.01 ) {   
         continue ;
         }
        




            // Look up the terrain layer index and sample the corresponding texture
          
            let color_from_diffuse = textureSample(base_color_texture, base_color_sampler, tiled_uv, terrain_layer_index);
            let color_from_normal = textureSample(normal_texture, normal_sampler, tiled_uv, terrain_layer_index);
            
             
            

            let color_from_diffuse_distorted = textureSample(base_color_texture, base_color_sampler, tiled_uv, terrain_layer_index_distorted);

            
            let distortion_lerp =   hsv_noise_sample.b    ; // make this lerp be noisy  
            let mixed_color_from_diffuse = mix(color_from_diffuse_distorted, color_from_diffuse , distortion_lerp ) ;


          
            var splat_strength_float =   splat_strength ;
            //from 0.0 to 1.0 


       
 

            if i == 0u {
                blended_color = mixed_color_from_diffuse;
                blended_normal = color_from_normal;
                highest_drawn_pixel_height = blend_height_strength_f;
            }else {



                //renders above 
                 if ( blend_height_strength_f > highest_drawn_pixel_height 
                    || splat_strength_float >  BLEND_HEIGHT_OVERRIDE_THRESHOLD
                 ) {

                    //if we are higher, full render 
                    highest_drawn_pixel_height = blend_height_strength_f;
                    splat_strength_float = splat_strength_float; 

                }else if ( splat_strength_float > hsv_noise_amount   ) {
  
                    //if we are rendering below , some pixels will kind of show through w noise 
                    splat_strength_float = splat_strength_float * hsv_noise_amount ; 

                }else {
                    //artificially reduce our splat strength since we are below and unlucky --  
                   splat_strength_float = splat_strength_float * hsv_noise_amount  * hsv_noise_amount ; 
                }   


                // smooth out the blending slightly 
                let splat_strength_float_noisy =   splat_strength_float  + (hsv_noise_amount * 0.2 )  - 0.1  ;

                  // Accumulate the blended color based on splat strength 
                blended_color = mix( blended_color, mixed_color_from_diffuse,  splat_strength_float_noisy );
                 blended_normal = mix( blended_normal, color_from_normal, splat_strength_float  );

               
            }

          //  blended_color = vec4<f32>( hsv_noise_sample.r ,0.0,0.0,1.0   );
             
        
    }

    


    

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