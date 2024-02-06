 

1. My code to rebuild the heightmap seems to work BUT it is revealing an issue -- like the old meshes are not being destroyed or something... 



2. move the heightmap data into the chunk comp 
  a. instead of rebuilding chunks when getting far and near them,  only rebuild them when they are flagged as needing rebuild. (like from being edited) 
  b. getting far and close just HIDES and SHOWS them, doesnt rebuild the mesh or texture 
  
  
  3. lods 
  
  Consider having only 1 LOD levels and retaining them both in memory to minimzie lag 