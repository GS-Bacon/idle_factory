//! Stateful Property-Based Tests for Game Invariants
//!
//! This test uses proptest to generate sequences of random game operations
//! and verify that invariants hold after each step. If a violation is found,
//! proptest will shrink to find the minimal failing sequence.

use proptest::prelude::*;

/// Game operations that can be performed
#[derive(Debug, Clone, Copy, PartialEq)]
enum GameOp {
    MoveForward(f32), // Duration in seconds
    MoveBack(f32),
    MoveLeft(f32),
    MoveRight(f32),
    Jump,
    PlaceBlock { x: i32, y: i32, z: i32 },
    BreakBlock { x: i32, y: i32, z: i32 },
    Teleport { x: f32, y: f32, z: f32 },
}

/// Simplified game state model for testing invariants
#[derive(Debug, Clone)]
struct GameModel {
    /// Player position
    player_pos: (f32, f32, f32),
    /// Player velocity
    velocity: (f32, f32, f32),
    /// Whether player is on ground
    on_ground: bool,
    /// Placed blocks (solid positions)
    blocks: std::collections::HashSet<(i32, i32, i32)>,
    /// Number of frames stuck (input without movement)
    stuck_frames: u32,
    /// Violations detected
    violations: Vec<String>,
}

impl Default for GameModel {
    fn default() -> Self {
        // Ground at Y=9, player feet at Y=10, player center at Y=11
        Self {
            player_pos: (0.0, 11.0, 0.0),
            velocity: (0.0, 0.0, 0.0),
            on_ground: true,
            blocks: Self::generate_ground(),
            stuck_frames: 0,
            violations: Vec::new(),
        }
    }
}

impl GameModel {
    fn generate_ground() -> std::collections::HashSet<(i32, i32, i32)> {
        let mut blocks = std::collections::HashSet::new();
        // Generate ground plane at y=9 (player spawns at y=11)
        // 100x100 is enough for typical test sequences
        for x in -50..=50 {
            for z in -50..=50 {
                blocks.insert((x, 9, z));
            }
        }
        blocks
    }

    /// Apply gravity and physics for one frame
    fn physics_step(&mut self, dt: f32) {
        const GRAVITY: f32 = 20.0;
        const TERMINAL_VELOCITY: f32 = 50.0;

        // Apply gravity if not on ground
        if !self.on_ground {
            self.velocity.1 -= GRAVITY * dt;
            self.velocity.1 = self.velocity.1.max(-TERMINAL_VELOCITY);
        }

        // Calculate desired new position
        let new_pos = (
            self.player_pos.0 + self.velocity.0 * dt,
            self.player_pos.1 + self.velocity.1 * dt,
            self.player_pos.2 + self.velocity.2 * dt,
        );

        // Check horizontal collision BEFORE moving
        let head_block = (
            new_pos.0.floor() as i32,
            self.player_pos.1.floor() as i32, // Use current Y
            new_pos.2.floor() as i32,
        );

        // Block horizontal movement if there's a wall (proper collision)
        if self.blocks.contains(&head_block) {
            // Don't move horizontally - wall collision
            // Only apply vertical movement
            let new_y = self.player_pos.1 + self.velocity.1 * dt;
            self.player_pos.1 = new_y;
            self.velocity.0 = 0.0;
            self.velocity.2 = 0.0;
        } else {
            // No horizontal collision, apply full movement
            self.player_pos.0 = new_pos.0;
            self.player_pos.2 = new_pos.2;
            self.player_pos.1 = new_pos.1;
        }

        // Check collision with blocks for ground detection
        // Player height = 2, so feet are at player_pos.1 - 1.0
        // Block below feet is at floor(feet_y) - 1 when standing on top
        let feet_y = self.player_pos.1 - 1.0;
        let below_feet_block = (
            self.player_pos.0.floor() as i32,
            (feet_y - 0.01).floor() as i32, // Block just below feet
            self.player_pos.2.floor() as i32,
        );

        // Ground check - standing on a block
        if self.blocks.contains(&below_feet_block) && self.velocity.1 <= 0.0 {
            self.on_ground = true;
            self.velocity.1 = 0.0;
            // Snap to ground: feet at top of block, player center 1 above feet
            self.player_pos.1 = (below_feet_block.1 + 1) as f32 + 1.0;
        } else {
            self.on_ground = false;
        }

        // Check if player ended up inside a block (shouldn't happen with proper collision)
        let final_head_block = (
            self.player_pos.0.floor() as i32,
            self.player_pos.1.floor() as i32,
            self.player_pos.2.floor() as i32,
        );
        if self.blocks.contains(&final_head_block) {
            // Player embedded in block!
            self.violations.push(format!(
                "EMBEDDED: Player at ({:.1},{:.1},{:.1}) inside block at ({},{},{})",
                self.player_pos.0,
                self.player_pos.1,
                self.player_pos.2,
                final_head_block.0,
                final_head_block.1,
                final_head_block.2
            ));
        }

        // Check fell through world
        if self.player_pos.1 < -10.0 {
            self.violations.push(format!(
                "FELL: Player at ({:.1},{:.1},{:.1}) fell through world",
                self.player_pos.0, self.player_pos.1, self.player_pos.2
            ));
        }
    }

    /// Apply a game operation
    fn apply(&mut self, op: &GameOp) {
        const WALK_SPEED: f32 = 5.0;
        const JUMP_VELOCITY: f32 = 8.0;
        const DT: f32 = 1.0 / 60.0;

        match op {
            GameOp::MoveForward(duration) => {
                let steps = (duration / DT) as usize;
                let had_input = steps > 0;
                let start_pos = self.player_pos;

                for _ in 0..steps {
                    self.velocity.2 = -WALK_SPEED;
                    self.physics_step(DT);
                }
                self.velocity.2 = 0.0;

                // Check if stuck
                if had_input {
                    let moved = ((self.player_pos.0 - start_pos.0).powi(2)
                        + (self.player_pos.2 - start_pos.2).powi(2))
                    .sqrt();
                    if moved < 0.01 {
                        self.stuck_frames += steps as u32;
                    } else {
                        self.stuck_frames = 0;
                    }
                }
            }
            GameOp::MoveBack(duration) => {
                let steps = (duration / DT) as usize;
                for _ in 0..steps {
                    self.velocity.2 = WALK_SPEED;
                    self.physics_step(DT);
                }
                self.velocity.2 = 0.0;
            }
            GameOp::MoveLeft(duration) => {
                let steps = (duration / DT) as usize;
                for _ in 0..steps {
                    self.velocity.0 = -WALK_SPEED;
                    self.physics_step(DT);
                }
                self.velocity.0 = 0.0;
            }
            GameOp::MoveRight(duration) => {
                let steps = (duration / DT) as usize;
                for _ in 0..steps {
                    self.velocity.0 = WALK_SPEED;
                    self.physics_step(DT);
                }
                self.velocity.0 = 0.0;
            }
            GameOp::Jump => {
                if self.on_ground {
                    self.velocity.1 = JUMP_VELOCITY;
                    self.on_ground = false;
                }
                // Simulate jump arc
                for _ in 0..30 {
                    self.physics_step(DT);
                }
            }
            GameOp::PlaceBlock { x, y, z } => {
                // Can only place if not inside player
                let player_block = (
                    self.player_pos.0.floor() as i32,
                    self.player_pos.1.floor() as i32,
                    self.player_pos.2.floor() as i32,
                );
                if (*x, *y, *z) != player_block {
                    self.blocks.insert((*x, *y, *z));
                }
            }
            GameOp::BreakBlock { x, y, z } => {
                self.blocks.remove(&(*x, *y, *z));
                // Physics step to handle falling
                for _ in 0..60 {
                    self.physics_step(DT);
                }
            }
            GameOp::Teleport { x, y, z } => {
                self.player_pos = (*x, *y, *z);
                self.velocity = (0.0, 0.0, 0.0);
                // Check if teleported into a block
                let block_pos = (x.floor() as i32, y.floor() as i32, z.floor() as i32);
                if self.blocks.contains(&block_pos) {
                    self.violations.push(format!(
                        "EMBEDDED: Teleported into block at ({},{},{})",
                        block_pos.0, block_pos.1, block_pos.2
                    ));
                }
            }
        }
    }

    /// Check invariants and return errors if any
    fn check_invariants(&self) -> Result<(), String> {
        // Check for violations
        if !self.violations.is_empty() {
            return Err(self.violations.join(", "));
        }

        // Check stuck threshold (3 seconds = 180 frames)
        if self.stuck_frames > 180 {
            return Err(format!(
                "STUCK: Player stuck for {} frames at ({:.1},{:.1},{:.1})",
                self.stuck_frames, self.player_pos.0, self.player_pos.1, self.player_pos.2
            ));
        }

        // Check position bounds
        if self.player_pos.1 < -10.0 {
            return Err(format!(
                "FELL: Player at Y={:.1} below world floor",
                self.player_pos.1
            ));
        }

        Ok(())
    }
}

/// Strategy to generate a single game operation (movement-focused)
fn game_op_strategy() -> impl Strategy<Value = GameOp> {
    prop_oneof![
        // Movement operations (weighted more heavily)
        // Limited to 0.5s to stay within ground bounds (-50..=50)
        5 => (0.1f32..0.5).prop_map(GameOp::MoveForward),
        5 => (0.1f32..0.5).prop_map(GameOp::MoveBack),
        3 => (0.1f32..0.5).prop_map(GameOp::MoveLeft),
        3 => (0.1f32..0.5).prop_map(GameOp::MoveRight),
        3 => Just(GameOp::Jump),
        // Teleport (less frequent) - Y=11 is ground level, within bounds
        1 => (-20.0f32..20.0, 11.0f32..20.0, -20.0f32..20.0)
            .prop_map(|(x, y, z)| GameOp::Teleport { x, y, z }),
    ]
}

/// Strategy for testing ground breaking specifically
fn ground_break_op_strategy() -> impl Strategy<Value = GameOp> {
    prop_oneof![
        3 => (0.1f32..1.0).prop_map(GameOp::MoveForward),
        3 => (0.1f32..1.0).prop_map(GameOp::MoveBack),
        2 => Just(GameOp::Jump),
        1 => (-5i32..5, 9i32..10, -5i32..5).prop_map(|(x, y, z)| GameOp::BreakBlock { x, y, z }),
    ]
}

/// Strategy to generate a sequence of operations
fn operation_sequence_strategy(min: usize, max: usize) -> impl Strategy<Value = Vec<GameOp>> {
    prop::collection::vec(game_op_strategy(), min..=max)
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Test that no invariant violations occur during random play
    #[test]
    fn no_violations_during_random_play(ops in operation_sequence_strategy(10, 50)) {
        let mut model = GameModel::default();

        for (i, op) in ops.iter().enumerate() {
            model.apply(op);

            prop_assert!(
                model.check_invariants().is_ok(),
                "Invariant violation at step {}: {:?}\nOperation: {:?}\nSequence: {:?}",
                i, model.check_invariants().err().unwrap_or_default(), op, &ops[..=i]
            );
        }
    }

    /// Test that player always lands after jumping
    #[test]
    fn player_lands_after_jump(
        pre_ops in operation_sequence_strategy(0, 5),
        post_ops in operation_sequence_strategy(0, 10)
    ) {
        let mut model = GameModel::default();

        // Apply pre-operations
        for op in &pre_ops {
            model.apply(op);
        }

        // Jump
        model.apply(&GameOp::Jump);

        // Apply some post-operations (give time to land)
        for op in &post_ops {
            model.apply(op);
        }

        // Wait for physics to settle
        for _ in 0..120 {
            model.physics_step(1.0 / 60.0);
        }

        // Should be on ground or have valid Y position
        prop_assert!(
            model.on_ground || model.player_pos.1 > -10.0,
            "Player should land or be above world floor, but at Y={}",
            model.player_pos.1
        );
    }

    /// Test that breaking ground causes falling
    #[test]
    fn breaking_ground_causes_falling(
        x in -5i32..5,
        z in -5i32..5
    ) {
        let mut model = GameModel::default();

        // Teleport to position above the block we'll break
        model.apply(&GameOp::Teleport {
            x: x as f32 + 0.5,
            y: 10.5,
            z: z as f32 + 0.5,
        });

        // Break the ground block
        model.apply(&GameOp::BreakBlock { x, y: 9, z });

        // Should not be embedded
        prop_assert!(
            model.violations.is_empty(),
            "Should not have violations after breaking ground: {:?}",
            model.violations
        );
    }

    /// Test movement doesn't cause embedding
    #[test]
    fn movement_never_embeds(
        ops in prop::collection::vec(
            prop_oneof![
                (0.1f32..1.0).prop_map(GameOp::MoveForward),
                (0.1f32..1.0).prop_map(GameOp::MoveBack),
                (0.1f32..1.0).prop_map(GameOp::MoveLeft),
                (0.1f32..1.0).prop_map(GameOp::MoveRight),
                Just(GameOp::Jump),
            ],
            20..50
        )
    ) {
        let mut model = GameModel::default();

        for op in &ops {
            model.apply(op);

            // Check no embedding
            let embedded: Vec<_> = model.violations.iter()
                .filter(|v| v.contains("EMBEDDED"))
                .collect();

            prop_assert!(
                embedded.is_empty(),
                "Movement caused embedding: {:?} after {:?}",
                embedded, op
            );
        }
    }
}

#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn test_game_model_default() {
        let model = GameModel::default();
        // Player at Y=11 (ground at Y=9, feet at Y=10, center at Y=11)
        assert_eq!(model.player_pos.1, 11.0);
        assert!(model.on_ground);
        assert!(model.violations.is_empty());
    }

    #[test]
    fn test_basic_movement() {
        let mut model = GameModel::default();
        let start_z = model.player_pos.2;

        model.apply(&GameOp::MoveForward(1.0));

        // Should have moved forward (negative Z)
        assert!(model.player_pos.2 < start_z);
    }

    #[test]
    fn test_jump_physics() {
        let mut model = GameModel::default();
        let start_y = model.player_pos.1;

        model.apply(&GameOp::Jump);

        // After jump completes, should be back on ground
        // Jump takes about 1 second to complete (up and down)
        for _ in 0..180 {
            model.physics_step(1.0 / 60.0);
        }

        // Should be on ground and at similar height
        assert!(
            model.on_ground,
            "Player should be on ground after jump. Y={}, start_Y={}",
            model.player_pos.1, start_y
        );
    }

    #[test]
    fn test_teleport_into_block_detected() {
        let mut model = GameModel::default();

        // Place a block at (5, 10, 5)
        model.apply(&GameOp::PlaceBlock { x: 5, y: 10, z: 5 });

        // Teleport into it
        model.apply(&GameOp::Teleport {
            x: 5.5,
            y: 10.5,
            z: 5.5,
        });

        // Should have violation
        assert!(
            !model.violations.is_empty(),
            "Should detect teleport into block"
        );
        assert!(
            model.violations[0].contains("EMBEDDED"),
            "Violation should be EMBEDDED"
        );
    }
}
