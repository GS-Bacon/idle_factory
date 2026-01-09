//! Visibility and occlusion culling for chunk rendering
//!
//! This module provides visibility determination for chunks based on:
//! - Section-level opacity (opaque sections can hide adjacent sections)
//! - Player position (underground vs surface)
//! - Chunk distance from camera
//!
//! For flat terrain without caves (current game state), this provides limited
//! benefit but sets up infrastructure for future underground areas.

use crate::constants::{CHUNK_HEIGHT, SECTIONS_PER_CHUNK, SECTION_HEIGHT};
use crate::world::section::ChunkSection;
use bevy::prelude::*;

/// Visibility flags for the 6 faces of a section
/// Bit 0-5: +X, -X, +Y, -Y, +Z, -Z
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SectionFaces(pub u8);

impl SectionFaces {
    pub const POSITIVE_X: u8 = 1 << 0;
    pub const NEGATIVE_X: u8 = 1 << 1;
    pub const POSITIVE_Y: u8 = 1 << 2;
    pub const NEGATIVE_Y: u8 = 1 << 3;
    pub const POSITIVE_Z: u8 = 1 << 4;
    pub const NEGATIVE_Z: u8 = 1 << 5;
    pub const ALL: u8 = 0b111111;

    /// Create with all faces visible
    pub fn all_visible() -> Self {
        Self(Self::ALL)
    }

    /// Create with no faces visible
    pub fn none_visible() -> Self {
        Self(0)
    }

    /// Check if any face is visible
    pub fn is_any_visible(&self) -> bool {
        self.0 != 0
    }

    /// Check if all faces are visible
    pub fn is_all_visible(&self) -> bool {
        self.0 == Self::ALL
    }

    /// Check if the positive X face is visible
    pub fn is_positive_x_visible(&self) -> bool {
        self.0 & Self::POSITIVE_X != 0
    }

    /// Check if the positive Y (top) face is visible
    pub fn is_positive_y_visible(&self) -> bool {
        self.0 & Self::POSITIVE_Y != 0
    }

    /// Mark a face as hidden (fully occluded by adjacent section)
    pub fn hide_face(&mut self, face: u8) {
        self.0 &= !face;
    }
}

/// Visibility information for a single section
#[derive(Clone, Copy, Debug)]
pub struct SectionVisibility {
    /// Which faces of this section are potentially visible
    pub visible_faces: SectionFaces,
    /// Is this section fully opaque (all blocks are solid)?
    pub is_opaque: bool,
    /// Is this section completely transparent (all air)?
    pub is_transparent: bool,
}

impl Default for SectionVisibility {
    fn default() -> Self {
        Self {
            visible_faces: SectionFaces::all_visible(),
            is_opaque: false,
            is_transparent: true,
        }
    }
}

impl SectionVisibility {
    /// Create visibility info for a section
    pub fn from_section(section: &ChunkSection) -> Self {
        match section {
            ChunkSection::Empty => Self {
                visible_faces: SectionFaces::all_visible(),
                is_opaque: false,
                is_transparent: true,
            },
            ChunkSection::Uniform(id) => {
                // All blocks are the same solid type
                // Assuming all non-air blocks are opaque for simplicity
                let _ = id; // Use the ID if we add transparent blocks later
                Self {
                    visible_faces: SectionFaces::all_visible(),
                    is_opaque: true,
                    is_transparent: false,
                }
            }
            ChunkSection::Paletted(_) => {
                // Mixed blocks - need to check if all are opaque
                // For now, assume not fully opaque
                Self {
                    visible_faces: SectionFaces::all_visible(),
                    is_opaque: false,
                    is_transparent: false,
                }
            }
        }
    }

    /// Check if this section should be rendered
    pub fn should_render(&self) -> bool {
        !self.is_transparent && self.visible_faces.is_any_visible()
    }
}

/// Visibility information for an entire chunk
#[derive(Clone, Debug)]
pub struct ChunkVisibility {
    /// Visibility info for each section (bottom to top)
    pub sections: Vec<SectionVisibility>,
    /// Quick check: should this chunk be rendered at all?
    pub any_visible: bool,
}

impl Default for ChunkVisibility {
    fn default() -> Self {
        Self {
            sections: vec![SectionVisibility::default(); SECTIONS_PER_CHUNK],
            any_visible: true,
        }
    }
}

impl ChunkVisibility {
    /// Create visibility info for a chunk from its sections
    pub fn from_sections(sections: &[ChunkSection]) -> Self {
        let visibility: Vec<SectionVisibility> = sections
            .iter()
            .map(SectionVisibility::from_section)
            .collect();

        let any_visible = visibility.iter().any(|s| s.should_render());

        Self {
            sections: visibility,
            any_visible,
        }
    }

    /// Update visibility based on adjacent chunks
    /// This hides faces that are completely occluded by adjacent opaque sections
    pub fn update_with_neighbors(
        &mut self,
        neighbor_pos_x: Option<&ChunkVisibility>,
        neighbor_neg_x: Option<&ChunkVisibility>,
        neighbor_pos_z: Option<&ChunkVisibility>,
        neighbor_neg_z: Option<&ChunkVisibility>,
    ) {
        let num_sections = self.sections.len();

        // First, collect opacity info to avoid borrow issues
        let opacity: Vec<bool> = self.sections.iter().map(|s| s.is_opaque).collect();

        for section_idx in 0..num_sections {
            // Check adjacent sections in neighboring chunks
            // If the adjacent section is fully opaque, hide that face

            // +X neighbor
            if let Some(neighbor) = neighbor_pos_x {
                if neighbor.sections[section_idx].is_opaque {
                    self.sections[section_idx]
                        .visible_faces
                        .hide_face(SectionFaces::POSITIVE_X);
                }
            }

            // -X neighbor
            if let Some(neighbor) = neighbor_neg_x {
                if neighbor.sections[section_idx].is_opaque {
                    self.sections[section_idx]
                        .visible_faces
                        .hide_face(SectionFaces::NEGATIVE_X);
                }
            }

            // +Z neighbor
            if let Some(neighbor) = neighbor_pos_z {
                if neighbor.sections[section_idx].is_opaque {
                    self.sections[section_idx]
                        .visible_faces
                        .hide_face(SectionFaces::POSITIVE_Z);
                }
            }

            // -Z neighbor
            if let Some(neighbor) = neighbor_neg_z {
                if neighbor.sections[section_idx].is_opaque {
                    self.sections[section_idx]
                        .visible_faces
                        .hide_face(SectionFaces::NEGATIVE_Z);
                }
            }

            // Check internal vertical adjacency using pre-collected opacity info
            // +Y (section above)
            if section_idx + 1 < num_sections && opacity[section_idx + 1] {
                self.sections[section_idx]
                    .visible_faces
                    .hide_face(SectionFaces::POSITIVE_Y);
            }

            // -Y (section below)
            if section_idx > 0 && opacity[section_idx - 1] {
                self.sections[section_idx]
                    .visible_faces
                    .hide_face(SectionFaces::NEGATIVE_Y);
            }
        }

        // Update overall visibility
        self.any_visible = self.sections.iter().any(|s| s.should_render());
    }

    /// Check if a specific section should be rendered
    pub fn should_render_section(&self, section_idx: usize) -> bool {
        self.sections
            .get(section_idx)
            .map(|s| s.should_render())
            .unwrap_or(false)
    }

    /// Get the Y coordinate range that needs rendering
    /// Returns (min_y, max_y) in world coordinates relative to chunk base
    pub fn render_y_range(&self) -> (i32, i32) {
        let mut min_section = SECTIONS_PER_CHUNK;
        let mut max_section = 0;

        for (idx, section) in self.sections.iter().enumerate() {
            if section.should_render() {
                min_section = min_section.min(idx);
                max_section = max_section.max(idx);
            }
        }

        if min_section > max_section {
            // Nothing to render
            (0, 0)
        } else {
            let min_y = (min_section as i32) * SECTION_HEIGHT;
            let max_y = ((max_section + 1) as i32) * SECTION_HEIGHT;
            (min_y, max_y.min(CHUNK_HEIGHT))
        }
    }
}

/// Determine if a chunk should be culled based on player position
/// Returns true if the chunk should be skipped
pub fn should_cull_chunk(chunk_coord: IVec2, player_pos: Vec3, view_distance: i32) -> bool {
    let player_chunk = IVec2::new(
        (player_pos.x / 16.0).floor() as i32,
        (player_pos.z / 16.0).floor() as i32,
    );

    let dx = (chunk_coord.x - player_chunk.x).abs();
    let dz = (chunk_coord.y - player_chunk.y).abs();

    // Cull if outside view distance
    dx > view_distance || dz > view_distance
}

/// Determine if player is underground
/// This can be used to skip rendering surface chunks when deep underground
pub fn is_player_underground(player_y: f32, ground_level: i32) -> bool {
    player_y < (ground_level - 5) as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_section_faces_default() {
        let faces = SectionFaces::all_visible();
        assert!(faces.is_all_visible());
        assert!(faces.is_any_visible());
    }

    #[test]
    fn test_section_faces_hide() {
        let mut faces = SectionFaces::all_visible();
        faces.hide_face(SectionFaces::POSITIVE_X);
        assert!(!faces.is_all_visible());
        assert!(faces.is_any_visible());
        assert!(!faces.is_positive_x_visible());
    }

    #[test]
    fn test_section_visibility_from_empty() {
        let section = ChunkSection::Empty;
        let vis = SectionVisibility::from_section(&section);
        assert!(vis.is_transparent);
        assert!(!vis.is_opaque);
        assert!(!vis.should_render());
    }

    #[test]
    fn test_section_visibility_from_uniform() {
        let section = ChunkSection::Uniform(crate::core::id::Id::new(1));
        let vis = SectionVisibility::from_section(&section);
        assert!(!vis.is_transparent);
        assert!(vis.is_opaque);
        assert!(vis.should_render());
    }

    #[test]
    fn test_chunk_visibility_render_range() {
        let mut sections = vec![ChunkSection::Empty; SECTIONS_PER_CHUNK];
        // Make bottom section have some blocks
        sections[0] = ChunkSection::Uniform(crate::core::id::Id::new(1));

        let vis = ChunkVisibility::from_sections(&sections);
        let (min_y, max_y) = vis.render_y_range();
        assert_eq!(min_y, 0);
        assert_eq!(max_y, SECTION_HEIGHT);
    }

    #[test]
    fn test_should_cull_chunk() {
        let player_pos = Vec3::new(0.0, 10.0, 0.0);
        let view_distance = 3;

        // Chunk at origin should not be culled
        assert!(!should_cull_chunk(IVec2::ZERO, player_pos, view_distance));

        // Chunk far away should be culled
        assert!(should_cull_chunk(
            IVec2::new(10, 10),
            player_pos,
            view_distance
        ));
    }

    #[test]
    fn test_is_player_underground() {
        assert!(!is_player_underground(10.0, 7));
        assert!(is_player_underground(-5.0, 7));
    }
}
