//! Mesh generation utilities for conveyors and wireframes
//!
//! This module contains functions for creating procedural meshes used in the game.

use bevy::asset::RenderAssetUsages;
use bevy::mesh::PrimitiveTopology;
use bevy::prelude::*;

use crate::constants::{BLOCK_SIZE, CONVEYOR_BELT_HEIGHT, CONVEYOR_BELT_WIDTH};
use crate::{ConveyorShape, Direction};

/// Create a wireframe mesh for conveyor preview (green outline with direction arrow)
pub fn create_conveyor_wireframe_mesh(direction: Direction) -> Mesh {
    let half_w = BLOCK_SIZE * 0.505; // Width
    let half_h = 0.25; // Height (conveyor is 0.5 tall - half block)
    let half_l = BLOCK_SIZE * 0.505; // Length

    // 8 corners of the conveyor bounding box (centered at y=0.25)
    let y_offset = 0.25;
    let corners = [
        Vec3::new(-half_w, y_offset - half_h, -half_l), // 0: bottom-back-left
        Vec3::new(half_w, y_offset - half_h, -half_l),  // 1: bottom-back-right
        Vec3::new(half_w, y_offset + half_h, -half_l),  // 2: top-back-right
        Vec3::new(-half_w, y_offset + half_h, -half_l), // 3: top-back-left
        Vec3::new(-half_w, y_offset - half_h, half_l),  // 4: bottom-front-left
        Vec3::new(half_w, y_offset - half_h, half_l),   // 5: bottom-front-right
        Vec3::new(half_w, y_offset + half_h, half_l),   // 6: top-front-right
        Vec3::new(-half_w, y_offset + half_h, half_l),  // 7: top-front-left
    ];

    // Arrow points (centered, then rotated for direction)
    let arrow_y = y_offset + half_h + 0.02; // Slightly above conveyor
    let arrow_base = 0.0;
    let arrow_tip = 0.35;
    let arrow_wing = 0.15;

    // Base arrow points in +X direction (East)
    let arrow_points = match direction {
        Direction::East => [
            (
                Vec3::new(arrow_base, arrow_y, 0.0),
                Vec3::new(arrow_tip, arrow_y, 0.0),
            ), // Main arrow
            (
                Vec3::new(arrow_tip, arrow_y, 0.0),
                Vec3::new(arrow_tip - arrow_wing, arrow_y, arrow_wing),
            ), // Left wing
            (
                Vec3::new(arrow_tip, arrow_y, 0.0),
                Vec3::new(arrow_tip - arrow_wing, arrow_y, -arrow_wing),
            ), // Right wing
        ],
        Direction::West => [
            (
                Vec3::new(-arrow_base, arrow_y, 0.0),
                Vec3::new(-arrow_tip, arrow_y, 0.0),
            ),
            (
                Vec3::new(-arrow_tip, arrow_y, 0.0),
                Vec3::new(-arrow_tip + arrow_wing, arrow_y, arrow_wing),
            ),
            (
                Vec3::new(-arrow_tip, arrow_y, 0.0),
                Vec3::new(-arrow_tip + arrow_wing, arrow_y, -arrow_wing),
            ),
        ],
        Direction::South => [
            (
                Vec3::new(0.0, arrow_y, arrow_base),
                Vec3::new(0.0, arrow_y, arrow_tip),
            ),
            (
                Vec3::new(0.0, arrow_y, arrow_tip),
                Vec3::new(arrow_wing, arrow_y, arrow_tip - arrow_wing),
            ),
            (
                Vec3::new(0.0, arrow_y, arrow_tip),
                Vec3::new(-arrow_wing, arrow_y, arrow_tip - arrow_wing),
            ),
        ],
        Direction::North => [
            (
                Vec3::new(0.0, arrow_y, -arrow_base),
                Vec3::new(0.0, arrow_y, -arrow_tip),
            ),
            (
                Vec3::new(0.0, arrow_y, -arrow_tip),
                Vec3::new(arrow_wing, arrow_y, -arrow_tip + arrow_wing),
            ),
            (
                Vec3::new(0.0, arrow_y, -arrow_tip),
                Vec3::new(-arrow_wing, arrow_y, -arrow_tip + arrow_wing),
            ),
        ],
    };

    // Build positions: 12 box edges + 3 arrow lines = 15 lines = 30 vertices
    let mut positions: Vec<[f32; 3]> = Vec::with_capacity(30);

    // Box edges (12 edges, 24 vertices)
    let edges = [
        // Bottom face
        (corners[0], corners[1]),
        (corners[1], corners[5]),
        (corners[5], corners[4]),
        (corners[4], corners[0]),
        // Top face
        (corners[3], corners[2]),
        (corners[2], corners[6]),
        (corners[6], corners[7]),
        (corners[7], corners[3]),
        // Vertical edges
        (corners[0], corners[3]),
        (corners[1], corners[2]),
        (corners[5], corners[6]),
        (corners[4], corners[7]),
    ];

    for (a, b) in edges {
        positions.push(a.to_array());
        positions.push(b.to_array());
    }

    // Arrow lines (3 lines, 6 vertices)
    for (a, b) in arrow_points {
        positions.push(a.to_array());
        positions.push(b.to_array());
    }

    let mut mesh = Mesh::new(PrimitiveTopology::LineList, RenderAssetUsages::RENDER_WORLD);
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh
}

/// Create arrow mesh for direction indication (LineList)
pub fn create_arrow_mesh(direction: Direction) -> Mesh {
    let arrow_y = 0.55; // Above conveyor (0.5 height + 0.05 offset)
    let arrow_base = 0.0;
    let arrow_tip = 0.4;
    let arrow_wing = 0.2;

    let arrow_points = match direction {
        Direction::East => [
            (
                Vec3::new(arrow_base, arrow_y, 0.0),
                Vec3::new(arrow_tip, arrow_y, 0.0),
            ),
            (
                Vec3::new(arrow_tip, arrow_y, 0.0),
                Vec3::new(arrow_tip - arrow_wing, arrow_y, arrow_wing),
            ),
            (
                Vec3::new(arrow_tip, arrow_y, 0.0),
                Vec3::new(arrow_tip - arrow_wing, arrow_y, -arrow_wing),
            ),
        ],
        Direction::West => [
            (
                Vec3::new(-arrow_base, arrow_y, 0.0),
                Vec3::new(-arrow_tip, arrow_y, 0.0),
            ),
            (
                Vec3::new(-arrow_tip, arrow_y, 0.0),
                Vec3::new(-arrow_tip + arrow_wing, arrow_y, arrow_wing),
            ),
            (
                Vec3::new(-arrow_tip, arrow_y, 0.0),
                Vec3::new(-arrow_tip + arrow_wing, arrow_y, -arrow_wing),
            ),
        ],
        Direction::North => [
            (
                Vec3::new(0.0, arrow_y, arrow_base),
                Vec3::new(0.0, arrow_y, arrow_tip),
            ),
            (
                Vec3::new(0.0, arrow_y, arrow_tip),
                Vec3::new(arrow_wing, arrow_y, arrow_tip - arrow_wing),
            ),
            (
                Vec3::new(0.0, arrow_y, arrow_tip),
                Vec3::new(-arrow_wing, arrow_y, arrow_tip - arrow_wing),
            ),
        ],
        Direction::South => [
            (
                Vec3::new(0.0, arrow_y, -arrow_base),
                Vec3::new(0.0, arrow_y, -arrow_tip),
            ),
            (
                Vec3::new(0.0, arrow_y, -arrow_tip),
                Vec3::new(arrow_wing, arrow_y, -arrow_tip + arrow_wing),
            ),
            (
                Vec3::new(0.0, arrow_y, -arrow_tip),
                Vec3::new(-arrow_wing, arrow_y, -arrow_tip + arrow_wing),
            ),
        ],
    };

    let mut positions: Vec<[f32; 3]> = Vec::with_capacity(6);
    for (a, b) in arrow_points {
        positions.push(a.to_array());
        positions.push(b.to_array());
    }

    let mut mesh = Mesh::new(PrimitiveTopology::LineList, RenderAssetUsages::RENDER_WORLD);
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh
}

/// Create a wireframe cube mesh (12 edges)
pub fn create_wireframe_cube_mesh() -> Mesh {
    let half = BLOCK_SIZE * 0.505; // Slightly larger to avoid z-fighting

    // 8 corners of the cube
    let corners = [
        Vec3::new(-half, -half, -half), // 0
        Vec3::new(half, -half, -half),  // 1
        Vec3::new(half, half, -half),   // 2
        Vec3::new(-half, half, -half),  // 3
        Vec3::new(-half, -half, half),  // 4
        Vec3::new(half, -half, half),   // 5
        Vec3::new(half, half, half),    // 6
        Vec3::new(-half, half, half),   // 7
    ];

    // 12 edges as line pairs (24 vertices total)
    let positions: Vec<[f32; 3]> = [
        // Bottom face edges
        (corners[0], corners[1]),
        (corners[1], corners[5]),
        (corners[5], corners[4]),
        (corners[4], corners[0]),
        // Top face edges
        (corners[3], corners[2]),
        (corners[2], corners[6]),
        (corners[6], corners[7]),
        (corners[7], corners[3]),
        // Vertical edges
        (corners[0], corners[3]),
        (corners[1], corners[2]),
        (corners[5], corners[6]),
        (corners[4], corners[7]),
    ]
    .iter()
    .flat_map(|(a, b)| vec![a.to_array(), b.to_array()])
    .collect();

    let mut mesh = Mesh::new(PrimitiveTopology::LineList, RenderAssetUsages::RENDER_WORLD);
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh
}

/// Create conveyor mesh based on connection shape
pub fn create_conveyor_mesh(shape: ConveyorShape) -> Mesh {
    let width = BLOCK_SIZE * CONVEYOR_BELT_WIDTH;
    let height = BLOCK_SIZE * CONVEYOR_BELT_HEIGHT;
    let half_width = width / 2.0;
    let half_height = height / 2.0;
    let half_block = BLOCK_SIZE / 2.0;

    match shape {
        ConveyorShape::Straight => {
            // Simple rectangular belt
            Cuboid::new(width, height, BLOCK_SIZE).into()
        }
        ConveyorShape::CornerLeft => {
            // L-shaped: main belt + left extension
            create_l_shaped_mesh(half_width, half_height, half_block, true)
        }
        ConveyorShape::CornerRight => {
            // L-shaped: main belt + right extension
            create_l_shaped_mesh(half_width, half_height, half_block, false)
        }
        ConveyorShape::TJunction => {
            // T-shaped: main belt + both side extensions
            create_t_shaped_mesh(half_width, half_height, half_block)
        }
        ConveyorShape::Splitter => {
            // Splitter: Y-shaped with 3 output directions (front, left, right)
            create_splitter_mesh(half_width, half_height, half_block)
        }
    }
}

/// Create L-shaped conveyor mesh
fn create_l_shaped_mesh(half_width: f32, half_height: f32, half_block: f32, is_left: bool) -> Mesh {
    // The conveyor faces -Z, so:
    // - Left is +X direction
    // - Right is -X direction
    let side_sign = if is_left { 1.0 } else { -1.0 };

    // Main belt vertices (along Z axis, width along X)
    // Side extension vertices (along X axis from the back half)
    let positions: Vec<[f32; 3]> = vec![
        // Main belt (8 vertices) - full length along Z
        [-half_width, -half_height, -half_block], // 0
        [half_width, -half_height, -half_block],  // 1
        [half_width, half_height, -half_block],   // 2
        [-half_width, half_height, -half_block],  // 3
        [-half_width, -half_height, half_block],  // 4
        [half_width, -half_height, half_block],   // 5
        [half_width, half_height, half_block],    // 6
        [-half_width, half_height, half_block],   // 7
        // Side extension (8 vertices) - extends from side at back half
        // Inner edge at half_width (or -half_width), outer edge at half_block
        [side_sign * half_width, -half_height, 0.0], // 8 - inner front
        [side_sign * half_block, -half_height, 0.0], // 9 - outer front
        [side_sign * half_block, half_height, 0.0],  // 10 - outer front top
        [side_sign * half_width, half_height, 0.0],  // 11 - inner front top
        [side_sign * half_width, -half_height, half_block], // 12 - inner back
        [side_sign * half_block, -half_height, half_block], // 13 - outer back
        [side_sign * half_block, half_height, half_block], // 14 - outer back top
        [side_sign * half_width, half_height, half_block], // 15 - inner back top
    ];

    // Normals for each vertex
    let normals: Vec<[f32; 3]> = vec![
        // Main belt
        [0.0, 0.0, -1.0],
        [0.0, 0.0, -1.0],
        [0.0, 0.0, -1.0],
        [0.0, 0.0, -1.0], // front
        [0.0, 0.0, 1.0],
        [0.0, 0.0, 1.0],
        [0.0, 0.0, 1.0],
        [0.0, 0.0, 1.0], // back
        // Side extension (simplified - all outward)
        [0.0, 0.0, -1.0],
        [side_sign, 0.0, 0.0],
        [side_sign, 0.0, 0.0],
        [0.0, 0.0, -1.0],
        [0.0, 0.0, 1.0],
        [side_sign, 0.0, 0.0],
        [side_sign, 0.0, 0.0],
        [0.0, 0.0, 1.0],
    ];

    let uvs: Vec<[f32; 2]> = vec![[0.0; 2]; 16];

    // Indices - each face needs to be wound correctly for back-face culling
    let indices = if is_left {
        vec![
            // Main belt faces
            0, 2, 1, 0, 3, 2, // front
            4, 5, 6, 4, 6, 7, // back
            0, 1, 5, 0, 5, 4, // bottom
            3, 6, 2, 3, 7, 6, // top
            0, 4, 7, 0, 7, 3, // left (closed since extension is on right/+X)
            1, 2, 6, 1, 6, 5, // right - open where extension connects
            // Side extension faces
            8, 10, 9, 8, 11, 10, // front
            12, 13, 14, 12, 14, 15, // back
            8, 9, 13, 8, 13, 12, // bottom
            11, 14, 10, 11, 15, 14, // top
            9, 10, 14, 9, 14, 13, // outer side
        ]
    } else {
        vec![
            // Main belt faces
            0, 2, 1, 0, 3, 2, // front
            4, 5, 6, 4, 6, 7, // back
            0, 1, 5, 0, 5, 4, // bottom
            3, 6, 2, 3, 7, 6, // top
            1, 2, 6, 1, 6, 5, // right (closed since extension is on left/-X)
            0, 4, 7, 0, 7, 3, // left - open where extension connects
            // Side extension faces (reversed winding for -X side)
            8, 9, 10, 8, 10, 11, // front
            12, 14, 13, 12, 15, 14, // back
            8, 13, 9, 8, 12, 13, // bottom
            11, 10, 14, 11, 14, 15, // top
            9, 14, 10, 9, 13, 14, // outer side
        ]
    };

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, default());
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(bevy::mesh::Indices::U32(indices));
    mesh
}

/// Create T-shaped conveyor mesh (both sides)
fn create_t_shaped_mesh(half_width: f32, half_height: f32, half_block: f32) -> Mesh {
    // Main belt + extensions on both sides
    let positions: Vec<[f32; 3]> = vec![
        // Main belt (8 vertices)
        [-half_width, -half_height, -half_block], // 0
        [half_width, -half_height, -half_block],  // 1
        [half_width, half_height, -half_block],   // 2
        [-half_width, half_height, -half_block],  // 3
        [-half_width, -half_height, half_block],  // 4
        [half_width, -half_height, half_block],   // 5
        [half_width, half_height, half_block],    // 6
        [-half_width, half_height, half_block],   // 7
        // Left extension (+X side, 8 vertices)
        [half_width, -half_height, 0.0],        // 8
        [half_block, -half_height, 0.0],        // 9
        [half_block, half_height, 0.0],         // 10
        [half_width, half_height, 0.0],         // 11
        [half_width, -half_height, half_block], // 12
        [half_block, -half_height, half_block], // 13
        [half_block, half_height, half_block],  // 14
        [half_width, half_height, half_block],  // 15
        // Right extension (-X side, 8 vertices)
        [-half_width, -half_height, 0.0],        // 16
        [-half_block, -half_height, 0.0],        // 17
        [-half_block, half_height, 0.0],         // 18
        [-half_width, half_height, 0.0],         // 19
        [-half_width, -half_height, half_block], // 20
        [-half_block, -half_height, half_block], // 21
        [-half_block, half_height, half_block],  // 22
        [-half_width, half_height, half_block],  // 23
    ];

    let normals: Vec<[f32; 3]> = vec![
        // Main belt
        [0.0, 0.0, -1.0],
        [0.0, 0.0, -1.0],
        [0.0, 0.0, -1.0],
        [0.0, 0.0, -1.0],
        [0.0, 0.0, 1.0],
        [0.0, 0.0, 1.0],
        [0.0, 0.0, 1.0],
        [0.0, 0.0, 1.0],
        // Left extension
        [0.0, 0.0, -1.0],
        [1.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [0.0, 0.0, -1.0],
        [0.0, 0.0, 1.0],
        [1.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0],
        // Right extension
        [0.0, 0.0, -1.0],
        [-1.0, 0.0, 0.0],
        [-1.0, 0.0, 0.0],
        [0.0, 0.0, -1.0],
        [0.0, 0.0, 1.0],
        [-1.0, 0.0, 0.0],
        [-1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0],
    ];

    let uvs: Vec<[f32; 2]> = vec![[0.0; 2]; 24];

    let indices: Vec<u32> = vec![
        // Main belt faces
        0, 2, 1, 0, 3, 2, // front
        4, 5, 6, 4, 6, 7, // back
        0, 1, 5, 0, 5, 4, // bottom
        3, 6, 2, 3, 7, 6, // top
        // Left and right main sides are open for extensions
        // Left extension (+X)
        8, 10, 9, 8, 11, 10, // front
        12, 13, 14, 12, 14, 15, // back
        8, 9, 13, 8, 13, 12, // bottom
        11, 14, 10, 11, 15, 14, // top
        9, 10, 14, 9, 14, 13, // outer side (+X face)
        // Right extension (-X)
        16, 17, 18, 16, 18, 19, // front
        20, 22, 21, 20, 23, 22, // back
        16, 21, 17, 16, 20, 21, // bottom
        19, 18, 22, 19, 22, 23, // top
        17, 22, 18, 17, 21, 22, // outer side (-X face)
    ];

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, default());
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(bevy::mesh::Indices::U32(indices));
    mesh
}

/// Create splitter conveyor mesh (Y-shaped with 3 output directions)
fn create_splitter_mesh(half_width: f32, half_height: f32, half_block: f32) -> Mesh {
    // Main belt (short, just the center) + 3 extensions (front, left, right)
    let positions: Vec<[f32; 3]> = vec![
        // Center hub (8 vertices) - small cube at center
        [-half_width, -half_height, -half_width], // 0
        [half_width, -half_height, -half_width],  // 1
        [half_width, half_height, -half_width],   // 2
        [-half_width, half_height, -half_width],  // 3
        [-half_width, -half_height, half_width],  // 4
        [half_width, -half_height, half_width],   // 5
        [half_width, half_height, half_width],    // 6
        [-half_width, half_height, half_width],   // 7
        // Front extension (-Z direction, 8 vertices)
        [-half_width, -half_height, -half_block], // 8
        [half_width, -half_height, -half_block],  // 9
        [half_width, half_height, -half_block],   // 10
        [-half_width, half_height, -half_block],  // 11
        [-half_width, -half_height, -half_width], // 12
        [half_width, -half_height, -half_width],  // 13
        [half_width, half_height, -half_width],   // 14
        [-half_width, half_height, -half_width],  // 15
        // Left extension (+X direction, 8 vertices)
        [half_width, -half_height, -half_width], // 16
        [half_block, -half_height, -half_width], // 17
        [half_block, half_height, -half_width],  // 18
        [half_width, half_height, -half_width],  // 19
        [half_width, -half_height, half_width],  // 20
        [half_block, -half_height, half_width],  // 21
        [half_block, half_height, half_width],   // 22
        [half_width, half_height, half_width],   // 23
        // Right extension (-X direction, 8 vertices)
        [-half_block, -half_height, -half_width], // 24
        [-half_width, -half_height, -half_width], // 25
        [-half_width, half_height, -half_width],  // 26
        [-half_block, half_height, -half_width],  // 27
        [-half_block, -half_height, half_width],  // 28
        [-half_width, -half_height, half_width],  // 29
        [-half_width, half_height, half_width],   // 30
        [-half_block, half_height, half_width],   // 31
    ];

    let normals: Vec<[f32; 3]> = vec![
        // Center hub (simplified normals)
        [0.0, 0.0, -1.0],
        [0.0, 0.0, -1.0],
        [0.0, 0.0, -1.0],
        [0.0, 0.0, -1.0],
        [0.0, 0.0, 1.0],
        [0.0, 0.0, 1.0],
        [0.0, 0.0, 1.0],
        [0.0, 0.0, 1.0],
        // Front extension
        [0.0, 0.0, -1.0],
        [0.0, 0.0, -1.0],
        [0.0, 0.0, -1.0],
        [0.0, 0.0, -1.0],
        [0.0, 0.0, -1.0],
        [0.0, 0.0, -1.0],
        [0.0, 0.0, -1.0],
        [0.0, 0.0, -1.0],
        // Left extension
        [1.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        // Right extension
        [-1.0, 0.0, 0.0],
        [-1.0, 0.0, 0.0],
        [-1.0, 0.0, 0.0],
        [-1.0, 0.0, 0.0],
        [-1.0, 0.0, 0.0],
        [-1.0, 0.0, 0.0],
        [-1.0, 0.0, 0.0],
        [-1.0, 0.0, 0.0],
    ];

    let uvs: Vec<[f32; 2]> = vec![[0.0; 2]; 32];

    let indices: Vec<u32> = vec![
        // Center hub - top face only (sides connect to extensions)
        3, 6, 2, 3, 7, 6, // top
        0, 1, 5, 0, 5, 4, // bottom
        // Front extension (-Z)
        8, 10, 9, 8, 11, 10, // front face
        8, 9, 13, 8, 13, 12, // bottom
        11, 14, 10, 11, 15, 14, // top
        8, 12, 15, 8, 15, 11, // left side
        9, 10, 14, 9, 14, 13, // right side
        // Left extension (+X)
        17, 18, 22, 17, 22, 21, // outer face (+X)
        16, 17, 21, 16, 21, 20, // bottom
        19, 22, 18, 19, 23, 22, // top
        16, 19, 18, 16, 18, 17, // front
        20, 21, 22, 20, 22, 23, // back
        // Right extension (-X)
        24, 28, 31, 24, 31, 27, // outer face (-X)
        24, 25, 29, 24, 29, 28, // bottom
        27, 31, 30, 27, 30, 26, // top
        24, 27, 26, 24, 26, 25, // front
        28, 29, 30, 28, 30, 31, // back
    ];

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, default());
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(bevy::mesh::Indices::U32(indices));
    mesh
}
