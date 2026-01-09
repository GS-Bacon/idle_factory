//! Game event system with depth protection
//!
//! These events will be used for multiplayer synchronization in the future.
//! The event system includes cycle prevention via depth tracking.

#![allow(dead_code)] // These events are prepared for future multiplayer support

pub mod game_events;
pub mod guarded_writer;
pub mod ui_events;

pub use game_events::*;
pub use guarded_writer::*;
pub use ui_events::*;

use crate::core::ItemId;
use bevy::prelude::*;
use std::collections::HashSet;
use std::sync::atomic::{AtomicU8, Ordering};

// ============================================================================
// Event System Configuration
// ============================================================================

/// イベントシステム設定
#[derive(Resource)]
pub struct EventSystemConfig {
    /// 最大連鎖深さ（循環防止）デフォルト: 16
    pub max_depth: u8,
    /// デバッグログ
    pub log_enabled: bool,
    /// 外部通知を除外するイベント（パフォーマンス用）
    pub external_exclude: HashSet<&'static str>,
}

impl Default for EventSystemConfig {
    fn default() -> Self {
        Self {
            max_depth: 16,
            log_enabled: false,
            external_exclude: HashSet::new(),
        }
    }
}

/// 現在の連鎖深さを追跡（AtomicU8で内部可変性を実現）
///
/// これにより、同一システム内で複数のGuardedEventWriterを使用可能。
#[derive(Resource, Default)]
pub struct EventDepth(pub AtomicU8);

impl EventDepth {
    /// 現在の深さを取得
    pub fn get(&self) -> u8 {
        self.0.load(Ordering::Relaxed)
    }

    /// 深さをインクリメント
    pub fn increment(&self) -> u8 {
        self.0.fetch_add(1, Ordering::Relaxed) + 1
    }

    /// 深さをリセット
    pub fn reset(&self) {
        self.0.store(0, Ordering::Relaxed);
    }
}

/// イベントエラー
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventError {
    MaxDepthExceeded,
}

/// フレーム開始時にリセット
pub fn reset_event_depth(depth: Res<EventDepth>) {
    depth.reset();
}

/// Plugin for event system base (depth tracking, config)
pub struct EventsPlugin;

impl Plugin for EventsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EventSystemConfig>()
            .init_resource::<EventDepth>()
            .add_systems(First, reset_event_depth);
    }
}

// ============================================================================
// Game Events
// ============================================================================

/// Event for block placement
#[derive(Event, Clone, Debug)]
pub struct BlockPlaceEvent {
    pub position: IVec3,
    pub item_id: ItemId,
    pub player_id: u64,
}

/// Event for block destruction
#[derive(Event, Clone, Debug)]
pub struct BlockBreakEvent {
    pub position: IVec3,
    pub player_id: u64,
}

/// Event for machine interaction
#[derive(Event, Clone, Debug)]
pub struct MachineInteractEvent {
    pub position: IVec3,
    pub action: MachineAction,
    pub player_id: u64,
}

/// Types of machine interactions
#[derive(Clone, Debug)]
pub enum MachineAction {
    Open,
    Close,
    AddItem(ItemId),
    TakeItem(ItemId),
}

/// Event for item transfer between machines/conveyors
#[derive(Event, Clone, Debug)]
pub struct ItemTransferEvent {
    pub from_pos: IVec3,
    pub to_pos: IVec3,
    pub item: ItemId,
    pub count: u32,
}

/// Event for quest progress
#[derive(Event, Clone, Debug)]
pub struct QuestProgressEvent {
    pub item_id: ItemId,
    pub amount: u32,
}

/// Event for spawning machines via E2E commands
#[derive(Event, Clone, Debug)]
pub struct SpawnMachineEvent {
    pub position: IVec3,
    pub machine_id: ItemId,
    pub direction: Option<u8>, // For conveyors: 0=North, 1=East, 2=South, 3=West
}

/// Plugin for game events
pub struct GameEventsPlugin;

impl Plugin for GameEventsPlugin {
    fn build(&self, app: &mut App) {
        // Add base event system (depth tracking, config)
        app.add_plugins(EventsPlugin);

        // Add game events
        app.add_event::<BlockPlaceEvent>()
            .add_event::<BlockBreakEvent>()
            .add_event::<MachineInteractEvent>()
            .add_event::<ItemTransferEvent>()
            .add_event::<QuestProgressEvent>()
            .add_event::<SpawnMachineEvent>()
            .add_plugins(GameEventsExtPlugin);
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_config_default() {
        let config = EventSystemConfig::default();
        assert_eq!(config.max_depth, 16);
        assert!(!config.log_enabled);
    }

    #[test]
    fn test_event_depth_default() {
        let depth = EventDepth::default();
        assert_eq!(depth.get(), 0);
    }

    #[test]
    fn test_event_error_eq() {
        assert_eq!(EventError::MaxDepthExceeded, EventError::MaxDepthExceeded);
    }

    #[test]
    fn test_external_exclude() {
        let mut config = EventSystemConfig::default();
        config.external_exclude.insert("ConveyorTransfer");
        config.external_exclude.insert("PlayerMoved");
        assert!(config.external_exclude.contains("ConveyorTransfer"));
        assert!(config.external_exclude.contains("PlayerMoved"));
        assert!(!config.external_exclude.contains("BlockPlaced"));
    }
}
