//! Utility functions for raycast, direction, and other common operations
//!
//! Note: These functions are also defined locally in main.rs for now.
//! This module exists for future refactoring to centralize utility functions.

#![allow(dead_code)]

use crate::Direction;
use bevy::prelude::*;
use std::f32::consts::PI;

/// Ray-AABB intersection test
/// Returns the distance along the ray to the intersection point, or None if no intersection
pub fn ray_aabb_intersection(
    ray_origin: Vec3,
    ray_direction: Vec3,
    box_min: Vec3,
    box_max: Vec3,
) -> Option<f32> {
    let inv_dir = Vec3::new(
        1.0 / ray_direction.x,
        1.0 / ray_direction.y,
        1.0 / ray_direction.z,
    );

    let t1 = (box_min.x - ray_origin.x) * inv_dir.x;
    let t2 = (box_max.x - ray_origin.x) * inv_dir.x;
    let t3 = (box_min.y - ray_origin.y) * inv_dir.y;
    let t4 = (box_max.y - ray_origin.y) * inv_dir.y;
    let t5 = (box_min.z - ray_origin.z) * inv_dir.z;
    let t6 = (box_max.z - ray_origin.z) * inv_dir.z;

    let tmin = t1.min(t2).max(t3.min(t4)).max(t5.min(t6));
    let tmax = t1.max(t2).min(t3.max(t4)).min(t5.max(t6));

    if tmax < 0.0 || tmin > tmax {
        None
    } else {
        Some(tmin)
    }
}

/// Ray-AABB intersection that also returns the hit normal
pub fn ray_aabb_intersection_with_normal(
    ray_origin: Vec3,
    ray_direction: Vec3,
    box_min: Vec3,
    box_max: Vec3,
) -> Option<(f32, Vec3)> {
    let inv_dir = Vec3::new(
        1.0 / ray_direction.x,
        1.0 / ray_direction.y,
        1.0 / ray_direction.z,
    );

    let tx1 = (box_min.x - ray_origin.x) * inv_dir.x;
    let tx2 = (box_max.x - ray_origin.x) * inv_dir.x;
    let ty1 = (box_min.y - ray_origin.y) * inv_dir.y;
    let ty2 = (box_max.y - ray_origin.y) * inv_dir.y;
    let tz1 = (box_min.z - ray_origin.z) * inv_dir.z;
    let tz2 = (box_max.z - ray_origin.z) * inv_dir.z;

    let tmin_x = tx1.min(tx2);
    let tmax_x = tx1.max(tx2);
    let tmin_y = ty1.min(ty2);
    let tmax_y = ty1.max(ty2);
    let tmin_z = tz1.min(tz2);
    let tmax_z = tz1.max(tz2);

    let tmin = tmin_x.max(tmin_y).max(tmin_z);
    let tmax = tmax_x.min(tmax_y).min(tmax_z);

    if tmax < 0.0 || tmin > tmax {
        return None;
    }

    // Determine which face was hit by finding which axis contributed to tmin
    let normal = if tmin == tmin_x {
        if ray_direction.x > 0.0 {
            Vec3::NEG_X
        } else {
            Vec3::X
        }
    } else if tmin == tmin_y {
        if ray_direction.y > 0.0 {
            Vec3::NEG_Y
        } else {
            Vec3::Y
        }
    } else if ray_direction.z > 0.0 {
        Vec3::NEG_Z
    } else {
        Vec3::Z
    };

    Some((tmin, normal))
}

/// Convert yaw angle to Direction
pub fn yaw_to_direction(yaw: f32) -> Direction {
    // Normalize yaw to 0..2PI
    let yaw = yaw.rem_euclid(std::f32::consts::TAU);
    // Split into 4 quadrants (45 degree offset for centered regions)
    if !(PI / 4.0..7.0 * PI / 4.0).contains(&yaw) {
        Direction::North
    } else if yaw < 3.0 * PI / 4.0 {
        Direction::West
    } else if yaw < 5.0 * PI / 4.0 {
        Direction::South
    } else {
        Direction::East
    }
}

/// Determine optimal conveyor direction based on adjacent conveyors and machines
pub fn auto_conveyor_direction(
    place_pos: IVec3,
    fallback_direction: Direction,
    conveyors: &[(IVec3, Direction)], // (position, direction)
    machines: &[IVec3],               // positions of miners, crushers, furnaces
) -> Direction {
    // Priority 1: Continue chain from adjacent conveyor pointing toward us
    for (conv_pos, conv_dir) in conveyors {
        let expected_target = *conv_pos + conv_dir.to_ivec3();
        if expected_target == place_pos {
            // This conveyor is pointing at our position, continue in same direction
            return *conv_dir;
        }
    }

    // Priority 2: Point away from adjacent machine (to receive items from it)
    for machine_pos in machines {
        let diff = place_pos - *machine_pos;
        if diff.x.abs() + diff.y.abs() + diff.z.abs() == 1 {
            // Adjacent machine - point away from it
            if diff.x == 1 {
                return Direction::East;
            }
            if diff.x == -1 {
                return Direction::West;
            }
            if diff.z == 1 {
                return Direction::South;
            }
            if diff.z == -1 {
                return Direction::North;
            }
        }
    }

    // Note: Previously had Priority 3 to align with adjacent conveyors,
    // but this prevented creating branches/splits. Now uses player direction.

    // Fallback: use player's facing direction
    fallback_direction
}

/// Convert keycode to character for text input
pub fn keycode_to_char(key_code: KeyCode, shift: bool) -> Option<char> {
    match key_code {
        KeyCode::KeyA => Some(if shift { 'A' } else { 'a' }),
        KeyCode::KeyB => Some(if shift { 'B' } else { 'b' }),
        KeyCode::KeyC => Some(if shift { 'C' } else { 'c' }),
        KeyCode::KeyD => Some(if shift { 'D' } else { 'd' }),
        KeyCode::KeyE => Some(if shift { 'E' } else { 'e' }),
        KeyCode::KeyF => Some(if shift { 'F' } else { 'f' }),
        KeyCode::KeyG => Some(if shift { 'G' } else { 'g' }),
        KeyCode::KeyH => Some(if shift { 'H' } else { 'h' }),
        KeyCode::KeyI => Some(if shift { 'I' } else { 'i' }),
        KeyCode::KeyJ => Some(if shift { 'J' } else { 'j' }),
        KeyCode::KeyK => Some(if shift { 'K' } else { 'k' }),
        KeyCode::KeyL => Some(if shift { 'L' } else { 'l' }),
        KeyCode::KeyM => Some(if shift { 'M' } else { 'm' }),
        KeyCode::KeyN => Some(if shift { 'N' } else { 'n' }),
        KeyCode::KeyO => Some(if shift { 'O' } else { 'o' }),
        KeyCode::KeyP => Some(if shift { 'P' } else { 'p' }),
        KeyCode::KeyQ => Some(if shift { 'Q' } else { 'q' }),
        KeyCode::KeyR => Some(if shift { 'R' } else { 'r' }),
        KeyCode::KeyS => Some(if shift { 'S' } else { 's' }),
        KeyCode::KeyT => Some(if shift { 'T' } else { 't' }),
        KeyCode::KeyU => Some(if shift { 'U' } else { 'u' }),
        KeyCode::KeyV => Some(if shift { 'V' } else { 'v' }),
        KeyCode::KeyW => Some(if shift { 'W' } else { 'w' }),
        KeyCode::KeyX => Some(if shift { 'X' } else { 'x' }),
        KeyCode::KeyY => Some(if shift { 'Y' } else { 'y' }),
        KeyCode::KeyZ => Some(if shift { 'Z' } else { 'z' }),
        KeyCode::Digit0 | KeyCode::Numpad0 => Some(if shift { ')' } else { '0' }),
        KeyCode::Digit1 | KeyCode::Numpad1 => Some(if shift { '!' } else { '1' }),
        KeyCode::Digit2 | KeyCode::Numpad2 => Some(if shift { '@' } else { '2' }),
        KeyCode::Digit3 | KeyCode::Numpad3 => Some(if shift { '#' } else { '3' }),
        KeyCode::Digit4 | KeyCode::Numpad4 => Some(if shift { '$' } else { '4' }),
        KeyCode::Digit5 | KeyCode::Numpad5 => Some(if shift { '%' } else { '5' }),
        KeyCode::Digit6 | KeyCode::Numpad6 => Some(if shift { '^' } else { '6' }),
        KeyCode::Digit7 | KeyCode::Numpad7 => Some(if shift { '&' } else { '7' }),
        KeyCode::Digit8 | KeyCode::Numpad8 => Some(if shift { '*' } else { '8' }),
        KeyCode::Digit9 | KeyCode::Numpad9 => Some(if shift { '(' } else { '9' }),
        KeyCode::Space => Some(' '),
        KeyCode::Minus => Some(if shift { '_' } else { '-' }),
        KeyCode::Equal => Some(if shift { '+' } else { '=' }),
        KeyCode::Slash => Some(if shift { '?' } else { '/' }),
        KeyCode::Period => Some(if shift { '>' } else { '.' }),
        KeyCode::Comma => Some(if shift { '<' } else { ',' }),
        _ => None,
    }
}

/// Parse item name to BlockType
pub fn parse_item_name(name: &str) -> Option<crate::BlockType> {
    use crate::BlockType;
    match name.to_lowercase().as_str() {
        "iron_ore" | "ironore" | "iron" => Some(BlockType::IronOre),
        "coal" => Some(BlockType::Coal),
        "stone" => Some(BlockType::Stone),
        "grass" => Some(BlockType::Grass),
        "iron_ingot" | "ironingot" => Some(BlockType::IronIngot),
        "copper_ore" | "copperore" | "copper" => Some(BlockType::CopperOre),
        "copper_ingot" | "copperingot" => Some(BlockType::CopperIngot),
        "miner" | "miner_block" | "minerblock" => Some(BlockType::MinerBlock),
        "conveyor" | "conveyor_block" | "conveyorblock" => Some(BlockType::ConveyorBlock),
        "crusher" | "crusher_block" | "crusherblock" => Some(BlockType::CrusherBlock),
        "furnace" | "furnace_block" | "furnaceblock" => Some(BlockType::FurnaceBlock),
        _ => None,
    }
}
