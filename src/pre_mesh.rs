use bevy::log::warn;
use crate::heightmap::HeightMapU16;
use bevy::prelude::{Mesh, Vec2};
use bevy::render::mesh::Indices;
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::render_resource::PrimitiveTopology::TriangleList;

 

const THRESHOLD: u16 = (0.0001 * 65535.0) as u16;

pub struct PreMesh {
    positions: Vec<[f32; 3]>,
    uvs: Vec<[f32; 2]>,
    normals: Vec<[f32; 3]>,
    indices: Vec<u32>,
}

impl PreMesh {
    fn new() -> Self {
        Self {
            positions: Vec::new(),
            uvs: Vec::new(),
            normals: Vec::new(),
            indices: Vec::new(),
        }
    }

    fn calculate_smooth_normals(&mut self) {
        let mut vertex_normals_accum: Vec<[f32; 3]> = vec![[0.0, 0.0, 0.0]; self.positions.len()];

        // Step 1: Calculate face normals and accumulate them for each vertex
        for i in (0..self.indices.len()).step_by(3) {
            let idx0 = self.indices[i] as usize;
            let idx1 = self.indices[i + 1] as usize;
            let idx2 = self.indices[i + 2] as usize;

            let v0 = self.positions[idx0];
            let v1 = self.positions[idx1];
            let v2 = self.positions[idx2];

            let normal = compute_normal(v0, v1, v2);

            // Step 2: Accumulate normals for each vertex of the face
            for &idx in &[idx0, idx1, idx2] {
                vertex_normals_accum[idx][0] += normal[0];
                vertex_normals_accum[idx][1] += normal[1];
                vertex_normals_accum[idx][2] += normal[2];
            }
        }

        // Step 3: Normalize accumulated normals to average them
        for normal in vertex_normals_accum.iter_mut() {
            let len =
                f32::sqrt(normal[0] * normal[0] + normal[1] * normal[1] + normal[2] * normal[2]);
            if len > 0.0 {
                normal[0] /= len;
                normal[1] /= len;
                normal[2] /= len;
            }
        }

        // Step 4: Assign averaged normals to the mesh
        self.normals = vertex_normals_accum;
    }

    fn add_triangle(&mut self, positions: [[f32; 3]; 3], uvs: [[f32; 2]; 3]) {
        // Add vertices and indices
        for psn in &positions {
            //   println!("psn {:?}", psn);
            self.positions.push(*psn);
        }
        let start_idx = self.positions.len() as u32 - 3;
        self.indices
            .extend(&[start_idx, start_idx + 1, start_idx + 2]);

        //stubbed in for now ...
        // let normal = compute_normal(positions[0], positions[1], positions[2]);
        //self.normals.extend([normal, normal, normal]);

        self.uvs.extend(uvs);
    }

    pub fn build(self) -> Mesh {
        let mut mesh = Mesh::new(TriangleList, RenderAssetUsages::default());
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, self.positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, self.uvs);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, self.normals);
        mesh.insert_indices(Indices::U32(self.indices));

       // mesh.generate_tangents().unwrap(); 
        mesh
    }

    /*

    Could improve this so that NO MATTER WHAT lod level, the edges are never decimated at all and always full resolution.  Only decimate the middle. (interim fix for stitching) .

    */
    pub fn from_heightmap_subsection(
        sub_heightmap: & HeightMapU16,

        height_scale: f32,
        lod_level: u8, // 0 is full quality, higher levels decimate the mesh

        texture_dimensions: [f32; 2],
    ) -> Self {
        let mut premesh = Self::new();

        let step_size = 1 << lod_level; // doubles the step for each LOD level using bit shifting

        let height_data = &sub_heightmap ;
        

        let bounds_pct: [[f32; 2]; 2] = [[0.0, 0.0], [1.0, 1.0]]; //1.0 is the max right ?

        let sub_heightmap_height = height_data.len();
        let sub_heightmap_width = height_data[0].len();

        println!("sub_heightmap_width {}", sub_heightmap_width);
        println!("sub_heightmap_height {}", sub_heightmap_height);

        let tex_dim_x = texture_dimensions.get(0).unwrap().clone();
        let tex_dim_y = texture_dimensions.get(1).unwrap().clone();

        let width_scale = 1.0;

        //tris completely below this will be skipped
        let scaled_min_threshold = (THRESHOLD as f32) * height_scale;

        //there is a weird bug where there are gaps in betweeen each chunk ...
        for x in (0..(tex_dim_x as usize - step_size) as usize).step_by(step_size) {
         for z in (0..(tex_dim_y as usize - step_size) as usize).step_by(step_size) {
          
          //  for z in (0..(tex_dim_y as usize - step_size) as usize).step_by(step_size) {
                let fx = (x) as f32 * width_scale;
                let fz = (z) as f32 * width_scale;

                let mut sample_allowed = true;
                //cant sample so we just continue
               if x + step_size >= sub_heightmap_width as usize {
                    sample_allowed = false;
                  //  warn!("x {}", x + step_size);
                }
                if z + step_size >= sub_heightmap_height as usize {
                    sample_allowed = false;
                  //  warn!("z {}", z + step_size);
                }  

                // height data is in format [y][x] 
                // 1. flipped x and z here ... 
                // 2. 
                let (lb, lf, rb, rf) = match sample_allowed {
                    true => {
                        let lb = height_data[z][x] as f32 * height_scale;
                        let lf = height_data[z+ step_size][x ] as f32 * height_scale;
                        let rb = height_data[z][x + step_size] as f32 * height_scale;
                        let rf = height_data[z + step_size][x + step_size] as f32 * height_scale;
                        (lb, lf, rb, rf)
                    }
                    false => (0.0, 0.0, 0.0, 0.0),
                };

                //if the triangle would be completely under the threshold,
                //do not add it to the mesh at all.  This makes a hole for collision
                //since this mesh is used to generate the collider
                if lb < scaled_min_threshold
                    && lf < scaled_min_threshold
                    && rb < scaled_min_threshold
                    && rf < scaled_min_threshold
                {
                    continue;
                }

                /* let lb = height_data[x][y] as f32 * height_scale;
                let lf = height_data[x][y + step_size] as f32 * height_scale;
                let rb = height_data[x + step_size][y] as f32 * height_scale;
                let rf = height_data[x + step_size][y + step_size] as f32 * height_scale;*/

                let uv_lb = compute_uv(fx, fz, bounds_pct, texture_dimensions);
                let uv_rb = compute_uv(fx + step_size as f32, fz, bounds_pct, texture_dimensions);
                let uv_rf = compute_uv(
                    fx + step_size as f32,
                    fz + step_size as f32,
                    bounds_pct,
                    texture_dimensions,
                );
                let uv_lf = compute_uv(fx, fz + step_size as f32, bounds_pct, texture_dimensions);

                let left_back = [fx, lb, fz];
                let right_back = [fx + step_size as f32, rb, fz];
                let right_front = [fx + step_size as f32, rf, fz + step_size as f32];
                let left_front = [fx, lf, fz + step_size as f32];

                premesh.add_triangle([left_front, right_back, left_back], [uv_lf, uv_rb, uv_lb]);
                premesh.add_triangle([right_front, right_back, left_front], [uv_rf, uv_rb, uv_lf]);
            }
        }

        premesh.calculate_smooth_normals();

        premesh
    }

    pub fn from_heightmap_subsection_greedy(
        sub_heightmap: & HeightMapU16,

        height_scale: f32,
        lod_level: u8, // 0 is full quality, higher levels decimate the mesh

        texture_dimensions: [f32; 2],
    ) -> Self {
        let mut premesh = Self::new();

        let step_size = 1 << 2; // doubles the step for each LOD level using bit shifting

        let height_data = &sub_heightmap ;
        //  let start_bound: Vec<usize> = vec![0, 0];

        //   let width = texture_dimensions[0]  ;
        //   let height = texture_dimensions[1]  ;

        // let bounds_pct = sub_heightmap.bounds_pct;

        let bounds_pct: [[f32; 2]; 2] = [[0.0, 0.0], [1.0, 1.0]]; //1.0 is the max right ?

        let sub_heightmap_height = height_data.len();
        let sub_heightmap_width = height_data[0].len();

        println!("sub_heightmap_width {}", sub_heightmap_width);
        println!("sub_heightmap_height {}", sub_heightmap_height);

        let tex_dim_x = texture_dimensions.get(0).unwrap().clone();
        let tex_dim_y = texture_dimensions.get(1).unwrap().clone();

        let width_scale = 1.0;

        //tris completely below this will be skipped
        let scaled_min_threshold = (THRESHOLD as f32) * height_scale;

        let similarity_threshold = scaled_min_threshold * 2.0;

        for x in (0..(tex_dim_x as usize - step_size) as usize).step_by(step_size) {
            // let mut greedy_y_start:Option<f32> = None;
            let mut current_greedy_height: Option<f32> = None;
            let mut greedy_points_z_start: Option<f32> = None; //fx

            let fx = (x) as f32 * width_scale;

            for y in (0..(tex_dim_y as usize - step_size) as usize).step_by(step_size) {
                let at_end_of_segment = y >= tex_dim_y as usize - (step_size * 2);

                let fz = (y) as f32 * width_scale;

                let mut sample_allowed = true;
                //cant sample so we just continue
                if x + step_size >= sub_heightmap_width as usize {
                    sample_allowed = false;
                    warn!("x {}", x + step_size);
                    continue;
                }
                if y + step_size >= sub_heightmap_height as usize {
                    sample_allowed = false;
                    warn!("y {}", y + step_size);
                    continue; 
                }

                // println!( "{} {} {} {} ", x , y , x+step_size, y + step_size   );
                let (mut lb, mut lf, mut rb, mut rf) = match sample_allowed {
                    true => {
                        let lb = height_data[y][x] as f32 * height_scale;
                        let lf = height_data[y+ step_size][x ] as f32 * height_scale;
                        let rb = height_data[y][x + step_size] as f32 * height_scale;
                        let rf = height_data[y + step_size][x + step_size] as f32 * height_scale;
                        (lb, lf, rb, rf)
                    }
                    false => (0.0, 0.0, 0.0, 0.0),
                };

                //if the triangle would be completely under the threshold,
                //do not add it to the mesh at all.  This makes a hole for collision
                //since this mesh is used to generate the collider
                if lb < scaled_min_threshold
                    && lf < scaled_min_threshold
                    && rb < scaled_min_threshold
                    && rf < scaled_min_threshold
                {
                    //this does goofy things w greedy !?
                    // continue;
                }

                if let Some(greedy_height) = current_greedy_height {
                    // let mut total_step_size = step_size.clone();

                    let differences = [
                        (lb - greedy_height).abs(),
                        (lf - greedy_height).abs(),
                        (rf - greedy_height).abs(),
                        (rb - greedy_height).abs(),
                    ];

                    // Check if all differences are within the threshold
                    let current_points_are_similar =
                        differences.iter().all(|&diff| diff <= similarity_threshold);

                    if current_points_are_similar && !at_end_of_segment {
                        //keep going -- continue

                        continue;
                    } else {
                        //end this segment and render the triangles

                        let start_fz = greedy_points_z_start.unwrap();

                        let uv_lb = compute_uv(fx, start_fz, bounds_pct, texture_dimensions);
                        let uv_rb = compute_uv(
                            fx + step_size as f32,
                            start_fz,
                            bounds_pct,
                            texture_dimensions,
                        );
                        let uv_rf = compute_uv(
                            fx + step_size as f32,
                            start_fz + step_size as f32,
                            bounds_pct,
                            texture_dimensions,
                        );
                        let uv_lf = compute_uv(
                            fx,
                            start_fz + step_size as f32,
                            bounds_pct,
                            texture_dimensions,
                        );

                        let left_back = [fx, greedy_height, start_fz];
                        let right_back = [fx + step_size as f32, greedy_height, start_fz];
                        let right_front = [fx + step_size as f32, greedy_height, fz];
                        let left_front = [fx, greedy_height, fz];

                       
                         if greedy_height >= scaled_min_threshold 
                        {
                                   premesh.add_triangle(
                                        [left_front, right_back, left_back],
                                        [uv_lf, uv_rb, uv_lb],
                                    );
                                    premesh.add_triangle(
                                        [right_front, right_back, left_front],
                                        [uv_rf, uv_rb, uv_lf],
                                    ); 
                                        
                        }
                          greedy_points_z_start = None;
                        current_greedy_height = None;


                       

                       
                    }
                } else {
                    let differences = [
                        (lb - lf).abs(),
                        (lb - rb).abs(),
                        (lb - rf).abs(),
                        (lf - rb).abs(),
                        (lf - rf).abs(),
                        (rb - rf).abs(),
                    ];

                    // Check if all differences are within the threshold
                    let current_points_are_similar =
                        differences.iter().all(|&diff| diff <= similarity_threshold);

                    // we start the greedy meshing here
                    if current_points_are_similar && !at_end_of_segment {
                        current_greedy_height = Some((lb + lf + rb + rf) / 4.0);
                        greedy_points_z_start = Some(fz);
                        continue;
                    }
                }

                //if the 4 points heights are very similar, skip adding this triangle and instead increment the
                //greedy counter and continue .   lf and rf will be way out, the others will be saved.

                //add normal mini tris here
                let uv_lb = compute_uv(fx, fz, bounds_pct, texture_dimensions);
                let uv_rb = compute_uv(fx + step_size as f32, fz, bounds_pct, texture_dimensions);
                let uv_rf = compute_uv(
                    fx + step_size as f32,
                    fz + step_size as f32,
                    bounds_pct,
                    texture_dimensions,
                );
                let uv_lf = compute_uv(fx, fz + step_size as f32, bounds_pct, texture_dimensions);

                let left_back = [fx, lb, fz];
                let right_back = [fx + step_size as f32, rb, fz];
                let right_front = [fx + step_size as f32, rf, fz + step_size as f32];
                let left_front = [fx, lf, fz + step_size as f32];


                if lb < scaled_min_threshold
                    && lf < scaled_min_threshold
                    && rb < scaled_min_threshold
                    && rf < scaled_min_threshold
                {
                     
                     continue;
                }
                premesh.add_triangle([left_front, right_back, left_back], [uv_lf, uv_rb, uv_lb]);
                premesh.add_triangle([right_front, right_back, left_front], [uv_rf, uv_rb, uv_lf]);
            } // z loop

            //if there is still a greedy segment left over ... lets render it !
            if let Some(greedy_height) = current_greedy_height {
                if let Some(fz) = greedy_points_z_start {
                    let start_fz = greedy_points_z_start.unwrap();

                    let uv_lb = compute_uv(fx, start_fz, bounds_pct, texture_dimensions);
                    let uv_rb = compute_uv(
                        fx + step_size as f32,
                        start_fz,
                        bounds_pct,
                        texture_dimensions,
                    );
                    let uv_rf =
                        compute_uv(fx + step_size as f32, fz, bounds_pct, texture_dimensions);
                    let uv_lf = compute_uv(fx, fz, bounds_pct, texture_dimensions);

                    let left_back = [fx, greedy_height, start_fz];
                    let right_back = [fx + step_size as f32, greedy_height, start_fz];
                    let right_front = [fx + step_size as f32, greedy_height, fz];
                    let left_front = [fx, greedy_height, fz];


                    if greedy_height < scaled_min_threshold 
                        {
                             
                             continue;
                        }


                    premesh
                        .add_triangle([left_front, right_back, left_back], [uv_lf, uv_rb, uv_lb]);
                    premesh
                        .add_triangle([right_front, right_back, left_front], [uv_rf, uv_rb, uv_lf]);
                } //if
            } // x loop
        } // x loop

        premesh.calculate_smooth_normals();

        premesh
    }
}

fn compute_normal(v0: [f32; 3], v1: [f32; 3], v2: [f32; 3]) -> [f32; 3] {
    let edge1 = [v1[0] - v0[0], v1[1] - v0[1], v1[2] - v0[2]];
    let edge2 = [v2[0] - v0[0], v2[1] - v0[1], v2[2] - v0[2]];

    // Cross product
    let normal = [
        edge1[1] * edge2[2] - edge1[2] * edge2[1],
        edge1[2] * edge2[0] - edge1[0] * edge2[2],
        edge1[0] * edge2[1] - edge1[1] * edge2[0],
    ];

    // normal

    [normal[0], normal[1], normal[2]] //is this busted ?
}

//is this right !!??
fn compute_uv(x: f32, y: f32, bounds: [[f32; 2]; 2], texture_dimensions: [f32; 2]) -> [f32; 2] {
    let start_bounds_x = bounds[0][0];
    let end_bounds_x = bounds[1][0];

    let start_bounds_y = bounds[0][1];
    let end_bounds_y = bounds[1][1];

    //x and y are the origin coords

    let uv_worldspace = [
        (x) / (end_bounds_x - start_bounds_x),
        (y) / (end_bounds_y - start_bounds_y),
    ];

    let uv = [
        uv_worldspace[0] / texture_dimensions[0],
        uv_worldspace[1] / texture_dimensions[1],
    ];

    // println!("uv {:?}", uv);

    uv
}
