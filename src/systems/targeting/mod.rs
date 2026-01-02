//! Target block and highlight systems
//!
//! Handles raycasting for block selection and visual highlighting
//!
//! ## Modules
//! - `raycast`: Update target block based on player view
//! - `highlight`: Visual highlighting of target blocks
//! - `conveyor`: Conveyor rotation and shape updates
//! - `guide`: Guide markers for placement

mod conveyor;
mod guide;
mod highlight;
mod raycast;

pub use conveyor::{rotate_conveyor_placement, update_conveyor_shapes};
pub use guide::update_guide_markers;
pub use highlight::{setup_highlight_cache, update_target_highlight, HighlightMeshCache};
pub use raycast::update_target_block;
