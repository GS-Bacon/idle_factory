//! Mesh generation for chunks using greedy meshing
//!
//! Contains the mesh generation algorithms including greedy meshing optimization.

use crate::constants::*;
use crate::core::ItemId;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;

use super::chunk::{ChunkData, ChunkLod};

impl ChunkData {
    /// Generate a combined mesh for the entire chunk with face culling using greedy meshing
    /// neighbor_checker: function to check if a block exists at world position (for cross-chunk checks)
    /// lod: Level of detail - affects which blocks are included in the mesh
    pub fn generate_mesh_with_neighbors<F>(
        &self,
        chunk_coord: IVec2,
        neighbor_checker: F,
        lod: ChunkLod,
    ) -> Mesh
    where
        F: Fn(IVec3) -> bool,
    {
        let min_y = lod.min_y();
        // Pre-allocate with estimated capacity (greedy meshing produces fewer quads)
        let estimated_faces = (CHUNK_SIZE * CHUNK_SIZE) as usize;
        let mut positions: Vec<[f32; 3]> = Vec::with_capacity(estimated_faces * 4);
        let mut normals: Vec<[f32; 3]> = Vec::with_capacity(estimated_faces * 4);
        let mut uvs: Vec<[f32; 2]> = Vec::with_capacity(estimated_faces * 4);
        let mut uv_layers: Vec<[f32; 2]> = Vec::with_capacity(estimated_faces * 4); // UV_1: texture layer index
        let mut colors: Vec<[f32; 4]> = Vec::with_capacity(estimated_faces * 4);
        let mut indices: Vec<u32> = Vec::with_capacity(estimated_faces * 6);

        // Cache chunk world offset
        let chunk_world_x = (chunk_coord.x * CHUNK_SIZE) as f32;
        let chunk_world_z = (chunk_coord.y * CHUNK_SIZE) as f32;

        // Helper to check if neighbor exists
        let has_neighbor = |x: i32, y: i32, z: i32, dx: i32, dy: i32, dz: i32| -> bool {
            let nx = x + dx;
            let ny = y + dy;
            let nz = z + dz;
            if (0..CHUNK_SIZE).contains(&nx)
                && (0..CHUNK_HEIGHT).contains(&ny)
                && (0..CHUNK_SIZE).contains(&nz)
            {
                self.blocks[Self::pos_to_index(nx, ny, nz)].is_some()
            } else if !(0..CHUNK_HEIGHT).contains(&ny) {
                false
            } else {
                let world_pos = IVec3::new(
                    chunk_coord.x * CHUNK_SIZE + nx,
                    ny,
                    chunk_coord.y * CHUNK_SIZE + nz,
                );
                neighbor_checker(world_pos)
            }
        };

        // Face data: (axis, positive, axis1_size, axis2_size)
        // axis: 0=X, 1=Y, 2=Z
        // positive: true for +X/+Y/+Z, false for -X/-Y/-Z
        let face_configs: [(usize, bool); 6] = [
            (1, true),  // +Y (top)
            (1, false), // -Y (bottom)
            (0, true),  // +X (east)
            (0, false), // -X (west)
            (2, true),  // +Z (south)
            (2, false), // -Z (north)
        ];

        // Get axis sizes
        let axis_sizes = [CHUNK_SIZE, CHUNK_HEIGHT, CHUNK_SIZE];

        for (axis, positive) in face_configs {
            let (axis1, axis2) = match axis {
                0 => (1, 2), // X: slice along Y-Z
                1 => (0, 2), // Y: slice along X-Z
                2 => (0, 1), // Z: slice along X-Y
                _ => unreachable!(),
            };

            let d = if positive { 1 } else { -1 };

            // Iterate through slices perpendicular to axis
            for slice in 0..axis_sizes[axis] {
                // Create mask for this slice
                // mask[u][v] = Some(ItemId) if face is visible
                let mut mask: Vec<Vec<Option<ItemId>>> =
                    vec![vec![None; axis_sizes[axis2] as usize]; axis_sizes[axis1] as usize];

                for u in 0..axis_sizes[axis1] {
                    for v in 0..axis_sizes[axis2] {
                        let (x, y, z) = match axis {
                            0 => (slice, u, v),
                            1 => (u, slice, v),
                            2 => (u, v, slice),
                            _ => unreachable!(),
                        };

                        // LOD: Skip blocks below min_y threshold
                        if y < min_y {
                            continue;
                        }

                        if let Some(item_id) = self.get_block(x, y, z) {
                            let (dx, dy, dz) = match axis {
                                0 => (d, 0, 0),
                                1 => (0, d, 0),
                                2 => (0, 0, d),
                                _ => unreachable!(),
                            };
                            if !has_neighbor(x, y, z, dx, dy, dz) {
                                mask[u as usize][v as usize] = Some(item_id);
                            }
                        }
                    }
                }

                // Greedy merge the mask into quads
                let mut processed =
                    vec![vec![false; axis_sizes[axis2] as usize]; axis_sizes[axis1] as usize];

                for u in 0..axis_sizes[axis1] as usize {
                    for v in 0..axis_sizes[axis2] as usize {
                        if processed[u][v] || mask[u][v].is_none() {
                            continue;
                        }

                        let item_id = mask[u][v].unwrap();

                        // Greedy meshing: expand quad in v direction (width)
                        let mut width = 1;
                        while v + width < axis_sizes[axis2] as usize
                            && !processed[u][v + width]
                            && mask[u][v + width] == Some(item_id)
                        {
                            width += 1;
                        }

                        // Expand quad in u direction (height)
                        let mut height = 1;
                        'height: while u + height < axis_sizes[axis1] as usize {
                            for dv in 0..width {
                                if processed[u + height][v + dv]
                                    || mask[u + height][v + dv] != Some(item_id)
                                {
                                    break 'height;
                                }
                            }
                            height += 1;
                        }

                        // Mark all cells in the quad as processed
                        for du in 0..height {
                            for dv in 0..width {
                                processed[u + du][v + dv] = true;
                            }
                        }

                        // Generate quad
                        // For Array Texture: UV coordinates are tile-based for proper tiling
                        // UV_0: (u, v) where u,v can be > 1 for multi-block quads
                        // UV_1: (texture_layer, 0) for shader to select texture layer

                        // Get texture index based on face direction (grass has different top/side)
                        let is_top_face = axis == 1 && positive;
                        let tex_layer = item_id.texture_index_for_face(is_top_face) as f32;

                        // Keep vertex colors for tinting/debugging (white = no tint)
                        let color_arr = [1.0_f32, 1.0, 1.0, 1.0];

                        // Calculate corner positions in u-v space
                        let u0f = u as f32;
                        let v0f = v as f32;
                        let u1f = (u + height) as f32;
                        let v1f = (v + width) as f32;

                        // Generate 4 vertices in CCW order when viewed from outside
                        // The order depends on face direction to ensure correct winding
                        let (verts, normal): ([[f32; 3]; 4], [f32; 3]) = match (axis, positive) {
                            (0, true) => {
                                // +X face: looking from +X toward -X
                                // Y is up, Z is left -> CCW: (y_low,z_low), (y_high,z_low), (y_high,z_high), (y_low,z_high)
                                let x = chunk_world_x + (slice + 1) as f32;
                                (
                                    [
                                        [x, u0f, chunk_world_z + v0f],
                                        [x, u1f, chunk_world_z + v0f],
                                        [x, u1f, chunk_world_z + v1f],
                                        [x, u0f, chunk_world_z + v1f],
                                    ],
                                    [1.0, 0.0, 0.0],
                                )
                            }
                            (0, false) => {
                                // -X face: looking from -X toward +X
                                // Y is up, Z is right -> CCW: (y_low,z_low), (y_low,z_high), (y_high,z_high), (y_high,z_low)
                                let x = chunk_world_x + slice as f32;
                                (
                                    [
                                        [x, u0f, chunk_world_z + v0f],
                                        [x, u0f, chunk_world_z + v1f],
                                        [x, u1f, chunk_world_z + v1f],
                                        [x, u1f, chunk_world_z + v0f],
                                    ],
                                    [-1.0, 0.0, 0.0],
                                )
                            }
                            (1, true) => {
                                // +Y face: looking from +Y down
                                // X is right, Z is up -> CCW: (x_low,z_low), (x_low,z_high), (x_high,z_high), (x_high,z_low)
                                let y = (slice + 1) as f32;
                                (
                                    [
                                        [chunk_world_x + u0f, y, chunk_world_z + v0f],
                                        [chunk_world_x + u0f, y, chunk_world_z + v1f],
                                        [chunk_world_x + u1f, y, chunk_world_z + v1f],
                                        [chunk_world_x + u1f, y, chunk_world_z + v0f],
                                    ],
                                    [0.0, 1.0, 0.0],
                                )
                            }
                            (1, false) => {
                                // -Y face: looking from -Y up
                                // X is left, Z is up -> CCW: (x_low,z_low), (x_high,z_low), (x_high,z_high), (x_low,z_high)
                                let y = slice as f32;
                                (
                                    [
                                        [chunk_world_x + u0f, y, chunk_world_z + v0f],
                                        [chunk_world_x + u1f, y, chunk_world_z + v0f],
                                        [chunk_world_x + u1f, y, chunk_world_z + v1f],
                                        [chunk_world_x + u0f, y, chunk_world_z + v1f],
                                    ],
                                    [0.0, -1.0, 0.0],
                                )
                            }
                            (2, true) => {
                                // +Z face: need +X × +Y = +Z
                                // CCW order: (x_low,y_low), (x_high,y_low), (x_high,y_high), (x_low,y_high)
                                let z = chunk_world_z + (slice + 1) as f32;
                                (
                                    [
                                        [chunk_world_x + u0f, v0f, z],
                                        [chunk_world_x + u1f, v0f, z],
                                        [chunk_world_x + u1f, v1f, z],
                                        [chunk_world_x + u0f, v1f, z],
                                    ],
                                    [0.0, 0.0, 1.0],
                                )
                            }
                            (2, false) => {
                                // -Z face: need +Y × +X = -Z
                                // CCW order: (x_low,y_low), (x_low,y_high), (x_high,y_high), (x_high,y_low)
                                let z = chunk_world_z + slice as f32;
                                (
                                    [
                                        [chunk_world_x + u0f, v0f, z],
                                        [chunk_world_x + u0f, v1f, z],
                                        [chunk_world_x + u1f, v1f, z],
                                        [chunk_world_x + u1f, v0f, z],
                                    ],
                                    [0.0, 0.0, -1.0],
                                )
                            }
                            _ => unreachable!(),
                        };

                        let base_idx = positions.len() as u32;

                        // Push vertices in CCW order
                        for vert in &verts {
                            positions.push(*vert);
                        }

                        for _ in 0..4 {
                            normals.push(normal);
                            colors.push(color_arr);
                            uv_layers.push([tex_layer, 0.0]); // UV_1: texture layer index
                        }

                        // UV coordinates for Minecraft-compatible texture mapping
                        // Bevy UV: V=0 is top, V=1 is bottom (image coordinate system)
                        // Side faces: Y+ is texture "up"
                        // Top face (+Y): Z- is texture "up"
                        // Bottom face (-Y): Z+ is texture "up"
                        //
                        // greedy meshing coords: u = axis1, v = axis2
                        // axis 0 (X faces): u = Y, v = Z
                        // axis 1 (Y faces): u = X, v = Z
                        // axis 2 (Z faces): u = X, v = Y
                        let w = width as f32; // size along v-axis
                        let h = height as f32; // size along u-axis

                        // Calculate UV for each vertex based on face direction
                        // Each face needs UV coords that put texture "up" in the correct direction
                        match (axis, positive) {
                            // +X face: looking at face, Z+ is right, Y+ is up
                            // Vertices: (y0,z0), (y1,z0), (y1,z1), (y0,z1)
                            // UV: u = z (right), v = h - y (up = low V)
                            (0, true) => {
                                uvs.push([v as f32, h]); // (y0,z0) -> (z0, h-y0) = (0, h)
                                uvs.push([v as f32, 0.0]); // (y1,z0) -> (z0, h-y1) = (0, 0)
                                uvs.push([(v as f32) + w, 0.0]); // (y1,z1) -> (z1, h-y1) = (w, 0)
                                uvs.push([(v as f32) + w, h]); // (y0,z1) -> (z1, h-y0) = (w, h)
                            }
                            // -X face: looking at face, Z- is right (mirrored), Y+ is up
                            // Vertices: (y0,z0), (y0,z1), (y1,z1), (y1,z0)
                            // UV: u = w - z (mirrored right), v = h - y
                            (0, false) => {
                                uvs.push([w - (v as f32), h]); // (y0,z0)
                                uvs.push([w - (v as f32) - w, h]); // (y0,z1) = (0, h)
                                uvs.push([w - (v as f32) - w, 0.0]); // (y1,z1) = (0, 0)
                                uvs.push([w - (v as f32), 0.0]); // (y1,z0)
                            }
                            // +Y face (top): looking down, X+ is right, Z+ is forward (down in texture)
                            // Vertices: (x0,z0), (x0,z1), (x1,z1), (x1,z0)
                            // UV: u = x, v = z
                            (1, true) => {
                                uvs.push([u as f32, v as f32]); // (x0,z0)
                                uvs.push([u as f32, (v as f32) + w]); // (x0,z1)
                                uvs.push([(u as f32) + h, (v as f32) + w]); // (x1,z1)
                                uvs.push([(u as f32) + h, v as f32]); // (x1,z0)
                            }
                            // -Y face (bottom): looking up, X+ is right, Z- is forward
                            // Vertices: (x0,z0), (x1,z0), (x1,z1), (x0,z1)
                            // UV: u = x, v = w - z (mirrored)
                            (1, false) => {
                                uvs.push([u as f32, w - (v as f32)]); // (x0,z0)
                                uvs.push([(u as f32) + h, w - (v as f32)]); // (x1,z0)
                                uvs.push([(u as f32) + h, w - (v as f32) - w]); // (x1,z1) = (h, 0)
                                uvs.push([u as f32, w - (v as f32) - w]); // (x0,z1) = (0, 0)
                            }
                            // +Z face: looking at face, X+ is right, Y+ is up
                            // Vertices: (x0,y0), (x1,y0), (x1,y1), (x0,y1)
                            // UV: u = x, v = w - y (Y+ = up = low V)
                            (2, true) => {
                                uvs.push([u as f32, w]); // (x0,y0) -> v = w - 0 = w
                                uvs.push([(u as f32) + h, w]); // (x1,y0)
                                uvs.push([(u as f32) + h, 0.0]); // (x1,y1) -> v = w - w = 0
                                uvs.push([u as f32, 0.0]); // (x0,y1)
                            }
                            // -Z face: looking at face, X- is right (mirrored), Y+ is up
                            // Vertices: (x0,y0), (x0,y1), (x1,y1), (x1,y0)
                            // UV: u = h - x (mirrored), v = w - y
                            (2, false) => {
                                uvs.push([h - (u as f32), w]); // (x0,y0)
                                uvs.push([h - (u as f32), 0.0]); // (x0,y1)
                                uvs.push([h - (u as f32) - h, 0.0]); // (x1,y1) = (0, 0)
                                uvs.push([h - (u as f32) - h, w]); // (x1,y0) = (0, w)
                            }
                            _ => unreachable!(),
                        }

                        indices.extend_from_slice(&[
                            base_idx,
                            base_idx + 1,
                            base_idx + 2,
                            base_idx,
                            base_idx + 2,
                            base_idx + 3,
                        ]);
                    }
                }
            }
        }

        tracing::info!(
            "Greedy mesh for chunk {:?}: {} vertices, {} indices",
            chunk_coord,
            positions.len(),
            indices.len()
        );

        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, default());
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_1, uv_layers); // Texture layer index for array texture
        mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
        mesh.insert_indices(Indices::U32(indices));
        mesh
    }

    /// Simple mesh generation without neighbor checking (for async tasks)
    /// Uses full LOD (all blocks)
    pub fn generate_mesh(&self, chunk_coord: IVec2) -> Mesh {
        self.generate_mesh_with_neighbors(chunk_coord, |_| false, ChunkLod::Full)
    }

    /// Simple mesh generation with LOD
    pub fn generate_mesh_with_lod(&self, chunk_coord: IVec2, lod: ChunkLod) -> Mesh {
        self.generate_mesh_with_neighbors(chunk_coord, |_| false, lod)
    }
}
