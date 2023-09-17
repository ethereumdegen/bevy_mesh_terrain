use bevy::prelude::Mesh;

use crate::heightmap::HeightMapU16;

use bevy::prelude::*;

use bevy::render::render_resource::PrimitiveTopology::TriangleList; 

pub struct PreMesh {
    vertices: Vec<[f32; 3]>,
    indices: Vec<u32>,
}

impl PreMesh {
    fn new() -> Self {
        Self {
            vertices: Vec::new(),
            indices: Vec::new(),
        }
    }

    fn add_triangle(&mut self, vertices: [[f32; 3]; 3]) {
        // Add vertices and indices
        for vertex in &vertices {
            self.vertices.push(*vertex);
        }
        let start_idx = self.vertices.len() as u32 - 3;
        self.indices.extend_from_slice(&[start_idx, start_idx + 1, start_idx + 2]);
    }
    
    
    pub fn build(self) -> Mesh{
        
            
            let mut mesh = Mesh::new( TriangleList );
           
            mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, self.vertices );
            mesh.set_indices(Some(bevy::render::mesh::Indices::U32(self.indices )));
            mesh
        
    
    
    }
}
 
 
impl From<&HeightMapU16> for PreMesh {
    
    fn from( heightmap: &HeightMapU16 ) -> Self {
        
         
            let mut store = Self::new();
        
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
        
            store
 

        
                        
    }
    
    
}
