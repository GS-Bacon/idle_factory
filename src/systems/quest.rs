//! Quest and delivery platform systems

use crate::components::*;
use crate::player::GlobalInventory;
use crate::{game_spec, BlockType, BLOCK_SIZE, PLATFORM_SIZE};
use bevy::prelude::*;

/// Quest definition structure (runtime representation)
pub struct QuestDef {
    #[allow(dead_code)]
    pub id: &'static str,
    pub description: &'static str,
    pub required_items: Vec<(BlockType, u32)>,
    pub rewards: Vec<(BlockType, u32)>,
    #[allow(dead_code)]
    pub unlocks: Vec<BlockType>,
}

/// Cached quest data to avoid allocations every frame
#[derive(Resource)]
pub struct QuestCache {
    pub main_quests: Vec<QuestDef>,
    pub sub_quests: Vec<QuestDef>,
}

impl Default for QuestCache {
    fn default() -> Self {
        Self {
            main_quests: build_main_quests(),
            sub_quests: build_sub_quests(),
        }
    }
}

/// Build main quests from game_spec (internal, called once)
fn build_main_quests() -> Vec<QuestDef> {
    game_spec::MAIN_QUESTS
        .iter()
        .map(|spec| QuestDef {
            id: spec.id,
            description: spec.description,
            required_items: spec.required_items.to_vec(),
            rewards: spec.rewards.to_vec(),
            unlocks: spec.unlocks.to_vec(),
        })
        .collect()
}

/// Build sub quests from game_spec (internal, called once)
fn build_sub_quests() -> Vec<QuestDef> {
    game_spec::SUB_QUESTS
        .iter()
        .map(|spec| QuestDef {
            id: spec.id,
            description: spec.description,
            required_items: spec.required_items.to_vec(),
            rewards: spec.rewards.to_vec(),
            unlocks: spec.unlocks.to_vec(),
        })
        .collect()
}

/// Get main quests from game_spec (Single Source of Truth)
/// Note: Prefer using QuestCache resource for hot paths
pub fn get_main_quests() -> Vec<QuestDef> {
    game_spec::MAIN_QUESTS
        .iter()
        .map(|spec| QuestDef {
            id: spec.id,
            description: spec.description,
            required_items: spec.required_items.to_vec(),
            rewards: spec.rewards.to_vec(),
            unlocks: spec.unlocks.to_vec(),
        })
        .collect()
}

/// Get sub quests from game_spec
pub fn get_sub_quests() -> Vec<QuestDef> {
    game_spec::SUB_QUESTS
        .iter()
        .map(|spec| QuestDef {
            id: spec.id,
            description: spec.description,
            required_items: spec.required_items.to_vec(),
            rewards: spec.rewards.to_vec(),
            unlocks: spec.unlocks.to_vec(),
        })
        .collect()
}

/// Legacy: backward compatibility
#[allow(dead_code)]
#[deprecated(note = "Use get_main_quests instead")]
pub fn get_quests() -> Vec<QuestDef> {
    get_main_quests()
}

/// Check quest progress (supports multiple required items)
pub fn quest_progress_check(
    platform_query: Query<&DeliveryPlatform>,
    mut current_quest: ResMut<CurrentQuest>,
    quest_cache: Res<QuestCache>,
) {
    if current_quest.completed {
        return;
    }

    let Ok(platform) = platform_query.get_single() else {
        return;
    };

    let Some(quest) = quest_cache.main_quests.get(current_quest.index) else {
        return;
    };

    // Check if all required items are delivered
    let all_satisfied = quest.required_items.iter().all(|(item, amount)| {
        let delivered = platform.delivered.get(item).copied().unwrap_or(0);
        delivered >= *amount
    });

    if all_satisfied {
        current_quest.completed = true;
    }
}

/// Claim quest rewards with Q key
pub fn quest_claim_rewards(
    key_input: Res<ButtonInput<KeyCode>>,
    mut current_quest: ResMut<CurrentQuest>,
    mut global_inventory: ResMut<GlobalInventory>,
    command_state: Res<CommandInputState>,
    quest_cache: Res<QuestCache>,
) {
    // Don't process while command input is open
    if command_state.open {
        return;
    }

    if !current_quest.completed || current_quest.rewards_claimed {
        return;
    }

    if !key_input.just_pressed(KeyCode::KeyQ) {
        return;
    }

    let Some(quest) = quest_cache.main_quests.get(current_quest.index) else {
        return;
    };

    // Add rewards to GlobalInventory (machines and items)
    for (block_type, amount) in &quest.rewards {
        global_inventory.add_item(*block_type, *amount);
    }

    current_quest.rewards_claimed = true;

    // Move to next quest
    if current_quest.index + 1 < quest_cache.main_quests.len() {
        current_quest.index += 1;
        current_quest.completed = false;
        current_quest.rewards_claimed = false;
    }
}

/// Check if GlobalInventory has enough items to deliver for the current quest
fn can_deliver_from_global_inventory(
    global_inventory: &GlobalInventory,
    quest: &QuestDef,
    platform: Option<&DeliveryPlatform>,
) -> bool {
    for (item, required) in &quest.required_items {
        let already_delivered = platform
            .map(|p| p.delivered.get(item).copied().unwrap_or(0))
            .unwrap_or(0);
        let remaining = required.saturating_sub(already_delivered);
        if remaining > 0 && global_inventory.get_count(*item) < remaining {
            return false;
        }
    }
    true
}

/// Update quest UI (supports multiple required items with progress bars)
#[allow(clippy::type_complexity, clippy::too_many_arguments)]
pub fn update_quest_ui(
    current_quest: Res<CurrentQuest>,
    platform_query: Query<&DeliveryPlatform>,
    global_inventory: Res<GlobalInventory>,
    mut text_query: Query<&mut Text, With<QuestUIText>>,
    mut button_query: Query<(&mut Visibility, &mut BackgroundColor), With<QuestDeliverButton>>,
    mut progress_item_query: Query<
        (&QuestProgressItem, &mut Visibility),
        Without<QuestDeliverButton>,
    >,
    mut progress_text_query: Query<(&QuestProgressText, &mut Text), Without<QuestUIText>>,
    mut progress_fill_query: Query<
        (&QuestProgressBarFill, &mut Node, &mut BackgroundColor),
        Without<QuestProgressItem>,
    >,
    quest_cache: Res<QuestCache>,
) {
    let Ok(mut text) = text_query.get_single_mut() else {
        return;
    };

    if current_quest.index >= quest_cache.main_quests.len() {
        **text = "🎉 全クエスト完了！".to_string();
        // Hide deliver button and progress bars
        for (mut vis, _) in button_query.iter_mut() {
            *vis = Visibility::Hidden;
        }
        for (_, mut vis) in progress_item_query.iter_mut() {
            *vis = Visibility::Hidden;
        }
        return;
    }

    let quest = &quest_cache.main_quests[current_quest.index];
    let platform = platform_query.get_single().ok();

    if current_quest.completed && !current_quest.rewards_claimed {
        // Quest complete - show rewards
        let rewards: Vec<String> = quest
            .rewards
            .iter()
            .map(|(bt, amt)| format!("  {} ×{}", bt.name(), amt))
            .collect();
        **text = format!(
            "✓ クエスト完了！\n\n報酬:\n{}\n\n[Q] 報酬を受け取る",
            rewards.join("\n")
        );
        // Hide deliver button and progress bars when completed
        for (mut vis, _) in button_query.iter_mut() {
            *vis = Visibility::Hidden;
        }
        for (_, mut vis) in progress_item_query.iter_mut() {
            *vis = Visibility::Hidden;
        }
    } else {
        // Show quest description
        **text = quest.description.to_string();

        // Update progress bars for each required item
        for (i, (item, required)) in quest.required_items.iter().enumerate() {
            let delivered = platform
                .map(|p| p.delivered.get(item).copied().unwrap_or(0))
                .unwrap_or(0);
            let in_storage = global_inventory.get_count(*item);
            let progress_pct = if *required > 0 {
                (delivered as f32 / *required as f32 * 100.0).min(100.0)
            } else {
                100.0
            };

            // Update visibility
            for (progress_item, mut vis) in progress_item_query.iter_mut() {
                if progress_item.0 == i {
                    *vis = Visibility::Visible;
                }
            }

            // Update text
            for (progress_text, mut txt) in progress_text_query.iter_mut() {
                if progress_text.0 == i {
                    let status_icon = if delivered >= *required { "✓" } else { "○" };
                    **txt = format!(
                        "{} {} ({}/{}) [在庫:{}]",
                        status_icon,
                        item.name(),
                        delivered.min(*required),
                        required,
                        in_storage
                    );
                }
            }

            // Update progress bar fill
            for (fill, mut node, mut bg) in progress_fill_query.iter_mut() {
                if fill.0 == i {
                    node.width = Val::Percent(progress_pct);
                    // Color based on progress
                    *bg = if delivered >= *required {
                        BackgroundColor(Color::srgba(0.2, 0.7, 0.3, 1.0)) // Green when complete
                    } else if in_storage >= required.saturating_sub(delivered) {
                        BackgroundColor(Color::srgba(0.5, 0.7, 0.2, 1.0)) // Yellow-green when can deliver
                    } else {
                        BackgroundColor(Color::srgba(0.3, 0.5, 0.6, 1.0)) // Blue when in progress
                    };
                }
            }
        }

        // Hide unused progress slots
        for (progress_item, mut vis) in progress_item_query.iter_mut() {
            if progress_item.0 >= quest.required_items.len() {
                *vis = Visibility::Hidden;
            }
        }

        // Show deliver button if can deliver from GlobalInventory
        let can_deliver = can_deliver_from_global_inventory(&global_inventory, quest, platform);
        for (mut vis, mut bg) in button_query.iter_mut() {
            if can_deliver {
                *vis = Visibility::Visible;
                *bg = BackgroundColor(Color::srgba(0.15, 0.5, 0.2, 0.95));
            } else {
                *vis = Visibility::Hidden;
            }
        }
    }
}

/// Handle deliver button click - consume from GlobalInventory and deliver to platform
#[allow(clippy::type_complexity)]
pub fn quest_deliver_button(
    mut current_quest: ResMut<CurrentQuest>,
    mut global_inventory: ResMut<GlobalInventory>,
    mut platform_query: Query<&mut DeliveryPlatform>,
    mut button_query: Query<
        (&Interaction, &mut BackgroundColor, &mut BorderColor),
        (Changed<Interaction>, With<QuestDeliverButton>),
    >,
    quest_cache: Res<QuestCache>,
) {
    if current_quest.completed {
        return;
    }

    let Some(quest) = quest_cache.main_quests.get(current_quest.index) else {
        return;
    };

    let Ok(mut platform) = platform_query.get_single_mut() else {
        return;
    };

    for (interaction, mut bg_color, mut border_color) in button_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                // Consume items from GlobalInventory and deliver to platform
                let mut success = true;
                for (item, required) in &quest.required_items {
                    let already_delivered = platform.delivered.get(item).copied().unwrap_or(0);
                    let remaining = required.saturating_sub(already_delivered);
                    if remaining > 0 {
                        if global_inventory.remove_item(*item, remaining) {
                            *platform.delivered.entry(*item).or_insert(0) += remaining;
                        } else {
                            success = false;
                            break;
                        }
                    }
                }

                if success {
                    // Check if quest is now complete
                    let all_satisfied = quest.required_items.iter().all(|(item, amount)| {
                        let delivered = platform.delivered.get(item).copied().unwrap_or(0);
                        delivered >= *amount
                    });
                    if all_satisfied {
                        current_quest.completed = true;
                    }
                }

                *border_color = BorderColor(Color::srgb(0.5, 1.0, 0.5));
            }
            Interaction::Hovered => {
                *bg_color = BackgroundColor(Color::srgba(0.3, 0.6, 0.3, 0.95));
                *border_color = BorderColor(Color::srgba(0.4, 0.8, 0.4, 1.0));
            }
            Interaction::None => {
                *bg_color = BackgroundColor(Color::srgba(0.2, 0.5, 0.2, 0.95));
                *border_color = BorderColor(Color::srgba(0.3, 0.6, 0.3, 1.0));
            }
        }
    }
}

/// Setup delivery platform near spawn point
pub fn setup_delivery_platform(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Platform position: 12x12 area starting at (20, 8, 10)
    let platform_origin = IVec3::new(20, 8, 10);

    // Create platform mesh (flat plate)
    let platform_mesh = meshes.add(Cuboid::new(
        PLATFORM_SIZE as f32 * BLOCK_SIZE,
        BLOCK_SIZE * 0.2,
        PLATFORM_SIZE as f32 * BLOCK_SIZE,
    ));

    let platform_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.2, 0.5, 0.3), // Green-ish for delivery area
        ..default()
    });

    // Spawn platform entity
    // Platform center: origin + half_size (in blocks), then offset by 0.5 for grid alignment
    commands.spawn((
        Mesh3d(platform_mesh),
        MeshMaterial3d(platform_material),
        Transform::from_translation(Vec3::new(
            platform_origin.x as f32 * BLOCK_SIZE + (PLATFORM_SIZE as f32 * BLOCK_SIZE / 2.0),
            platform_origin.y as f32 * BLOCK_SIZE + 0.1,
            platform_origin.z as f32 * BLOCK_SIZE + (PLATFORM_SIZE as f32 * BLOCK_SIZE / 2.0),
        )),
        DeliveryPlatform::default(),
    ));

    // Spawn delivery port markers (visual indicators at edges)
    // Use tall vertical markers for better visibility
    let port_mesh = meshes.add(Cuboid::new(
        BLOCK_SIZE * 0.3,
        BLOCK_SIZE * 0.8,
        BLOCK_SIZE * 0.3,
    ));
    let port_material = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 0.9, 0.2), // Bright yellow for ports
        emissive: bevy::color::LinearRgba::new(0.5, 0.45, 0.1, 1.0),
        ..default()
    });

    // Create 16 ports along edges (4 per side)
    let port_positions = [
        // North edge (z = 10)
        IVec3::new(22, 8, 10),
        IVec3::new(25, 8, 10),
        IVec3::new(28, 8, 10),
        IVec3::new(31, 8, 10),
        // South edge (z = 21)
        IVec3::new(22, 8, 21),
        IVec3::new(25, 8, 21),
        IVec3::new(28, 8, 21),
        IVec3::new(31, 8, 21),
        // West edge (x = 20)
        IVec3::new(20, 8, 12),
        IVec3::new(20, 8, 15),
        IVec3::new(20, 8, 18),
        IVec3::new(20, 8, 21),
        // East edge (x = 31)
        IVec3::new(31, 8, 12),
        IVec3::new(31, 8, 15),
        IVec3::new(31, 8, 18),
        IVec3::new(31, 8, 21),
    ];

    for port_pos in port_positions {
        commands.spawn((
            Mesh3d(port_mesh.clone()),
            MeshMaterial3d(port_material.clone()),
            Transform::from_translation(Vec3::new(
                port_pos.x as f32 * BLOCK_SIZE + 0.5,
                port_pos.y as f32 * BLOCK_SIZE + 0.5,
                port_pos.z as f32 * BLOCK_SIZE + 0.5,
            )),
        ));
    }
}

/// Update delivery UI text
pub fn update_delivery_ui(
    platform_query: Query<&DeliveryPlatform>,
    mut text_query: Query<&mut Text, With<DeliveryUIText>>,
) {
    let Ok(platform) = platform_query.get_single() else {
        return;
    };

    let Ok(mut text) = text_query.get_single_mut() else {
        return;
    };

    if platform.delivered.is_empty() {
        **text = "=== Deliveries ===\nNo items delivered".to_string();
    } else {
        let items: Vec<String> = platform
            .delivered
            .iter()
            .map(|(block_type, count)| format!("{}: {}", block_type.name(), count))
            .collect();
        **text = format!("=== Deliveries ===\n{}", items.join("\n"));
    }
}

/// Load 3D models for machines and conveyors (if available)
pub fn load_machine_models(
    asset_server: Res<AssetServer>,
    mut models: ResMut<MachineModels>,
    mut item_sprites: ResMut<ItemSprites>,
) {
    // Try to load conveyor models
    models.conveyor_straight =
        Some(asset_server.load("models/machines/conveyor/straight.glb#Scene0"));
    models.conveyor_corner_left =
        Some(asset_server.load("models/machines/conveyor/corner_left.glb#Scene0"));
    models.conveyor_corner_right =
        Some(asset_server.load("models/machines/conveyor/corner_right.glb#Scene0"));
    models.conveyor_t_junction =
        Some(asset_server.load("models/machines/conveyor/t_junction.glb#Scene0"));
    models.conveyor_splitter =
        Some(asset_server.load("models/machines/conveyor/splitter.glb#Scene0"));

    // Try to load machine models
    models.miner = Some(asset_server.load("models/machines/miner.glb#Scene0"));
    models.furnace = Some(asset_server.load("models/machines/furnace.glb#Scene0"));
    models.crusher = Some(asset_server.load("models/machines/crusher.glb#Scene0"));

    // Try to load item models (for conveyor display)
    models.item_iron_ore = Some(asset_server.load("models/items/iron_ore.glb#Scene0"));
    models.item_copper_ore = Some(asset_server.load("models/items/copper_ore.glb#Scene0"));
    models.item_coal = Some(asset_server.load("models/items/coal.glb#Scene0"));
    models.item_stone = Some(asset_server.load("models/items/stone.glb#Scene0"));
    models.item_iron_ingot = Some(asset_server.load("models/items/iron_ingot.glb#Scene0"));
    models.item_copper_ingot = Some(asset_server.load("models/items/copper_ingot.glb#Scene0"));

    // Will check if loaded in update system
    models.loaded = false;

    // Load item sprites for UI
    use crate::BlockType;
    item_sprites.textures.insert(
        BlockType::IronOre,
        asset_server.load("textures/items/iron_ore.png"),
    );
    item_sprites.textures.insert(
        BlockType::CopperOre,
        asset_server.load("textures/items/copper_ore.png"),
    );
    item_sprites.textures.insert(
        BlockType::Coal,
        asset_server.load("textures/items/coal.png"),
    );
    item_sprites.textures.insert(
        BlockType::Stone,
        asset_server.load("textures/items/stone.png"),
    );
    item_sprites.textures.insert(
        BlockType::IronIngot,
        asset_server.load("textures/items/iron_ingot.png"),
    );
    item_sprites.textures.insert(
        BlockType::CopperIngot,
        asset_server.load("textures/items/copper_ingot.png"),
    );
    item_sprites.textures.insert(
        BlockType::MinerBlock,
        asset_server.load("textures/items/miner.png"),
    );
    item_sprites.textures.insert(
        BlockType::ConveyorBlock,
        asset_server.load("textures/items/conveyor.png"),
    );
    item_sprites.textures.insert(
        BlockType::FurnaceBlock,
        asset_server.load("textures/items/furnace.png"),
    );
    item_sprites.textures.insert(
        BlockType::CrusherBlock,
        asset_server.load("textures/items/crusher.png"),
    );
}
