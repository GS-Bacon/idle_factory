//! Conveyor systems: transfer, visuals

use crate::components::{can_crush, Machine};
use crate::constants::{CONVEYOR_ITEM_SPACING, CONVEYOR_SPEED, PLATFORM_SIZE};
use crate::events::game_events::{ConveyorTransfer, ItemDelivered};
use crate::player::GlobalInventory;
use crate::{
    BlockType, Conveyor, ConveyorItemVisual, ConveyorShape, DeliveryPlatform, Direction,
    MachineModels, BLOCK_SIZE, CONVEYOR_BELT_HEIGHT, CONVEYOR_ITEM_SIZE,
};
use bevy::prelude::*;
use bevy::time::Fixed;
use std::collections::{HashMap, HashSet};
use tracing::info;

/// Conveyor transfer logic - move items along conveyor chain (supports multiple items per conveyor)
#[allow(clippy::too_many_arguments)]
pub fn conveyor_transfer(
    time: Res<Time>,
    mut commands: Commands,
    mut conveyor_query: Query<(Entity, &mut Conveyor)>,
    mut machine_query: Query<&mut Machine>,
    platform_query: Query<(&Transform, &DeliveryPlatform)>,
    mut global_inventory: ResMut<GlobalInventory>,
    mut transfer_events: EventWriter<ConveyorTransfer>,
    mut delivery_events: EventWriter<ItemDelivered>,
) {
    // Build lookup maps
    let conveyor_positions: HashMap<IVec3, Entity> = conveyor_query
        .iter()
        .map(|(e, c)| (c.position, e))
        .collect();

    // Collect furnace and crusher positions from Machine components
    let mut furnace_positions: HashMap<IVec3, Entity> = HashMap::new();
    let mut crusher_positions: HashMap<IVec3, Entity> = HashMap::new();

    for (entity, machine) in machine_query.iter().map(|m| (Entity::PLACEHOLDER, m)) {
        match machine.spec.block_type {
            BlockType::FurnaceBlock => {
                furnace_positions.insert(machine.position, entity);
            }
            BlockType::CrusherBlock => {
                crusher_positions.insert(machine.position, entity);
            }
            _ => {}
        }
    }

    // Check if position is on delivery platform
    let platform_bounds: Option<(IVec3, IVec3)> = platform_query.iter().next().map(|(t, _)| {
        let center = crate::world_to_grid(t.translation);
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
        item_type: BlockType, // For event emission
        target: TransferTarget,
    }
    enum TransferTarget {
        Conveyor(Entity, IVec3), // Target conveyor entity and position
        Furnace(IVec3),
        Crusher(IVec3),
        Delivery,
    }

    let mut actions: Vec<TransferAction> = Vec::new();

    // Collect events to send
    let mut conveyor_transfer_items: Vec<(IVec3, IVec3, BlockType)> = Vec::new();
    let mut delivered_items: Vec<(BlockType, u32)> = Vec::new();

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
                let start_idx = *splitter_indices
                    .get(&entity)
                    .unwrap_or(&conveyor.last_output_index);
                // Rotate the array to start from the current index
                let mut rotated = Vec::with_capacity(3);
                for i in 0..3 {
                    rotated.push(outputs[(start_idx + i) % 3]);
                }
                rotated
            } else {
                // Normal conveyor: use output_direction (may differ for corners)
                vec![conveyor.position + conveyor.output_direction.to_ivec3()]
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
                            item_type: item.block_type,
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
                        item_type: item.block_type,
                        target: TransferTarget::Conveyor(next_entity, next_pos),
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
                        item_type: item.block_type,
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
                        item_type: item.block_type,
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
    // Group sources by target conveyor for zipper merge (HashSet for O(1) dedup)
    let mut sources_by_target: HashMap<Entity, HashSet<Entity>> = HashMap::new();
    for action in &actions {
        if let TransferTarget::Conveyor(target, _) = action.target {
            sources_by_target
                .entry(target)
                .or_default()
                .insert(action.source_entity);
        }
    }

    // Determine which source is allowed for each target (zipper logic)
    // When multiple sources try to feed into the same conveyor, only one is allowed per tick
    let allowed_source: HashMap<Entity, Entity> = sources_by_target
        .iter()
        .filter_map(|(target, sources)| {
            if sources.len() <= 1 {
                // Only one source, always allow
                sources.iter().next().map(|s| (*target, *s))
            } else {
                // Multiple sources - use zipper logic with last_input_source
                conveyor_query.get(*target).ok().map(|(_, c)| {
                    let mut sorted_sources: Vec<Entity> = sources.iter().copied().collect();
                    sorted_sources.sort_by_key(|e| e.index());
                    let idx = c.last_input_source % sorted_sources.len();
                    (*target, sorted_sources[idx])
                })
            }
        })
        .collect();

    // Track which targets successfully received an item (to update last_input_source)
    let mut targets_to_update: HashSet<Entity> = HashSet::new();

    // First pass: check which conveyor-to-conveyor transfers can proceed
    // This avoids borrow conflicts
    // Value is Some((progress, lateral_offset)) if can accept, None otherwise
    let conveyor_transfer_ok: HashMap<(Entity, usize), Option<(f32, f32)>> = actions
        .iter()
        .filter_map(|action| {
            if let TransferTarget::Conveyor(target_entity, _) = action.target {
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
            TransferTarget::Conveyor(target_entity, target_pos) => {
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
                    targets_to_update.insert(target_entity);
                    // Collect event for ConveyorTransfer
                    conveyor_transfer_items.push((action.source_pos, target_pos, action.item_type));
                }
            }
            TransferTarget::Furnace(furnace_pos) => {
                let mut accepted = false;
                for mut machine in machine_query.iter_mut() {
                    if machine.spec.block_type != BlockType::FurnaceBlock
                        || machine.position != furnace_pos
                    {
                        continue;
                    }
                    // Calculate input ports based on furnace facing direction
                    // FURNACE spec: Back=ore(slot0), Left=fuel(slot1), Right=fuel(slot1)
                    let back_port = machine.position + machine.facing.opposite().to_ivec3();
                    let left_port = machine.position + machine.facing.left().to_ivec3();
                    let right_port = machine.position + machine.facing.right().to_ivec3();

                    // Determine which port the conveyor is at
                    let at_back = action.source_pos == back_port;
                    let at_left = action.source_pos == left_port;
                    let at_right = action.source_pos == right_port;

                    if !at_back && !at_left && !at_right {
                        break; // Not at any input port, reject
                    }

                    // Accept items based on port type
                    let input_count = machine.slots.inputs.first().map(|s| s.count).unwrap_or(0);
                    let input_type = machine.slots.inputs.first().and_then(|s| s.item_type);
                    let can_accept = match item.block_type {
                        BlockType::Coal => {
                            // Fuel only from left or right ports
                            (at_left || at_right) && machine.slots.fuel < 64
                        }
                        BlockType::IronOre
                        | BlockType::CopperOre
                        | BlockType::IronDust
                        | BlockType::CopperDust => {
                            // Ore/Dust only from back port
                            at_back
                                && (input_type.is_none() || input_type == Some(item.block_type))
                                && input_count < 64
                        }
                        _ => false,
                    };
                    if can_accept {
                        match item.block_type {
                            BlockType::Coal => machine.slots.fuel += 1,
                            BlockType::IronOre
                            | BlockType::CopperOre
                            | BlockType::IronDust
                            | BlockType::CopperDust => {
                                if let Some(input_slot) = machine.slots.inputs.first_mut() {
                                    input_slot.item_type = Some(item.block_type);
                                    input_slot.count += 1;
                                }
                            }
                            _ => {}
                        }
                        accepted = true;
                    }
                    break;
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
                for mut machine in machine_query.iter_mut() {
                    if machine.spec.block_type != BlockType::CrusherBlock
                        || machine.position != crusher_pos
                    {
                        continue;
                    }
                    // Check if conveyor is at input port (back of crusher)
                    let input_port = machine.position + machine.facing.opposite().to_ivec3();
                    if action.source_pos != input_port {
                        break; // Not at input port, reject
                    }

                    let input_count = machine.slots.inputs.first().map(|s| s.count).unwrap_or(0);
                    let input_type = machine.slots.inputs.first().and_then(|s| s.item_type);
                    let can_accept_item = can_crush(item.block_type)
                        && (input_type.is_none() || input_type == Some(item.block_type))
                        && input_count < 64;
                    if can_accept_item {
                        if let Some(input_slot) = machine.slots.inputs.first_mut() {
                            input_slot.item_type = Some(item.block_type);
                            input_slot.count += 1;
                        }
                        accepted = true;
                    }
                    break;
                }
                if accepted {
                    if let Some(visual) = item.visual_entity {
                        commands.entity(visual).despawn();
                    }
                    source_conv.items.remove(action.item_index);
                }
            }
            TransferTarget::Delivery => {
                // Deliver the item to GlobalInventory
                global_inventory.add_item_by_id(item.block_type.into(), 1);
                let total = global_inventory.get_count_by_id(item.block_type.into());
                info!(category = "QUEST", action = "deliver", item = ?item.block_type, total = total, "Item delivered to storage");
                if let Some(visual) = item.visual_entity {
                    commands.entity(visual).despawn();
                }
                source_conv.items.remove(action.item_index);
                // Collect event for ItemDelivered
                delivered_items.push((action.item_type, 1));
            }
        }
    }

    // Second pass: add items to target conveyors at their calculated join progress
    for (target_entity, block_type, progress, visual, lateral_offset) in conveyor_adds {
        if let Ok((_, mut target_conv)) = conveyor_query.get_mut(target_entity) {
            target_conv.add_item_with_visual(block_type.into(), progress, visual, lateral_offset);
        }
    }

    // Update last_input_source for conveyors that received items (zipper merge)
    for target_entity in targets_to_update {
        if let Ok((_, mut target_conv)) = conveyor_query.get_mut(target_entity) {
            target_conv.last_input_source += 1;
        }
    }

    // Send collected events
    for (from_pos, to_pos, item) in conveyor_transfer_items {
        transfer_events.send(ConveyorTransfer {
            from_pos,
            to_pos,
            item,
        });
    }
    for (item, count) in delivered_items {
        delivery_events.send(ItemDelivered { item, count });
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
            // Store previous values for interpolation (before updating)
            conveyor.items[i].previous_progress = conveyor.items[i].progress;
            conveyor.items[i].previous_lateral_offset = conveyor.items[i].lateral_offset;

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
/// Uses 3D GLB models when available, falls back to colored cubes
/// Uses interpolation for smooth rendering between FixedUpdate ticks
pub fn update_conveyor_item_visuals(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    models: Res<MachineModels>,
    fixed_time: Res<Time<Fixed>>,
    mut conveyor_query: Query<&mut Conveyor>,
    mut visual_query: Query<&mut Transform, With<ConveyorItemVisual>>,
) {
    // Fallback mesh for items without GLB models
    let fallback_mesh = meshes.add(Cuboid::new(
        BLOCK_SIZE * CONVEYOR_ITEM_SIZE,
        BLOCK_SIZE * CONVEYOR_ITEM_SIZE,
        BLOCK_SIZE * CONVEYOR_ITEM_SIZE,
    ));

    // Item model scale (GLB models are 8x8x8 voxels = 0.5 blocks, scale down for conveyor)
    const ITEM_MODEL_SCALE: f32 = 0.5;

    // Interpolation factor (0.0 = at previous tick, 1.0 = at current tick)
    let alpha = fixed_time.overstep_fraction();

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
            Direction::East => Vec3::new(0.0, 0.0, 1.0), // Right is +Z (South)
            Direction::West => Vec3::new(0.0, 0.0, -1.0), // Right is -Z (North)
            Direction::South => Vec3::new(-1.0, 0.0, 0.0), // Right is -X (West)
            Direction::North => Vec3::new(1.0, 0.0, 0.0), // Right is +X (East)
        };

        for item in conveyor.items.iter_mut() {
            // Interpolate between previous and current values for smooth rendering
            let interpolated_progress =
                item.previous_progress + (item.progress - item.previous_progress) * alpha;
            let interpolated_lateral = item.previous_lateral_offset
                + (item.lateral_offset - item.previous_lateral_offset) * alpha;

            // Calculate position: progress 0.0 = entry (-0.5), 1.0 = exit (+0.5)
            let forward_offset = (interpolated_progress - 0.5) * BLOCK_SIZE;
            let lateral_offset_world = interpolated_lateral * BLOCK_SIZE;
            let item_pos =
                base_pos + direction_vec * forward_offset + lateral_vec * lateral_offset_world;

            match item.visual_entity {
                None => {
                    // Try to spawn with GLB model, fall back to colored cube
                    let entity = if let Some(scene_handle) = models.get_item_model(item.block_type)
                    {
                        // Spawn GLB model
                        commands
                            .spawn((
                                SceneRoot(scene_handle),
                                Transform::from_translation(item_pos)
                                    .with_scale(Vec3::splat(ITEM_MODEL_SCALE)),
                                GlobalTransform::default(),
                                Visibility::default(),
                                InheritedVisibility::default(),
                                ViewVisibility::default(),
                                ConveyorItemVisual,
                            ))
                            .id()
                    } else {
                        // Fallback: spawn colored cube
                        let material = materials.add(StandardMaterial {
                            base_color: item.block_type.color(),
                            ..default()
                        });
                        commands
                            .spawn((
                                Mesh3d(fallback_mesh.clone()),
                                MeshMaterial3d(material),
                                Transform::from_translation(item_pos),
                                ConveyorItemVisual,
                            ))
                            .id()
                    };
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
