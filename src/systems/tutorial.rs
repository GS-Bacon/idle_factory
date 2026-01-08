//! Tutorial system - tracks player actions and advances tutorial steps

use bevy::prelude::*;

use crate::block_type::BlockType;
use crate::components::{
    InventoryUI, TutorialAction, TutorialPanel, TutorialProgress, TutorialProgressBarBg,
    TutorialProgressBarFill, TutorialProgressText, TutorialShown, TutorialStepText, TUTORIAL_STEPS,
};
use crate::core::ItemId;
use crate::player::LocalPlatformInventory;

/// Event for tutorial action notifications
#[derive(Event)]
pub enum TutorialEvent {
    /// Player moved
    PlayerMoved { distance: f32 },
    /// Player broke a block
    BlockBroken,
    /// Player opened inventory
    InventoryOpened,
    /// Player placed a machine
    MachinePlaced(BlockType),
    /// Player placed a conveyor
    ConveyorPlaced { position: IVec3 },
    /// An item was produced
    ItemProduced(BlockType),
}

/// Track player movement for tutorial
pub fn track_movement(
    player_query: Query<&Transform, (With<crate::components::Player>, Changed<Transform>)>,
    progress: Res<TutorialProgress>,
    mut last_pos: Local<Option<Vec3>>,
    mut events: EventWriter<TutorialEvent>,
) {
    if progress.completed {
        return;
    }

    let Ok(transform) = player_query.get_single() else {
        return;
    };

    let pos = transform.translation;
    if let Some(last) = *last_pos {
        let delta = pos.distance(last);
        if delta > 0.01 && delta < 10.0 {
            // Ignore teleports
            events.send(TutorialEvent::PlayerMoved { distance: delta });
        }
    }
    *last_pos = Some(pos);
}

/// Track inventory open for tutorial
pub fn track_inventory_open(
    inventory_query: Query<&Visibility, With<InventoryUI>>,
    mut last_open: Local<bool>,
    mut events: EventWriter<TutorialEvent>,
    progress: Res<TutorialProgress>,
) {
    if progress.completed {
        return;
    }

    let is_open = inventory_query
        .get_single()
        .map(|v| *v == Visibility::Visible)
        .unwrap_or(false);
    if is_open && !*last_open {
        events.send(TutorialEvent::InventoryOpened);
    }
    *last_open = is_open;
}

/// Process tutorial events and advance progress
pub fn process_tutorial_events(
    mut events: EventReader<TutorialEvent>,
    mut progress: ResMut<TutorialProgress>,
) {
    if progress.completed {
        events.clear();
        return;
    }

    let Some(step) = progress.current() else {
        return;
    };

    for event in events.read() {
        let should_advance = match (&step.action, event) {
            // Move tutorial
            (TutorialAction::Move { distance }, TutorialEvent::PlayerMoved { distance: d }) => {
                progress.move_distance += d;
                progress.move_distance >= *distance as f32
            }
            // Break block tutorial
            (TutorialAction::BreakBlock, TutorialEvent::BlockBroken) => true,
            // Open inventory tutorial
            (TutorialAction::OpenInventory, TutorialEvent::InventoryOpened) => true,
            // Place machine tutorial
            (TutorialAction::PlaceMachine(expected), TutorialEvent::MachinePlaced(placed)) => {
                *expected == *placed
            }
            // Place conveyors tutorial
            (
                TutorialAction::PlaceConveyors { count },
                TutorialEvent::ConveyorPlaced { position },
            ) => {
                // Check if consecutive
                let is_consecutive = if let Some(last) = progress.last_conveyor_pos {
                    let diff = *position - last;
                    diff.abs().max_element() == 1
                        && (diff.x.abs() + diff.y.abs() + diff.z.abs()) == 1
                } else {
                    true // First conveyor is always valid
                };

                if is_consecutive {
                    progress.conveyor_count += 1;
                    progress.last_conveyor_pos = Some(*position);
                } else {
                    // Reset if not consecutive
                    progress.conveyor_count = 1;
                    progress.last_conveyor_pos = Some(*position);
                }

                progress.conveyor_count >= *count
            }
            // Produce item tutorial
            (TutorialAction::ProduceItem(expected), TutorialEvent::ItemProduced(produced)) => {
                *expected == *produced
            }
            _ => false,
        };

        if should_advance {
            info!(
                "Tutorial step completed: {} ({})",
                step.id, step.description
            );
            progress.advance();

            if progress.completed {
                info!("All tutorials completed!");
            }
            return;
        }
    }
}

/// Update tutorial UI panel
#[allow(clippy::type_complexity, clippy::too_many_arguments)]
pub fn update_tutorial_ui(
    progress: Res<TutorialProgress>,
    tutorial_shown: Res<TutorialShown>,
    mut panel_query: Query<&mut Visibility, With<TutorialPanel>>,
    mut quest_query: Query<&mut Visibility, (With<crate::QuestUI>, Without<TutorialPanel>)>,
    mut text_query: Query<&mut Text, With<TutorialStepText>>,
    mut progress_text_query: Query<
        (&mut Text, &mut Visibility),
        (
            With<TutorialProgressText>,
            Without<TutorialStepText>,
            Without<TutorialPanel>,
            Without<crate::QuestUI>,
            Without<TutorialProgressBarBg>,
        ),
    >,
    mut progress_bar_bg_query: Query<
        &mut Visibility,
        (
            With<TutorialProgressBarBg>,
            Without<TutorialPanel>,
            Without<TutorialProgressText>,
            Without<crate::QuestUI>,
        ),
    >,
    mut progress_bar_fill_query: Query<&mut Node, With<TutorialProgressBarFill>>,
) {
    let Ok(mut panel_vis) = panel_query.get_single_mut() else {
        return;
    };

    // Quest UI visibility: only show when tutorial is completed
    if let Ok(mut quest_vis) = quest_query.get_single_mut() {
        *quest_vis = if progress.completed {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }

    // Hide tutorial panel if tutorials completed or initial popup still showing
    if progress.completed || !tutorial_shown.0 {
        *panel_vis = Visibility::Hidden;
        return;
    }

    // Show tutorial panel during tutorial
    *panel_vis = Visibility::Visible;

    if let Some(step) = progress.current() {
        // Update step text
        if let Ok(mut text) = text_query.get_single_mut() {
            **text = format!(
                "[T] {} ({}/{})\n{}",
                step.description,
                progress.current_step + 1,
                TUTORIAL_STEPS.len(),
                step.hint
            );
        }

        // Check if current step has a count-based action
        let count_info: Option<(u32, u32)> = match &step.action {
            TutorialAction::Move { distance } => Some((progress.move_distance as u32, *distance)),
            TutorialAction::PlaceConveyors { count } => Some((progress.conveyor_count, *count)),
            _ => None,
        };

        // Update progress bar visibility and values
        if let Some((current, target)) = count_info {
            // Show progress text
            if let Ok((mut text, mut vis)) = progress_text_query.get_single_mut() {
                **text = format!("{}/{}", current.min(target), target);
                *vis = Visibility::Visible;
            }

            // Show progress bar background
            if let Ok(mut bar_vis) = progress_bar_bg_query.get_single_mut() {
                *bar_vis = Visibility::Visible;
            }

            // Update progress bar fill width
            if let Ok(mut node) = progress_bar_fill_query.get_single_mut() {
                let percent = (current as f32 / target as f32 * 100.0).min(100.0);
                node.width = Val::Percent(percent);
            }
        } else {
            // Hide progress bar for non-count actions
            if let Ok((_, mut vis)) = progress_text_query.get_single_mut() {
                *vis = Visibility::Hidden;
            }
            if let Ok(mut bar_vis) = progress_bar_bg_query.get_single_mut() {
                *bar_vis = Visibility::Hidden;
            }
        }
    }
}

/// Check if platform inventory has new items for production tracking
pub fn track_production(
    platform_inventory: LocalPlatformInventory,
    mut last_counts: Local<std::collections::HashMap<ItemId, u32>>,
    mut events: EventWriter<TutorialEvent>,
    progress: Res<TutorialProgress>,
) {
    if progress.completed {
        return;
    }

    // Check for new items
    for (item_id, count) in platform_inventory.get_all_items() {
        let last = last_counts.get(&item_id).copied().unwrap_or(0);
        if count > last {
            // Convert ItemId to BlockType for tutorial event
            if let Ok(block_type) = BlockType::try_from(item_id) {
                events.send(TutorialEvent::ItemProduced(block_type));
            }
        }
        last_counts.insert(item_id, count);
    }
}
