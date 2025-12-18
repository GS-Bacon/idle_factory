// src/gameplay/delivery_platform.rs
//! 納品プラットフォームシステム
//! - 12x12のグリッドベースの納品エリア
//! - コンベアからのアイテム受け入れ
//! - クエスト進捗への連携

use bevy::prelude::*;
use std::collections::HashMap;

use super::grid::{Direction, SimulationGrid};
use super::quest::{QuestManager, QuestRegistry, QuestStatus};

/// 納品プラットフォームのサイズ
pub const PLATFORM_SIZE: i32 = 12;

/// 納品ポートの位置と状態
#[derive(Debug, Clone)]
pub struct DeliveryPort {
    pub local_x: i32,
    pub local_z: i32,
    pub direction: Direction, // アイテム受け入れ方向
    pub is_active: bool,
    pub filter: Option<String>, // フィルタ（特定アイテムのみ受け入れ）
}

impl DeliveryPort {
    pub fn new(local_x: i32, local_z: i32, direction: Direction) -> Self {
        Self {
            local_x,
            local_z,
            direction,
            is_active: true,
            filter: None,
        }
    }

    pub fn with_filter(mut self, item_id: impl Into<String>) -> Self {
        self.filter = Some(item_id.into());
        self
    }

    /// アイテムを受け入れ可能かチェック
    pub fn can_accept(&self, item_id: &str) -> bool {
        if !self.is_active {
            return false;
        }
        if let Some(filter) = &self.filter {
            return filter == item_id;
        }
        true
    }
}

/// 納品プラットフォーム（コンポーネント）
#[derive(Component)]
pub struct DeliveryPlatform {
    pub grid_pos: IVec3,        // ワールド座標での位置
    pub ports: Vec<DeliveryPort>,
    pub delivered_items: HashMap<String, u32>, // 納品済みアイテム
    pub linked_quest_id: Option<String>,       // 関連クエストID
}

impl DeliveryPlatform {
    pub fn new(grid_pos: IVec3) -> Self {
        // デフォルトで4辺にポートを配置（各辺3ポート）
        let mut ports = Vec::new();

        // 北側（Z-）
        for x in 3..=8 {
            if x % 2 == 0 {
                ports.push(DeliveryPort::new(x, 0, Direction::North));
            }
        }

        // 南側（Z+）
        for x in 3..=8 {
            if x % 2 == 0 {
                ports.push(DeliveryPort::new(x, PLATFORM_SIZE - 1, Direction::South));
            }
        }

        // 東側（X+）
        for z in 3..=8 {
            if z % 2 == 0 {
                ports.push(DeliveryPort::new(PLATFORM_SIZE - 1, z, Direction::East));
            }
        }

        // 西側（X-）
        for z in 3..=8 {
            if z % 2 == 0 {
                ports.push(DeliveryPort::new(0, z, Direction::West));
            }
        }

        Self {
            grid_pos,
            ports,
            delivered_items: HashMap::new(),
            linked_quest_id: None,
        }
    }

    /// ポートのワールド座標を取得
    pub fn get_port_world_pos(&self, port: &DeliveryPort) -> IVec3 {
        IVec3::new(
            self.grid_pos.x + port.local_x,
            self.grid_pos.y,
            self.grid_pos.z + port.local_z,
        )
    }

    /// アクティブなポート数を取得
    pub fn active_port_count(&self) -> usize {
        self.ports.iter().filter(|p| p.is_active).count()
    }

    /// ポートの有効/無効を設定
    pub fn set_port_active(&mut self, index: usize, active: bool) {
        if let Some(port) = self.ports.get_mut(index) {
            port.is_active = active;
        }
    }

    /// アイテムを納品
    pub fn deliver_item(&mut self, item_id: &str, amount: u32) {
        *self.delivered_items.entry(item_id.to_string()).or_insert(0) += amount;
    }

    /// 納品済みアイテム数を取得
    pub fn get_delivered(&self, item_id: &str) -> u32 {
        *self.delivered_items.get(item_id).unwrap_or(&0)
    }

    /// クエストをリンク
    pub fn link_quest(&mut self, quest_id: impl Into<String>) {
        self.linked_quest_id = Some(quest_id.into());
    }
}

/// 納品イベント
#[derive(Event)]
pub struct ItemDeliveredEvent {
    pub platform_entity: Entity,
    pub item_id: String,
    pub amount: u32,
}

/// プラットフォーム生成イベント
#[derive(Event)]
pub struct SpawnPlatformEvent {
    pub pos: IVec3,
}

/// 納品プラットフォームマーカー（ビジュアル用）
#[derive(Component)]
pub struct DeliveryPlatformVisual;

/// 納品ポートマーカー（ビジュアル用）
#[derive(Component)]
pub struct DeliveryPortVisual {
    pub port_index: usize,
}

/// 納品プラットフォームプラグイン
pub struct DeliveryPlatformPlugin;

impl Plugin for DeliveryPlatformPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ItemDeliveredEvent>()
            .add_event::<SpawnPlatformEvent>()
            .add_systems(Startup, spawn_initial_platform)
            .add_systems(
                Update,
                (
                    handle_spawn_platform,
                    receive_items_from_conveyors,
                    sync_delivery_to_quest,
                    update_port_availability,
                ),
            );
    }
}

/// 初期プラットフォームを生成
fn spawn_initial_platform(mut ev_spawn: EventWriter<SpawnPlatformEvent>) {
    // 原点付近にプラットフォームを配置
    ev_spawn.send(SpawnPlatformEvent {
        pos: IVec3::new(-6, 0, -6), // 中心が原点になるように配置
    });
}

/// プラットフォーム生成ハンドラ
fn handle_spawn_platform(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut ev_spawn: EventReader<SpawnPlatformEvent>,
) {
    for event in ev_spawn.read() {
        let platform = DeliveryPlatform::new(event.pos);

        // プラットフォーム本体のメッシュ
        let platform_mesh = meshes.add(Cuboid::new(
            PLATFORM_SIZE as f32,
            0.2,
            PLATFORM_SIZE as f32,
        ));

        let platform_material = materials.add(StandardMaterial {
            base_color: Color::srgb(0.2, 0.3, 0.5),
            metallic: 0.3,
            perceptual_roughness: 0.8,
            ..default()
        });

        let platform_entity = commands
            .spawn((
                Mesh3d(platform_mesh),
                MeshMaterial3d(platform_material),
                Transform::from_translation(Vec3::new(
                    event.pos.x as f32 + PLATFORM_SIZE as f32 / 2.0,
                    event.pos.y as f32 - 0.1,
                    event.pos.z as f32 + PLATFORM_SIZE as f32 / 2.0,
                )),
                DeliveryPlatformVisual,
                platform,
            ))
            .id();

        // ポートのビジュアルを生成
        spawn_port_visuals(&mut commands, &mut meshes, &mut materials, platform_entity, event.pos);

        info!(
            "Spawned delivery platform at {:?}",
            event.pos
        );
    }
}

/// ポートビジュアルを生成
fn spawn_port_visuals(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    _platform_entity: Entity,
    platform_pos: IVec3,
) {
    let port_mesh = meshes.add(Cuboid::new(0.8, 0.3, 0.8));
    let active_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.2, 0.8, 0.2),
        emissive: LinearRgba::rgb(0.1, 0.4, 0.1),
        ..default()
    });

    // デフォルトポート位置（各辺3つずつ）
    let port_positions = [
        // 北側
        (4, 0),
        (6, 0),
        (8, 0),
        // 南側
        (4, 11),
        (6, 11),
        (8, 11),
        // 東側
        (11, 4),
        (11, 6),
        (11, 8),
        // 西側
        (0, 4),
        (0, 6),
        (0, 8),
    ];

    for (index, (local_x, local_z)) in port_positions.iter().enumerate() {
        commands.spawn((
            Mesh3d(port_mesh.clone()),
            MeshMaterial3d(active_material.clone()),
            Transform::from_translation(Vec3::new(
                platform_pos.x as f32 + *local_x as f32 + 0.5,
                platform_pos.y as f32 + 0.15,
                platform_pos.z as f32 + *local_z as f32 + 0.5,
            )),
            DeliveryPortVisual { port_index: index },
        ));
    }
}

/// コンベアからアイテムを受け取る
fn receive_items_from_conveyors(
    mut platforms: Query<(Entity, &mut DeliveryPlatform)>,
    mut grid: ResMut<SimulationGrid>,
    mut ev_delivered: EventWriter<ItemDeliveredEvent>,
) {
    for (entity, mut platform) in platforms.iter_mut() {
        // まずポート情報を収集（借用を分離）
        let port_infos: Vec<(IVec3, Direction, Option<String>)> = platform
            .ports
            .iter()
            .filter(|p| p.is_active)
            .map(|port| {
                (
                    platform.get_port_world_pos(port),
                    port.direction,
                    port.filter.clone(),
                )
            })
            .collect();

        // 各ポートを処理
        for (port_pos, direction, filter) in port_infos {
            let input_pos = port_pos + direction.opposite().to_ivec3();

            // 入力位置のコンベアからアイテムを取得
            if let Some(machine) = grid.machines.get_mut(&input_pos) {
                if let super::grid::Machine::Conveyor(ref mut conveyor) = machine.machine_type {
                    // 完全に進んだアイテムを探す
                    let mut items_to_remove = Vec::new();

                    for (i, slot) in conveyor.inventory.iter().enumerate() {
                        if slot.progress >= 1.0 {
                            let can_accept = if let Some(ref f) = filter {
                                f == &slot.item_id
                            } else {
                                true
                            };
                            if can_accept {
                                items_to_remove.push((i, slot.item_id.clone(), slot.count));
                            }
                        }
                    }

                    // アイテムを納品
                    for (i, item_id, count) in items_to_remove.into_iter().rev() {
                        conveyor.inventory.remove(i);
                        platform.deliver_item(&item_id, count);

                        ev_delivered.send(ItemDeliveredEvent {
                            platform_entity: entity,
                            item_id: item_id.clone(),
                            amount: count,
                        });
                    }
                }
            }
        }
    }
}

/// 納品をクエスト進捗に反映
fn sync_delivery_to_quest(
    mut ev_delivered: EventReader<ItemDeliveredEvent>,
    platforms: Query<&DeliveryPlatform>,
    mut quest_manager: ResMut<QuestManager>,
    quest_registry: Res<QuestRegistry>,
) {
    for event in ev_delivered.read() {
        if let Ok(platform) = platforms.get(event.platform_entity) {
            if let Some(quest_id) = &platform.linked_quest_id {
                // リンクされたクエストに納品
                let progress = quest_manager.get_or_create(quest_id);
                if progress.status == QuestStatus::Active {
                    progress.deliver(&event.item_id, event.amount);
                    info!(
                        "Delivered {} x{} to quest {}",
                        event.item_id, event.amount, quest_id
                    );
                }
            } else {
                // アクティブなクエスト全てに納品
                for (quest_id, quest) in quest_registry.quests.iter() {
                    let progress = quest_manager.get_or_create(quest_id);
                    if progress.status == QuestStatus::Active {
                        // このアイテムが要件に含まれているかチェック
                        let is_required = quest
                            .requirements
                            .iter()
                            .any(|req| req.item_id == event.item_id);

                        if is_required {
                            progress.deliver(&event.item_id, event.amount);
                        }
                    }
                }
            }
        }
    }
}

/// クエストマネージャーのポート数に基づいてポートの有効状態を更新
fn update_port_availability(
    quest_manager: Res<QuestManager>,
    mut platforms: Query<&mut DeliveryPlatform>,
    mut port_visuals: Query<(&DeliveryPortVisual, &mut MeshMaterial3d<StandardMaterial>)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let unlocked_count = quest_manager.unlocked_ports as usize;

    for mut platform in platforms.iter_mut() {
        // ポートの有効/無効を更新
        for (i, port) in platform.ports.iter_mut().enumerate() {
            port.is_active = i < unlocked_count;
        }
    }

    // ビジュアルを更新
    let inactive_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.3, 0.3, 0.3),
        ..default()
    });

    let active_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.2, 0.8, 0.2),
        emissive: LinearRgba::rgb(0.1, 0.4, 0.1),
        ..default()
    });

    for (visual, mut material) in port_visuals.iter_mut() {
        if visual.port_index < unlocked_count {
            *material = MeshMaterial3d(active_material.clone());
        } else {
            *material = MeshMaterial3d(inactive_material.clone());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_platform_creation() {
        let platform = DeliveryPlatform::new(IVec3::ZERO);
        assert_eq!(platform.ports.len(), 12); // 各辺3ポート × 4辺
    }

    #[test]
    fn test_port_filter() {
        let mut port = DeliveryPort::new(0, 0, Direction::North);
        assert!(port.can_accept("iron_ingot"));
        assert!(port.can_accept("copper_wire"));

        port = port.with_filter("iron_ingot");
        assert!(port.can_accept("iron_ingot"));
        assert!(!port.can_accept("copper_wire"));
    }

    #[test]
    fn test_delivery() {
        let mut platform = DeliveryPlatform::new(IVec3::ZERO);
        assert_eq!(platform.get_delivered("iron_ingot"), 0);

        platform.deliver_item("iron_ingot", 50);
        assert_eq!(platform.get_delivered("iron_ingot"), 50);

        platform.deliver_item("iron_ingot", 30);
        assert_eq!(platform.get_delivered("iron_ingot"), 80);
    }

    #[test]
    fn test_port_world_position() {
        let platform = DeliveryPlatform::new(IVec3::new(10, 5, 20));
        let port = &platform.ports[0];
        let world_pos = platform.get_port_world_pos(port);

        assert_eq!(world_pos.x, 10 + port.local_x);
        assert_eq!(world_pos.y, 5);
        assert_eq!(world_pos.z, 20 + port.local_z);
    }
}
