//! Machine systems: Miner, Conveyor, Furnace, Crusher
//!
//! This module contains all machine-related game logic systems.

use crate::components::{
    CommandInputState, CrusherProgressBar, CrusherSlotButton, CrusherSlotCount, CrusherUI,
    CursorLockState, FurnaceUI, InteractingCrusher, InteractingFurnace, InteractingMiner,
    InventoryOpen, MachineProgressBar, MachineSlotButton, MachineSlotCount, MachineSlotType,
    MinerBufferButton, MinerBufferCountText, MinerClearButton, MinerUI, PlayerCamera,
};
use crate::player::Inventory;
use crate::world::WorldData;
use crate::{
    ray_aabb_intersection, set_ui_open_state, BlockType, Conveyor, ConveyorItemVisual,
    ConveyorShape, Crusher, DeliveryPlatform, Direction, Furnace, Miner, BLOCK_SIZE,
    CONVEYOR_BELT_HEIGHT, CONVEYOR_ITEM_SIZE, CRUSH_TIME, MINE_TIME, REACH_DISTANCE, SMELT_TIME,
};
use crate::constants::{CONVEYOR_ITEM_SPACING, CONVEYOR_SPEED, PLATFORM_SIZE};
use bevy::prelude::*;
use bevy::window::CursorGrabMode;
use std::collections::HashMap;
use tracing::info;

// === Furnace Systems ===

/// Handle furnace right-click interaction (open/close UI)
#[allow(clippy::too_many_arguments)]
pub fn furnace_interact(
    key_input: Res<ButtonInput<KeyCode>>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    camera_query: Query<&GlobalTransform, With<PlayerCamera>>,
    furnace_query: Query<(Entity, &Transform), With<Furnace>>,
    mut interacting: ResMut<InteractingFurnace>,
    mut furnace_ui_query: Query<&mut Visibility, With<FurnaceUI>>,
    mut windows: Query<&mut Window>,
    inventory_open: Res<InventoryOpen>,
    command_state: Res<CommandInputState>,
    cursor_state: Res<CursorLockState>,
) {
    // Don't process when inventory, command input is open or game is paused (input matrix: Right Click)
    if inventory_open.0 || command_state.open || cursor_state.paused {
        return;
    }

    // ESC or E key to close furnace UI (when open)
    let e_pressed = key_input.just_pressed(KeyCode::KeyE);
    let esc_pressed = key_input.just_pressed(KeyCode::Escape);

    // If already interacting, close the UI with E or ESC
    if interacting.0.is_some() && (e_pressed || esc_pressed) {
        interacting.0 = None;
        if let Ok(mut vis) = furnace_ui_query.get_single_mut() {
            *vis = Visibility::Hidden;
        }
        let mut window = windows.single_mut();
        if esc_pressed {
            // ESC: Browser releases pointer lock automatically in WASM
            // Don't set paused=true - JS will auto-relock via data-ui-open observer (BUG-6 fix)
            window.cursor_options.grab_mode = CursorGrabMode::None;
            window.cursor_options.visible = true;
            // Don't set paused - let JS handle auto-relock
            set_ui_open_state(false);
        } else {
            // E key: Keep cursor locked (no browser interference)
            window.cursor_options.grab_mode = CursorGrabMode::Locked;
            window.cursor_options.visible = false;
            set_ui_open_state(false);
        }
        return;
    }

    // Only open furnace UI with right-click
    if !mouse_button.just_pressed(MouseButton::Right) {
        return;
    }

    let window = windows.single();
    let cursor_locked = window.cursor_options.grab_mode != CursorGrabMode::None;
    if !cursor_locked {
        return;
    }

    let Ok(camera_transform) = camera_query.get_single() else {
        return;
    };

    let ray_origin = camera_transform.translation();
    let ray_direction = camera_transform.forward().as_vec3();

    // Find closest furnace intersection
    let mut closest_furnace: Option<(Entity, f32)> = None;
    let half_size = BLOCK_SIZE / 2.0;

    for (entity, furnace_transform) in furnace_query.iter() {
        let furnace_pos = furnace_transform.translation;
        if let Some(t) = ray_aabb_intersection(
            ray_origin,
            ray_direction,
            furnace_pos - Vec3::splat(half_size),
            furnace_pos + Vec3::splat(half_size),
        ) {
            if t > 0.0 && t < REACH_DISTANCE {
                let is_closer = closest_furnace.is_none_or(|f| t < f.1);
                if is_closer {
                    closest_furnace = Some((entity, t));
                }
            }
        }
    }

    // Open furnace UI
    if let Some((entity, _)) = closest_furnace {
        interacting.0 = Some(entity);
        if let Ok(mut vis) = furnace_ui_query.get_single_mut() {
            *vis = Visibility::Visible;
        }
        // Unlock cursor for UI interaction
        let mut window = windows.single_mut();
        window.cursor_options.grab_mode = CursorGrabMode::None;
        window.cursor_options.visible = true;
        set_ui_open_state(true);
    }
}

/// Handle slot click interactions when furnace UI is open
pub fn furnace_ui_input(
    interacting: Res<InteractingFurnace>,
    mut furnace_query: Query<&mut Furnace>,
    mut inventory: ResMut<Inventory>,
    mut slot_query: Query<
        (&Interaction, &MachineSlotButton, &mut BackgroundColor, &mut BorderColor),
        Changed<Interaction>,
    >,
) {
    let Some(furnace_entity) = interacting.0 else {
        return;
    };

    let Ok(mut furnace) = furnace_query.get_mut(furnace_entity) else {
        return;
    };

    for (interaction, slot_button, mut bg_color, mut border_color) in slot_query.iter_mut() {
        let slot_type = slot_button.0;

        match *interaction {
            Interaction::Pressed => {
                match slot_type {
                    MachineSlotType::Fuel => {
                        // Add coal from inventory (max 64)
                        const MAX_FUEL: u32 = 64;
                        if furnace.fuel < MAX_FUEL && inventory.consume_item(BlockType::Coal, 1) {
                            furnace.fuel += 1;
                        }
                    }
                    MachineSlotType::Input => {
                        // Add ore from inventory (prioritize iron, then copper)
                        if furnace.can_add_input(BlockType::IronOre)
                            && inventory.consume_item(BlockType::IronOre, 1)
                        {
                            furnace.input_type = Some(BlockType::IronOre);
                            furnace.input_count += 1;
                        } else if furnace.can_add_input(BlockType::CopperOre)
                            && inventory.consume_item(BlockType::CopperOre, 1)
                        {
                            furnace.input_type = Some(BlockType::CopperOre);
                            furnace.input_count += 1;
                        }
                    }
                    MachineSlotType::Output => {
                        // Take output ingot to inventory
                        if furnace.output_count > 0 {
                            if let Some(output_type) = furnace.output_type {
                                furnace.output_count -= 1;
                                inventory.add_item(output_type, 1);
                                if furnace.output_count == 0 {
                                    furnace.output_type = None;
                                }
                            }
                        }
                    }
                }
                *border_color = BorderColor(Color::srgb(1.0, 1.0, 0.0));
            }
            Interaction::Hovered => {
                *border_color = BorderColor(Color::srgb(0.8, 0.8, 0.8));
                // Brighten background slightly
                let base = match slot_type {
                    MachineSlotType::Fuel => Color::srgb(0.25, 0.25, 0.25),
                    MachineSlotType::Input => Color::srgb(0.7, 0.6, 0.5),
                    MachineSlotType::Output => Color::srgb(0.9, 0.9, 0.95),
                };
                *bg_color = BackgroundColor(base);
            }
            Interaction::None => {
                *border_color = BorderColor(Color::srgba(0.4, 0.4, 0.4, 1.0));
                *bg_color = BackgroundColor(match slot_type {
                    MachineSlotType::Fuel => Color::srgb(0.15, 0.15, 0.15),
                    MachineSlotType::Input => Color::srgb(0.6, 0.5, 0.4),
                    MachineSlotType::Output => Color::srgb(0.8, 0.8, 0.85),
                });
            }
        }
    }
}

/// Smelting logic - convert ore + coal to ingot
pub fn furnace_smelting(time: Res<Time>, mut furnace_query: Query<&mut Furnace>) {
    for mut furnace in furnace_query.iter_mut() {
        // Need fuel, input ore, and valid recipe to smelt
        let Some(input_ore) = furnace.input_type else {
            furnace.progress = 0.0;
            continue;
        };

        if furnace.fuel == 0 || furnace.input_count == 0 {
            furnace.progress = 0.0;
            continue;
        }

        let output_ingot = Furnace::get_smelt_output(input_ore);

        // Check output slot compatibility
        let output_compatible = match (furnace.output_type, output_ingot) {
            (None, Some(_)) => true,
            (Some(current), Some(new)) => current == new && furnace.output_count < 64,
            _ => false,
        };

        if output_compatible {
            furnace.progress += time.delta_secs() / SMELT_TIME;

            // When progress reaches 1.0, complete smelting
            if furnace.progress >= 1.0 {
                furnace.progress = 0.0;
                furnace.fuel -= 1;
                furnace.input_count -= 1;
                if furnace.input_count == 0 {
                    furnace.input_type = None;
                }
                furnace.output_type = output_ingot;
                furnace.output_count += 1;
            }
        } else {
            furnace.progress = 0.0;
        }
    }
}

/// Update furnace UI slot counts and progress bar
pub fn update_furnace_ui(
    interacting: Res<InteractingFurnace>,
    furnace_query: Query<&Furnace>,
    mut slot_count_query: Query<(&MachineSlotCount, &mut Text)>,
    mut progress_bar_query: Query<&mut Node, With<MachineProgressBar>>,
) {
    let Some(furnace_entity) = interacting.0 else {
        return;
    };

    let Ok(furnace) = furnace_query.get(furnace_entity) else {
        return;
    };

    // Update slot counts
    for (slot_count, mut text) in slot_count_query.iter_mut() {
        **text = match slot_count.0 {
            MachineSlotType::Fuel => format!("{}", furnace.fuel),
            MachineSlotType::Input => format!("{}", furnace.input_count),
            MachineSlotType::Output => format!("{}", furnace.output_count),
        };
    }

    // Update progress bar
    for mut node in progress_bar_query.iter_mut() {
        node.width = Val::Percent(furnace.progress * 100.0);
    }
}

/// Furnace output to conveyor
pub fn furnace_output(
    mut furnace_query: Query<(&Transform, &mut Furnace)>,
    mut conveyor_query: Query<&mut Conveyor>,
) {
    for (transform, mut furnace) in furnace_query.iter_mut() {
        let Some(output_type) = furnace.output_type else {
            continue;
        };

        if furnace.output_count == 0 {
            continue;
        }

        // Get furnace position from Transform
        let furnace_pos = IVec3::new(
            transform.translation.x.floor() as i32,
            transform.translation.y.floor() as i32,
            transform.translation.z.floor() as i32,
        );

        // Check for adjacent conveyors (horizontal + above)
        let adjacent_positions = [
            furnace_pos + IVec3::new(1, 0, 0),  // east
            furnace_pos + IVec3::new(-1, 0, 0), // west
            furnace_pos + IVec3::new(0, 0, 1),  // south
            furnace_pos + IVec3::new(0, 0, -1), // north
            furnace_pos + IVec3::new(0, 1, 0),  // above
        ];

        'outer: for mut conveyor in conveyor_query.iter_mut() {
            for pos in &adjacent_positions {
                if conveyor.position == *pos {
                    let join_progress = if *pos == furnace_pos + IVec3::new(0, 1, 0) {
                        Some(0.0) // Above: always join at entry
                    } else {
                        conveyor.get_join_progress(furnace_pos)
                    };

                    if let Some(progress) = join_progress {
                        if conveyor.can_accept_item(progress) {
                            conveyor.add_item(output_type, progress);
                            furnace.output_count -= 1;
                            if furnace.output_count == 0 {
                                furnace.output_type = None;
                            }
                            break 'outer;
                        }
                    }
                }
            }
        }
    }
}

// === Crusher Systems ===

/// Crusher processing - doubles ore
pub fn crusher_processing(time: Res<Time>, mut crusher_query: Query<&mut Crusher>) {
    for mut crusher in crusher_query.iter_mut() {
        // Need input ore to process
        let Some(input_ore) = crusher.input_type else {
            crusher.progress = 0.0;
            continue;
        };

        if crusher.input_count == 0 {
            crusher.progress = 0.0;
            continue;
        }

        // Check output slot compatibility (same ore type or empty, max 64)
        let output_compatible = match crusher.output_type {
            None => true,
            Some(current) => current == input_ore && crusher.output_count < 63, // 63 because we add 2
        };

        if output_compatible {
            crusher.progress += time.delta_secs() / CRUSH_TIME;

            // When progress reaches 1.0, complete crushing
            if crusher.progress >= 1.0 {
                crusher.progress = 0.0;
                crusher.input_count -= 1;
                if crusher.input_count == 0 {
                    crusher.input_type = None;
                }
                crusher.output_type = Some(input_ore);
                crusher.output_count += 2; // Double output!
            }
        } else {
            crusher.progress = 0.0;
        }
    }
}

/// Handle crusher right-click interaction (open/close UI)
#[allow(clippy::too_many_arguments)]
pub fn crusher_interact(
    key_input: Res<ButtonInput<KeyCode>>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    camera_query: Query<&GlobalTransform, With<PlayerCamera>>,
    crusher_query: Query<(Entity, &Transform), With<Crusher>>,
    mut interacting: ResMut<InteractingCrusher>,
    inventory_open: Res<InventoryOpen>,
    interacting_furnace: Res<InteractingFurnace>,
    mut crusher_ui_query: Query<&mut Visibility, With<CrusherUI>>,
    mut windows: Query<&mut Window>,
    command_state: Res<CommandInputState>,
    cursor_state: Res<CursorLockState>,
) {
    // Don't open crusher if inventory, furnace is open, command input is active, or game is paused (input matrix: Right Click)
    if inventory_open.0
        || interacting_furnace.0.is_some()
        || command_state.open
        || cursor_state.paused
    {
        return;
    }

    let e_pressed = key_input.just_pressed(KeyCode::KeyE);
    let esc_pressed = key_input.just_pressed(KeyCode::Escape);

    // If already interacting, close the UI with E or ESC
    if interacting.0.is_some() && (e_pressed || esc_pressed) {
        interacting.0 = None;
        if let Ok(mut vis) = crusher_ui_query.get_single_mut() {
            *vis = Visibility::Hidden;
        }
        let mut window = windows.single_mut();
        if esc_pressed {
            // ESC: Browser releases pointer lock automatically in WASM
            // Don't set paused=true - JS will auto-relock via data-ui-open observer (BUG-6 fix)
            window.cursor_options.grab_mode = CursorGrabMode::None;
            window.cursor_options.visible = true;
            set_ui_open_state(false);
        } else {
            window.cursor_options.grab_mode = CursorGrabMode::Locked;
            window.cursor_options.visible = false;
            set_ui_open_state(false);
        }
        return;
    }

    // Only open crusher UI with right-click
    if !mouse_button.just_pressed(MouseButton::Right) {
        return;
    }

    let window = windows.single();
    let cursor_locked = window.cursor_options.grab_mode != CursorGrabMode::None;
    if !cursor_locked {
        return;
    }

    let Ok(camera_transform) = camera_query.get_single() else {
        return;
    };

    let ray_origin = camera_transform.translation();
    let ray_direction = camera_transform.forward().as_vec3();

    // Find closest crusher intersection
    let mut closest_crusher: Option<(Entity, f32)> = None;
    let half_size = BLOCK_SIZE / 2.0;

    for (entity, crusher_transform) in crusher_query.iter() {
        let crusher_pos = crusher_transform.translation;
        if let Some(t) = ray_aabb_intersection(
            ray_origin,
            ray_direction,
            crusher_pos - Vec3::splat(half_size),
            crusher_pos + Vec3::splat(half_size),
        ) {
            if t > 0.0 && t < REACH_DISTANCE {
                let is_closer = closest_crusher.is_none_or(|f| t < f.1);
                if is_closer {
                    closest_crusher = Some((entity, t));
                }
            }
        }
    }

    // Open crusher UI
    if let Some((entity, _)) = closest_crusher {
        interacting.0 = Some(entity);
        if let Ok(mut vis) = crusher_ui_query.get_single_mut() {
            *vis = Visibility::Visible;
        }
        // Unlock cursor for UI interaction
        let mut window = windows.single_mut();
        window.cursor_options.grab_mode = CursorGrabMode::None;
        window.cursor_options.visible = true;
        set_ui_open_state(true);
    }
}

/// Handle crusher slot click interactions
pub fn crusher_ui_input(
    interacting: Res<InteractingCrusher>,
    mut crusher_query: Query<&mut Crusher>,
    mut inventory: ResMut<Inventory>,
    mut slot_query: Query<
        (
            &Interaction,
            &CrusherSlotButton,
            &mut BackgroundColor,
            &mut BorderColor,
        ),
        Changed<Interaction>,
    >,
) {
    let Some(crusher_entity) = interacting.0 else {
        return;
    };

    let Ok(mut crusher) = crusher_query.get_mut(crusher_entity) else {
        return;
    };

    for (interaction, slot_button, mut bg_color, mut border_color) in slot_query.iter_mut() {
        let slot_type = slot_button.0;

        match *interaction {
            Interaction::Pressed => {
                match slot_type {
                    MachineSlotType::Fuel => {
                        // Crusher has no fuel slot - do nothing
                    }
                    MachineSlotType::Input => {
                        // Add ore from inventory (prioritize iron, then copper, max 64)
                        const MAX_INPUT: u32 = 64;
                        if crusher.input_count < MAX_INPUT
                            && (crusher.input_type.is_none()
                                || crusher.input_type == Some(BlockType::IronOre))
                            && inventory.consume_item(BlockType::IronOre, 1)
                        {
                            crusher.input_type = Some(BlockType::IronOre);
                            crusher.input_count += 1;
                        } else if crusher.input_count < MAX_INPUT
                            && (crusher.input_type.is_none()
                                || crusher.input_type == Some(BlockType::CopperOre))
                            && inventory.consume_item(BlockType::CopperOre, 1)
                        {
                            crusher.input_type = Some(BlockType::CopperOre);
                            crusher.input_count += 1;
                        }
                    }
                    MachineSlotType::Output => {
                        // Take output ore to inventory
                        if crusher.output_count > 0 {
                            if let Some(output_type) = crusher.output_type {
                                crusher.output_count -= 1;
                                inventory.add_item(output_type, 1);
                                if crusher.output_count == 0 {
                                    crusher.output_type = None;
                                }
                            }
                        }
                    }
                }
                *border_color = BorderColor(Color::srgb(1.0, 1.0, 0.0));
            }
            Interaction::Hovered => {
                *border_color = BorderColor(Color::srgb(0.8, 0.8, 0.8));
                let base = match slot_type {
                    MachineSlotType::Fuel => Color::srgb(0.5, 0.4, 0.35),
                    MachineSlotType::Input => Color::srgb(0.6, 0.5, 0.45),
                    MachineSlotType::Output => Color::srgb(0.7, 0.6, 0.55),
                };
                *bg_color = BackgroundColor(base);
            }
            Interaction::None => {
                *border_color = BorderColor(Color::srgba(0.4, 0.4, 0.4, 1.0));
                *bg_color = BackgroundColor(match slot_type {
                    MachineSlotType::Fuel => Color::srgb(0.5, 0.4, 0.35),
                    MachineSlotType::Input => Color::srgb(0.5, 0.4, 0.35),
                    MachineSlotType::Output => Color::srgb(0.6, 0.5, 0.45),
                });
            }
        }
    }
}

/// Crusher output to conveyor
pub fn crusher_output(
    mut crusher_query: Query<&mut Crusher>,
    mut conveyor_query: Query<&mut Conveyor>,
) {
    for mut crusher in crusher_query.iter_mut() {
        let Some(output_type) = crusher.output_type else {
            continue;
        };

        if crusher.output_count == 0 {
            continue;
        }

        // Check for adjacent conveyors (horizontal + above)
        let adjacent_positions = [
            crusher.position + IVec3::new(1, 0, 0),  // east
            crusher.position + IVec3::new(-1, 0, 0), // west
            crusher.position + IVec3::new(0, 0, 1),  // south
            crusher.position + IVec3::new(0, 0, -1), // north
            crusher.position + IVec3::new(0, 1, 0),  // above
        ];

        'outer: for mut conveyor in conveyor_query.iter_mut() {
            for pos in &adjacent_positions {
                if conveyor.position == *pos {
                    let join_progress = if *pos == crusher.position + IVec3::new(0, 1, 0) {
                        Some(0.0) // Above: always join at entry
                    } else {
                        conveyor.get_join_progress(crusher.position)
                    };

                    if let Some(progress) = join_progress {
                        if conveyor.can_accept_item(progress) {
                            conveyor.add_item(output_type, progress);
                            crusher.output_count -= 1;
                            if crusher.output_count == 0 {
                                crusher.output_type = None;
                            }
                            break 'outer;
                        }
                    }
                }
            }
        }
    }
}

// === Miner Systems ===

/// Handle miner right-click interaction (open/close UI)
#[allow(clippy::too_many_arguments)]
pub fn miner_interact(
    key_input: Res<ButtonInput<KeyCode>>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    camera_query: Query<&GlobalTransform, With<PlayerCamera>>,
    miner_query: Query<(Entity, &Transform), With<Miner>>,
    mut interacting: ResMut<InteractingMiner>,
    inventory_open: Res<InventoryOpen>,
    interacting_furnace: Res<InteractingFurnace>,
    interacting_crusher: Res<InteractingCrusher>,
    mut miner_ui_query: Query<&mut Visibility, With<MinerUI>>,
    mut windows: Query<&mut Window>,
    command_state: Res<CommandInputState>,
    cursor_state: Res<CursorLockState>,
) {
    // Don't open miner if other UI is open
    if inventory_open.0
        || interacting_furnace.0.is_some()
        || interacting_crusher.0.is_some()
        || command_state.open
        || cursor_state.paused
    {
        return;
    }

    let e_pressed = key_input.just_pressed(KeyCode::KeyE);
    let esc_pressed = key_input.just_pressed(KeyCode::Escape);

    // If already interacting, close the UI with E or ESC
    if interacting.0.is_some() && (e_pressed || esc_pressed) {
        interacting.0 = None;
        if let Ok(mut vis) = miner_ui_query.get_single_mut() {
            *vis = Visibility::Hidden;
        }
        let mut window = windows.single_mut();
        if esc_pressed {
            window.cursor_options.grab_mode = CursorGrabMode::None;
            window.cursor_options.visible = true;
            set_ui_open_state(false);
        } else {
            window.cursor_options.grab_mode = CursorGrabMode::Locked;
            window.cursor_options.visible = false;
            set_ui_open_state(false);
        }
        return;
    }

    // Only open miner UI with right-click
    if !mouse_button.just_pressed(MouseButton::Right) {
        return;
    }

    let window = windows.single();
    let cursor_locked = window.cursor_options.grab_mode != CursorGrabMode::None;
    if !cursor_locked {
        return;
    }

    let Ok(camera_transform) = camera_query.get_single() else {
        return;
    };

    let ray_origin = camera_transform.translation();
    let ray_direction = camera_transform.forward().as_vec3();

    // Find closest miner intersection
    let mut closest_miner: Option<(Entity, f32)> = None;
    let half_size = BLOCK_SIZE / 2.0;

    for (entity, miner_transform) in miner_query.iter() {
        let miner_pos = miner_transform.translation;
        if let Some(t) = ray_aabb_intersection(
            ray_origin,
            ray_direction,
            miner_pos - Vec3::splat(half_size),
            miner_pos + Vec3::splat(half_size),
        ) {
            if t > 0.0 && t < REACH_DISTANCE {
                let is_closer = closest_miner.is_none_or(|f| t < f.1);
                if is_closer {
                    closest_miner = Some((entity, t));
                }
            }
        }
    }

    // Open miner UI
    if let Some((entity, _)) = closest_miner {
        interacting.0 = Some(entity);
        if let Ok(mut vis) = miner_ui_query.get_single_mut() {
            *vis = Visibility::Visible;
        }
        // Unlock cursor for UI interaction
        let mut window = windows.single_mut();
        window.cursor_options.grab_mode = CursorGrabMode::None;
        window.cursor_options.visible = true;
        set_ui_open_state(true);
    }
}

/// Handle miner UI button clicks (take buffer, discard buffer)
#[allow(clippy::type_complexity)]
pub fn miner_ui_input(
    interacting: Res<InteractingMiner>,
    mut miner_query: Query<&mut Miner>,
    mut inventory: ResMut<Inventory>,
    mut buffer_btn_query: Query<
        (&Interaction, &mut BackgroundColor, &mut BorderColor),
        (With<MinerBufferButton>, Changed<Interaction>),
    >,
    mut clear_btn_query: Query<
        (&Interaction, &mut BackgroundColor, &mut BorderColor),
        (
            With<MinerClearButton>,
            Without<MinerBufferButton>,
            Changed<Interaction>,
        ),
    >,
) {
    let Some(miner_entity) = interacting.0 else {
        return;
    };

    let Ok(mut miner) = miner_query.get_mut(miner_entity) else {
        return;
    };

    // Buffer button (take all items)
    for (interaction, mut bg_color, mut border_color) in buffer_btn_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                // Take all items from buffer to inventory
                if let Some((block_type, count)) = miner.buffer.take() {
                    inventory.add_item(block_type, count);
                }
                *border_color = BorderColor(Color::srgb(1.0, 1.0, 0.0));
            }
            Interaction::Hovered => {
                *border_color = BorderColor(Color::srgb(0.8, 0.8, 0.8));
                *bg_color = BackgroundColor(Color::srgb(0.6, 0.5, 0.45));
            }
            Interaction::None => {
                *border_color = BorderColor(Color::srgba(0.4, 0.4, 0.4, 1.0));
                *bg_color = BackgroundColor(Color::srgb(0.5, 0.4, 0.35));
            }
        }
    }

    // Clear button (discard buffer)
    for (interaction, mut bg_color, mut border_color) in clear_btn_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                // Discard buffer contents
                miner.buffer = None;
                *border_color = BorderColor(Color::srgb(1.0, 0.3, 0.3));
            }
            Interaction::Hovered => {
                *border_color = BorderColor(Color::srgb(1.0, 0.5, 0.5));
                *bg_color = BackgroundColor(Color::srgb(0.7, 0.3, 0.3));
            }
            Interaction::None => {
                *border_color = BorderColor(Color::srgba(0.8, 0.3, 0.3, 1.0));
                *bg_color = BackgroundColor(Color::srgb(0.6, 0.2, 0.2));
            }
        }
    }
}

/// Update miner UI buffer count display
pub fn update_miner_ui(
    interacting: Res<InteractingMiner>,
    miner_query: Query<&Miner>,
    mut text_query: Query<&mut Text, With<MinerBufferCountText>>,
) {
    let Some(miner_entity) = interacting.0 else {
        return;
    };

    let Ok(miner) = miner_query.get(miner_entity) else {
        return;
    };

    // Update buffer count text
    for mut text in text_query.iter_mut() {
        if let Some((block_type, count)) = &miner.buffer {
            **text = format!("{}\n{}", block_type.name(), count);
        } else {
            **text = "Empty".to_string();
        }
    }
}

/// Mining logic - automatically mine blocks below the miner
pub fn miner_mining(time: Res<Time>, mut miner_query: Query<&mut Miner>, world_data: Res<WorldData>) {
    for mut miner in miner_query.iter_mut() {
        // Skip if buffer is full
        if let Some((_, count)) = miner.buffer {
            if count >= 64 {
                continue;
            }
        }

        // Find block below miner to determine resource type
        let below_pos = miner.position + IVec3::new(0, -1, 0);
        let Some(&block_type) = world_data.get_block(below_pos) else {
            miner.progress = 0.0;
            continue;
        };

        // Only mine resource blocks (not grass/stone)
        let resource_type = match block_type {
            BlockType::IronOre => BlockType::IronOre,
            BlockType::Coal => BlockType::Coal,
            BlockType::CopperOre => BlockType::CopperOre,
            BlockType::Stone => BlockType::Stone,
            _ => {
                // Can't mine this block type, skip
                miner.progress = 0.0;
                continue;
            }
        };

        // Mine progress
        miner.progress += time.delta_secs() / MINE_TIME;

        if miner.progress >= 1.0 {
            miner.progress = 0.0;

            // Generate resource infinitely (don't remove block)
            // Add to buffer
            if let Some((buf_type, ref mut count)) = miner.buffer {
                if buf_type == resource_type {
                    *count += 1;
                }
            } else {
                miner.buffer = Some((resource_type, 1));
            }
        }
    }
}

/// Visual feedback for miner activity (pulse scale when mining)
pub fn miner_visual_feedback(time: Res<Time>, mut miner_query: Query<(&Miner, &mut Transform)>) {
    for (miner, mut transform) in miner_query.iter_mut() {
        // If progress > 0, the miner is actively mining
        if miner.progress > 0.0 {
            // Pulse effect: scale between 0.95 and 1.05 based on progress
            let pulse = 1.0 + 0.05 * (miner.progress * std::f32::consts::TAU * 2.0).sin();
            transform.scale = Vec3::splat(pulse);
        } else if miner.buffer.is_some() {
            // Buffer full but not mining: slight glow/scale up
            let pulse = 1.0 + 0.02 * (time.elapsed_secs() * 3.0).sin();
            transform.scale = Vec3::splat(pulse);
        } else {
            // Idle: reset scale
            transform.scale = Vec3::ONE;
        }
    }
}

/// Output from miner to adjacent conveyor
pub fn miner_output(
    mut miner_query: Query<&mut Miner>,
    mut conveyor_query: Query<&mut Conveyor>,
) {
    for mut miner in miner_query.iter_mut() {
        let Some((block_type, count)) = miner.buffer else {
            continue;
        };
        if count == 0 {
            continue;
        }

        // Check for adjacent conveyors (horizontal + above)
        let adjacent_positions = [
            miner.position + IVec3::new(1, 0, 0),  // east
            miner.position + IVec3::new(-1, 0, 0), // west
            miner.position + IVec3::new(0, 0, 1),  // south
            miner.position + IVec3::new(0, 0, -1), // north
            miner.position + IVec3::new(0, 1, 0),  // above
        ];

        'outer: for mut conveyor in conveyor_query.iter_mut() {
            for pos in &adjacent_positions {
                if conveyor.position == *pos {
                    // Use get_join_progress to determine if item can join and at what progress
                    // For above conveyor, allow joining at entry (0.0) for any direction
                    let join_progress = if *pos == miner.position + IVec3::new(0, 1, 0) {
                        Some(0.0) // Above: always join at entry
                    } else {
                        conveyor.get_join_progress(miner.position)
                    };

                    if let Some(progress) = join_progress {
                        if conveyor.can_accept_item(progress) {
                            conveyor.add_item(block_type, progress);
                            if let Some((_, ref mut buf_count)) = miner.buffer {
                                *buf_count -= 1;
                                if *buf_count == 0 {
                                    miner.buffer = None;
                                }
                            }
                            break 'outer;
                        }
                    }
                }
            }
        }
    }
}

// === Conveyor Systems ===

/// Conveyor transfer logic - move items along conveyor chain (supports multiple items per conveyor)
#[allow(clippy::too_many_arguments)]
pub fn conveyor_transfer(
    time: Res<Time>,
    mut commands: Commands,
    mut conveyor_query: Query<(Entity, &mut Conveyor)>,
    mut furnace_query: Query<(&Transform, &mut Furnace)>,
    mut crusher_query: Query<&mut Crusher>,
    mut platform_query: Query<(&Transform, &mut DeliveryPlatform)>,
) {
    // Build lookup maps
    let conveyor_positions: HashMap<IVec3, Entity> = conveyor_query
        .iter()
        .map(|(e, c)| (c.position, e))
        .collect();

    // Collect furnace positions
    let furnace_positions: HashMap<IVec3, Entity> = furnace_query
        .iter()
        .map(|(t, _)| {
            let pos = IVec3::new(
                t.translation.x.floor() as i32,
                t.translation.y.floor() as i32,
                t.translation.z.floor() as i32,
            );
            (pos, Entity::PLACEHOLDER) // We'll look up by position
        })
        .collect();

    // Collect crusher positions
    let crusher_positions: HashMap<IVec3, Entity> = crusher_query
        .iter()
        .map(|c| (c.position, Entity::PLACEHOLDER))
        .collect();

    // Check if position is on delivery platform
    let platform_bounds: Option<(IVec3, IVec3)> = platform_query.iter().next().map(|(t, _)| {
        let center = IVec3::new(
            t.translation.x.floor() as i32,
            t.translation.y.floor() as i32,
            t.translation.z.floor() as i32,
        );
        let half = PLATFORM_SIZE / 2;
        (
            IVec3::new(center.x - half, center.y, center.z - half),
            IVec3::new(center.x + half, center.y, center.z + half),
        )
    });

    // Transfer actions to apply
    struct TransferAction {
        source_entity: Entity,
        source_pos: IVec3, // Position of source conveyor (for join progress calculation)
        item_index: usize,
        target: TransferTarget,
    }
    enum TransferTarget {
        Conveyor(Entity), // Target conveyor entity
        Furnace(IVec3),
        Crusher(IVec3),
        Delivery,
    }

    let mut actions: Vec<TransferAction> = Vec::new();

    // Track splitter output indices for round-robin (entity -> next output index)
    let mut splitter_indices: HashMap<Entity, usize> = HashMap::new();

    // First pass: update progress and collect transfer actions
    for (entity, conveyor) in conveyor_query.iter() {
        for (idx, item) in conveyor.items.iter().enumerate() {
            // Only transfer items that reached the end
            if item.progress < 1.0 {
                continue;
            }

            // Determine output position(s) based on shape
            let output_positions: Vec<IVec3> = if conveyor.shape == ConveyorShape::Splitter {
                // Splitter: try front, left, right in round-robin order
                let outputs = conveyor.get_splitter_outputs();
                let start_idx =
                    *splitter_indices.get(&entity).unwrap_or(&conveyor.last_output_index);
                // Rotate the array to start from the current index
                let mut rotated = Vec::with_capacity(3);
                for i in 0..3 {
                    rotated.push(outputs[(start_idx + i) % 3]);
                }
                rotated
            } else {
                // Normal conveyor: front only
                vec![conveyor.position + conveyor.direction.to_ivec3()]
            };

            // Try each output position in order
            let mut found_target = false;
            for next_pos in output_positions {
                // Check if next position is on delivery platform
                if let Some((min, max)) = platform_bounds {
                    if next_pos.x >= min.x
                        && next_pos.x <= max.x
                        && next_pos.y >= min.y
                        && next_pos.y <= max.y
                        && next_pos.z >= min.z
                        && next_pos.z <= max.z
                    {
                        actions.push(TransferAction {
                            source_entity: entity,
                            source_pos: conveyor.position,
                            item_index: idx,
                            target: TransferTarget::Delivery,
                        });
                        // Update splitter index for next item
                        if conveyor.shape == ConveyorShape::Splitter {
                            let current = splitter_indices
                                .entry(entity)
                                .or_insert(conveyor.last_output_index);
                            *current = (*current + 1) % 3;
                        }
                        found_target = true;
                        break;
                    }
                }

                // Check if next position has a conveyor
                if let Some(&next_entity) = conveyor_positions.get(&next_pos) {
                    actions.push(TransferAction {
                        source_entity: entity,
                        source_pos: conveyor.position,
                        item_index: idx,
                        target: TransferTarget::Conveyor(next_entity),
                    });
                    if conveyor.shape == ConveyorShape::Splitter {
                        let current = splitter_indices
                            .entry(entity)
                            .or_insert(conveyor.last_output_index);
                        *current = (*current + 1) % 3;
                    }
                    found_target = true;
                    break;
                } else if furnace_positions.contains_key(&next_pos) {
                    actions.push(TransferAction {
                        source_entity: entity,
                        source_pos: conveyor.position,
                        item_index: idx,
                        target: TransferTarget::Furnace(next_pos),
                    });
                    if conveyor.shape == ConveyorShape::Splitter {
                        let current = splitter_indices
                            .entry(entity)
                            .or_insert(conveyor.last_output_index);
                        *current = (*current + 1) % 3;
                    }
                    found_target = true;
                    break;
                } else if crusher_positions.contains_key(&next_pos) {
                    actions.push(TransferAction {
                        source_entity: entity,
                        source_pos: conveyor.position,
                        item_index: idx,
                        target: TransferTarget::Crusher(next_pos),
                    });
                    if conveyor.shape == ConveyorShape::Splitter {
                        let current = splitter_indices
                            .entry(entity)
                            .or_insert(conveyor.last_output_index);
                        *current = (*current + 1) % 3;
                    }
                    found_target = true;
                    break;
                }
            }

            // If no target found for splitter, still advance the index to try next output next time
            if !found_target && conveyor.shape == ConveyorShape::Splitter {
                let current = splitter_indices
                    .entry(entity)
                    .or_insert(conveyor.last_output_index);
                *current = (*current + 1) % 3;
            }
        }
    }

    // Sort actions by item_index descending so we can remove without index shifting issues
    actions.sort_by(|a, b| b.item_index.cmp(&a.item_index));

    // === ZIPPER MERGE LOGIC ===
    // Group sources by target conveyor for zipper merge
    let mut sources_by_target: HashMap<Entity, Vec<Entity>> = HashMap::new();
    for action in &actions {
        if let TransferTarget::Conveyor(target) = action.target {
            let sources = sources_by_target.entry(target).or_default();
            if !sources.contains(&action.source_entity) {
                sources.push(action.source_entity);
            }
        }
    }

    // Determine which source is allowed for each target (zipper logic)
    // When multiple sources try to feed into the same conveyor, only one is allowed per tick
    let allowed_source: HashMap<Entity, Entity> = sources_by_target
        .iter()
        .filter_map(|(target, sources)| {
            if sources.len() <= 1 {
                // Only one source, always allow
                sources.first().map(|s| (*target, *s))
            } else {
                // Multiple sources - use zipper logic with last_input_source
                conveyor_query.get(*target).ok().map(|(_, c)| {
                    let mut sorted_sources: Vec<Entity> = sources.clone();
                    sorted_sources.sort_by_key(|e| e.index());
                    let idx = c.last_input_source % sorted_sources.len();
                    (*target, sorted_sources[idx])
                })
            }
        })
        .collect();

    // Track which targets successfully received an item (to update last_input_source)
    let mut targets_to_update: Vec<Entity> = Vec::new();

    // First pass: check which conveyor-to-conveyor transfers can proceed
    // This avoids borrow conflicts
    // Value is Some((progress, lateral_offset)) if can accept, None otherwise
    let conveyor_transfer_ok: HashMap<(Entity, usize), Option<(f32, f32)>> = actions
        .iter()
        .filter_map(|action| {
            if let TransferTarget::Conveyor(target_entity) = action.target {
                let result = conveyor_query.get(target_entity).ok().and_then(|(_, c)| {
                    // Calculate join info (progress, lateral_offset) based on source position
                    c.get_join_info(action.source_pos)
                        .filter(|&(progress, _)| c.can_accept_item(progress))
                });
                Some(((action.source_entity, action.item_index), result))
            } else {
                None
            }
        })
        .collect();

    // Collect conveyor adds for second pass (to avoid borrow conflicts)
    // Tuple: (target_entity, block_type, join_progress, visual_entity, lateral_offset)
    let mut conveyor_adds: Vec<(Entity, BlockType, f32, Option<Entity>, f32)> = Vec::new();

    // Apply transfers
    for action in actions {
        let Ok((_, mut source_conv)) = conveyor_query.get_mut(action.source_entity) else {
            continue;
        };

        if action.item_index >= source_conv.items.len() {
            continue;
        }

        let item = source_conv.items[action.item_index].clone();

        match action.target {
            TransferTarget::Conveyor(target_entity) => {
                // Zipper merge: check if this source is allowed for this target
                if let Some(&allowed) = allowed_source.get(&target_entity) {
                    if allowed != action.source_entity {
                        // This source is not allowed this tick (zipper logic)
                        continue;
                    }
                }

                // Check pre-computed result - Some((progress, lateral_offset)) if can accept
                let join_info = conveyor_transfer_ok
                    .get(&(action.source_entity, action.item_index))
                    .copied()
                    .flatten();

                if let Some((progress, lateral_offset)) = join_info {
                    // Keep visual entity for seamless transfer (BUG-3 fix)
                    let visual = item.visual_entity;
                    source_conv.items.remove(action.item_index);
                    // Queue add to target conveyor with visual and lateral offset
                    conveyor_adds.push((
                        target_entity,
                        item.block_type,
                        progress,
                        visual,
                        lateral_offset,
                    ));
                    // Mark target for last_input_source update
                    if !targets_to_update.contains(&target_entity) {
                        targets_to_update.push(target_entity);
                    }
                }
            }
            TransferTarget::Furnace(furnace_pos) => {
                let mut accepted = false;
                for (furnace_transform, mut furnace) in furnace_query.iter_mut() {
                    let pos = IVec3::new(
                        furnace_transform.translation.x.floor() as i32,
                        furnace_transform.translation.y.floor() as i32,
                        furnace_transform.translation.z.floor() as i32,
                    );
                    if pos == furnace_pos {
                        let can_accept = match item.block_type {
                            BlockType::Coal => furnace.fuel < 64,
                            BlockType::IronOre | BlockType::CopperOre => {
                                furnace.can_add_input(item.block_type) && furnace.input_count < 64
                            }
                            _ => false,
                        };
                        if can_accept {
                            match item.block_type {
                                BlockType::Coal => furnace.fuel += 1,
                                BlockType::IronOre | BlockType::CopperOre => {
                                    furnace.input_type = Some(item.block_type);
                                    furnace.input_count += 1;
                                }
                                _ => {}
                            }
                            accepted = true;
                        }
                        break;
                    }
                }
                if accepted {
                    if let Some(visual) = item.visual_entity {
                        commands.entity(visual).despawn();
                    }
                    source_conv.items.remove(action.item_index);
                }
            }
            TransferTarget::Crusher(crusher_pos) => {
                let mut accepted = false;
                for mut crusher in crusher_query.iter_mut() {
                    if crusher.position == crusher_pos {
                        let can_accept = Crusher::can_crush(item.block_type)
                            && (crusher.input_type.is_none()
                                || crusher.input_type == Some(item.block_type))
                            && crusher.input_count < 64;
                        if can_accept {
                            crusher.input_type = Some(item.block_type);
                            crusher.input_count += 1;
                            accepted = true;
                        }
                        break;
                    }
                }
                if accepted {
                    if let Some(visual) = item.visual_entity {
                        commands.entity(visual).despawn();
                    }
                    source_conv.items.remove(action.item_index);
                }
            }
            TransferTarget::Delivery => {
                // Deliver the item to platform
                if let Some((_, mut platform)) = platform_query.iter_mut().next() {
                    let count = platform.delivered.entry(item.block_type).or_insert(0);
                    *count += 1;
                    info!(category = "QUEST", action = "deliver", item = ?item.block_type, total = *count, "Item delivered");
                }
                if let Some(visual) = item.visual_entity {
                    commands.entity(visual).despawn();
                }
                source_conv.items.remove(action.item_index);
            }
        }
    }

    // Second pass: add items to target conveyors at their calculated join progress
    for (target_entity, block_type, progress, visual, lateral_offset) in conveyor_adds {
        if let Ok((_, mut target_conv)) = conveyor_query.get_mut(target_entity) {
            target_conv.add_item_with_visual(block_type, progress, visual, lateral_offset);
        }
    }

    // Update last_input_source for conveyors that received items (zipper merge)
    for target_entity in targets_to_update {
        if let Ok((_, mut target_conv)) = conveyor_query.get_mut(target_entity) {
            target_conv.last_input_source += 1;
        }
    }

    // Persist splitter output indices
    for (entity, new_index) in splitter_indices {
        if let Ok((_, mut conv)) = conveyor_query.get_mut(entity) {
            conv.last_output_index = new_index;
        }
    }

    // Update progress for all items on all conveyors
    let delta = time.delta_secs() / CONVEYOR_SPEED;
    let lateral_decay = time.delta_secs() * 3.0; // Decay rate for lateral offset (BUG-5 fix)
    for (_, mut conveyor) in conveyor_query.iter_mut() {
        let item_count = conveyor.items.len();
        for i in 0..item_count {
            // Decay lateral offset towards center
            if conveyor.items[i].lateral_offset.abs() > 0.01 {
                let sign = conveyor.items[i].lateral_offset.signum();
                conveyor.items[i].lateral_offset -= sign * lateral_decay;
                // Clamp to prevent overshooting
                if sign * conveyor.items[i].lateral_offset < 0.0 {
                    conveyor.items[i].lateral_offset = 0.0;
                }
            } else {
                conveyor.items[i].lateral_offset = 0.0;
            }

            if conveyor.items[i].progress < 1.0 {
                // Check if blocked by item ahead (higher progress)
                let current_progress = conveyor.items[i].progress;
                let blocked = conveyor.items.iter().any(|other| {
                    other.progress > current_progress
                        && other.progress - current_progress < CONVEYOR_ITEM_SPACING
                });
                if !blocked {
                    conveyor.items[i].progress += delta;
                    if conveyor.items[i].progress > 1.0 {
                        conveyor.items[i].progress = 1.0;
                    }
                }
            }
        }
    }
}

/// Update conveyor item visuals - spawn/despawn/move items on conveyors (multiple items)
pub fn update_conveyor_item_visuals(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut conveyor_query: Query<&mut Conveyor>,
    mut visual_query: Query<&mut Transform, With<ConveyorItemVisual>>,
) {
    let item_mesh = meshes.add(Cuboid::new(
        BLOCK_SIZE * CONVEYOR_ITEM_SIZE,
        BLOCK_SIZE * CONVEYOR_ITEM_SIZE,
        BLOCK_SIZE * CONVEYOR_ITEM_SIZE,
    ));

    for mut conveyor in conveyor_query.iter_mut() {
        // Position items on top of the belt (belt height + item size/2)
        let item_y = conveyor.position.y as f32 * BLOCK_SIZE
            + CONVEYOR_BELT_HEIGHT
            + CONVEYOR_ITEM_SIZE / 2.0;
        let base_pos = Vec3::new(
            conveyor.position.x as f32 * BLOCK_SIZE + 0.5,
            item_y,
            conveyor.position.z as f32 * BLOCK_SIZE + 0.5,
        );
        let direction_vec = conveyor.direction.to_ivec3().as_vec3();
        // Perpendicular vector for lateral offset (BUG-5 fix, BUG-9 fix)
        // Positive lateral_offset = right side of conveyor direction
        let lateral_vec = match conveyor.direction {
            Direction::East => Vec3::new(0.0, 0.0, 1.0),   // Right is +Z (South)
            Direction::West => Vec3::new(0.0, 0.0, -1.0),  // Right is -Z (North)
            Direction::South => Vec3::new(-1.0, 0.0, 0.0), // Right is -X (West)
            Direction::North => Vec3::new(1.0, 0.0, 0.0),  // Right is +X (East)
        };

        for item in conveyor.items.iter_mut() {
            // Calculate position: progress 0.0 = entry (-0.5), 1.0 = exit (+0.5)
            let forward_offset = (item.progress - 0.5) * BLOCK_SIZE;
            let lateral_offset_world = item.lateral_offset * BLOCK_SIZE;
            let item_pos =
                base_pos + direction_vec * forward_offset + lateral_vec * lateral_offset_world;

            match item.visual_entity {
                None => {
                    // Spawn visual
                    let material = materials.add(StandardMaterial {
                        base_color: item.block_type.color(),
                        ..default()
                    });
                    let entity = commands
                        .spawn((
                            Mesh3d(item_mesh.clone()),
                            MeshMaterial3d(material),
                            Transform::from_translation(item_pos),
                            ConveyorItemVisual,
                        ))
                        .id();
                    item.visual_entity = Some(entity);
                }
                Some(entity) => {
                    // Update position
                    if let Ok(mut transform) = visual_query.get_mut(entity) {
                        transform.translation = item_pos;
                    }
                }
            }
        }
    }
}

/// Update crusher UI slot counts and progress bar
pub fn update_crusher_ui(
    interacting: Res<InteractingCrusher>,
    crusher_query: Query<&Crusher>,
    mut slot_count_query: Query<(&CrusherSlotCount, &mut Text)>,
    mut progress_bar_query: Query<&mut Node, With<CrusherProgressBar>>,
) {
    let Some(crusher_entity) = interacting.0 else {
        return;
    };

    let Ok(crusher) = crusher_query.get(crusher_entity) else {
        return;
    };

    // Update slot counts
    for (slot_count, mut text) in slot_count_query.iter_mut() {
        **text = match slot_count.0 {
            MachineSlotType::Fuel => String::new(), // Crusher has no fuel
            MachineSlotType::Input => format!("{}", crusher.input_count),
            MachineSlotType::Output => format!("{}", crusher.output_count),
        };
    }

    // Update progress bar
    for mut node in progress_bar_query.iter_mut() {
        node.width = Val::Percent(crusher.progress * 100.0);
    }
}
