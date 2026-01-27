//! Mesh generation for Blockbench models

use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;

use super::raw::{RawElement, RawFace};
use super::BlockbenchLoadError;

/// Generate Bevy Mesh from bbmodel elements
pub(crate) fn generate_mesh(
    elements: &[RawElement],
    resolution: UVec2,
) -> Result<Mesh, BlockbenchLoadError> {
    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut normals: Vec<[f32; 3]> = Vec::new();
    let mut uvs: Vec<[f32; 2]> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();

    // Scale factor: 16 pixels = 1 game unit (Minecraft convention)
    let scale = 1.0 / 16.0;

    for element in elements {
        add_element_mesh(
            element,
            resolution,
            scale,
            &mut positions,
            &mut normals,
            &mut uvs,
            &mut indices,
        );
    }

    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U32(indices));

    Ok(mesh)
}

/// Add a single element's geometry to the mesh
pub(crate) fn add_element_mesh(
    element: &RawElement,
    resolution: UVec2,
    scale: f32,
    positions: &mut Vec<[f32; 3]>,
    normals: &mut Vec<[f32; 3]>,
    uvs: &mut Vec<[f32; 2]>,
    indices: &mut Vec<u32>,
) {
    let from = element.from;
    let to = element.to;
    let origin = element.origin;
    let rotation = element.rotation;

    // Calculate the 8 corners of the cube
    // Blockbench uses Y-up, same as Bevy
    let corners = [
        [from[0], from[1], from[2]], // 0: ---
        [to[0], from[1], from[2]],   // 1: +--
        [to[0], to[1], from[2]],     // 2: ++-
        [from[0], to[1], from[2]],   // 3: -+-
        [from[0], from[1], to[2]],   // 4: --+
        [to[0], from[1], to[2]],     // 5: +-+
        [to[0], to[1], to[2]],       // 6: +++
        [from[0], to[1], to[2]],     // 7: -++
    ];

    // Apply rotation around origin if needed
    let corners: Vec<[f32; 3]> = if rotation[0] != 0.0 || rotation[1] != 0.0 || rotation[2] != 0.0 {
        let rot = Quat::from_euler(
            EulerRot::YXZ,
            rotation[1].to_radians(),
            rotation[0].to_radians(),
            rotation[2].to_radians(),
        );
        corners
            .iter()
            .map(|c| {
                let p = Vec3::from(*c) - Vec3::from(origin);
                let rotated = rot * p + Vec3::from(origin);
                [rotated.x, rotated.y, rotated.z]
            })
            .collect()
    } else {
        corners.to_vec()
    };

    // Transform corners to game space (scale and center)
    let transformed: Vec<[f32; 3]> = corners
        .iter()
        .map(|c| {
            [
                (c[0] - 8.0) * scale, // Center at 0 (Blockbench default is 0-16)
                (c[1] - 8.0) * scale,
                (c[2] - 8.0) * scale,
            ]
        })
        .collect();

    // Face definitions: (corner indices, normal, face data accessor)
    let face_defs: [([usize; 4], [f32; 3], &Option<RawFace>); 6] = [
        ([0, 1, 2, 3], [0.0, 0.0, -1.0], &element.faces.north),
        ([5, 4, 7, 6], [0.0, 0.0, 1.0], &element.faces.south),
        ([1, 5, 6, 2], [1.0, 0.0, 0.0], &element.faces.east),
        ([4, 0, 3, 7], [-1.0, 0.0, 0.0], &element.faces.west),
        ([3, 2, 6, 7], [0.0, 1.0, 0.0], &element.faces.up),
        ([4, 5, 1, 0], [0.0, -1.0, 0.0], &element.faces.down),
    ];

    for (corner_indices, normal, face_opt) in &face_defs {
        let Some(face) = face_opt else { continue };

        let base_idx = positions.len() as u32;

        // Add vertices for this face
        for &ci in corner_indices {
            positions.push(transformed[ci]);
            normals.push(*normal);
        }

        // Calculate UV coordinates (convert from pixel to 0-1 range)
        let uv = face.uv;
        let u1 = uv[0] / resolution.x as f32;
        let v1 = uv[1] / resolution.y as f32;
        let u2 = uv[2] / resolution.x as f32;
        let v2 = uv[3] / resolution.y as f32;

        // Apply UV based on face rotation
        let face_uvs = match face.rotation {
            0 => [[u1, v2], [u2, v2], [u2, v1], [u1, v1]],
            90 => [[u1, v1], [u1, v2], [u2, v2], [u2, v1]],
            180 => [[u2, v1], [u1, v1], [u1, v2], [u2, v2]],
            270 => [[u2, v2], [u2, v1], [u1, v1], [u1, v2]],
            _ => [[u1, v2], [u2, v2], [u2, v1], [u1, v1]],
        };

        for uv_coord in &face_uvs {
            uvs.push(*uv_coord);
        }

        // Add indices (two triangles)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blockbench::raw::RawFaces;

    fn make_test_element() -> RawElement {
        RawElement {
            uuid: "test".to_string(),
            element_type: "cube".to_string(),
            name: "cube".to_string(),
            box_uv: false,
            from: [0.0, 0.0, 0.0],
            to: [16.0, 16.0, 16.0],
            origin: [8.0, 8.0, 8.0],
            rotation: [0.0, 0.0, 0.0],
            faces: RawFaces {
                north: Some(RawFace {
                    uv: [0.0, 0.0, 16.0, 16.0],
                    texture: Some(0),
                    rotation: 0,
                }),
                south: Some(RawFace {
                    uv: [0.0, 0.0, 16.0, 16.0],
                    texture: Some(0),
                    rotation: 0,
                }),
                east: Some(RawFace {
                    uv: [0.0, 0.0, 16.0, 16.0],
                    texture: Some(0),
                    rotation: 0,
                }),
                west: Some(RawFace {
                    uv: [0.0, 0.0, 16.0, 16.0],
                    texture: Some(0),
                    rotation: 0,
                }),
                up: Some(RawFace {
                    uv: [0.0, 0.0, 16.0, 16.0],
                    texture: Some(0),
                    rotation: 0,
                }),
                down: Some(RawFace {
                    uv: [0.0, 0.0, 16.0, 16.0],
                    texture: Some(0),
                    rotation: 0,
                }),
            },
        }
    }

    #[test]
    fn test_generate_mesh_single_cube() {
        let elements = vec![make_test_element()];

        let mesh = generate_mesh(&elements, UVec2::new(16, 16)).unwrap();

        // 6 faces * 4 vertices = 24 vertices
        let positions = mesh.attribute(Mesh::ATTRIBUTE_POSITION).unwrap();
        let pos_count = match positions {
            bevy::mesh::VertexAttributeValues::Float32x3(v) => v.len(),
            _ => 0,
        };
        assert_eq!(pos_count, 24);

        // 6 faces * 2 triangles * 3 indices = 36 indices
        let indices = mesh.indices().unwrap();
        assert_eq!(indices.len(), 36);
    }

    #[test]
    fn test_generate_mesh_rotated_cube() {
        let mut element = make_test_element();
        element.rotation = [0.0, 45.0, 0.0]; // 45 degree rotation on Y axis

        let elements = vec![element];

        let mesh = generate_mesh(&elements, UVec2::new(16, 16)).unwrap();

        // Should still have 24 vertices even with rotation
        let positions = mesh.attribute(Mesh::ATTRIBUTE_POSITION).unwrap();
        let pos_count = match positions {
            bevy::mesh::VertexAttributeValues::Float32x3(v) => v.len(),
            _ => 0,
        };
        assert_eq!(pos_count, 24);
    }
}
