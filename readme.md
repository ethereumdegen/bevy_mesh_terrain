
 ## Bevy Mesh Terrain


### Installing
```
cargo add bevy_mesh_terrain 
```


NOTE: Make SURE you copy the terrain.wsgl shader into your assets directory properly!  Otherwise your terrain will not render. 

 
### Description 

 A very bevy-centric terrain plugin that takes advantage of entities, components and systems as much as possible to be as easy to understand and interact with as possible. 
 
 You spawn an entity and give it the 'TerrainConfig' and 'TerrainData' components, and then the plugin systems will spawn child entities which are each of the rendered chunks. 
 In this way, it works similarly to a voxel chunking system ( a la minecraft) except using heightfields (2d) instead of voxels (3d). 
 

 ![terrain1](https://github.com/ethereumdegen/bevy_mesh_terrain/assets/6249263/cc4ed950-dd54-430f-a40e-dc1df76d303f)


  
 
 
 ## How it works 
 
 ( See examples folder )
 
 1. You load a heightfield image into bevy asset server (R16 format - single color channel of 16 bits per pixel) 
 
 2. Pass this handle into this terrain plugin so that it will generate the heightfield data (Note: you could also set the heightfield data yourself manually)
 
 3. The plugin systems automatically spawn 'chunk' entities by sampling the heightfield data.  Chunks are only built and spawned when they are near the TerrainViewer component so attach that to your main camera. 


## Texture Types 

*Height Map Texture*
The source of height ! This is a grayscale image in R16 format and if values are totally dead black then the shader will actually make alpha == 1 right there so you can blow holes in the terrain for caves and such (youll prob want to put rock doodads there to make it look nice!)

*Color Array Texture*
This is the source of the actual terrain textures. In this configuration, there are only 4 different types you get in order to improve performance of the shader and gpu memory . So for example, grass, dirt, snow, and sand. You can edit that file of course for different looks. These pop through based on the splat map..

*Splat Map Texture*
This is an RGBA texture where the intensity of each channel controls the alpha of each of the Color Array Textures. So for example if the R channel is white and the rest black, the entire terrain will be grass. This is what you would typically edit in a 'terrain editor' to paint your grass in one area, dirt in another area, snow on mountains, etc.





## Bevy versions

Currently supports bevy 0.11  

 

### Reference Material 
see https://github.com/bevyengine/bevy/blob/main/examples/shader/shader_material.rs

 
 
 
