// src/ui/inventory_ui.rs
//! インベントリUIシステム
//! - ステート管理: Closed/PlayerInventory/Container
//! - ドラッグ&ドロップ
//! - 動的ツールチップ
//! - クラフトリスト
//! - Minecraft風レイアウト

use bevy::prelude::*;
use bevy::window::{CursorGrabMode, PrimaryWindow};
use crate::gameplay::inventory::{PlayerInventory, EquipmentSlots, ItemRegistry, EquipmentSlotType};
use crate::core::registry::RecipeRegistry;

/// インベントリUIのステート
#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum InventoryUiState {
    #[default]
    Closed,          // HUDのみ
    PlayerInventory, // 'E'キー: [装備] + [Player在庫] + [クラフトリスト]
    Container,       // 右クリック: [装備] + [Player在庫] + [コンテナ在庫]
}

/// 現在開いているコンテナの情報
#[derive(Resource, Default)]
pub struct OpenContainer {
    pub pos: Option<IVec3>,
}

/// ドラッグ中のアイテム情報
#[derive(Resource, Default)]
pub struct DraggedItem {
    pub item_id: Option<String>,
    pub count: u32,
    pub source_slot: Option<SlotIdentifier>,
}

/// スロットの識別子
#[derive(Debug, Clone, PartialEq)]
pub enum SlotIdentifier {
    PlayerInventory(usize),
    Equipment(EquipmentSlotType),
    Container(usize),
}

/// UIルートマーカー
#[derive(Component)]
pub struct InventoryUiRoot;

/// スロットコンポーネント
#[derive(Component)]
pub struct UiSlot {
    pub identifier: SlotIdentifier,
}

/// スプリットダイアログマーカー
#[derive(Component)]
pub struct SplitDialog;

/// ツールチップコンポーネント
#[derive(Component)]
pub struct Tooltip;

/// クラフトボタン
#[derive(Component)]
pub struct CraftButton {
    pub recipe_id: String,
}

/// ソートボタン
#[derive(Component)]
pub struct SortButton;

/// ゴミ箱スロット
#[derive(Component)]
pub struct TrashSlot;

/// ホットバーHUDマーカー
#[derive(Component)]
pub struct HotbarHud;

/// イベント
#[derive(Event)]
pub struct OpenInventoryEvent;

#[derive(Event)]
pub struct CloseInventoryEvent;

#[derive(Event)]
pub struct OpenContainerEvent {
    pub pos: IVec3,
}

#[derive(Event)]
pub struct SortInventoryEvent;

#[derive(Event)]
pub struct CraftItemEvent {
    pub recipe_id: String,
}

/// インベントリUIプラグイン
pub struct InventoryUiPlugin;

impl Plugin for InventoryUiPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<InventoryUiState>()
            .init_resource::<OpenContainer>()
            .init_resource::<DraggedItem>()
            .add_event::<OpenInventoryEvent>()
            .add_event::<CloseInventoryEvent>()
            .add_event::<OpenContainerEvent>()
            .add_event::<SortInventoryEvent>()
            .add_event::<CraftItemEvent>()
            .add_systems(Update, (
                handle_inventory_key,
                handle_open_inventory_event,
                handle_close_inventory_event,
                handle_open_container_event,
                handle_escape_key,
            ))
            .add_systems(OnEnter(InventoryUiState::PlayerInventory), (
                spawn_player_inventory_ui,
                release_cursor,
            ))
            .add_systems(OnEnter(InventoryUiState::Container), (
                spawn_container_ui,
                release_cursor,
            ))
            .add_systems(OnExit(InventoryUiState::PlayerInventory), despawn_inventory_ui)
            .add_systems(OnExit(InventoryUiState::Container), despawn_inventory_ui)
            .add_systems(OnEnter(InventoryUiState::Closed), spawn_hotbar_hud)
            .add_systems(OnExit(InventoryUiState::Closed), despawn_hotbar_hud)
            .add_systems(Update, (
                update_slot_visuals,
                handle_slot_interaction,
                handle_sort_button,
                handle_craft_button,
                update_tooltip,
            ).run_if(not(in_state(InventoryUiState::Closed))))
            .add_systems(Update, update_hotbar_hud.run_if(in_state(InventoryUiState::Closed)));
    }
}

/// 'E'キーでインベントリを開く
fn handle_inventory_key(
    keyboard: Res<ButtonInput<KeyCode>>,
    current_state: Res<State<InventoryUiState>>,
    mut next_state: ResMut<NextState<InventoryUiState>>,
) {
    if keyboard.just_pressed(KeyCode::KeyE) {
        match current_state.get() {
            InventoryUiState::Closed => {
                next_state.set(InventoryUiState::PlayerInventory);
            }
            InventoryUiState::PlayerInventory => {
                next_state.set(InventoryUiState::Closed);
            }
            _ => {}
        }
    }
}

/// Escキーでインベントリを閉じる
fn handle_escape_key(
    keyboard: Res<ButtonInput<KeyCode>>,
    current_state: Res<State<InventoryUiState>>,
    mut next_state: ResMut<NextState<InventoryUiState>>,
) {
    if keyboard.just_pressed(KeyCode::Escape) {
        if *current_state.get() != InventoryUiState::Closed {
            next_state.set(InventoryUiState::Closed);
        }
    }
}

/// インベントリオープンイベント処理
fn handle_open_inventory_event(
    mut events: EventReader<OpenInventoryEvent>,
    mut next_state: ResMut<NextState<InventoryUiState>>,
) {
    for _ in events.read() {
        next_state.set(InventoryUiState::PlayerInventory);
    }
}

/// インベントリクローズイベント処理
fn handle_close_inventory_event(
    mut events: EventReader<CloseInventoryEvent>,
    mut next_state: ResMut<NextState<InventoryUiState>>,
) {
    for _ in events.read() {
        next_state.set(InventoryUiState::Closed);
    }
}

/// コンテナオープンイベント処理
fn handle_open_container_event(
    mut events: EventReader<OpenContainerEvent>,
    mut next_state: ResMut<NextState<InventoryUiState>>,
    mut open_container: ResMut<OpenContainer>,
) {
    for event in events.read() {
        open_container.pos = Some(event.pos);
        next_state.set(InventoryUiState::Container);
    }
}

/// プレイヤーインベントリUIを生成（Minecraft風）
fn spawn_player_inventory_ui(
    mut commands: Commands,
    player_inventory: Res<PlayerInventory>,
    equipment: Res<EquipmentSlots>,
    recipe_registry: Res<RecipeRegistry>,
) {
    const SLOT_SIZE: f32 = 54.0;
    const SLOT_GAP: f32 = 4.0;

    commands
        .spawn((
            InventoryUiRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.8)),
        ))
        .with_children(|parent| {
            // メインコンテナ (Minecraft風)
            parent
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Row,
                        padding: UiRect::all(Val::Px(20.0)),
                        column_gap: Val::Px(20.0),
                        position_type: PositionType::Relative,
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
                ))
                .with_children(|parent| {
                    // 左側: 装備スロット (縦並び)
                    spawn_equipment_panel_mc(parent, &equipment, SLOT_SIZE, SLOT_GAP);

                    // 中央: インベントリ + ホットバー
                    parent
                        .spawn(Node {
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Px(20.0),
                            ..default()
                        })
                        .with_children(|parent| {
                            spawn_main_inventory_panel_mc(parent, &player_inventory, SLOT_SIZE, SLOT_GAP);
                        });

                    // 右側: クラフトリスト
                    spawn_craft_list_panel(parent, &recipe_registry);

                    // ゴミ箱スロット（右下に絶対配置）
                    parent
                        .spawn(Node {
                            position_type: PositionType::Absolute,
                            bottom: Val::Px(20.0),
                            right: Val::Px(20.0),
                            ..default()
                        })
                        .with_children(|parent| {
                            parent
                                .spawn((
                                    TrashSlot,
                                    Button,
                                    Node {
                                        width: Val::Px(SLOT_SIZE),
                                        height: Val::Px(SLOT_SIZE),
                                        justify_content: JustifyContent::Center,
                                        align_items: AlignItems::Center,
                                        border: UiRect::all(Val::Px(2.0)),
                                        ..default()
                                    },
                                    BackgroundColor(Color::srgb(0.6, 0.2, 0.2)),
                                    BorderColor(Color::srgb(0.8, 0.3, 0.3)),
                                ))
                                .with_children(|parent| {
                                    parent.spawn((
                                        Text::new("Trash"),
                                        TextFont { font_size: 12.0, ..default() },
                                        TextColor(Color::WHITE),
                                    ));
                                });
                        });
                });
        });

    info!("Player inventory UI spawned (Minecraft style)");
}

/// コンテナUIを生成
fn spawn_container_ui(
    mut commands: Commands,
    player_inventory: Res<PlayerInventory>,
    equipment: Res<EquipmentSlots>,
) {
    commands
        .spawn((
            InventoryUiRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.8)),
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Row,
                        padding: UiRect::all(Val::Px(20.0)),
                        column_gap: Val::Px(20.0),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
                ))
                .with_children(|parent| {
                    spawn_equipment_panel(parent, &equipment);
                    spawn_inventory_panel(parent, &player_inventory);
                    spawn_container_panel(parent);
                });
        });

    info!("Container UI spawned");
}

/// 装備パネルを生成（Minecraft風、アイコン付き）
fn spawn_equipment_panel_mc(parent: &mut ChildBuilder, equipment: &EquipmentSlots, slot_size: f32, slot_gap: f32) {
    parent
        .spawn(Node {
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(10.0),
            ..default()
        })
        .with_children(|parent| {
            // タイトル部分（インベントリと同じ高さに調整）
            parent
                .spawn(Node {
                    height: Val::Px(38.0), // ソートボタンを含むヘッダーと同じ高さ
                    align_items: AlignItems::Center,
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("Equipment"),
                        TextFont { font_size: 20.0, ..default() },
                        TextColor(Color::WHITE),
                    ));
                });

            // 装備スロット（アイコン付き、row_gapで間隔調整）
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(slot_gap),
                    ..default()
                })
                .with_children(|parent| {
                    for (slot_type, icon_text) in [
                        (EquipmentSlotType::Head, "H"),
                        (EquipmentSlotType::Chest, "C"),
                        (EquipmentSlotType::Legs, "L"),
                        (EquipmentSlotType::Feet, "F"),
                        (EquipmentSlotType::Tool, "T"),
                    ] {
                        spawn_slot_with_icon(parent, SlotIdentifier::Equipment(slot_type), equipment.get(slot_type), slot_size, icon_text);
                    }
                });
        });
}

/// 装備パネルを生成（旧版、互換性のため残す）
fn spawn_equipment_panel(parent: &mut ChildBuilder, equipment: &EquipmentSlots) {
    spawn_equipment_panel_mc(parent, equipment, 54.0, 4.0);
}

/// メインインベントリパネルを生成（Minecraft風）
fn spawn_main_inventory_panel_mc(parent: &mut ChildBuilder, inventory: &PlayerInventory, slot_size: f32, slot_gap: f32) {
    parent
        .spawn(Node {
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(10.0),
            ..default()
        })
        .with_children(|parent| {
            // タイトルとソートボタン
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::Center,
                    width: Val::Px(10.0 * slot_size + 9.0 * slot_gap),
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("Inventory"),
                        TextFont { font_size: 20.0, ..default() },
                        TextColor(Color::WHITE),
                    ));

                    parent
                        .spawn((
                            SortButton,
                            Button,
                            Node {
                                padding: UiRect::all(Val::Px(8.0)),
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.4, 0.4, 0.4)),
                        ))
                        .with_children(|parent| {
                            parent.spawn((
                                Text::new("Sort"),
                                TextFont { font_size: 14.0, ..default() },
                                TextColor(Color::WHITE),
                            ));
                        });
                });

            // メインインベントリ (10x5 = 50スロット、スロット0-49)
            parent
                .spawn(Node {
                    display: Display::Grid,
                    grid_template_columns: RepeatedGridTrack::flex(10, 1.0),
                    grid_template_rows: RepeatedGridTrack::flex(5, 1.0),
                    row_gap: Val::Px(slot_gap),
                    column_gap: Val::Px(slot_gap),
                    ..default()
                })
                .with_children(|parent| {
                    for i in 0..50 {
                        spawn_slot_sized(parent, SlotIdentifier::PlayerInventory(i), &inventory.slots[i], slot_size);
                    }
                });

            // ホットバー (1x10 = 10スロット、スロット50-59)
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(5.0),
                    margin: UiRect::top(Val::Px(10.0)),
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn((
                        Text::new("Hotbar"),
                        TextFont { font_size: 16.0, ..default() },
                        TextColor(Color::WHITE),
                    ));

                    parent
                        .spawn(Node {
                            display: Display::Grid,
                            grid_template_columns: RepeatedGridTrack::flex(10, 1.0),
                            grid_template_rows: RepeatedGridTrack::flex(1, 1.0),
                            column_gap: Val::Px(slot_gap),
                            ..default()
                        })
                        .with_children(|parent| {
                            for i in 50..60 {
                                spawn_slot_sized(parent, SlotIdentifier::PlayerInventory(i), &inventory.slots[i], slot_size);
                            }
                        });
                });
        });
}

/// インベントリパネルを生成（旧版、互換性のため残す）
fn spawn_inventory_panel(parent: &mut ChildBuilder, inventory: &PlayerInventory) {
    spawn_main_inventory_panel_mc(parent, inventory, 54.0, 4.0);
}

/// クラフトリストパネルを生成
fn spawn_craft_list_panel(parent: &mut ChildBuilder, recipe_registry: &RecipeRegistry) {
    parent
        .spawn(Node {
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(10.0),
            width: Val::Px(300.0),
            ..default()
        })
        .with_children(|parent| {
            parent.spawn((
                Text::new("Crafting"),
                TextFont { font_size: 24.0, ..default() },
                TextColor(Color::WHITE),
            ));

            // スクロールビュー
            parent
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        overflow: Overflow::scroll_y(),
                        max_height: Val::Px(400.0),
                        row_gap: Val::Px(5.0),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.15, 0.15, 0.15)),
                ))
                .with_children(|parent| {
                    // レシピボタンを動的生成
                    for (recipe_id, recipe) in &recipe_registry.map {
                        parent
                            .spawn((
                                CraftButton { recipe_id: recipe_id.clone() },
                                Button,
                                Node {
                                    padding: UiRect::all(Val::Px(10.0)),
                                    ..default()
                                },
                                BackgroundColor(Color::srgb(0.3, 0.3, 0.3)),
                            ))
                            .with_children(|parent| {
                                parent.spawn((
                                    Text::new(&recipe.name),
                                    TextFont { font_size: 16.0, ..default() },
                                    TextColor(Color::WHITE),
                                ));
                            });
                    }
                });
        });
}

/// コンテナパネルを生成
fn spawn_container_panel(parent: &mut ChildBuilder) {
    parent
        .spawn(Node {
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(10.0),
            ..default()
        })
        .with_children(|parent| {
            parent.spawn((
                Text::new("Container"),
                TextFont { font_size: 24.0, ..default() },
                TextColor(Color::WHITE),
            ));

            // コンテナスロット (仮で20スロット)
            parent
                .spawn(Node {
                    display: Display::Grid,
                    grid_template_columns: RepeatedGridTrack::flex(4, 1.0),
                    grid_template_rows: RepeatedGridTrack::flex(5, 1.0),
                    row_gap: Val::Px(5.0),
                    column_gap: Val::Px(5.0),
                    ..default()
                })
                .with_children(|parent| {
                    for i in 0..20 {
                        spawn_slot(
                            parent,
                            SlotIdentifier::Container(i),
                            &crate::gameplay::inventory::InventorySlot::empty(),
                        );
                    }
                });
        });
}

/// 個別スロットを生成（サイズ指定版）
fn spawn_slot_sized(
    parent: &mut ChildBuilder,
    identifier: SlotIdentifier,
    slot_data: &crate::gameplay::inventory::InventorySlot,
    size: f32,
) {
    parent
        .spawn((
            UiSlot { identifier },
            Button,
            Node {
                width: Val::Px(size),
                height: Val::Px(size),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.3, 0.3, 0.3)),
            BorderColor(Color::srgb(0.5, 0.5, 0.5)),
        ))
        .with_children(|parent| {
            if let Some(item_id) = &slot_data.item_id {
                parent.spawn((
                    Text::new(format!("{}\n{}", item_id, slot_data.count)),
                    TextFont { font_size: 12.0, ..default() },
                    TextColor(Color::WHITE),
                ));
            }
        });
}

/// アイコン付きスロットを生成（装備用）
fn spawn_slot_with_icon(
    parent: &mut ChildBuilder,
    identifier: SlotIdentifier,
    slot_data: &crate::gameplay::inventory::InventorySlot,
    size: f32,
    icon_text: &str,
) {
    parent
        .spawn((
            UiSlot { identifier },
            Button,
            Node {
                width: Val::Px(size),
                height: Val::Px(size),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: UiRect::all(Val::Px(2.0)),
                position_type: PositionType::Relative,
                ..default()
            },
            BackgroundColor(Color::srgb(0.3, 0.3, 0.3)),
            BorderColor(Color::srgb(0.5, 0.5, 0.5)),
        ))
        .with_children(|parent| {
            // アイコン（左上）
            parent.spawn((
                Text::new(icon_text),
                TextFont { font_size: 10.0, ..default() },
                TextColor(Color::srgba(1.0, 1.0, 1.0, 0.5)),
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(2.0),
                    top: Val::Px(2.0),
                    ..default()
                },
            ));

            // アイテム情報（中央）
            if let Some(item_id) = &slot_data.item_id {
                parent.spawn((
                    Text::new(format!("{}\n{}", item_id, slot_data.count)),
                    TextFont { font_size: 11.0, ..default() },
                    TextColor(Color::WHITE),
                ));
            }
        });
}

/// 個別スロットを生成（旧版、互換性のため残す）
fn spawn_slot(
    parent: &mut ChildBuilder,
    identifier: SlotIdentifier,
    slot_data: &crate::gameplay::inventory::InventorySlot,
) {
    spawn_slot_sized(parent, identifier, slot_data, 54.0);
}

/// UIを削除
fn despawn_inventory_ui(mut commands: Commands, query: Query<Entity, With<InventoryUiRoot>>) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
}

/// スロットの見た目を更新
fn update_slot_visuals(
    player_inventory: Res<PlayerInventory>,
    equipment: Res<EquipmentSlots>,
    mut slot_query: Query<(&UiSlot, &Children, &mut BackgroundColor), With<Button>>,
    mut text_query: Query<&mut Text>,
) {
    if !player_inventory.is_changed() && !equipment.is_changed() {
        return;
    }

    for (ui_slot, children, mut bg_color) in &mut slot_query {
        let slot_data = match &ui_slot.identifier {
            SlotIdentifier::PlayerInventory(i) => {
                if *i < player_inventory.slots.len() {
                    &player_inventory.slots[*i]
                } else {
                    continue;
                }
            }
            SlotIdentifier::Equipment(slot_type) => equipment.get(*slot_type),
            SlotIdentifier::Container(_) => continue,
        };

        // 背景色を更新
        if slot_data.is_empty() {
            *bg_color = BackgroundColor(Color::srgb(0.3, 0.3, 0.3));
        } else {
            *bg_color = BackgroundColor(Color::srgb(0.4, 0.4, 0.5));
        }

        // テキストを更新
        for &child in children.iter() {
            if let Ok(mut text) = text_query.get_mut(child) {
                if let Some(item_id) = &slot_data.item_id {
                    **text = format!("{}\n{}", item_id, slot_data.count);
                } else {
                    **text = String::new();
                }
            }
        }
    }
}

/// スロットのインタラクション処理（簡易版）
fn handle_slot_interaction(
    interaction_query: Query<(&Interaction, &UiSlot), (Changed<Interaction>, With<Button>)>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut dragged: ResMut<DraggedItem>,
    mut player_inventory: ResMut<PlayerInventory>,
    mut equipment: ResMut<EquipmentSlots>,
    item_registry: Res<ItemRegistry>,
) {
    for (interaction, ui_slot) in &interaction_query {
        if *interaction == Interaction::Pressed {
            let _is_ctrl = keyboard.pressed(KeyCode::ControlLeft) || keyboard.pressed(KeyCode::ControlRight);

            // 現在のスロットデータを取得
            let slot_data = match &ui_slot.identifier {
                SlotIdentifier::PlayerInventory(i) => {
                    if *i < player_inventory.slots.len() {
                        &player_inventory.slots[*i]
                    } else {
                        continue;
                    }
                }
                SlotIdentifier::Equipment(slot_type) => equipment.get(*slot_type),
                SlotIdentifier::Container(_) => continue,
            };

            // 通常のドラッグ&ドロップ
            if dragged.item_id.is_none() {
                // ドラッグ開始
                if !slot_data.is_empty() {
                    dragged.item_id = slot_data.item_id.clone();
                    dragged.count = slot_data.count;
                    dragged.source_slot = Some(ui_slot.identifier.clone());

                    // スロットをクリア
                    match &ui_slot.identifier {
                        SlotIdentifier::PlayerInventory(i) => {
                            player_inventory.slots[*i].clear();
                        }
                        SlotIdentifier::Equipment(slot_type) => {
                            equipment.get_mut(*slot_type).clear();
                        }
                        _ => {}
                    }
                }
            } else {
                // ドロップ
                let dragged_item_id = dragged.item_id.clone().unwrap();
                let dragged_count = dragged.count;

                // 対象スロットに配置
                match &ui_slot.identifier {
                    SlotIdentifier::PlayerInventory(i) => {
                        if *i < player_inventory.slots.len() {
                            let target_slot = &mut player_inventory.slots[*i];

                            if target_slot.is_empty() {
                                target_slot.item_id = Some(dragged_item_id.clone());
                                target_slot.count = dragged_count;
                                dragged.item_id = None;
                                dragged.count = 0;
                                dragged.source_slot = None;
                            } else if target_slot.item_id.as_ref() == Some(&dragged_item_id) {
                                let max_stack = item_registry
                                    .get(&dragged_item_id)
                                    .map(|d| d.max_stack)
                                    .unwrap_or(999);

                                let space = max_stack - target_slot.count;
                                let add = dragged_count.min(space);
                                target_slot.count += add;
                                dragged.count -= add;

                                if dragged.count == 0 {
                                    dragged.item_id = None;
                                    dragged.source_slot = None;
                                }
                            } else {
                                // スワップ
                                let temp_id = target_slot.item_id.clone();
                                let temp_count = target_slot.count;

                                target_slot.item_id = Some(dragged_item_id.clone());
                                target_slot.count = dragged_count;

                                dragged.item_id = temp_id;
                                dragged.count = temp_count;
                            }
                        }
                    }
                    SlotIdentifier::Equipment(slot_type) => {
                        let target_slot = equipment.get_mut(*slot_type);

                        if target_slot.is_empty() {
                            target_slot.item_id = Some(dragged_item_id.clone());
                            target_slot.count = dragged_count;
                            dragged.item_id = None;
                            dragged.count = 0;
                            dragged.source_slot = None;
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}

/// ソートボタン処理
fn handle_sort_button(
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<SortButton>)>,
    mut player_inventory: ResMut<PlayerInventory>,
) {
    for interaction in &interaction_query {
        if *interaction == Interaction::Pressed {
            player_inventory.sort();
            info!("Inventory sorted");
        }
    }
}

/// クラフトボタン処理（簡易版）
fn handle_craft_button(
    interaction_query: Query<(&Interaction, &CraftButton), Changed<Interaction>>,
    mut craft_event: EventWriter<CraftItemEvent>,
) {
    for (interaction, craft_button) in &interaction_query {
        if *interaction == Interaction::Pressed {
            craft_event.send(CraftItemEvent {
                recipe_id: craft_button.recipe_id.clone(),
            });
            info!("Crafting: {}", craft_button.recipe_id);
        }
    }
}

/// ツールチップ更新（簡易版）
fn update_tooltip(
    mut commands: Commands,
    slot_query: Query<(&Interaction, &UiSlot, &GlobalTransform), (Changed<Interaction>, With<Button>)>,
    player_inventory: Res<PlayerInventory>,
    equipment: Res<EquipmentSlots>,
    item_registry: Res<ItemRegistry>,
    tooltip_query: Query<Entity, With<Tooltip>>,
) {
    // 既存のツールチップを削除
    for entity in &tooltip_query {
        commands.entity(entity).despawn_recursive();
    }

    // ホバー中のスロットを探す
    for (interaction, ui_slot, transform) in &slot_query {
        if *interaction == Interaction::Hovered {
            let slot_data = match &ui_slot.identifier {
                SlotIdentifier::PlayerInventory(i) => {
                    if *i < player_inventory.slots.len() {
                        &player_inventory.slots[*i]
                    } else {
                        continue;
                    }
                }
                SlotIdentifier::Equipment(slot_type) => equipment.get(*slot_type),
                _ => continue,
            };

            if let Some(item_id) = &slot_data.item_id {
                if let Some(item_data) = item_registry.get(item_id) {
                    let mut tooltip_text = format!("{}\n", item_data.name);

                    for (key, value) in &item_data.custom_properties {
                        tooltip_text.push_str(&format!("{}: {}\n", key, value));
                    }

                    commands.spawn((
                        Tooltip,
                        Node {
                            position_type: PositionType::Absolute,
                            left: Val::Px(transform.translation().x + 70.0),
                            top: Val::Px(transform.translation().y),
                            padding: UiRect::all(Val::Px(10.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.9)),
                        Text::new(tooltip_text),
                        TextFont { font_size: 16.0, ..default() },
                        TextColor(Color::WHITE),
                    ));
                }
            }
        }
    }
}

/// インベントリUI表示時にカーソルを解放
fn release_cursor(
    mut window_query: Query<&mut Window, With<PrimaryWindow>>,
) {
    if let Ok(mut window) = window_query.get_single_mut() {
        window.cursor_options.grab_mode = CursorGrabMode::None;
        window.cursor_options.visible = true;
    }
}

/// ホットバーHUDを生成
fn spawn_hotbar_hud(
    mut commands: Commands,
    player_inventory: Res<PlayerInventory>,
) {
    const SLOT_SIZE: f32 = 54.0;
    const SLOT_GAP: f32 = 4.0;

    commands
        .spawn((
            HotbarHud,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::End,
                padding: UiRect::bottom(Val::Px(20.0)),
                ..default()
            },
        ))
        .with_children(|parent| {
            parent
                .spawn(Node {
                    display: Display::Grid,
                    grid_template_columns: RepeatedGridTrack::flex(10, 1.0),
                    grid_template_rows: RepeatedGridTrack::flex(1, 1.0),
                    column_gap: Val::Px(SLOT_GAP),
                    padding: UiRect::all(Val::Px(8.0)),
                    ..default()
                })
                .with_children(|parent| {
                    for i in 50..60 {
                        spawn_hotbar_slot(parent, i, &player_inventory.slots[i], SLOT_SIZE);
                    }
                });
        });

    info!("Hotbar HUD spawned");
}

/// ホットバー用スロットを生成
fn spawn_hotbar_slot(
    parent: &mut ChildBuilder,
    index: usize,
    slot_data: &crate::gameplay::inventory::InventorySlot,
    size: f32,
) {
    parent
        .spawn((
            UiSlot { identifier: SlotIdentifier::PlayerInventory(index) },
            Node {
                width: Val::Px(size),
                height: Val::Px(size),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.8)),
            BorderColor(Color::srgb(0.5, 0.5, 0.5)),
        ))
        .with_children(|parent| {
            if let Some(item_id) = &slot_data.item_id {
                parent.spawn((
                    Text::new(format!("{}\n{}", item_id, slot_data.count)),
                    TextFont { font_size: 11.0, ..default() },
                    TextColor(Color::WHITE),
                ));
            }
        });
}

/// ホットバーHUDを削除
fn despawn_hotbar_hud(
    mut commands: Commands,
    query: Query<Entity, With<HotbarHud>>,
) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
}

/// ホットバーHUDを更新
fn update_hotbar_hud(
    player_inventory: Res<PlayerInventory>,
    mut slot_query: Query<(&UiSlot, &Children, &mut BackgroundColor), Without<Button>>,
    mut text_query: Query<&mut Text>,
) {
    if !player_inventory.is_changed() {
        return;
    }

    for (ui_slot, children, mut bg_color) in &mut slot_query {
        if let SlotIdentifier::PlayerInventory(i) = &ui_slot.identifier {
            if *i >= 50 && *i < 60 {
                let slot_data = &player_inventory.slots[*i];

                // 背景色を更新
                if slot_data.is_empty() {
                    *bg_color = BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.8));
                } else {
                    *bg_color = BackgroundColor(Color::srgba(0.2, 0.2, 0.3, 0.9));
                }

                // テキストを更新
                for &child in children.iter() {
                    if let Ok(mut text) = text_query.get_mut(child) {
                        if let Some(item_id) = &slot_data.item_id {
                            **text = format!("{}\n{}", item_id, slot_data.count);
                        } else {
                            **text = String::new();
                        }
                    }
                }
            }
        }
    }
}
