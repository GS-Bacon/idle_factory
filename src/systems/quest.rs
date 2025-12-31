//! Quest and delivery platform systems

use crate::components::*;
use crate::player::Inventory;
use crate::{game_spec, BlockType, BLOCK_SIZE, PLATFORM_SIZE};
use bevy::prelude::*;

/// Quest definition structure
pub struct QuestDef {
    pub description: &'static str,
    pub required_item: BlockType,
    pub required_amount: u32,
    pub rewards: Vec<(BlockType, u32)>,
}

/// Quest definitions from game_spec (Single Source of Truth)
pub fn get_quests() -> Vec<QuestDef> {
    game_spec::QUESTS
        .iter()
        .map(|spec| QuestDef {
            description: spec.description,
            required_item: spec.required_item,
            required_amount: spec.required_amount,
            rewards: spec.rewards.to_vec(),
        })
        .collect()
}

/// Check quest progress
pub fn quest_progress_check(
    platform_query: Query<&DeliveryPlatform>,
    mut current_quest: ResMut<CurrentQuest>,
) {
    if current_quest.completed {
        return;
    }

    let Ok(platform) = platform_query.get_single() else {
        return;
    };

    let quests = get_quests();
    let Some(quest) = quests.get(current_quest.index) else {
        return;
    };

    let delivered = platform.delivered.get(&quest.required_item).copied().unwrap_or(0);
    if delivered >= quest.required_amount {
        current_quest.completed = true;
    }
}

/// Claim quest rewards with Q key
pub fn quest_claim_rewards(
    key_input: Res<ButtonInput<KeyCode>>,
    mut current_quest: ResMut<CurrentQuest>,
    mut inventory: ResMut<Inventory>,
    command_state: Res<CommandInputState>,
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

    let quests = get_quests();
    let Some(quest) = quests.get(current_quest.index) else {
        return;
    };

    // Add rewards to inventory
    for (block_type, amount) in &quest.rewards {
        inventory.add_item(*block_type, *amount);
    }

    current_quest.rewards_claimed = true;

    // Move to next quest
    if current_quest.index + 1 < quests.len() {
        current_quest.index += 1;
        current_quest.completed = false;
        current_quest.rewards_claimed = false;
    }
}

/// Update quest UI
pub fn update_quest_ui(
    current_quest: Res<CurrentQuest>,
    platform_query: Query<&DeliveryPlatform>,
    mut text_query: Query<&mut Text, With<QuestUIText>>,
) {
    let Ok(mut text) = text_query.get_single_mut() else {
        return;
    };

    let quests = get_quests();

    if current_quest.index >= quests.len() {
        **text = "=== Quest ===\nAll quests completed!".to_string();
        return;
    }

    let quest = &quests[current_quest.index];
    let delivered = platform_query
        .get_single()
        .map(|p| p.delivered.get(&quest.required_item).copied().unwrap_or(0))
        .unwrap_or(0);

    if current_quest.completed && !current_quest.rewards_claimed {
        let rewards: Vec<String> = quest.rewards
            .iter()
            .map(|(bt, amt)| format!("{} x{}", bt.name(), amt))
            .collect();
        **text = format!(
            "=== Quest Complete! ===\n{}\n\nRewards:\n{}\n\n[Q] Claim Rewards",
            quest.description,
            rewards.join("\n")
        );
    } else {
        **text = format!(
            "=== Quest ===\n{}\nProgress: {}/{}",
            quest.description,
            delivered.min(quest.required_amount),
            quest.required_amount
        );
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
    let port_mesh = meshes.add(Cuboid::new(BLOCK_SIZE * 0.3, BLOCK_SIZE * 0.8, BLOCK_SIZE * 0.3));
    let port_material = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 0.9, 0.2), // Bright yellow for ports
        emissive: bevy::color::LinearRgba::new(0.5, 0.45, 0.1, 1.0),
        ..default()
    });

    // Create 16 ports along edges (4 per side)
    let port_positions = [
        // North edge (z = 10)
        IVec3::new(22, 8, 10), IVec3::new(25, 8, 10), IVec3::new(28, 8, 10), IVec3::new(31, 8, 10),
        // South edge (z = 21)
        IVec3::new(22, 8, 21), IVec3::new(25, 8, 21), IVec3::new(28, 8, 21), IVec3::new(31, 8, 21),
        // West edge (x = 20)
        IVec3::new(20, 8, 12), IVec3::new(20, 8, 15), IVec3::new(20, 8, 18), IVec3::new(20, 8, 21),
        // East edge (x = 31)
        IVec3::new(31, 8, 12), IVec3::new(31, 8, 15), IVec3::new(31, 8, 18), IVec3::new(31, 8, 21),
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
) {
    // Try to load conveyor models
    models.conveyor_straight = Some(asset_server.load("models/machines/conveyor/straight.glb#Scene0"));
    models.conveyor_corner_left = Some(asset_server.load("models/machines/conveyor/corner_left.glb#Scene0"));
    models.conveyor_corner_right = Some(asset_server.load("models/machines/conveyor/corner_right.glb#Scene0"));
    models.conveyor_t_junction = Some(asset_server.load("models/machines/conveyor/t_junction.glb#Scene0"));
    models.conveyor_splitter = Some(asset_server.load("models/machines/conveyor/splitter.glb#Scene0"));

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
}
