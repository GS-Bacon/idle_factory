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
use crate::ui::main_menu::AppState;

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

/// クリエイティブモードアイテムボタン
#[derive(Component)]
pub struct CreativeItemButton {
    pub item_id: String,
}

/// ソートボタン
#[derive(Component)]
pub struct SortButton;

/// 装備パネルマーカー（クリエイティブモードで非表示にするため）
#[derive(Component)]
pub struct EquipmentPanel;

/// メインインベントリパネルマーカー（クリエイティブモードで非表示にするため）
#[derive(Component)]
pub struct MainInventoryPanel;

/// クリエイティブアイテムカタログマーカー
#[derive(Component)]
pub struct CreativeItemList;

/// クリエイティブ表示モード
#[derive(Resource, Default, PartialEq, Debug)]
pub enum CreativeViewMode {
    #[default]
    Catalog,    // アイテムカタログ
    Inventory,  // プレイヤーインベントリ
}

/// 表示切替ボタン
#[derive(Component)]
pub struct ViewToggleButton;

/// ドラッグ中のアイテム表示マーカー
#[derive(Component)]
pub struct DraggedItemVisual;

/// ゴミ箱スロット
#[derive(Component)]
pub struct TrashSlot;

/// ホットバーHUDマーカー
#[derive(Component)]
pub struct HotbarHud;

/// ホットバー上のアイテム名表示マーカー
#[derive(Component)]
pub struct HotbarItemName;

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
            .init_resource::<CreativeViewMode>()
            .add_event::<OpenInventoryEvent>()
            .add_event::<CloseInventoryEvent>()
            .add_event::<OpenContainerEvent>()
            .add_event::<SortInventoryEvent>()
            .add_event::<CraftItemEvent>()
            .add_systems(Update, (
                handle_inventory_key.run_if(in_state(crate::ui::settings_ui::SettingsUiState::Closed)),
                handle_open_inventory_event,
                handle_close_inventory_event,
                handle_open_container_event,
                handle_escape_key.run_if(in_state(crate::ui::settings_ui::SettingsUiState::Closed)),
            ))
            .add_systems(OnEnter(InventoryUiState::PlayerInventory), (
                spawn_player_inventory_ui,
                initialize_creative_visibility,
                release_cursor,
            ))
            .add_systems(OnEnter(InventoryUiState::Container), (
                spawn_container_ui,
                release_cursor,
            ))
            .add_systems(OnExit(InventoryUiState::PlayerInventory), (despawn_inventory_ui, spawn_hotbar_hud_if_not_creative, grab_cursor))
            .add_systems(OnExit(InventoryUiState::Container), (despawn_inventory_ui, grab_cursor))
            // ホットバーHUDはInGame開始時にスポーン、終了時にデスポーン
            .add_systems(OnEnter(AppState::InGame), spawn_hotbar_hud)
            .add_systems(OnExit(AppState::InGame), despawn_hotbar_hud)
            // インベントリ開閉時の表示切替
            .add_systems(OnEnter(InventoryUiState::PlayerInventory), (spawn_hotbar_hud_if_creative, hide_hotbar_hud_if_not_creative))
            .add_systems(OnExit(InventoryUiState::Closed), hide_hotbar_hud_if_not_creative)
            .add_systems(Update, (
                (
                    handle_slot_interaction,
                    handle_creative_item_button,
                    handle_drag_drop_release,
                    update_slot_visuals, // ドラッグ&ドロップ処理の後に必ず実行
                ).chain(),
                handle_sort_button,
                handle_craft_button,
                handle_view_toggle_button,
                update_creative_view_visibility,
                update_tooltip,
                update_dragged_item_visual,
            ).run_if(not(in_state(InventoryUiState::Closed))))
            .add_systems(Update, update_hotbar_hud);
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
    if keyboard.just_pressed(KeyCode::Escape)
        && *current_state.get() != InventoryUiState::Closed
    {
        next_state.set(InventoryUiState::Closed);
        // カーソルのグラブはOnExitシステムで自動的に行われる
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
    item_registry: Res<ItemRegistry>,
    game_mode: Res<crate::gameplay::commands::GameMode>,
    config: Res<crate::core::config::GameConfig>,
) {
    const SLOT_SIZE: f32 = 54.0;
    const SLOT_GAP: f32 = 4.0;

    // enable_ui_blurが有効な場合、背景を暗くしてぼやけた効果を出す
    let bg_alpha = if config.enable_ui_blur { 0.95 } else { 0.7 };

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
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, bg_alpha)),
        ))
        .with_children(|parent| {
            // メインコンテナ - CSS Gridで自動整列
            parent
                .spawn((
                    Node {
                        display: Display::Grid,
                        // 列: 装備パネル | メインインベントリ | クラフトリスト(オプション)
                        grid_template_columns: vec![
                            GridTrack::auto(),  // 装備パネル
                            GridTrack::auto(),  // メインインベントリ
                            GridTrack::auto(),  // クラフトリスト
                        ],
                        // 行: 全て自動サイズ
                        grid_auto_rows: GridTrack::auto(),
                        padding: UiRect::all(Val::Px(20.0)),
                        column_gap: Val::Px(20.0),
                        row_gap: Val::Px(10.0),
                        align_items: AlignItems::Start, // グリッドセル内で上揃え
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
                ))
                .with_children(|parent| {
                    // 左側: 装備スロット (縦並び) - CSS Gridで自動整列されるのでmargin不要
                    parent
                        .spawn((
                            EquipmentPanel,
                            Node {
                                flex_direction: FlexDirection::Column,
                                row_gap: Val::Px(SLOT_GAP),
                                align_self: AlignSelf::Center, // 中央揃え（メインインベントリと揃える）
                                justify_content: JustifyContent::Center,
                                ..default()
                            },
                        ))
                        .with_children(|parent| {
                            spawn_equipment_panel_mc(parent, &equipment, SLOT_SIZE, SLOT_GAP);
                        });

                    // 中央: インベントリ OR アイテムカタログ（クリエイティブモード）
                    if *game_mode == crate::gameplay::commands::GameMode::Creative {
                        // クリエイティブモード: グリッド部分とホットバーを分離し、グリッドのみをトグル
                        // 中央列: インベントリ/カタロググリッド + ホットバー
                        parent
                            .spawn(Node {
                                flex_direction: FlexDirection::Column,
                                row_gap: Val::Px(10.0),
                                ..default()
                            })
                            .with_children(|parent| {
                                // グリッド切り替えコンテナ（相対位置、2つのグリッドを同じ位置に配置）
                                parent
                                    .spawn(Node {
                                        position_type: PositionType::Relative,
                                        width: Val::Auto,
                                        height: Val::Px(350.0), // 明示的な高さを設定（グリッド高さ分を確保）
                                        ..default()
                                    })
                                    .with_children(|parent| {
                                        // メインインベントリグリッド（絶対位置、初期状態では非表示）
                                        parent
                                            .spawn((
                                                MainInventoryPanel,
                                                Node {
                                                    position_type: PositionType::Absolute,
                                                    flex_direction: FlexDirection::Column,
                                                    row_gap: Val::Px(10.0),
                                                    ..default()
                                                },
                                                Visibility::Hidden, // 初期状態は非表示
                                            ))
                                            .with_children(|parent| {
                                                spawn_main_inventory_grid_only(parent, &player_inventory, SLOT_SIZE, SLOT_GAP);
                                            });

                                        // アイテムカタロググリッド（絶対位置、初期状態では表示）
                                        parent
                                            .spawn((
                                                CreativeItemList,
                                                Node {
                                                    position_type: PositionType::Absolute,
                                                    flex_direction: FlexDirection::Column,
                                                    row_gap: Val::Px(10.0),
                                                    ..default()
                                                },
                                                Visibility::Visible, // 初期状態は表示
                                            ))
                                            .with_children(|parent| {
                                                spawn_creative_item_grid(parent, &item_registry, SLOT_SIZE, SLOT_GAP);
                                            });
                                    });

                                // ホットバー行（ホットバー + トグルボタン + ゴミ箱）
                                parent
                                    .spawn(Node {
                                        flex_direction: FlexDirection::Row,
                                        column_gap: Val::Px(10.0),
                                        margin: UiRect::top(Val::Px(10.0)),
                                        align_items: AlignItems::Center,
                                        ..default()
                                    })
                                    .with_children(|parent| {
                                        // ホットバーグリッド（10スロット）
                                        parent
                                            .spawn(Node {
                                                display: Display::Grid,
                                                grid_template_columns: RepeatedGridTrack::flex(10, 1.0),
                                                grid_template_rows: RepeatedGridTrack::flex(1, 1.0),
                                                column_gap: Val::Px(SLOT_GAP),
                                                ..default()
                                            })
                                            .with_children(|parent| {
                                                for i in 50..60 {
                                                    spawn_slot_sized(parent, SlotIdentifier::PlayerInventory(i), &player_inventory.slots[i], SLOT_SIZE);
                                                }
                                            });

                                        // トグルボタン
                                        parent.spawn((
                                            ViewToggleButton,
                                            Button,
                                            Node {
                                                width: Val::Px(SLOT_SIZE),
                                                height: Val::Px(SLOT_SIZE),
                                                justify_content: JustifyContent::Center,
                                                align_items: AlignItems::Center,
                                                border: UiRect::all(Val::Px(2.0)),
                                                ..default()
                                            },
                                            BackgroundColor(Color::srgb(0.4, 0.4, 0.5)),
                                            BorderColor(Color::srgb(0.6, 0.6, 0.6)),
                                        ))
                                        .with_children(|parent| {
                                            parent.spawn((
                                                Text::new("⇄"),
                                                TextFont { font_size: 32.0, ..default() },
                                                TextColor(Color::WHITE),
                                            ));
                                        });

                                        // ゴミ箱スロット
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
                    } else {
                        // サバイバルモード: インベントリのみ
                        parent
                            .spawn((
                                MainInventoryPanel,
                                Node {
                                    flex_direction: FlexDirection::Column,
                                    row_gap: Val::Px(20.0),
                                    ..default()
                                },
                            ))
                            .with_children(|parent| {
                                spawn_main_inventory_panel_mc(parent, &player_inventory, SLOT_SIZE, SLOT_GAP);
                            });
                    }

                    // 右側: クラフトリスト（サバイバルモードのみ）
                    if *game_mode != crate::gameplay::commands::GameMode::Creative {
                        spawn_craft_list_panel(parent, &recipe_registry);
                    }
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
            // メインコンテナ - CSS Gridで自動整列
            parent
                .spawn((
                    Node {
                        display: Display::Grid,
                        // 列: 装備パネル | メインインベントリ | コンテナ
                        grid_template_columns: vec![
                            GridTrack::auto(),  // 装備パネル
                            GridTrack::auto(),  // メインインベントリ
                            GridTrack::auto(),  // コンテナ
                        ],
                        grid_auto_rows: GridTrack::auto(),
                        padding: UiRect::all(Val::Px(20.0)),
                        column_gap: Val::Px(20.0),
                        row_gap: Val::Px(10.0),
                        align_items: AlignItems::Start, // グリッドセル内で上揃え
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
                ))
                .with_children(|parent| {
                    // 装備パネル - CSS Gridで自動整列されるのでmargin不要
                    parent
                        .spawn(Node {
                            flex_direction: FlexDirection::Column,
                            align_self: AlignSelf::Start, // 上揃え
                            ..default()
                        })
                        .with_children(|parent| {
                            spawn_equipment_panel(parent, &equipment);
                        });
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
            // タイトル部分は削除（Equipment表記を非表示に）

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
            position_type: PositionType::Relative, // ソートボタンの絶対位置の基準
            ..default()
        })
        .with_children(|parent| {
            // ソートボタン（グリッド右上に絶対位置で配置）
            parent
                .spawn((
                    SortButton,
                    Button,
                    Node {
                        position_type: PositionType::Absolute,
                        right: Val::Px(0.0),
                        top: Val::Px(-35.0), // グリッドの上に配置
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

            // ホットバー行 (ホットバー + ゴミ箱)
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(10.0),
                    margin: UiRect::top(Val::Px(10.0)),
                    align_items: AlignItems::End,
                    ..default()
                })
                .with_children(|parent| {
                    // ホットバー
                    parent
                        .spawn(Node {
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Px(5.0),
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

                    // ゴミ箱スロット
                    parent
                        .spawn((
                            TrashSlot,
                            Button,
                            Node {
                                width: Val::Px(slot_size),
                                height: Val::Px(slot_size),
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
}

/// メインインベントリグリッドのみを生成（ホットバーなし、クリエイティブモード用）
fn spawn_main_inventory_grid_only(parent: &mut ChildBuilder, inventory: &PlayerInventory, slot_size: f32, slot_gap: f32) {
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
            // Spacing node to maintain grid position (removed "Inventory" label)
            parent.spawn(Node {
                height: Val::Px(25.0),
                ..default()
            });

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
}

/// ホットバーのみを生成（共有用）
#[allow(dead_code)]
fn spawn_hotbar(parent: &mut ChildBuilder, inventory: &PlayerInventory, slot_size: f32, slot_gap: f32) {
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

/// クリエイティブモードアイテムリストパネルを生成
#[allow(dead_code)]
fn spawn_creative_item_list(parent: &mut ChildBuilder, item_registry: &ItemRegistry) {
    parent
        .spawn(Node {
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(10.0),
            width: Val::Px(400.0),
            ..default()
        })
        .with_children(|parent| {
            // Spacing node to maintain grid position (removed "Creative Items" label)
            parent.spawn(Node {
                height: Val::Px(30.0),
                ..default()
            });

            // スクロールビュー（グリッド表示）
            parent
                .spawn((
                    Node {
                        display: Display::Grid,
                        grid_template_columns: RepeatedGridTrack::flex(5, 1.0),
                        grid_auto_rows: GridTrack::px(60.0),
                        overflow: Overflow::scroll_y(),
                        max_height: Val::Px(400.0),
                        row_gap: Val::Px(5.0),
                        column_gap: Val::Px(5.0),
                        padding: UiRect::all(Val::Px(5.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.15, 0.15, 0.15)),
                ))
                .with_children(|parent| {
                    // アイテムボタンをグリッドで生成
                    for (item_id, item_data) in &item_registry.items {
                        parent
                            .spawn((
                                CreativeItemButton { item_id: item_id.clone() },
                                Button,
                                Node {
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    padding: UiRect::all(Val::Px(5.0)),
                                    ..default()
                                },
                                BackgroundColor(Color::srgb(0.3, 0.3, 0.3)),
                            ))
                            .with_children(|parent| {
                                parent.spawn((
                                    Text::new(&item_data.name),
                                    TextFont { font_size: 12.0, ..default() },
                                    TextColor(Color::WHITE),
                                ));
                            });
                    }
                });
        });
}

/// クリエイティブアイテムグリッドを生成（10x5固定グリッド、インベントリと同じサイズ）
fn spawn_creative_item_grid(parent: &mut ChildBuilder, item_registry: &ItemRegistry, slot_size: f32, slot_gap: f32) {
    // Spacing node to maintain grid position (removed "Creative Items" label)
    parent.spawn(Node {
        height: Val::Px(30.0),
        ..default()
    });

    // アイテムIDリストを取得（ソート済み）
    let mut item_ids: Vec<String> = item_registry.items.keys().cloned().collect();
    item_ids.sort();

    // 10x5グリッド（50スロット、インベントリのメイングリッドと同じ）
    const GRID_COLS: usize = 10;
    const GRID_ROWS: usize = 5;
    const TOTAL_SLOTS: usize = GRID_COLS * GRID_ROWS;

    parent
        .spawn((
            Node {
                display: Display::Grid,
                grid_template_columns: RepeatedGridTrack::flex(GRID_COLS as u16, 1.0),
                grid_template_rows: RepeatedGridTrack::flex(GRID_ROWS as u16, 1.0),
                row_gap: Val::Px(slot_gap),
                column_gap: Val::Px(slot_gap),
                padding: UiRect::all(Val::Px(5.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.15, 0.15, 0.15)),
        ))
        .with_children(|parent| {
            // 40スロット全てを生成
            for i in 0..TOTAL_SLOTS {
                if i < item_ids.len() {
                    // アイテムボタンを生成
                    let item_id = &item_ids[i];
                    if let Some(item_data) = item_registry.items.get(item_id) {
                        parent
                            .spawn((
                                CreativeItemButton { item_id: item_id.clone() },
                                Button,
                                Node {
                                    width: Val::Px(slot_size),
                                    height: Val::Px(slot_size),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    border: UiRect::all(Val::Px(2.0)),
                                    ..default()
                                },
                                BackgroundColor(Color::srgb(0.3, 0.3, 0.3)),
                                BorderColor(Color::srgb(0.5, 0.5, 0.5)),
                            ))
                            .with_children(|parent| {
                                parent.spawn((
                                    Text::new(&item_data.name),
                                    TextFont { font_size: 11.0, ..default() },
                                    TextColor(Color::WHITE),
                                ));
                            });
                    }
                } else {
                    // 空スロットを生成
                    parent.spawn((
                        Node {
                            width: Val::Px(slot_size),
                            height: Val::Px(slot_size),
                            border: UiRect::all(Val::Px(2.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.1, 0.1, 0.1)),
                        BorderColor(Color::srgb(0.3, 0.3, 0.3)),
                    ));
                }
            }
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
            // 常にテキストエンティティを生成（空の場合も）
            let text_content = if let Some(item_id) = &slot_data.item_id {
                format!("{}\n{}", item_id, slot_data.count)
            } else {
                String::new()
            };

            parent.spawn((
                Text::new(text_content),
                TextFont { font_size: 12.0, ..default() },
                TextColor(Color::WHITE),
            ));
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

            // アイテム情報（中央）- 常にテキストエンティティを生成
            let text_content = if let Some(item_id) = &slot_data.item_id {
                format!("{}\n{}", item_id, slot_data.count)
            } else {
                String::new()
            };

            parent.spawn((
                Text::new(text_content),
                TextFont { font_size: 11.0, ..default() },
                TextColor(Color::WHITE),
            ));
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
    // change detectionを削除して常に更新（ドラッグ&ドロップの即時反映のため）

    let mut updated_count = 0;
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

        // 背景色を更新（変更がある場合のみ）
        let new_color = if slot_data.is_empty() {
            Color::srgb(0.3, 0.3, 0.3)
        } else {
            Color::srgb(0.4, 0.4, 0.5)
        };

        if bg_color.0 != new_color {
            *bg_color = BackgroundColor(new_color);
        }

        // テキストを更新
        for &child in children.iter() {
            if let Ok(mut text) = text_query.get_mut(child) {
                let new_text = if let Some(item_id) = &slot_data.item_id {
                    format!("{}\n{}", item_id, slot_data.count)
                } else {
                    String::new()
                };

                if **text != new_text {
                    **text = new_text.clone();
                    updated_count += 1;

                    // PlayerInventoryスロットの更新を特別にログ出力
                    if let SlotIdentifier::PlayerInventory(i) = &ui_slot.identifier {
                        if *i >= 50 && *i < 60 {
                            info!("[UPDATE VISUAL] Hotbar slot {} updated to: {}", i, new_text);
                        }
                    }
                }
            }
        }
    }

    if updated_count > 0 {
        info!("[UPDATE VISUAL] Updated {} slot visuals", updated_count);
    }
}

/// スロットのインタラクション処理（簡易版）
#[allow(clippy::type_complexity)]
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

/// マウスリリース時のドロップ処理
fn handle_drag_drop_release(
    mouse_button: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    slots: Query<(&GlobalTransform, &ComputedNode, &UiSlot), With<Button>>,
    mut dragged: ResMut<DraggedItem>,
    mut player_inventory: ResMut<PlayerInventory>,
    mut equipment: ResMut<EquipmentSlots>,
    item_registry: Res<ItemRegistry>,
) {
    // マウスボタンがリリースされ、かつドラッグ中のアイテムがある場合
    if mouse_button.just_released(MouseButton::Left) {
        info!("[DEBUG] Mouse button released. Dragged item: {:?}", dragged.item_id);
    }

    if mouse_button.just_released(MouseButton::Left) && dragged.item_id.is_some() {
        info!("[DRAG RELEASE] Checking for slots under cursor...");

        // マウスカーソルの位置を取得
        let cursor_pos = windows
            .get_single()
            .ok()
            .and_then(|window| window.cursor_position());

        if let Some(cursor_pos) = cursor_pos {
            info!("[DEBUG] Cursor position: {:?}", cursor_pos);

            let mut found_slot = false;
            // すべてのスロットをチェックして、カーソルの下にあるものを探す
            for (global_transform, computed_node, ui_slot) in &slots {
                let slot_pos = global_transform.translation().truncate();
                let slot_size = computed_node.size();

                // スロットの境界を計算（中心座標から）
                let min_x = slot_pos.x - slot_size.x / 2.0;
                let max_x = slot_pos.x + slot_size.x / 2.0;
                let min_y = slot_pos.y - slot_size.y / 2.0;
                let max_y = slot_pos.y + slot_size.y / 2.0;

                // カーソルがスロットの範囲内にあるかチェック
                if cursor_pos.x >= min_x && cursor_pos.x <= max_x &&
                   cursor_pos.y >= min_y && cursor_pos.y <= max_y {
                    found_slot = true;
                    info!("[DRAG RELEASE] Found slot under cursor: {:?}", ui_slot.identifier);
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
                                info!("[DROP SUCCESS] Dropped {} x{} to PlayerInventory slot {} (now: {:?})",
                                    dragged_item_id, dragged_count, i, target_slot);
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
                                info!("Stacked {} x{} to slot {}", dragged_item_id, add, i);
                            } else {
                                // スワップ
                                let temp_id = target_slot.item_id.clone();
                                let temp_count = target_slot.count;

                                target_slot.item_id = Some(dragged_item_id.clone());
                                target_slot.count = dragged_count;

                                dragged.item_id = temp_id;
                                dragged.count = temp_count;
                                info!("Swapped items at slot {}", i);
                            }
                        }
                        return; // 処理完了
                    }
                    SlotIdentifier::Equipment(slot_type) => {
                        let target_slot = equipment.get_mut(*slot_type);

                        if target_slot.is_empty() {
                            target_slot.item_id = Some(dragged_item_id.clone());
                            target_slot.count = dragged_count;
                            dragged.item_id = None;
                            dragged.count = 0;
                            dragged.source_slot = None;
                            info!("Dropped {} x{} to equipment slot {:?}", dragged_item_id, dragged_count, slot_type);
                        }
                        return; // 処理完了
                    }
                    _ => {}
                }
                }
            }

            if !found_slot {
                info!("[DEBUG] No slot found under cursor");
            }
        } else {
            info!("[DEBUG] Could not get cursor position");
        }

        // どのスロットにもドロップしなかった場合、元のスロットに戻す（クリエイティブアイテムの場合は破棄）
        if let Some(source) = dragged.source_slot.clone() {
            info!("[DEBUG] Returning item to source slot: {:?}", source);
            match source {
                SlotIdentifier::PlayerInventory(i) => {
                    if i < player_inventory.slots.len() {
                        player_inventory.slots[i].item_id = dragged.item_id.clone();
                        player_inventory.slots[i].count = dragged.count;
                    }
                }
                SlotIdentifier::Equipment(slot_type) => {
                    equipment.get_mut(slot_type).item_id = dragged.item_id.clone();
                    equipment.get_mut(slot_type).count = dragged.count;
                }
                _ => {}
            }
        } else {
            info!("[DEBUG] No source slot (creative item), clearing drag state");
        }

        // ドラッグ状態をクリア
        info!("[DEBUG] Clearing drag state");
        dragged.item_id = None;
        dragged.count = 0;
        dragged.source_slot = None;
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

/// クリエイティブモードアイテムボタン処理（ドラッグ&ドロップ対応）
fn handle_creative_item_button(
    interaction_query: Query<(&Interaction, &CreativeItemButton), Changed<Interaction>>,
    mut dragged: ResMut<DraggedItem>,
) {
    for (interaction, item_button) in &interaction_query {
        info!("[DEBUG] CreativeItemButton interaction: {:?}, item: {}", interaction, item_button.item_id);
        if *interaction == Interaction::Pressed {
            // クリエイティブアイテムからのドラッグ開始
            if dragged.item_id.is_none() {
                dragged.item_id = Some(item_button.item_id.clone());
                dragged.count = 64; // クリエイティブモードは64個
                dragged.source_slot = None; // クリエイティブアイテムは無限なので元に戻さない

                info!("[DRAG START] Started dragging {} x64 from creative catalog", item_button.item_id);
            } else {
                info!("[DEBUG] Already dragging: {:?}", dragged.item_id);
            }
        }
    }
}

/// クリエイティブモード表示切替ボタン処理
fn handle_view_toggle_button(
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<ViewToggleButton>)>,
    mut view_mode: ResMut<CreativeViewMode>,
) {
    for interaction in &interaction_query {
        if *interaction == Interaction::Pressed {
            *view_mode = match *view_mode {
                CreativeViewMode::Catalog => CreativeViewMode::Inventory,
                CreativeViewMode::Inventory => CreativeViewMode::Catalog,
            };
            info!("Toggled creative view mode: {:?}", *view_mode);
        }
    }
}

/// クリエイティブモード初期表示設定
fn initialize_creative_visibility(
    view_mode: Res<CreativeViewMode>,
    game_mode: Res<crate::gameplay::commands::GameMode>,
    mut inventory_query: Query<&mut Visibility, (With<MainInventoryPanel>, Without<CreativeItemList>)>,
    mut creative_list_query: Query<&mut Visibility, (With<CreativeItemList>, Without<MainInventoryPanel>)>,
) {
    // クリエイティブモードでない場合は何もしない
    if *game_mode != crate::gameplay::commands::GameMode::Creative {
        return;
    }

    // Catalogモード: アイテムカタログ表示、インベントリは非表示
    // Inventoryモード: インベントリ表示、アイテムカタログは非表示
    let show_catalog = *view_mode == CreativeViewMode::Catalog;

    // メインインベントリパネルの可視性を設定
    for mut visibility in &mut inventory_query {
        *visibility = if show_catalog { Visibility::Hidden } else { Visibility::Visible };
    }

    // クリエイティブアイテムカタログの可視性を設定
    for mut visibility in &mut creative_list_query {
        *visibility = if show_catalog { Visibility::Visible } else { Visibility::Hidden };
    }
}

/// クリエイティブモード表示の可視性を更新
fn update_creative_view_visibility(
    view_mode: Res<CreativeViewMode>,
    game_mode: Res<crate::gameplay::commands::GameMode>,
    mut inventory_query: Query<&mut Visibility, (With<MainInventoryPanel>, Without<CreativeItemList>)>,
    mut creative_list_query: Query<&mut Visibility, (With<CreativeItemList>, Without<MainInventoryPanel>)>,
) {
    // クリエイティブモードでない場合は何もしない
    if *game_mode != crate::gameplay::commands::GameMode::Creative {
        return;
    }

    // CreativeViewModeが変更された場合のみ更新
    if !view_mode.is_changed() {
        return;
    }

    // Catalogモード: アイテムカタログ表示、インベントリは非表示
    // Inventoryモード: インベントリ表示、アイテムカタログは非表示
    let show_catalog = *view_mode == CreativeViewMode::Catalog;

    // メインインベントリパネルの可視性を設定
    for mut visibility in &mut inventory_query {
        *visibility = if show_catalog { Visibility::Hidden } else { Visibility::Visible };
    }

    // クリエイティブアイテムカタログの可視性を設定
    for mut visibility in &mut creative_list_query {
        *visibility = if show_catalog { Visibility::Visible } else { Visibility::Hidden };
    }
}

/// ツールチップ更新（簡易版）
#[allow(clippy::type_complexity)]
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

/// インベントリUI終了時にカーソルをグラブ
fn grab_cursor(
    mut window_query: Query<&mut Window, With<PrimaryWindow>>,
) {
    if let Ok(mut window) = window_query.get_single_mut() {
        window.cursor_options.grab_mode = CursorGrabMode::Locked;
        window.cursor_options.visible = false;
    }
}

/// ホットバーHUDを生成
fn spawn_hotbar_hud(
    mut commands: Commands,
    player_inventory: Res<PlayerInventory>,
    item_registry: Res<ItemRegistry>,
) {
    const SLOT_SIZE: f32 = 54.0;
    const SLOT_GAP: f32 = 4.0;

    commands
        .spawn((
            HotbarHud,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::FlexEnd, // 縦方向で下に配置
                align_items: AlignItems::Center, // 横方向で中央に配置
                padding: UiRect::bottom(Val::Px(20.0)),
                flex_direction: FlexDirection::Column,
                ..default()
            },
        ))
        .with_children(|parent| {
            // アイテム名表示（ホットバーの上）
            let selected_slot = &player_inventory.slots[player_inventory.selected_hotbar_slot];
            let item_name = if let Some(item_id) = &selected_slot.item_id {
                item_registry.get(item_id)
                    .map(|data| data.name.clone())
                    .unwrap_or_else(|| item_id.clone())
            } else {
                String::new()
            };

            parent.spawn((
                HotbarItemName,
                Text::new(item_name),
                TextFont { font_size: 24.0, ..default() },
                TextColor(Color::WHITE),
                Node {
                    margin: UiRect::bottom(Val::Px(10.0)),
                    align_self: AlignSelf::Center,
                    ..default()
                },
            ));

            // ホットバースロット
            parent
                .spawn(Node {
                    display: Display::Grid,
                    grid_template_columns: RepeatedGridTrack::flex(10, 1.0),
                    grid_template_rows: RepeatedGridTrack::flex(1, 1.0),
                    column_gap: Val::Px(SLOT_GAP),
                    padding: UiRect::all(Val::Px(8.0)),
                    align_self: AlignSelf::Center,
                    ..default()
                })
                .with_children(|parent| {
                    for i in 50..60 {
                        let is_selected = i == player_inventory.selected_hotbar_slot;
                        spawn_hotbar_slot(parent, i, &player_inventory.slots[i], SLOT_SIZE, is_selected);
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
    is_selected: bool,
) {
    let (bg_color, border_color, border_width) = if is_selected {
        // 選択中のスロット: 明るい背景、白い太い枠
        (Color::srgba(0.3, 0.3, 0.4, 0.9), Color::WHITE, 3.0)
    } else {
        // 非選択のスロット: 暗い背景、通常の枠
        (Color::srgba(0.1, 0.1, 0.1, 0.8), Color::srgb(0.5, 0.5, 0.5), 2.0)
    };

    parent
        .spawn((
            UiSlot { identifier: SlotIdentifier::PlayerInventory(index) },
            Node {
                width: Val::Px(size),
                height: Val::Px(size),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: UiRect::all(Val::Px(border_width)),
                ..default()
            },
            BackgroundColor(bg_color),
            BorderColor(border_color),
        ))
        .with_children(|parent| {
            // 常にテキストエンティティを生成（空の場合も）
            let text_content = if let Some(item_id) = &slot_data.item_id {
                format!("{}\n{}", item_id, slot_data.count)
            } else {
                String::new()
            };

            parent.spawn((
                Text::new(text_content),
                TextFont { font_size: 11.0, ..default() },
                TextColor(Color::WHITE),
            ));
        });
}

/// ホットバーHUDを削除
#[allow(dead_code)]
fn despawn_hotbar_hud(
    mut commands: Commands,
    query: Query<Entity, With<HotbarHud>>,
) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
}

/// クリエイティブモードの場合のみホットバーHUDを生成
fn spawn_hotbar_hud_if_creative(
    commands: Commands,
    player_inventory: Res<PlayerInventory>,
    item_registry: Res<ItemRegistry>,
    game_mode: Res<crate::gameplay::commands::GameMode>,
) {
    // クリエイティブモードの場合のみ生成
    if *game_mode == crate::gameplay::commands::GameMode::Creative {
        spawn_hotbar_hud(commands, player_inventory, item_registry);
    }
}

/// サバイバルモードの場合のみホットバーHUDを表示
fn spawn_hotbar_hud_if_not_creative(
    mut query: Query<&mut Visibility, With<HotbarHud>>,
    game_mode: Res<crate::gameplay::commands::GameMode>,
) {
    // サバイバルモードの場合のみ表示
    if *game_mode == crate::gameplay::commands::GameMode::Survival {
        for mut visibility in &mut query {
            *visibility = Visibility::Visible;
        }
    }
}

/// サバイバルモードの場合のみホットバーHUDを非表示
fn hide_hotbar_hud_if_not_creative(
    mut query: Query<&mut Visibility, With<HotbarHud>>,
    game_mode: Res<crate::gameplay::commands::GameMode>,
) {
    // サバイバルモードの場合のみ非表示
    if *game_mode == crate::gameplay::commands::GameMode::Survival {
        for mut visibility in &mut query {
            *visibility = Visibility::Hidden;
        }
    }
}

/// ホットバーHUDを更新
#[allow(clippy::type_complexity)]
fn update_hotbar_hud(
    player_inventory: Res<PlayerInventory>,
    item_registry: Res<ItemRegistry>,
    mut slot_query: Query<(&UiSlot, Option<&Children>, &mut BackgroundColor, &mut BorderColor, &mut Node), Without<Button>>,
    mut text_query: Query<&mut Text, (Without<HotbarItemName>, Without<crate::ui::command_ui::CommandHistoryText>, Without<crate::ui::command_ui::CommandInputText>, Without<crate::ui::command_ui::CommandSuggestions>)>,
    mut item_name_query: Query<&mut Text, With<HotbarItemName>>,
) {

    // アイテム名を更新
    if let Ok(mut text) = item_name_query.get_single_mut() {
        let selected_slot = &player_inventory.slots[player_inventory.selected_hotbar_slot];
        **text = if let Some(item_id) = &selected_slot.item_id {
            item_registry.get(item_id)
                .map(|data| data.name.clone())
                .unwrap_or_else(|| item_id.clone())
        } else {
            String::new()
        };
    }

    for (ui_slot, children_opt, mut bg_color, mut border_color, mut node) in &mut slot_query {
        if let SlotIdentifier::PlayerInventory(i) = &ui_slot.identifier {
            if *i >= 50 && *i < 60 {
                let slot_data = &player_inventory.slots[*i];
                let is_selected = *i == player_inventory.selected_hotbar_slot;

                // 選択状態に応じて背景色と枠色を更新
                if is_selected {
                    *bg_color = BackgroundColor(Color::srgba(0.3, 0.3, 0.4, 0.9));
                    *border_color = BorderColor(Color::WHITE);
                    node.border = UiRect::all(Val::Px(3.0));
                } else {
                    if slot_data.is_empty() {
                        *bg_color = BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.8));
                    } else {
                        *bg_color = BackgroundColor(Color::srgba(0.2, 0.2, 0.3, 0.9));
                    }
                    *border_color = BorderColor(Color::srgb(0.5, 0.5, 0.5));
                    node.border = UiRect::all(Val::Px(2.0));
                }

                // テキストを更新（Childrenが存在する場合）
                if let Some(children) = children_opt {
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
}

/// ドラッグ中のアイテムを視覚的に表示
fn update_dragged_item_visual(
    dragged: Res<DraggedItem>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut commands: Commands,
    visual_query: Query<Entity, With<DraggedItemVisual>>,
    mut node_query: Query<&mut Node, With<DraggedItemVisual>>,
) {
    let is_dragging = dragged.item_id.is_some();
    let visual_exists = !visual_query.is_empty();

    if is_dragging {
        // ドラッグ中: ビジュアルを表示または更新
        if let Some(cursor_pos) = windows.get_single().ok().and_then(|w| w.cursor_position()) {
            if !visual_exists {
                // ビジュアルを新規作成
                let item_name = dragged.item_id.as_ref().unwrap();
                let text = format!("{} x{}", item_name, dragged.count);

                commands.spawn((
                    DraggedItemVisual,
                    Text::new(text),
                    TextFont {
                        font_size: 16.0,
                        ..default()
                    },
                    TextColor(Color::srgb(1.0, 1.0, 0.0)), // 黄色
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Px(cursor_pos.x + 10.0),
                        top: Val::Px(cursor_pos.y + 10.0),
                        ..default()
                    },
                ));
            } else {
                // 既存のビジュアルの位置を更新
                for mut node in &mut node_query {
                    node.left = Val::Px(cursor_pos.x + 10.0);
                    node.top = Val::Px(cursor_pos.y + 10.0);
                }
            }
        }
    } else if visual_exists {
        // ドラッグ終了: ビジュアルを削除
        for entity in &visual_query {
            commands.entity(entity).despawn_recursive();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::app::App;
    use bevy::ecs::system::RunSystemOnce;
    use crate::gameplay::inventory::ItemRegistry;

    // ========================================
    // テスト用自動UI操作ヘルパー
    // ========================================

    /// テスト用: スロットにアイテムを直接設定
    fn set_slot_item(inventory: &mut PlayerInventory, slot_index: usize, item_id: &str, count: u32) {
        if slot_index < inventory.slots.len() {
            inventory.slots[slot_index] = crate::gameplay::inventory::InventorySlot::new(item_id.to_string(), count);
        }
    }

    /// テスト用: スロットをクリア
    #[allow(dead_code)]
    fn clear_slot(inventory: &mut PlayerInventory, slot_index: usize) {
        if slot_index < inventory.slots.len() {
            inventory.slots[slot_index].clear();
        }
    }

    /// テスト用: ドラッグ&ドロップをシミュレート
    fn simulate_drag_drop(
        inventory: &mut PlayerInventory,
        from_slot: usize,
        to_slot: usize,
        item_registry: &ItemRegistry,
    ) -> bool {
        if from_slot >= inventory.slots.len() || to_slot >= inventory.slots.len() {
            return false;
        }

        let source = inventory.slots[from_slot].clone();
        if source.is_empty() {
            return false;
        }

        let target = &mut inventory.slots[to_slot];

        if target.is_empty() {
            // 空スロットへ移動
            target.item_id = source.item_id.clone();
            target.count = source.count;
            inventory.slots[from_slot].clear();
            true
        } else if target.item_id == source.item_id {
            // 同種アイテムをスタック
            let max_stack = source.item_id.as_ref()
                .and_then(|id| item_registry.get(id))
                .map(|d| d.max_stack)
                .unwrap_or(999);

            let space = max_stack.saturating_sub(target.count);
            let add = source.count.min(space);
            target.count += add;

            if add == source.count {
                inventory.slots[from_slot].clear();
            } else {
                inventory.slots[from_slot].count -= add;
            }
            true
        } else {
            // スワップ
            let temp = inventory.slots[to_slot].clone();
            inventory.slots[to_slot] = source;
            inventory.slots[from_slot] = temp;
            true
        }
    }

    /// テスト用: スロット内容を検証
    fn assert_slot_contains(inventory: &PlayerInventory, slot_index: usize, expected_item: Option<&str>, expected_count: u32) {
        let slot = &inventory.slots[slot_index];
        assert_eq!(slot.item_id.as_deref(), expected_item, "Slot {} item mismatch", slot_index);
        assert_eq!(slot.count, expected_count, "Slot {} count mismatch", slot_index);
    }

    /// テスト用: スロットが空であることを検証
    fn assert_slot_empty(inventory: &PlayerInventory, slot_index: usize) {
        assert!(inventory.slots[slot_index].is_empty(), "Slot {} should be empty", slot_index);
    }

    // ========================================
    // 自動UIテスト
    // ========================================

    #[test]
    fn test_drag_drop_to_empty_slot() {
        let mut inventory = PlayerInventory::new(60);
        let item_registry = ItemRegistry::default();

        // スロット0にアイテムを設定
        set_slot_item(&mut inventory, 0, "stone", 10);

        // スロット0からスロット5へドラッグ&ドロップ
        let result = simulate_drag_drop(&mut inventory, 0, 5, &item_registry);

        assert!(result, "Drag drop should succeed");
        assert_slot_empty(&inventory, 0);
        assert_slot_contains(&inventory, 5, Some("stone"), 10);
    }

    #[test]
    fn test_drag_drop_stack_same_item() {
        let mut inventory = PlayerInventory::new(60);
        let item_registry = ItemRegistry::default();

        // スロット0と1に同じアイテムを設定
        set_slot_item(&mut inventory, 0, "stone", 50);
        set_slot_item(&mut inventory, 1, "stone", 30);

        // スロット0からスロット1へドラッグ&ドロップ（スタック）
        let result = simulate_drag_drop(&mut inventory, 0, 1, &item_registry);

        assert!(result, "Drag drop should succeed");
        assert_slot_empty(&inventory, 0);
        assert_slot_contains(&inventory, 1, Some("stone"), 80);
    }

    #[test]
    fn test_drag_drop_swap_different_items() {
        let mut inventory = PlayerInventory::new(60);
        let item_registry = ItemRegistry::default();

        // 異なるアイテムを設定
        set_slot_item(&mut inventory, 0, "stone", 10);
        set_slot_item(&mut inventory, 1, "iron", 20);

        // スロット0からスロット1へドラッグ&ドロップ（スワップ）
        let result = simulate_drag_drop(&mut inventory, 0, 1, &item_registry);

        assert!(result, "Drag drop should succeed");
        assert_slot_contains(&inventory, 0, Some("iron"), 20);
        assert_slot_contains(&inventory, 1, Some("stone"), 10);
    }

    #[test]
    fn test_hotbar_slot_selection() {
        let mut inventory = PlayerInventory::new(60);

        // ホットバースロットにアイテムを設定
        set_slot_item(&mut inventory, 50, "pickaxe", 1);
        set_slot_item(&mut inventory, 52, "sword", 1);

        // 選択スロットを変更
        inventory.selected_hotbar_slot = 52;

        assert_eq!(inventory.selected_hotbar_slot, 52);
        assert_slot_contains(&inventory, 52, Some("sword"), 1);
    }

    #[test]
    fn test_inventory_sort() {
        let mut inventory = PlayerInventory::new(60);

        // バラバラにアイテムを配置
        set_slot_item(&mut inventory, 5, "stone", 10);
        set_slot_item(&mut inventory, 2, "iron", 5);
        set_slot_item(&mut inventory, 8, "stone", 20);

        // ソート実行
        inventory.sort();

        // ソート後: 同じアイテムがまとまり、ID順に並ぶ
        // iron(5), stone(30)の順になるはず
        let mut found_items: Vec<(String, u32)> = Vec::new();
        for slot in &inventory.slots[0..50] {
            if let Some(id) = &slot.item_id {
                found_items.push((id.clone(), slot.count));
            }
        }

        assert!(!found_items.is_empty(), "Should have items after sort");
        // アイテムの合計数が維持されていることを確認
        let stone_total: u32 = found_items.iter()
            .filter(|(id, _)| id == "stone")
            .map(|(_, c)| c)
            .sum();
        let iron_total: u32 = found_items.iter()
            .filter(|(id, _)| id == "iron")
            .map(|(_, c)| c)
            .sum();

        assert_eq!(stone_total, 30, "Stone total should be 30");
        assert_eq!(iron_total, 5, "Iron total should be 5");
    }

    #[test]
    fn test_hotbar_highlight_updates() {
        // Appを作成
        let mut app = App::new();
        app.add_plugins(bevy::state::app::StatesPlugin);

        // 必要なリソースを追加
        app.insert_resource(PlayerInventory::new(60));
        app.insert_resource(ItemRegistry::default());
        app.init_state::<InventoryUiState>();

        // update_hotbar_hudシステムを追加
        app.add_systems(Update, update_hotbar_hud);

        // ホットバーHUDを手動で生成
        let _ = app.world_mut().run_system_once(|mut commands: Commands, inventory: Res<PlayerInventory>| {
            const SLOT_SIZE: f32 = 54.0;
            commands.spawn(HotbarHud).with_children(|parent| {
                for i in 50..60 {
                    let is_selected = i == inventory.selected_hotbar_slot;
                    spawn_hotbar_slot(parent, i, &inventory.slots[i], SLOT_SIZE, is_selected);
                }
            });
        });

        // 初期状態: スロット50が選択されている
        app.update();

        // スロット50の枠色を確認（クエリを直接使用）
        {
            let world = app.world_mut();
            let mut query = world.query_filtered::<(&UiSlot, &BorderColor), Without<Button>>();
            let mut found_selected = false;
            for (ui_slot, border_color) in query.iter(world) {
                if let SlotIdentifier::PlayerInventory(i) = &ui_slot.identifier {
                    if *i == 50 {
                        assert_eq!(border_color.0, Color::WHITE, "Slot 50 should have white border");
                        found_selected = true;
                    } else if *i >= 50 && *i < 60 {
                        assert_eq!(border_color.0, Color::srgb(0.5, 0.5, 0.5), "Slot {} should have gray border", i);
                    }
                }
            }
            assert!(found_selected, "Selected slot 50 should be found");
        }

        // スロット52に変更
        app.world_mut().resource_mut::<PlayerInventory>().selected_hotbar_slot = 52;
        app.update();

        // スロット52の枠色を確認
        {
            let world = app.world_mut();
            let mut query = world.query_filtered::<(&UiSlot, &BorderColor), Without<Button>>();
            let mut found_new_selected = false;
            for (ui_slot, border_color) in query.iter(world) {
                if let SlotIdentifier::PlayerInventory(i) = &ui_slot.identifier {
                    if *i == 52 {
                        assert_eq!(border_color.0, Color::WHITE, "Slot 52 should have white border");
                        found_new_selected = true;
                    } else if *i == 50 {
                        assert_eq!(border_color.0, Color::srgb(0.5, 0.5, 0.5), "Slot 50 should now have gray border");
                    }
                }
            }
            assert!(found_new_selected, "Selected slot 52 should be found");
        }
    }

    #[test]
    fn test_empty_hotbar_slot_has_text_entity() {
        // Appを作成
        let mut app = App::new();

        // 空のスロットを生成
        let _ = app.world_mut().run_system_once(|mut commands: Commands| {
            commands.spawn(Node::default()).with_children(|parent| {
                let empty_slot = crate::gameplay::inventory::InventorySlot::empty();
                spawn_hotbar_slot(parent, 50, &empty_slot, 54.0, false);
            });
        });

        // スロットに子エンティティ（テキスト）が存在するか確認
        {
            let world = app.world_mut();
            let mut slot_query = world.query_filtered::<&Children, With<UiSlot>>();
            let mut text_query = world.query::<&Text>();
            let mut has_text_child = false;

            for children in slot_query.iter(world) {
                for &child in children.iter() {
                    if text_query.get(world, child).is_ok() {
                        has_text_child = true;
                        break;
                    }
                }
            }

            assert!(has_text_child, "Empty hotbar slot should have a text child entity");
        }
    }
}
