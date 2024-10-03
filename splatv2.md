



### Splat V2


SplatV2 is a big redesign of how the 'splat' control maps data works.  It will be a multi-staged pipeline.


1. For each level, there will be a splat folder for EACH TextureLayer.  In each, there will be a grayscale image for EACH  chunk.  (many will be blank -- maybe those just wont exist?  Could do a binary tree or whatever... anyways )

2. There will be a special processing stage which will 

  he raw painted splat data will consist of a grayscale image for EACH layer    





## LOADING 

Try to build the ChunkSplatData by using the  splat folder 

If there are missing images, assume they are 'blank' , there is no data there . 