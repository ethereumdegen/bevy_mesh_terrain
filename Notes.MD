


### Ideal Design

- The user provides a 'terrain config file' which we have a special asset loader for .  
That terrain config file could be a special ZIP file which contains a manifest and all of the textures we need for the terrain .


A custom asset loader will load that zip file, loading the manifest and all of the assets into memory and the asset server. 


Then, the camera position is monitored, perhaps with a component and the camera translation. 
From that camera position, we will spawn entities which are terrain chunks around the camera at fixed distances and these will have terrain planes on them . 

