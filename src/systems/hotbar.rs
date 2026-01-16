//! Hotbar UI systems

use crate::components::*;
use crate::input::{GameAction, InputManager};
use crate::player::{LocalPlayer, PlayerInventory};
use crate::systems::block_operations::LocalPlayerInventory;
use bevy::prelude::*;

/// Update hotbar UI display
pub fn update_hotbar_ui(
    local_player: Option<Res<LocalPlayer>>,
    inventory_query: Query<&PlayerInventory>,
    item_sprites: Res<ItemSprites>,
    asset_server: Res<AssetServer>,
    mut slot_query: Query<(&HotbarSlot, &mut BackgroundColor, &mut BorderColor)>,
    mut count_query: Query<(&HotbarSlotCount, &mut Text)>,
    mut image_query: Query<(&HotbarSlotImage, &mut ImageNode, &mut Visibility)>,
) {
    // Get local player's inventory
    let Some(local_player) = local_player else {
        return;
    };
    let Ok(inventory) = inventory_query.get(local_player.0) else {
        return;
    };

    // Update slot backgrounds - always run (for selection highlight)
    for (slot, mut bg, mut border) in slot_query.iter_mut() {
        let is_selected = inventory.selected_slot == slot.0;
        let has_item = inventory.get_slot_item_id(slot.0).is_some();

        if is_selected {
            // Selected slot - same highlight for empty and filled
            *bg = BackgroundColor(Color::srgba(0.4, 0.4, 0.2, 0.9));
            *border = BorderColor(Color::srgba(1.0, 1.0, 0.5, 1.0));
        } else if has_item {
            // Non-selected filled slot
            *bg = BackgroundColor(Color::srgba(0.3, 0.3, 0.3, 0.8));
            *border = BorderColor(Color::srgba(0.5, 0.5, 0.5, 1.0));
        } else {
            // Non-selected empty slot
            *bg = BackgroundColor(Color::srgba(0.2, 0.2, 0.2, 0.8));
            *border = BorderColor(Color::srgba(0.4, 0.4, 0.4, 1.0));
        }
    }

    // Check if any sprite assets are still loading
    let sprites_loading = item_sprites.textures.values().any(|h| {
        !matches!(
            asset_server.get_load_state(h),
            Some(bevy::asset::LoadState::Loaded)
        )
    });

    // Update sprites only when resource changes or sprites are loading
    // (need to keep checking while loading to catch when they finish)
    if !item_sprites.is_changed() && !sprites_loading {
        return;
    }

    // Update slot sprite images with visibility control
    for (slot_image, mut image_node, mut visibility) in image_query.iter_mut() {
        if let Some(item_id) = inventory.get_slot_item_id(slot_image.0) {
            if let Some(sprite_handle) = item_sprites.get_id(item_id) {
                image_node.image = sprite_handle.clone();
                *visibility = Visibility::Inherited;
            } else {
                *visibility = Visibility::Hidden;
            }
        } else {
            *visibility = Visibility::Hidden;
        }
    }

    // Update slot counts (only show number when count > 1)
    for (slot_count, mut text) in count_query.iter_mut() {
        if inventory.get_slot_item_id(slot_count.0).is_some() {
            let count = inventory.get_slot_count(slot_count.0);
            if count > 1 {
                **text = count.to_string();
            } else {
                **text = String::new();
            }
        } else {
            **text = String::new();
        }
    }
}

/// Update the hotbar item name display to show the selected item's name
pub fn update_hotbar_item_name(
    local_player: Option<Res<LocalPlayer>>,
    inventory_query: Query<&PlayerInventory>,
    inventory_open: Res<InventoryOpen>,
    mut text_query: Query<(&mut Text, &mut Node), With<HotbarItemNameText>>,
) {
    let Ok((mut text, mut node)) = text_query.get_single_mut() else {
        return;
    };

    // Hide when inventory is open
    if inventory_open.0 {
        text.0 = String::new();
        return;
    }

    // Get local player's inventory
    let Some(local_player) = local_player else {
        text.0 = String::new();
        return;
    };
    let Ok(inventory) = inventory_query.get(local_player.0) else {
        text.0 = String::new();
        return;
    };

    // Show selected item name
    if let Some(item_id) = inventory.selected_item_id() {
        // Get display name using ItemId's display_name() method
        let name = item_id.display_name().to_string();
        text.0 = name.clone();
        // Center the text by adjusting margin based on text length
        let char_width = 8.0; // Approximate character width
        node.margin.left = Val::Px(-(name.len() as f32 * char_width / 2.0));
    } else {
        text.0 = String::new();
    }
}

/// Select slot with number keys (1-9) or scroll wheel
pub fn select_block_type(
    input: Res<InputManager>,
    mut mouse_wheel: EventReader<bevy::input::mouse::MouseWheel>,
    mut local_player_inventory: LocalPlayerInventory,
    input_resources: InputStateResourcesWithCursor,
) {
    use crate::HOTBAR_SLOTS;

    // Use InputState to check if hotbar selection is allowed (see CLAUDE.md input matrix)
    let input_state = input_resources.get_state();
    if !input_state.allows_hotbar() {
        // Still need to drain events to prevent accumulation
        for _ in mouse_wheel.read() {}
        return;
    }

    // Get mutable access to local player's inventory
    let Some(mut inventory) = local_player_inventory.get_mut() else {
        // Still need to drain events to prevent accumulation
        for _ in mouse_wheel.read() {}
        return;
    };

    // Handle mouse wheel scroll (cycles through hotbar slots 0-8 only)
    for event in mouse_wheel.read() {
        let scroll = event.y;
        if scroll > 0.0 {
            // Scroll up - previous slot (within hotbar)
            if inventory.selected_slot > 0 {
                inventory.selected_slot -= 1;
            } else {
                inventory.selected_slot = HOTBAR_SLOTS - 1;
            }
        } else if scroll < 0.0 {
            // Scroll down - next slot (within hotbar)
            if inventory.selected_slot < HOTBAR_SLOTS - 1 {
                inventory.selected_slot += 1;
            } else {
                inventory.selected_slot = 0;
            }
        }
    }

    // Number keys 1-9 select hotbar slots directly via InputManager
    const HOTBAR_ACTIONS: [(GameAction, usize); 9] = [
        (GameAction::Hotbar1, 0),
        (GameAction::Hotbar2, 1),
        (GameAction::Hotbar3, 2),
        (GameAction::Hotbar4, 3),
        (GameAction::Hotbar5, 4),
        (GameAction::Hotbar6, 5),
        (GameAction::Hotbar7, 6),
        (GameAction::Hotbar8, 7),
        (GameAction::Hotbar9, 8),
    ];
    for (action, slot) in HOTBAR_ACTIONS {
        if input.just_pressed(action) {
            inventory.selected_slot = slot;
        }
    }
}

/// Track currently displayed held item to avoid unnecessary respawns
#[derive(Resource, Default)]
pub struct HeldItemDisplayState {
    pub current_item: Option<crate::core::ItemId>,
    pub scene_entity: Option<Entity>,
}

/// Update 3D held item display based on selected hotbar item
/// Shows cube mesh for simple items, and 3D model scenes for machines
#[allow(clippy::too_many_arguments)]
pub fn update_held_item_3d(
    mut commands: Commands,
    local_player: Option<Res<LocalPlayer>>,
    inventory_query: Query<&PlayerInventory>,
    cache: Option<Res<HeldItem3DCache>>,
    machine_models: Option<Res<MachineModels>>,
    mut display_state: ResMut<HeldItemDisplayState>,
    mut cube_query: Query<
        (
            Entity,
            &mut MeshMaterial3d<StandardMaterial>,
            &mut Visibility,
        ),
        With<HeldItem3D>,
    >,
    scene_query: Query<Entity, With<HeldItem3DScene>>,
) {
    use bevy::render::view::RenderLayers;

    // Get local player's inventory
    let Some(local_player) = local_player else {
        hide_all(
            &mut cube_query,
            &scene_query,
            &mut commands,
            &mut display_state,
        );
        return;
    };
    let Ok(inventory) = inventory_query.get(local_player.0) else {
        hide_all(
            &mut cube_query,
            &scene_query,
            &mut commands,
            &mut display_state,
        );
        return;
    };

    // Get selected item
    let Some(item_id) = inventory.selected_item_id() else {
        hide_all(
            &mut cube_query,
            &scene_query,
            &mut commands,
            &mut display_state,
        );
        return;
    };

    // Check if item changed
    if display_state.current_item == Some(item_id) {
        return; // No change, skip update
    }

    // Remove old scene entity if exists
    if let Some(old_entity) = display_state.scene_entity.take() {
        commands.entity(old_entity).despawn_recursive();
    }

    // Check if this item has a machine model
    let has_machine_model = machine_models
        .as_ref()
        .map(|m| m.get_held_item_scene(item_id).is_some())
        .unwrap_or(false);

    if has_machine_model {
        // Hide cube mesh
        if let Ok((_, _, mut visibility)) = cube_query.get_single_mut() {
            *visibility = Visibility::Hidden;
        }

        // Spawn scene for machine model
        if let Some(models) = machine_models.as_ref() {
            if let Some(scene_handle) = models.get_held_item_scene(item_id) {
                // Get the parent (HeldItem3D) to spawn the scene as sibling
                if let Ok((cube_entity, _, _)) = cube_query.get_single() {
                    // Spawn scene as sibling of cube with same transform style
                    let scene_entity = commands
                        .spawn((
                            HeldItem3DScene,
                            SceneRoot(scene_handle),
                            Transform::from_xyz(0.5, -0.4, -0.8)
                                .with_rotation(Quat::from_euler(EulerRot::YXZ, -0.3, 0.2, 0.1))
                                .with_scale(Vec3::splat(0.3)), // Smaller scale for hand display
                            Visibility::Inherited,
                            RenderLayers::layer(1), // Same overlay layer as cube
                        ))
                        .id();

                    // Make scene a sibling of cube (same parent)
                    if let Some(parent) = commands.get_entity(cube_entity).map(|_| {
                        // Get parent by querying - we need to find the camera parent
                        cube_entity
                    }) {
                        if let Some(mut entity_commands) = commands.get_entity(parent) {
                            // Insert scene as sibling by adding to same parent
                            // The cube is a child of the camera, so we need to add scene there too
                            entity_commands.add_child(scene_entity);
                        }
                    }

                    display_state.scene_entity = Some(scene_entity);
                }
            }
        }
    } else {
        // Show cube mesh for non-machine items
        if let Some(cache) = cache.as_ref() {
            if let Ok((_, mut material, mut visibility)) = cube_query.get_single_mut() {
                if let Some(block_material) = cache.materials.get(&item_id) {
                    material.0 = block_material.clone();
                    *visibility = Visibility::Inherited;
                } else {
                    *visibility = Visibility::Hidden;
                }
            }
        }
    }

    display_state.current_item = Some(item_id);
}

/// Hide all held item displays
fn hide_all(
    cube_query: &mut Query<
        (
            Entity,
            &mut MeshMaterial3d<StandardMaterial>,
            &mut Visibility,
        ),
        With<HeldItem3D>,
    >,
    scene_query: &Query<Entity, With<HeldItem3DScene>>,
    commands: &mut Commands,
    display_state: &mut HeldItemDisplayState,
) {
    // Hide cube
    if let Ok((_, _, mut visibility)) = cube_query.get_single_mut() {
        *visibility = Visibility::Hidden;
    }

    // Despawn scene
    for entity in scene_query.iter() {
        commands.entity(entity).despawn_recursive();
    }

    display_state.current_item = None;
    display_state.scene_entity = None;
}
