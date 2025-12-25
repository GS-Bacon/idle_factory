use bevy::prelude::*;
use crate::gameplay::inventory::PlayerInventory;
use crate::rendering::voxel_loader::VoxelAssets;

/// 右手に持っているアイテムのマーカー
#[derive(Component)]
pub struct HeldItem {
    pub item_id: String,
}

/// プレイヤーカメラのマーカー
#[derive(Component)]
pub struct PlayerCamera;

pub struct HeldItemPlugin;

impl Plugin for HeldItemPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update_held_item);
    }
}

/// ホットバーで選択されたアイテムを右手に表示
fn update_held_item(
    mut commands: Commands,
    inventory: Res<PlayerInventory>,
    voxel_assets: Res<VoxelAssets>,
    camera_query: Query<Entity, With<PlayerCamera>>,
    held_item_query: Query<(Entity, &HeldItem)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // 選択されたホットバースロットのアイテムを取得
    let selected_slot = inventory.selected_hotbar_slot;
    let selected_item_id = inventory.slots.get(selected_slot)
        .and_then(|slot| slot.item_id.as_ref());

    // 現在持っているアイテムを確認
    let current_held_item = held_item_query.iter().next();

    match (selected_item_id, current_held_item) {
        (Some(item_id), Some((held_entity, held_item))) => {
            // アイテムが変更された場合
            if &held_item.item_id != item_id {
                // 古いアイテムを削除
                commands.entity(held_entity).despawn_recursive();

                // 新しいアイテムを生成
                spawn_held_item(&mut commands, &camera_query, &voxel_assets, item_id, &mut meshes, &mut materials);
            }
        }
        (Some(item_id), None) => {
            // アイテムを持っていない状態から持つ状態へ
            spawn_held_item(&mut commands, &camera_query, &voxel_assets, item_id, &mut meshes, &mut materials);
        }
        (None, Some((held_entity, _))) => {
            // アイテムを持っている状態から持たない状態へ
            commands.entity(held_entity).despawn_recursive();
        }
        (None, None) => {
            // 何も変更なし
        }
    }
}

/// 右手にアイテムを生成
fn spawn_held_item(
    commands: &mut Commands,
    camera_query: &Query<Entity, With<PlayerCamera>>,
    voxel_assets: &Res<VoxelAssets>,
    item_id: &str,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    let Ok(camera_entity) = camera_query.get_single() else {
        warn!("カメラが見つかりません");
        return;
    };

    // VoxelAssetsからモデルを検索
    // item_idそのまま、またはitem_idから "_item" 等を除去して検索
    let model_key = item_id.trim_end_matches("_item");

    if let Some(voxels) = voxel_assets.models.get(model_key).or_else(|| voxel_assets.models.get(item_id)) {
        // ボクセルデータがある場合: ボクセルごとにキューブを配置
        spawn_voxel_model(commands, camera_entity, item_id, voxels, meshes, materials);
    } else {
        // フォールバック: 汎用キューブ
        spawn_fallback_cube(commands, camera_entity, item_id, meshes, materials);
    }
}

/// VoxelAssetsのボクセルデータからモデルを生成
fn spawn_voxel_model(
    commands: &mut Commands,
    camera_entity: Entity,
    item_id: &str,
    voxels: &[crate::rendering::voxel_loader::VoxelData],
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    // ボクセルモデルのスケール (手持ち用に縮小)
    const VOXEL_SCALE: f32 = 0.015;

    // モデルの中心を計算
    let center = if voxels.is_empty() {
        Vec3::ZERO
    } else {
        let sum: Vec3 = voxels.iter().map(|v| v.pos).sum();
        sum / voxels.len() as f32
    };

    // 小さなキューブメッシュ (共有)
    let cube_mesh = meshes.add(Cuboid::new(VOXEL_SCALE, VOXEL_SCALE, VOXEL_SCALE));

    // カメラの子としてアイテムを配置
    commands.entity(camera_entity).with_children(|parent| {
        parent.spawn((
            HeldItem {
                item_id: item_id.to_string(),
            },
            Transform::from_xyz(0.3, -0.2, -0.4)
                .with_rotation(Quat::from_rotation_y(std::f32::consts::PI / 6.0)),
            Visibility::default(),
        )).with_children(|model_parent| {
            // 各ボクセルをキューブとして配置
            for voxel in voxels {
                let material = materials.add(StandardMaterial {
                    base_color: Color::srgba(voxel.color[0], voxel.color[1], voxel.color[2], voxel.color[3]),
                    ..default()
                });

                model_parent.spawn((
                    Mesh3d(cube_mesh.clone()),
                    MeshMaterial3d(material),
                    Transform::from_translation((voxel.pos - center) * VOXEL_SCALE),
                ));
            }
        });
    });
}

/// フォールバック: 汎用キューブ表示
fn spawn_fallback_cube(
    commands: &mut Commands,
    camera_entity: Entity,
    item_id: &str,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    let mesh_handle = meshes.add(Cuboid::new(0.2, 0.2, 0.2));
    let material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.8, 0.8, 0.8),
        ..default()
    });

    commands.entity(camera_entity).with_children(|parent| {
        parent.spawn((
            HeldItem {
                item_id: item_id.to_string(),
            },
            Mesh3d(mesh_handle),
            MeshMaterial3d(material),
            Transform::from_xyz(0.3, -0.2, -0.4)
                .with_rotation(Quat::from_rotation_y(std::f32::consts::PI / 6.0)),
        ));
    });
}
