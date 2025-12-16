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
    _voxel_assets: &Res<VoxelAssets>,
    item_id: &str,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    let Ok(camera_entity) = camera_query.get_single() else {
        warn!("カメラが見つかりません");
        return;
    };

    // 簡易実装: 全てのアイテムを小さいキューブで表示
    // TODO: VoxelAssetsからボクセルデータを読み込んで表示
    let mesh_handle = meshes.add(Cuboid::new(0.2, 0.2, 0.2));

    // マテリアル作成（アイテムごとに色を変えることも可能）
    let material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.8, 0.8, 0.8),
        ..default()
    });

    // カメラの子としてアイテムを配置
    // 位置: カメラの右下（First-person view）
    commands.entity(camera_entity).with_children(|parent| {
        parent.spawn((
            HeldItem {
                item_id: item_id.to_string(),
            },
            Mesh3d(mesh_handle),
            MeshMaterial3d(material),
            Transform::from_xyz(0.3, -0.2, -0.4) // 右下前方
                .with_rotation(Quat::from_rotation_y(std::f32::consts::PI / 6.0)),
        ));
    });
}
