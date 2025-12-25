// src/ui/inventory_ui/render.rs
//! インベントリUI描画関連

use bevy::prelude::*;
use crate::gameplay::inventory::{PlayerInventory, EquipmentSlots, ItemRegistry, EquipmentSlotType, InventorySlot};
use crate::core::registry::RecipeRegistry;
use crate::ui::styles::{colors, sizes, fonts};
use super::types::*;

pub(super) fn spawn_player_inventory_ui(
    mut commands: Commands,
    player_inventory: Res<PlayerInventory>,
    equipment: Res<EquipmentSlots>,
    recipe_registry: Res<RecipeRegistry>,
    item_registry: Res<ItemRegistry>,
    game_mode: Res<crate::gameplay::commands::GameMode>,
    config: Res<crate::core::config::GameConfig>,
) {
    let slot_size = sizes::SLOT;
    let slot_gap = sizes::SLOT_GAP;

    // enable_ui_blurが有効な場合、背景を暗くしてぼやけた効果を出す
    let bg_alpha = if config.enable_ui_blur { 0.85 } else { 0.75 };

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
            // メインコンテナ - モダンなGlassmorphism風パネル
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
                        padding: UiRect::all(Val::Px(sizes::PANEL_PADDING)),
                        column_gap: Val::Px(sizes::PANEL_GAP),
                        row_gap: Val::Px(10.0),
                        align_items: AlignItems::Start, // グリッドセル内で上揃え
                        border: UiRect::all(Val::Px(sizes::BORDER_THIN)),
                        ..default()
                    },
                    BackgroundColor(colors::BG_PANEL),
                    BorderColor(colors::BORDER),
                    BorderRadius::all(Val::Px(sizes::RADIUS_LG)),
                ))
                .with_children(|parent| {
                    // 左側: 装備スロット (縦並び) - CSS Gridで自動整列されるのでmargin不要
                    parent
                        .spawn((
                            EquipmentPanel,
                            Node {
                                flex_direction: FlexDirection::Column,
                                row_gap: Val::Px(slot_gap),
                                align_self: AlignSelf::Center, // 中央揃え（メインインベントリと揃える）
                                justify_content: JustifyContent::Center,
                                ..default()
                            },
                        ))
                        .with_children(|parent| {
                            spawn_equipment_panel_mc(parent, &equipment, slot_size, slot_gap);
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
                                                spawn_main_inventory_grid_only(parent, &player_inventory, slot_size, slot_gap);
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
                                                spawn_creative_item_grid(parent, &item_registry, slot_size, slot_gap);
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
                                                column_gap: Val::Px(slot_gap),
                                                ..default()
                                            })
                                            .with_children(|parent| {
                                                for i in 50..60 {
                                                    spawn_slot_sized(parent, SlotIdentifier::PlayerInventory(i), &player_inventory.slots[i], slot_size);
                                                }
                                            });

                                        // トグルボタン - モダンなスタイル
                                        parent.spawn((
                                            ViewToggleButton,
                                            Button,
                                            Node {
                                                width: Val::Px(slot_size),
                                                height: Val::Px(slot_size),
                                                justify_content: JustifyContent::Center,
                                                align_items: AlignItems::Center,
                                                border: UiRect::all(Val::Px(sizes::BORDER_NORMAL)),
                                                ..default()
                                            },
                                            BackgroundColor(colors::BUTTON_DEFAULT),
                                            BorderColor(colors::BORDER),
                                            BorderRadius::all(Val::Px(sizes::RADIUS_SM)),
                                        ))
                                        .with_children(|parent| {
                                            parent.spawn((
                                                Text::new("⇄"),
                                                TextFont { font_size: fonts::TITLE_MD, ..default() },
                                                TextColor(colors::TEXT_PRIMARY),
                                            ));
                                        });

                                        // ゴミ箱スロット - モダンなスタイル
                                        parent
                                            .spawn((
                                                TrashSlot,
                                                Button,
                                                Node {
                                                    width: Val::Px(slot_size),
                                                    height: Val::Px(slot_size),
                                                    justify_content: JustifyContent::Center,
                                                    align_items: AlignItems::Center,
                                                    border: UiRect::all(Val::Px(sizes::BORDER_NORMAL)),
                                                    ..default()
                                                },
                                                BackgroundColor(colors::DANGER),
                                                BorderColor(Color::srgba(0.95, 0.35, 0.40, 0.8)),
                                                BorderRadius::all(Val::Px(sizes::RADIUS_SM)),
                                            ))
                                            .with_children(|parent| {
                                                parent.spawn((
                                                    Text::new("Trash"),
                                                    TextFont { font_size: fonts::CAPTION, ..default() },
                                                    TextColor(colors::TEXT_PRIMARY),
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
                                spawn_main_inventory_panel_mc(parent, &player_inventory, slot_size, slot_gap);
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
pub(super) fn spawn_container_ui(
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
pub(super) fn spawn_equipment_panel_mc(parent: &mut ChildBuilder, equipment: &EquipmentSlots, slot_size: f32, slot_gap: f32) {
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
pub(super) fn spawn_equipment_panel(parent: &mut ChildBuilder, equipment: &EquipmentSlots) {
    spawn_equipment_panel_mc(parent, equipment, 54.0, 4.0);
}

/// メインインベントリパネルを生成（Minecraft風）
pub(super) fn spawn_main_inventory_panel_mc(parent: &mut ChildBuilder, inventory: &PlayerInventory, slot_size: f32, slot_gap: f32) {
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
pub(super) fn spawn_main_inventory_grid_only(parent: &mut ChildBuilder, inventory: &PlayerInventory, slot_size: f32, slot_gap: f32) {
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
pub(super) fn spawn_hotbar(parent: &mut ChildBuilder, inventory: &PlayerInventory, slot_size: f32, slot_gap: f32) {
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
pub(super) fn spawn_inventory_panel(parent: &mut ChildBuilder, inventory: &PlayerInventory) {
    spawn_main_inventory_panel_mc(parent, inventory, 54.0, 4.0);
}

/// クラフトリストパネルを生成
pub(super) fn spawn_craft_list_panel(parent: &mut ChildBuilder, recipe_registry: &RecipeRegistry) {
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
pub(super) fn spawn_creative_item_list(parent: &mut ChildBuilder, item_registry: &ItemRegistry) {
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
pub(super) fn spawn_creative_item_grid(parent: &mut ChildBuilder, item_registry: &ItemRegistry, slot_size: f32, slot_gap: f32) {
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
pub(super) fn spawn_container_panel(parent: &mut ChildBuilder) {
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
pub(super) fn spawn_slot_sized(
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
pub(super) fn spawn_slot_with_icon(
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
pub(super) fn spawn_slot(
    parent: &mut ChildBuilder,
    identifier: SlotIdentifier,
    slot_data: &crate::gameplay::inventory::InventorySlot,
) {
    spawn_slot_sized(parent, identifier, slot_data, 54.0);
}

