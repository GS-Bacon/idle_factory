//! AI自動評価・改善フィードバックループシステム
//!
//! AIが自動でゲームをプレイし、複数のペルソナ視点で評価・改善提案を行う。
//! コアコンセプト（creative-sandbox, no-combat, stress-free）を守りながら改善を推進。

use bevy::prelude::*;

mod types;
mod systems;
mod analyzer;

pub use types::*;
pub use systems::*;
pub use analyzer::*;

/// フィードバックループプラグイン
pub struct FeedbackLoopPlugin;

impl Plugin for FeedbackLoopPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PersonaRegistry>()
            .init_resource::<GoalRegistry>()
            .init_resource::<PlaySessionRecorder>()
            .init_resource::<EvaluationConfig>()
            .init_resource::<MetaEvaluationState>()
            .add_event::<StartEvaluationEvent>()
            .add_event::<GoalCompletedEvent>()
            .add_event::<StuckDetectedEvent>()
            .add_systems(Update, (
                detect_stuck_points,
                check_goal_conditions,
                record_game_events,
                update_efficiency_metrics,
            ).chain());
    }
}
