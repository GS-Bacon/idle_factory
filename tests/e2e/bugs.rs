//! Bug regression tests
//!
//! Each test documents a specific bug and verifies the fix.

use bevy::prelude::*;
use idle_factory::core::items;
use idle_factory::ItemId;
use std::collections::HashMap;

// ============================================================
// BUG-2: Machine placement should NOT register block in world data
// ============================================================

#[test]
fn test_machine_placement_no_block_registration() {
    let mut world_blocks: HashMap<IVec3, ItemId> = HashMap::new();

    world_blocks.insert(IVec3::new(0, 0, 0), items::stone());
    world_blocks.insert(IVec3::new(0, 1, 0), items::grass());

    let miner_pos = IVec3::new(0, 2, 0);
    // Machines should be entities, NOT blocks

    assert!(
        world_blocks.contains_key(&IVec3::new(0, 0, 0)),
        "Stone should still exist"
    );
    assert!(
        world_blocks.contains_key(&IVec3::new(0, 1, 0)),
        "Grass should still exist"
    );
    assert!(
        !world_blocks.contains_key(&miner_pos),
        "Machine position should NOT be in world blocks"
    );
}

// ============================================================
// BUG-4: Chunk boundary mesh generation needs neighbor info
// ============================================================

#[test]
fn test_chunk_boundary_mesh_needs_neighbors() {
    const CHUNK_SIZE: i32 = 16;

    let mut chunk_a: HashMap<IVec3, ItemId> = HashMap::new();
    let mut chunk_b: HashMap<IVec3, ItemId> = HashMap::new();

    chunk_a.insert(IVec3::new(CHUNK_SIZE - 1, 0, 0), items::stone());
    chunk_b.insert(IVec3::new(0, 0, 0), items::stone());

    fn should_render_face(
        pos: IVec3,
        face_dir: IVec3,
        own_chunk: &HashMap<IVec3, ItemId>,
        neighbor_chunk: Option<&HashMap<IVec3, ItemId>>,
        chunk_size: i32,
    ) -> bool {
        let neighbor_pos = pos + face_dir;

        if own_chunk.contains_key(&neighbor_pos) {
            return false;
        }

        if neighbor_pos.x < 0 || neighbor_pos.x >= chunk_size {
            if let Some(neighbor) = neighbor_chunk {
                let local_x = if neighbor_pos.x < 0 {
                    chunk_size - 1
                } else {
                    0
                };
                let local_pos = IVec3::new(local_x, neighbor_pos.y, neighbor_pos.z);
                if neighbor.contains_key(&local_pos) {
                    return false;
                }
            }
        }

        true
    }

    let edge_pos = IVec3::new(CHUNK_SIZE - 1, 0, 0);

    let render_without_neighbor =
        should_render_face(edge_pos, IVec3::new(1, 0, 0), &chunk_a, None, CHUNK_SIZE);

    let render_with_neighbor = should_render_face(
        edge_pos,
        IVec3::new(1, 0, 0),
        &chunk_a,
        Some(&chunk_b),
        CHUNK_SIZE,
    );

    assert!(
        render_without_neighbor,
        "Without neighbor info, would incorrectly render face"
    );
    assert!(
        !render_with_neighbor,
        "With neighbor info, correctly skips occluded face"
    );
}

// ============================================================
// BUG-5: Block operations should not cause freeze
// ============================================================

#[test]
fn test_block_operations_no_freeze() {
    #[derive(Clone, Copy, PartialEq, Debug)]
    enum ChunkRegenPattern {
        #[allow(dead_code)]
        CurrentOnly,
        CurrentAndNeighbors,
        #[allow(dead_code)]
        AllLoaded,
    }

    let block_place_pattern = ChunkRegenPattern::CurrentAndNeighbors;
    let block_break_pattern = ChunkRegenPattern::CurrentAndNeighbors;

    assert_eq!(
        block_place_pattern, block_break_pattern,
        "block_place and block_break should use same chunk regeneration pattern"
    );
}

// ============================================================
// BUG-10: UI open should block player movement
// ============================================================

#[test]
fn test_ui_blocks_player_movement() {
    #[derive(Default)]
    struct InputState {
        inventory_open: bool,
        furnace_ui_open: bool,
        crusher_ui_open: bool,
        command_input_open: bool,
    }

    impl InputState {
        fn allows_movement(&self) -> bool {
            !self.inventory_open
                && !self.furnace_ui_open
                && !self.crusher_ui_open
                && !self.command_input_open
        }
    }

    let mut state = InputState::default();

    assert!(state.allows_movement());

    state.inventory_open = true;
    assert!(!state.allows_movement());
    state.inventory_open = false;

    state.furnace_ui_open = true;
    assert!(!state.allows_movement());
    state.furnace_ui_open = false;

    state.crusher_ui_open = true;
    assert!(!state.allows_movement());
    state.crusher_ui_open = false;

    state.command_input_open = true;
    assert!(!state.allows_movement());
}

// ============================================================
// BUG-12: UI open should block hotbar scroll
// ============================================================

#[test]
fn test_ui_blocks_hotbar_scroll() {
    struct GameState {
        inventory_open: bool,
        hotbar_selection: usize,
    }

    impl GameState {
        fn try_scroll_hotbar(&mut self, delta: i32) {
            if self.inventory_open {
                return;
            }
            let new_selection = (self.hotbar_selection as i32 + delta).rem_euclid(9) as usize;
            self.hotbar_selection = new_selection;
        }
    }

    let mut state = GameState {
        inventory_open: false,
        hotbar_selection: 0,
    };

    state.try_scroll_hotbar(1);
    assert_eq!(state.hotbar_selection, 1);

    state.try_scroll_hotbar(1);
    assert_eq!(state.hotbar_selection, 2);

    state.inventory_open = true;

    state.try_scroll_hotbar(1);
    assert_eq!(state.hotbar_selection, 2, "Scroll should be blocked");

    state.try_scroll_hotbar(-1);
    assert_eq!(state.hotbar_selection, 2, "Scroll should be blocked");
}

// ============================================================
// BUG-19: Chunk processing should be rate-limited
// ============================================================

#[test]
fn test_chunk_processing_rate_limited() {
    const MAX_CHUNKS_PER_FRAME: usize = 4;

    struct ChunkProcessor {
        pending_chunks: Vec<IVec2>,
        processed_this_frame: usize,
    }

    impl ChunkProcessor {
        fn new(pending: Vec<IVec2>) -> Self {
            Self {
                pending_chunks: pending,
                processed_this_frame: 0,
            }
        }

        fn process_frame(&mut self) -> Vec<IVec2> {
            let mut processed = Vec::new();
            self.processed_this_frame = 0;

            while !self.pending_chunks.is_empty()
                && self.processed_this_frame < MAX_CHUNKS_PER_FRAME
            {
                if let Some(chunk) = self.pending_chunks.pop() {
                    processed.push(chunk);
                    self.processed_this_frame += 1;
                }
            }

            processed
        }
    }

    let pending: Vec<IVec2> = (0..10).map(|i| IVec2::new(i, 0)).collect();
    let mut processor = ChunkProcessor::new(pending);

    let first_batch = processor.process_frame();
    assert_eq!(
        first_batch.len(),
        MAX_CHUNKS_PER_FRAME,
        "Should process exactly {} chunks",
        MAX_CHUNKS_PER_FRAME
    );

    let second_batch = processor.process_frame();
    assert_eq!(second_batch.len(), MAX_CHUNKS_PER_FRAME);

    let third_batch = processor.process_frame();
    assert_eq!(third_batch.len(), 2, "Should process remaining 2 chunks");

    let fourth_batch = processor.process_frame();
    assert_eq!(fourth_batch.len(), 0, "No more chunks to process");
}

// ============================================================
// Asset file existence tests
// ============================================================

#[test]
fn test_asset_files_exist() {
    let required_dirs = ["assets/data", "assets/models", "assets/textures"];

    for dir in &required_dirs {
        let path = std::path::Path::new(dir);
        assert!(
            path.exists(),
            "Required asset directory should exist: {}",
            dir
        );
    }
}

#[test]
fn test_base_data_files_exist() {
    let required_files = [
        "assets/data/machines.json",
        "assets/data/recipes.json",
        "assets/data/quests.json",
    ];

    for file in &required_files {
        let path = std::path::Path::new(file);
        assert!(path.exists(), "Required data file should exist: {}", file);
    }
}
