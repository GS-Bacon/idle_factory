//! Utility functions for raycast, direction, and other common operations

#![allow(dead_code)]

use crate::Direction;
use bevy::prelude::*;
use std::f32::consts::PI;

// ============================================================================
// Coordinate Types (NewType Pattern)
// ============================================================================

/// Grid coordinate (integer, block units)
/// Use this for block positions, chunk coordinates, machine positions, etc.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub struct GridPos(pub IVec3);

/// World coordinate (floating point)
/// Use this for entity positions, ray origins, visual positions, etc.
#[derive(Clone, Copy, Debug, Default)]
pub struct WorldPos(pub Vec3);

impl GridPos {
    pub const ZERO: GridPos = GridPos(IVec3::ZERO);

    #[inline]
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        GridPos(IVec3::new(x, y, z))
    }

    /// Convert to world position (center of the block)
    #[inline]
    pub fn to_world(self) -> WorldPos {
        WorldPos(Vec3::new(
            self.0.x as f32 + 0.5,
            self.0.y as f32 + 0.5,
            self.0.z as f32 + 0.5,
        ))
    }

    /// Convert to world position (corner of the block, no offset)
    #[inline]
    pub fn to_world_corner(self) -> WorldPos {
        WorldPos(Vec3::new(self.0.x as f32, self.0.y as f32, self.0.z as f32))
    }

    #[inline]
    pub fn x(&self) -> i32 {
        self.0.x
    }

    #[inline]
    pub fn y(&self) -> i32 {
        self.0.y
    }

    #[inline]
    pub fn z(&self) -> i32 {
        self.0.z
    }
}

impl WorldPos {
    pub const ZERO: WorldPos = WorldPos(Vec3::ZERO);

    #[inline]
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        WorldPos(Vec3::new(x, y, z))
    }

    /// Convert to grid position (floor to integer)
    #[inline]
    pub fn to_grid(self) -> GridPos {
        GridPos(IVec3::new(
            self.0.x.floor() as i32,
            self.0.y.floor() as i32,
            self.0.z.floor() as i32,
        ))
    }

    #[inline]
    pub fn x(&self) -> f32 {
        self.0.x
    }

    #[inline]
    pub fn y(&self) -> f32 {
        self.0.y
    }

    #[inline]
    pub fn z(&self) -> f32 {
        self.0.z
    }
}

// Conversion traits
impl From<WorldPos> for GridPos {
    #[inline]
    fn from(pos: WorldPos) -> Self {
        pos.to_grid()
    }
}

impl From<GridPos> for WorldPos {
    #[inline]
    fn from(pos: GridPos) -> Self {
        pos.to_world_corner()
    }
}

impl From<IVec3> for GridPos {
    #[inline]
    fn from(v: IVec3) -> Self {
        GridPos(v)
    }
}

impl From<GridPos> for IVec3 {
    #[inline]
    fn from(pos: GridPos) -> Self {
        pos.0
    }
}

impl From<Vec3> for WorldPos {
    #[inline]
    fn from(v: Vec3) -> Self {
        WorldPos(v)
    }
}

impl From<WorldPos> for Vec3 {
    #[inline]
    fn from(pos: WorldPos) -> Self {
        pos.0
    }
}

// Arithmetic operations for GridPos
impl std::ops::Add for GridPos {
    type Output = GridPos;
    #[inline]
    fn add(self, rhs: GridPos) -> Self::Output {
        GridPos(self.0 + rhs.0)
    }
}

impl std::ops::Add<IVec3> for GridPos {
    type Output = GridPos;
    #[inline]
    fn add(self, rhs: IVec3) -> Self::Output {
        GridPos(self.0 + rhs)
    }
}

impl std::ops::Sub for GridPos {
    type Output = GridPos;
    #[inline]
    fn sub(self, rhs: GridPos) -> Self::Output {
        GridPos(self.0 - rhs.0)
    }
}

impl std::ops::Sub<IVec3> for GridPos {
    type Output = GridPos;
    #[inline]
    fn sub(self, rhs: IVec3) -> Self::Output {
        GridPos(self.0 - rhs)
    }
}

// Helper functions for common conversions (for gradual migration)

/// Convert world coordinates (Vec3) to grid coordinates (IVec3)
/// This is the canonical way to convert floating point to integer coordinates.
#[inline]
pub fn world_to_grid(pos: Vec3) -> IVec3 {
    IVec3::new(
        pos.x.floor() as i32,
        pos.y.floor() as i32,
        pos.z.floor() as i32,
    )
}

/// Convert grid coordinates (IVec3) to world coordinates (Vec3)
/// Returns the corner of the block (not center).
#[inline]
pub fn grid_to_world(pos: IVec3) -> Vec3 {
    Vec3::new(pos.x as f32, pos.y as f32, pos.z as f32)
}

/// Convert grid coordinates to world coordinates (center of block)
#[inline]
pub fn grid_to_world_center(pos: IVec3) -> Vec3 {
    Vec3::new(pos.x as f32 + 0.5, pos.y as f32 + 0.5, pos.z as f32 + 0.5)
}

/// DDA (Digital Differential Analyzer) result for voxel raycast
#[derive(Clone, Copy, Debug)]
pub struct DdaHit {
    /// The voxel position that was hit
    pub position: IVec3,
    /// The face normal (pointing away from the hit surface)
    pub normal: IVec3,
    /// Distance along the ray to the hit
    pub distance: f32,
}

/// Perform DDA voxel traversal to find the first voxel that satisfies the check function.
///
/// # Arguments
/// * `ray_origin` - Starting point of the ray
/// * `ray_direction` - Direction of the ray (should be normalized)
/// * `max_distance` - Maximum distance to search
/// * `check_fn` - Function that returns true if the voxel at given position should be hit
///
/// # Returns
/// The first DdaHit if a voxel passes the check, None otherwise
pub fn dda_raycast<F>(
    ray_origin: Vec3,
    ray_direction: Vec3,
    max_distance: f32,
    check_fn: F,
) -> Option<DdaHit>
where
    F: Fn(IVec3) -> bool,
{
    // Current voxel position
    let mut current = IVec3::new(
        ray_origin.x.floor() as i32,
        ray_origin.y.floor() as i32,
        ray_origin.z.floor() as i32,
    );

    // Direction sign for stepping (+1 or -1 for each axis)
    let step = IVec3::new(
        if ray_direction.x >= 0.0 { 1 } else { -1 },
        if ray_direction.y >= 0.0 { 1 } else { -1 },
        if ray_direction.z >= 0.0 { 1 } else { -1 },
    );

    // How far along the ray we need to travel for one voxel step on each axis
    let t_delta = Vec3::new(
        if ray_direction.x.abs() < 1e-8 {
            f32::MAX
        } else {
            (1.0 / ray_direction.x).abs()
        },
        if ray_direction.y.abs() < 1e-8 {
            f32::MAX
        } else {
            (1.0 / ray_direction.y).abs()
        },
        if ray_direction.z.abs() < 1e-8 {
            f32::MAX
        } else {
            (1.0 / ray_direction.z).abs()
        },
    );

    // Distance to next voxel boundary for each axis
    let mut t_max = Vec3::new(
        if ray_direction.x >= 0.0 {
            ((current.x + 1) as f32 - ray_origin.x) / ray_direction.x.abs().max(1e-8)
        } else {
            (ray_origin.x - current.x as f32) / ray_direction.x.abs().max(1e-8)
        },
        if ray_direction.y >= 0.0 {
            ((current.y + 1) as f32 - ray_origin.y) / ray_direction.y.abs().max(1e-8)
        } else {
            (ray_origin.y - current.y as f32) / ray_direction.y.abs().max(1e-8)
        },
        if ray_direction.z >= 0.0 {
            ((current.z + 1) as f32 - ray_origin.z) / ray_direction.z.abs().max(1e-8)
        } else {
            (ray_origin.z - current.z as f32) / ray_direction.z.abs().max(1e-8)
        },
    );

    // Track which axis we stepped on last (for face normal)
    let mut last_step_axis = 0; // 0=x, 1=y, 2=z
    let mut current_distance = 0.0f32;

    // Maximum number of steps (prevent infinite loop)
    let max_steps = (max_distance * 2.0) as i32;

    for _ in 0..max_steps {
        // Check current voxel
        if check_fn(current) {
            let normal = match last_step_axis {
                0 => IVec3::new(-step.x, 0, 0),
                1 => IVec3::new(0, -step.y, 0),
                _ => IVec3::new(0, 0, -step.z),
            };
            return Some(DdaHit {
                position: current,
                normal,
                distance: current_distance,
            });
        }

        // Step to next voxel
        if t_max.x < t_max.y && t_max.x < t_max.z {
            if t_max.x > max_distance {
                break;
            }
            current_distance = t_max.x;
            current.x += step.x;
            t_max.x += t_delta.x;
            last_step_axis = 0;
        } else if t_max.y < t_max.z {
            if t_max.y > max_distance {
                break;
            }
            current_distance = t_max.y;
            current.y += step.y;
            t_max.y += t_delta.y;
            last_step_axis = 1;
        } else {
            if t_max.z > max_distance {
                break;
            }
            current_distance = t_max.z;
            current.z += step.z;
            t_max.z += t_delta.z;
            last_step_axis = 2;
        }
    }

    None
}

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

/// Parse item name to BlockType using strum's EnumString
pub fn parse_item_name(name: &str) -> Option<crate::BlockType> {
    use std::str::FromStr;
    crate::BlockType::from_str(name).ok()
}
