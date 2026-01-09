//! Generic machine systems (Phase C Data-Driven Design)
//!
//! These systems work with the generic `Machine` component,
//! using `MachineSpec` to determine behavior.

use crate::components::Machine;
use crate::core::{items, ItemId};
use crate::events::game_events::{MachineCompleted, MachineStarted};
use crate::events::GuardedEventWriter;
use crate::game_spec::{find_recipe, MachineType, ProcessType};
use crate::world::biome::{BiomeMap, BiomeType};
use crate::Conveyor;
use bevy::prelude::*;
use std::collections::HashMap;

/// Event result from tick_recipe: (started_inputs, completed_outputs)
type RecipeEventResult = Option<(Option<Vec<(ItemId, u32)>>, Option<Vec<(ItemId, u32)>>)>;

/// Generic machine tick system - processes all Machine components
pub fn generic_machine_tick(
    time: Res<Time>,
    biome_map: Res<BiomeMap>,
    mut machine_query: Query<(Entity, &mut Machine)>,
    mut conveyor_query: Query<(Entity, &mut Conveyor)>,
    mut started_events: GuardedEventWriter<MachineStarted>,
    mut completed_events: GuardedEventWriter<MachineCompleted>,
) {
    let delta = time.delta_secs();

    // Build conveyor position map for O(1) lookup
    let conveyor_map: HashMap<IVec3, Entity> = conveyor_query
        .iter()
        .map(|(entity, conveyor)| (conveyor.position, entity))
        .collect();

    // Collect events to send after iteration
    let mut started: Vec<(Entity, Vec<(ItemId, u32)>)> = Vec::new();
    let mut completed: Vec<(Entity, Vec<(ItemId, u32)>)> = Vec::new();

    for (entity, mut machine) in machine_query.iter_mut() {
        match machine.spec.process_type {
            ProcessType::AutoGenerate => {
                let result = tick_auto_generate(
                    &mut machine,
                    delta,
                    &biome_map,
                    &conveyor_map,
                    &mut conveyor_query,
                );
                if let Some(output_id) = result {
                    completed.push((entity, vec![(output_id, 1)]));
                }
            }
            ProcessType::Recipe(machine_type) => {
                let result = tick_recipe(
                    &mut machine,
                    delta,
                    machine_type,
                    &conveyor_map,
                    &mut conveyor_query,
                );
                if let Some((started_inputs, completed_outputs)) = result {
                    if let Some(inputs) = started_inputs {
                        started.push((entity, inputs));
                    }
                    if let Some(outputs) = completed_outputs {
                        completed.push((entity, outputs));
                    }
                }
            }
            ProcessType::Transfer => {
                // Conveyors are handled separately
            }
        }
    }

    // Send events
    for (entity, inputs) in started {
        let _ = started_events.send(MachineStarted { entity, inputs });
    }
    for (entity, outputs) in completed {
        let _ = completed_events.send(MachineCompleted { entity, outputs });
    }
}

/// Tick for auto-generating machines (like Miner)
/// Returns Some(output_item_id) when an item is produced
fn tick_auto_generate(
    machine: &mut Machine,
    delta: f32,
    biome_map: &BiomeMap,
    conveyor_map: &HashMap<IVec3, Entity>,
    conveyor_query: &mut Query<(Entity, &mut Conveyor)>,
) -> Option<ItemId> {
    let spec = machine.spec;

    // Check if output buffer has space
    let output_slot = machine.slots.outputs.first_mut();
    let can_output = output_slot
        .map(|s| s.count < spec.buffer_size)
        .unwrap_or(false);

    if !can_output {
        return None;
    }

    // Progress mining
    machine.progress += delta / spec.process_time;

    let mut produced = None;
    if machine.progress >= 1.0 {
        machine.progress = 0.0;
        machine.tick_count = machine.tick_count.wrapping_add(1);

        // Determine what to mine based on biome
        let biome = biome_map.get_biome(machine.position);
        let mined_id = get_biome_output(biome, machine.tick_count);

        // Add to output buffer
        if let Some(output) = machine.slots.outputs.first_mut() {
            if output.item_id.is_none() || output.item_id == Some(mined_id) {
                output.item_id = Some(mined_id);
                output.count += 1;
                produced = Some(mined_id);
            }
        }
    }

    // Try to output to conveyor
    try_output_to_conveyor(machine, conveyor_map, conveyor_query);
    produced
}

/// Tick for recipe-based machines (Furnace, Crusher, Assembler)
/// Returns Some((started_inputs, completed_outputs)) for event emission
/// - started_inputs: Some when processing started (inputs consumed)
/// - completed_outputs: Some when processing completed (outputs produced)
fn tick_recipe(
    machine: &mut Machine,
    delta: f32,
    machine_type: MachineType,
    conveyor_map: &HashMap<IVec3, Entity>,
    conveyor_query: &mut Query<(Entity, &mut Conveyor)>,
) -> RecipeEventResult {
    let spec = machine.spec;

    // Get input item
    let input_item_id = machine.slots.inputs.first().and_then(|s| s.item_id);

    // Find recipe
    let input_id = input_item_id?;
    let recipe = find_recipe(machine_type, input_id)?;

    // Check fuel requirement
    if spec.requires_fuel && machine.slots.fuel == 0 {
        return None;
    }

    // Check if we have enough input
    let input_slot = &machine.slots.inputs[0];
    let required_count = recipe.inputs.first().map(|i| i.count).unwrap_or(1);
    if input_slot.count < required_count {
        return None;
    }

    // Check if output has space
    let output_item_id: Option<ItemId> = recipe.outputs.first().map(|o| o.item);
    let output_count = recipe.outputs.first().map(|o| o.count).unwrap_or(1);

    let output_slot = machine.slots.outputs.first();
    let can_output = output_slot
        .map(|s| {
            s.count + output_count <= spec.buffer_size
                && (s.item_id.is_none() || s.item_id == output_item_id)
        })
        .unwrap_or(false);

    if !can_output {
        return None;
    }

    // Track if we just started processing
    let was_idle = machine.progress == 0.0;

    // Progress processing
    machine.progress += delta / recipe.craft_time;

    // Determine started event (only when transitioning from idle to processing)
    let started_inputs = if was_idle && machine.progress > 0.0 && machine.progress < 1.0 {
        Some(vec![(input_id, required_count)])
    } else {
        None
    };

    let mut completed_outputs = None;
    if machine.progress >= 1.0 {
        machine.progress = 0.0;

        // Consume input
        if let Some(input_slot) = machine.slots.inputs.first_mut() {
            input_slot.take(required_count);
        }

        // Consume fuel if required
        if spec.requires_fuel {
            if let Some(fuel_req) = &recipe.fuel {
                machine.slots.fuel = machine.slots.fuel.saturating_sub(fuel_req.amount);
            }
        }

        // Produce output
        if let (Some(item_id), Some(output_slot)) =
            (output_item_id, machine.slots.outputs.first_mut())
        {
            output_slot.add_id(item_id, output_count);
            completed_outputs = Some(vec![(item_id, output_count)]);
        }
    }

    // Try to output to conveyor
    try_output_to_conveyor(machine, conveyor_map, conveyor_query);

    // Return event info if anything happened
    if started_inputs.is_some() || completed_outputs.is_some() {
        Some((started_inputs, completed_outputs))
    } else {
        None
    }
}

/// Try to output items to a connected conveyor (O(1) lookup)
fn try_output_to_conveyor(
    machine: &mut Machine,
    conveyor_map: &HashMap<IVec3, Entity>,
    conveyor_query: &mut Query<(Entity, &mut Conveyor)>,
) {
    use crate::components::ConveyorItem;

    let output_pos = machine.output_position();

    // O(1) lookup for conveyor at output position
    let Some(&conveyor_entity) = conveyor_map.get(&output_pos) else {
        return;
    };

    // Get the conveyor component
    let Ok((_, mut conveyor)) = conveyor_query.get_mut(conveyor_entity) else {
        return;
    };

    // Check if conveyor can accept items
    if conveyor.items.len() >= crate::constants::CONVEYOR_MAX_ITEMS {
        return;
    }

    // Get item from output slot
    let Some(output_slot) = machine.slots.outputs.first_mut() else {
        return;
    };

    if output_slot.is_empty() {
        return;
    }

    let Some(item_id) = output_slot.item_id else {
        return;
    };

    // Transfer one item
    output_slot.take(1);
    conveyor.items.push(ConveyorItem::new(item_id, 0.0));
}

/// Get mining output based on biome (returns ItemId)
fn get_biome_output(biome: BiomeType, tick: u32) -> ItemId {
    use crate::game_spec::biome_mining_spec::*;

    let probabilities: &[(ItemId, u32)] = match biome {
        BiomeType::Iron => &IRON_BIOME,
        BiomeType::Copper => &COPPER_BIOME,
        BiomeType::Coal => &COAL_BIOME,
        BiomeType::Stone => &STONE_BIOME,
        BiomeType::Mixed => &MIXED_BIOME,
        BiomeType::Unmailable => &STONE_BIOME,
    };

    // Simple deterministic selection based on tick
    let total: u32 = probabilities.iter().map(|(_, p)| p).sum();
    let roll = tick % total;

    let mut acc = 0;
    for (item_id, prob) in probabilities {
        acc += prob;
        if roll < acc {
            return *item_id;
        }
    }

    items::stone()
}

// =============================================================================
// Generic Machine Interaction & UI Systems
// =============================================================================

use crate::components::{
    CursorLockState, GenericMachineProgressBar, GenericMachineSlotButton, GenericMachineSlotCount,
    GenericMachineUI, InteractingMachine, InventoryOpen, PlayerCamera,
};
use crate::input::{GameAction, InputManager};
use crate::player::{LocalPlayer, PlayerInventory};
use crate::systems::cursor;
use crate::REACH_DISTANCE;
use bevy::window::CursorGrabMode;

/// Generic machine interaction (open/close UI)
#[allow(clippy::too_many_arguments)]
pub fn generic_machine_interact(
    input: Res<InputManager>,
    camera_query: Query<&GlobalTransform, With<PlayerCamera>>,
    machine_query: Query<(Entity, &Transform, &Machine)>,
    mut interacting: ResMut<InteractingMachine>,
    inventory_open: Res<InventoryOpen>,
    mut ui_query: Query<(&GenericMachineUI, &mut Visibility)>,
    mut windows: Query<&mut Window>,
    mut cursor_state: ResMut<CursorLockState>,
) {
    let window = windows.single();
    let cursor_locked = window.cursor_options.grab_mode != CursorGrabMode::None;

    // Don't interact if inventory is open
    if inventory_open.0 {
        return;
    }

    let e_pressed = input.just_pressed(GameAction::ToggleInventory);
    let esc_pressed = input.just_pressed(GameAction::Cancel);

    // Close UI with E or ESC
    if interacting.0.is_some() && (e_pressed || esc_pressed) {
        let machine_id = machine_query
            .get(interacting.0.unwrap())
            .map(|(_, _, m)| m.spec.id)
            .unwrap_or("");

        // Hide UI
        for (ui, mut vis) in ui_query.iter_mut() {
            if ui.machine_id == machine_id {
                *vis = Visibility::Hidden;
            }
        }

        interacting.0 = None;

        // Lock cursor (unless ESC)
        if !esc_pressed {
            if let Ok(mut window) = windows.get_single_mut() {
                cursor::lock_cursor(&mut window);
            }
            cursor_state.skip_inventory_toggle = true;
        }
        return;
    }

    // Open UI with right-click when cursor locked
    if !input.just_pressed(GameAction::SecondaryAction) || !cursor_locked {
        return;
    }

    let Ok(camera_transform) = camera_query.get_single() else {
        return;
    };

    let ray_origin = camera_transform.translation();
    let ray_dir = camera_transform.forward().as_vec3();

    // Find closest machine
    let mut closest: Option<(Entity, f32, &'static str)> = None;
    for (entity, transform, machine) in machine_query.iter() {
        let to_machine = transform.translation - ray_origin;
        let dist = to_machine.dot(ray_dir);
        if dist > 0.0 && dist < REACH_DISTANCE {
            let closest_point = ray_origin + ray_dir * dist;
            let diff = (closest_point - transform.translation).length();
            if diff < 0.7 {
                // Within machine hitbox
                if closest.is_none() || dist < closest.unwrap().1 {
                    closest = Some((entity, dist, machine.spec.id));
                }
            }
        }
    }

    if let Some((entity, _, machine_id)) = closest {
        interacting.0 = Some(entity);

        // Show UI
        for (ui, mut vis) in ui_query.iter_mut() {
            if ui.machine_id == machine_id {
                *vis = Visibility::Inherited;
            }
        }

        // Unlock cursor
        if let Ok(mut window) = windows.get_single_mut() {
            cursor::unlock_cursor(&mut window);
        }
    }
}

/// Update generic machine UI slot counts and progress bar
pub fn update_generic_machine_ui(
    interacting: Res<InteractingMachine>,
    machine_query: Query<&Machine>,
    mut slot_count_query: Query<(&GenericMachineSlotCount, &mut Text)>,
    mut progress_bar_query: Query<&mut Node, With<GenericMachineProgressBar>>,
) {
    let Some(entity) = interacting.0 else {
        return;
    };

    let Ok(machine) = machine_query.get(entity) else {
        return;
    };

    // Update slot counts
    for (slot_count, mut text) in slot_count_query.iter_mut() {
        let display = if slot_count.is_fuel {
            format_count(machine.slots.fuel)
        } else if slot_count.is_input {
            machine
                .slots
                .inputs
                .get(slot_count.slot_id as usize)
                .map(format_slot)
                .unwrap_or_default()
        } else {
            machine
                .slots
                .outputs
                .get(slot_count.slot_id as usize)
                .map(format_slot)
                .unwrap_or_default()
        };
        **text = display;
    }

    // Update progress bar
    for mut node in progress_bar_query.iter_mut() {
        node.width = Val::Percent(machine.progress * 100.0);
    }
}

/// Format slot count for display
fn format_slot(slot: &crate::components::MachineSlot) -> String {
    if slot.is_empty() {
        String::new()
    } else {
        let short_name = slot.get_item_id().map(|id| id.short_name()).unwrap_or("");
        if slot.count > 1 {
            format!("{}{}", short_name, slot.count)
        } else {
            short_name.to_string()
        }
    }
}

fn format_count(count: u32) -> String {
    if count == 0 {
        String::new()
    } else {
        count.to_string()
    }
}

/// Visual feedback for machine activity (pulse scale when processing)
pub fn machine_visual_feedback(
    _time: Res<Time>,
    mut machine_query: Query<(&Machine, &mut Transform)>,
) {
    for (machine, mut transform) in machine_query.iter_mut() {
        if machine.progress > 0.0 {
            // Pulse effect while processing
            let pulse = 1.0 + 0.05 * (machine.progress * std::f32::consts::TAU * 2.0).sin();
            transform.scale = Vec3::splat(pulse);
        } else {
            // Reset scale
            transform.scale = Vec3::ONE;
        }
    }
}

/// Handle generic machine UI input (slot clicks)
#[allow(clippy::too_many_arguments)]
pub fn generic_machine_ui_input(
    interacting: Res<InteractingMachine>,
    mut machine_query: Query<&mut Machine>,
    local_player: Option<Res<LocalPlayer>>,
    mut inventory_query: Query<&mut PlayerInventory>,
    mut slot_btn_query: Query<
        (
            &Interaction,
            &GenericMachineSlotButton,
            &mut BackgroundColor,
        ),
        Changed<Interaction>,
    >,
) {
    let Some(entity) = interacting.0 else {
        return;
    };

    let Ok(mut machine) = machine_query.get_mut(entity) else {
        return;
    };

    let Some(local_player) = local_player else {
        return;
    };

    let Ok(mut inventory) = inventory_query.get_mut(local_player.0) else {
        return;
    };

    for (interaction, slot_btn, mut bg_color) in slot_btn_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                // Take from output / put to input
                if slot_btn.is_input {
                    // Try to put selected item into input slot
                    if let Some(selected_id) = inventory.selected_item_id() {
                        // Valid item check - must have a name registered
                        if selected_id.name().is_some() {
                            if let Some(input_slot) =
                                machine.slots.inputs.get_mut(slot_btn.slot_id as usize)
                            {
                                if (input_slot.item_id.is_none()
                                    || input_slot.item_id == Some(selected_id))
                                    && inventory.consume_item_by_id(selected_id, 1)
                                {
                                    input_slot.add_id(selected_id, 1);
                                }
                            }
                        }
                    }
                } else if slot_btn.is_fuel {
                    // Put coal into fuel slot
                    let coal_id = items::coal();
                    if let Some(selected_id) = inventory.selected_item_id() {
                        if selected_id == coal_id && inventory.consume_item_by_id(coal_id, 1) {
                            machine.slots.fuel += 1;
                        }
                    }
                } else {
                    // Take from output slot
                    if let Some(output_slot) =
                        machine.slots.outputs.get_mut(slot_btn.slot_id as usize)
                    {
                        if let Some(item_id) = output_slot.item_id {
                            let taken = output_slot.take(1);
                            if taken > 0 {
                                inventory.add_item_by_id(item_id, taken);
                            }
                        }
                    }
                }
                *bg_color = BackgroundColor(Color::srgb(0.4, 0.4, 0.5));
            }
            Interaction::Hovered => {
                *bg_color = BackgroundColor(Color::srgb(0.25, 0.25, 0.3));
            }
            Interaction::None => {
                *bg_color = BackgroundColor(Color::srgb(0.15, 0.15, 0.2));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::MachineSlot;
    use crate::game_spec::{CRUSHER, FURNACE, MINER};

    #[test]
    fn test_machine_slot_operations() {
        use crate::core::items;

        let mut slot = MachineSlot::empty();
        assert!(slot.is_empty());

        // Add items
        slot.add_id(items::iron_ore(), 5);
        assert_eq!(slot.count, 5);
        assert_eq!(slot.get_item_id(), Some(items::iron_ore()));

        // Can't add different type
        let added = slot.add_id(items::copper_ore(), 3);
        assert_eq!(added, 0);
        assert_eq!(slot.count, 5);

        // Can add same type
        slot.add_id(items::iron_ore(), 3);
        assert_eq!(slot.count, 8);

        // Take items
        let taken = slot.take(3);
        assert_eq!(taken, 3);
        assert_eq!(slot.count, 5);

        // Take all
        slot.take(5);
        assert!(slot.is_empty());
    }

    #[test]
    fn test_machine_creation() {
        let machine = Machine::new(
            &MINER,
            IVec3::new(0, 0, 0),
            crate::components::Direction::North,
        );
        assert_eq!(machine.spec.id, "miner");
        assert_eq!(machine.progress, 0.0);
        assert_eq!(machine.slots.outputs.len(), 1);
    }

    #[test]
    fn test_furnace_machine_creation() {
        let machine = Machine::new(
            &FURNACE,
            IVec3::new(0, 0, 0),
            crate::components::Direction::North,
        );
        assert_eq!(machine.spec.id, "furnace");
        assert!(machine.spec.requires_fuel);
        assert_eq!(machine.slots.inputs.len(), 1);
        assert_eq!(machine.slots.outputs.len(), 1);
    }

    #[test]
    fn test_crusher_machine_creation() {
        let machine = Machine::new(
            &CRUSHER,
            IVec3::new(0, 0, 0),
            crate::components::Direction::North,
        );
        assert_eq!(machine.spec.id, "crusher");
        assert!(!machine.spec.requires_fuel);
    }

    #[test]
    fn test_biome_output_deterministic() {
        use crate::world::biome::BiomeType;

        // Same tick should give same output
        let out1 = get_biome_output(BiomeType::Iron, 100);
        let out2 = get_biome_output(BiomeType::Iron, 100);
        assert_eq!(out1, out2);

        // Different biomes can give different outputs
        let iron_out = get_biome_output(BiomeType::Iron, 0);
        let copper_out = get_biome_output(BiomeType::Copper, 0);
        // At tick 0, both should give their primary ore
        assert_eq!(iron_out, items::iron_ore());
        assert_eq!(copper_out, items::copper_ore());
    }
}
