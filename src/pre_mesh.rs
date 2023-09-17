use bevy::prelude::Mesh;
use bevy::render::mesh::Indices;

use crate::heightmap::HeightMapU16;

use bevy::prelude::*;

use bevy::render::render_resource::PrimitiveTopology::TriangleList; 

pub struct PreMesh {
    positions: Vec<[f32; 3]>,
    uvs: Vec<[f32;2]>,
    normals: Vec<[f32;3]>,
    indices: Vec<u32>,
}

impl PreMesh {
    fn new() -> Self {
        Self {
            positions: Vec::new(),
            uvs: Vec::new(),
            normals:Vec::new(),
            indices: Vec::new(),
        }
    }

    fn add_triangle(&mut self, positions: [[f32; 3]; 3]) {
        // Add vertices and indices
        for psn in &positions {
            println!("psn {:?}", psn);
            self.positions.push(*psn);
        }
        let start_idx = self.positions.len() as u32 - 3;
        self.indices.extend (&[start_idx, start_idx + 1, start_idx + 2]);  //if 0 1 2 then may be upside down 
        
        self.normals.extend(   [ [0., 1., 0.] ,[0., 1., 0.], [0., 1., 0.]  ] );
        self.uvs.extend([  [0., 0.], [0., 0.], [0., 0.]  ]) ; 
    }
    
    
    
    
    pub fn build(self) -> Mesh{ 
            
            let mut mesh = Mesh::new( TriangleList );
            mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, self.positions);
            mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, self.uvs);
            mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, self.normals);
            mesh.set_indices(Some(Indices::U32(self.indices)));
            mesh 
    
    }
    
    pub fn from_heightmap_subsection( 
          heightmap: &HeightMapU16,
          bounds_pct: [ [f32 ; 2]  ;2 ], 
        
          ) -> Self {
        
        let mut premesh = Self::new();
          
        let height_scale = 0.001; // Adjust as needed
        let width = heightmap.len();
        let height = heightmap[0].len();
          
        let start_bound = [ (width as f32 * bounds_pct[0][0]) as usize, (height as f32 * bounds_pct[0][1]) as usize  ];
        let end_bound = [ (width as f32 * bounds_pct[1][0]) as usize, (height as f32 * bounds_pct[1][1]) as usize  ];
          
        
         
           for x in start_bound[0]..end_bound[0] - 1 {
                for y in start_bound[1]..end_bound[1] - 1 {
                    let fx = (x - start_bound[0]) as f32;
                    let fz =  (y - start_bound[1]) as f32; 
        
                    let lb = heightmap[x][y] as f32 * height_scale;
                    let lf = heightmap[x][y + 1] as f32 * height_scale;
                    let rb = heightmap[x + 1][y] as f32 * height_scale;
                    let rf = heightmap[x + 1][y + 1] as f32 * height_scale;
                    
                    println!("sampled: {} {} {} {} ", lb,lf,rb,rf);
        
                    let left_back = [fx, lb, fz];
                    let right_back = [fx + 1.0, rb, fz];
                    let right_front = [fx + 1.0, rf, fz + 1.0];
                    let left_front = [fx, lf, fz + 1.0];
        
                    premesh.add_triangle([left_front, right_back, left_back]);
                    premesh.add_triangle([right_front, right_back, left_front]);
                }
            }
            
         /*
        for x in 0..width - 1 {
        for y in 0..height - 1 {
                // Vertex indices
                let lb = x * height + y;
                let rb = (x + 1) * height + y;
                let lf = x * height + (y + 1);
                let rf = (x + 1) * height + (y + 1);
    
                // Add two triangles
             //   premesh.indices.extend(&[lb as u32, rb as u32, lf as u32]);
             //   premesh.indices.extend(&[lf as u32, rb as u32, rf as u32]);
    
                // TODO: Compute normals using cross product method and push to normals vector
            }
        }
         */
        
 

        
        premesh    
        
    }
    
    
}
 
 /*
impl From<&HeightMapU16> for PreMesh {
    
    fn from( heightmap: &HeightMapU16 ) -> Self {
       let mut premesh = Self::new();
          
        let height_scale = 0.1; // Adjust as needed
        let width = heightmap.len();
        let height = heightmap[0].len();
          
          // Calculate positions and UVs
       /* for x in 0..width {
            for y in 0..height {
                let h = heightmap[x][y] as f32 * height_scale;
                premesh.positions.push([x as f32, h, y as f32]);
                premesh.uvs.push([x as f32 / (width as f32 - 1.0), y as f32 / (height as f32 - 1.0)]);
            }
        }*/
         
           for x in 0..heightmap.len() - 1 {
                for y in 0..heightmap[x].len() - 1 {
                    let fx = x as f32;
                    let fz = y as f32; 
        
                    let lb = heightmap[x][y] as f32 * height_scale;
                    let lf = heightmap[x][y + 1] as f32 * height_scale;
                    let rb = heightmap[x + 1][y] as f32 * height_scale;
                    let rf = heightmap[x + 1][y + 1] as f32 * height_scale;
        
                    let left_back = [fx, lb, fz];
                    let right_back = [fx + 1.0, rb, fz];
                    let right_front = [fx + 1.0, rf, fz + 1.0];
                    let left_front = [fx, lf, fz + 1.0];
        
                    premesh.add_triangle([left_front, right_back, left_back]);
                    premesh.add_triangle([right_front, right_back, left_front]);
                }
            }
            
         
        for x in 0..width - 1 {
        for y in 0..height - 1 {
                // Vertex indices
                let lb = x * height + y;
                let rb = (x + 1) * height + y;
                let lf = x * height + (y + 1);
                let rf = (x + 1) * height + (y + 1);
    
                // Add two triangles
             //   premesh.indices.extend(&[lb as u32, rb as u32, lf as u32]);
             //   premesh.indices.extend(&[lf as u32, rb as u32, rf as u32]);
    
                // TODO: Compute normals using cross product method and push to normals vector
            }
        }
         
            
        /*
            for x in 0..heightmap.len() - 1 {
                for y in 0..heightmap[x].len() - 1 {
                    let fx = x as f32;
                    let fz = y as f32;
                    let height_scale = 0.1; // You can adjust this as needed
        
                    let lb = heightmap[x][y] as f32 * height_scale;
                    let lf = heightmap[x][y + 1] as f32 * height_scale;
                    let rb = heightmap[x + 1][y] as f32 * height_scale;
                    let rf = heightmap[x + 1][y + 1] as f32 * height_scale;
        
                    let left_back = [fx, lb, fz];
                    let right_back = [fx + 1.0, rb, fz];
                    let right_front = [fx + 1.0, rf, fz + 1.0];
                    let left_front = [fx, lf, fz + 1.0];
        
                    store.add_triangle([left_front, right_back, left_back]);
                    store.add_triangle([right_front, right_back, left_front]);
                }
            }
        
            store*/
 

        
        premesh    
    }
    
    
}
*/