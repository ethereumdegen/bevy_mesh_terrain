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

    fn add_triangle(&mut self, positions: [[f32; 3]; 3], uvs: [[f32; 2]; 3]) {
        // Add vertices and indices
        for psn in &positions {
         //   println!("psn {:?}", psn);
            self.positions.push(*psn);
        }
        let start_idx = self.positions.len() as u32 - 3;
        self.indices.extend (&[start_idx, start_idx + 1, start_idx + 2]);   
        
        //stubbed in for now ... 
        let normal = compute_normal(positions[0], positions[1], positions[2]);
        self.normals.extend([normal, normal, normal]);
        
        
        self.uvs.extend( uvs ) ; 
    }
    
     
    
    pub fn build(self) -> Mesh{ 
            
            let mut mesh = Mesh::new( TriangleList );
            mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, self.positions);
            mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, self.uvs);
            mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, self.normals);
            mesh.set_indices(Some(Indices::U32(self.indices)));
            mesh 
    
    }
    
    /*
    
    Could improve this so that NO MATTER WHAT lod level, the edges are never decimated at all and always full resolution.  Only decimate the middle. (interim fix for stitching) . 
    
    */
    pub fn from_heightmap_subsection( 
          heightmap: &HeightMapU16,
          lod_level: u8, // 0 is full quality, higher levels decimate the mesh 
          bounds_pct: [ [f32 ; 2]  ;2 ],   //must be between 0 and 1 
        
          ) -> Self {
        
        let mut premesh = Self::new();
          
         let step_size = 1 << lod_level; // doubles the step for each LOD level using bit shifting 
       
          
        let height_scale = 0.004; // Adjust as needed
        let width = heightmap.len() - 0;
        let height = heightmap[0].len() - 0;
          
        let start_bound = [ (width as f32 * bounds_pct[0][0]) as usize, (height as f32 * bounds_pct[0][1]) as usize  ];
        let end_bound = [ (width as f32 * bounds_pct[1][0]) as usize , (height as f32 * bounds_pct[1][1]) as usize   ];
          
        let texture_dimensions =  [ (width ) as f32 , (height ) as f32 ]; 
          
          for x in (start_bound[0]..end_bound[0]).step_by(step_size) {
            for y in (start_bound[1]..end_bound[1]).step_by(step_size) {
                
                let fx = (x - start_bound[0]) as f32;
                let fz = (y - start_bound[1]) as f32;
                
                //cant sample so we just continue 
                if  x+step_size >= width {  continue;}
                if  y+step_size >= height {  continue;}
                
    
                let lb = heightmap[x][y] as f32 * height_scale;
                let lf = heightmap[x][y + step_size] as f32 * height_scale; 
                let rb = heightmap[x + step_size][y] as f32 * height_scale;
                let rf = heightmap[x + step_size][y + step_size] as f32 * height_scale;
                
                let uv_lb = compute_uv(fx, fz, bounds_pct, texture_dimensions);
                let uv_rb = compute_uv(fx + step_size as f32, fz, bounds_pct, texture_dimensions);
                let uv_rf = compute_uv(fx + step_size as f32, fz + step_size as f32, bounds_pct, texture_dimensions);
                let uv_lf = compute_uv(fx, fz + step_size as f32, bounds_pct, texture_dimensions);
                
                let left_back = [fx, lb, fz];
                let right_back = [fx + step_size as f32, rb, fz];
                let right_front = [fx + step_size as f32, rf, fz + step_size as f32];
                let left_front = [fx, lf, fz + step_size as f32];
    
                premesh.add_triangle([left_front, right_back, left_back], [uv_lf, uv_rb, uv_lb]);
                premesh.add_triangle([right_front, right_back, left_front], [uv_rf, uv_rb, uv_lf]);
            }
        }
        
 

        
        premesh    
        
    }
    
    
}
 
 
 fn compute_normal(v0: [f32; 3], v1: [f32; 3], v2: [f32; 3]) -> [f32; 3] {
    let edge1 = [
        v1[0] - v0[0],
        v1[1] - v0[1],
        v1[2] - v0[2]
    ];
    let edge2 = [
        v2[0] - v0[0],
        v2[1] - v0[1],
        v2[2] - v0[2]
    ];

    // Cross product
    [
        edge1[1] * edge2[2] - edge1[2] * edge2[1],
        edge1[2] * edge2[0] - edge1[0] * edge2[2],
        edge1[0] * edge2[1] - edge1[1] * edge2[0]
    ]
}
 
 //is this right !!?? 
 fn compute_uv(x: f32, y: f32, bounds: [[f32; 2]; 2], texture_dimensions: [f32; 2]) -> [f32; 2] {
     
     let start_bounds_x = bounds[0][0];
     let end_bounds_x = bounds[1][0];
     
     let start_bounds_y = bounds[0][1];
     let end_bounds_y = bounds[1][1];
     
     //x and y are the origin coords 
     
    let uv_worldspace = [
        (x ) / (end_bounds_x - start_bounds_x),
        (y ) / (end_bounds_y - start_bounds_y)
    ];
    
    let uv = [
        uv_worldspace[0] / texture_dimensions[0],
        uv_worldspace[1] / texture_dimensions[1],  
        
    ];
    
   // println!("uv {:?}", uv);
     
    
    uv
}
  