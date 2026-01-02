# v0.2 UI コード抜粋

## 1. GlobalInventory表示パネル (src/setup/ui/inventory_ui.rs)

```rust
// === GlobalInventory panel (machine storage) ===
parent
    .spawn((
        GlobalInventoryPanel,
        Node {
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(2.0),
            padding: UiRect::all(Val::Px(4.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.15, 0.12, 0.1, 0.9)),
    ))
    .with_children(|panel| {
        // Title
        panel.spawn((
            Text::new("機械ストレージ"),
            TextFont {
                font_size: 12.0,
                ..default()
            },
            TextColor(Color::srgba(0.9, 0.8, 0.6, 1.0)),
            Node {
                margin: UiRect::bottom(Val::Px(4.0)),
                ..default()
            },
        ));

        // Item rows (2x2 grid)
        panel
            .spawn((Node {
                flex_direction: FlexDirection::Row,
                flex_wrap: FlexWrap::Wrap,
                column_gap: Val::Px(8.0),
                row_gap: Val::Px(2.0),
                ..default()
            },))
            .with_children(|grid| {
                for &block_type in GLOBAL_INVENTORY_ITEMS {
                    grid.spawn((
                        GlobalInventoryRow(block_type),
                        Node {
                            flex_direction: FlexDirection::Row,
                            align_items: AlignItems::Center,
                            column_gap: Val::Px(4.0),
                            min_width: Val::Px(100.0),
                            ..default()
                        },
                    ))
                    .with_children(|row| {
                        // Slot-like icon background
                        row.spawn((
                            Node {
                                width: Val::Px(SLOT_SIZE * 0.6),
                                height: Val::Px(SLOT_SIZE * 0.6),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                border: UiRect::all(Val::Px(SLOT_BORDER)),
                                ..default()
                            },
                            BackgroundColor(Color::srgba(0.2, 0.18, 0.15, 0.95)),
                            BorderColor(Color::srgba(0.4, 0.35, 0.3, 1.0)),
                        ))
                        .with_children(|slot| {
                            slot.spawn((
                                Text::new(block_type.short_name()),
                                TextFont {
                                    font_size: 8.0,
                                    ..default()
                                },
                                TextColor(Color::WHITE),
                            ));
                        });

                        // Item name and count
                        row.spawn((
                            GlobalInventoryCountText(block_type),
                            Text::new(format!("{}: 0", block_type.name())),
                            TextFont {
                                font_size: 11.0,
                                ..default()
                            },
                            TextColor(Color::srgba(0.8, 0.8, 0.7, 1.0)),
                        ));
                    });
                }
            });
    });
```

## 2. クエスト納品ボタン (src/setup/ui/mod.rs)

```rust
// Deliver button (shown when quest is completable)
parent
    .spawn((
        Button,
        QuestDeliverButton,
        Node {
            width: Val::Px(100.0),
            height: Val::Px(30.0),
            margin: UiRect::top(Val::Px(8.0)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border: UiRect::all(Val::Px(2.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.2, 0.4, 0.2, 0.9)),
        BorderColor(Color::srgba(0.3, 0.6, 0.3, 1.0)),
        Visibility::Hidden,
    ))
    .with_children(|btn| {
        btn.spawn((
            Text::new("納品"),
            TextFont {
                font_size: 14.0,
                ..default()
            },
            TextColor(Color::WHITE),
        ));
    });
```

## 3. クエストUI進捗表示 (src/systems/quest.rs)

```rust
// Show progress for each required item
let progress: Vec<String> = quest.required_items.iter().map(|(item, amount)| {
    let delivered = platform
        .map(|p| p.delivered.get(item).copied().unwrap_or(0))
        .unwrap_or(0);
    let in_inventory = global_inventory.get_count(*item);
    format!("{}: {}/{} (在庫: {})", item.name(), delivered.min(*amount), amount, in_inventory)
}).collect();
```

## UI色設計

| 要素 | 背景色 | ボーダー色 | テキスト色 |
|------|--------|-----------|-----------|
| GlobalInventoryパネル | srgba(0.15, 0.12, 0.1, 0.9) | - | - |
| タイトル | - | - | srgba(0.9, 0.8, 0.6, 1.0) |
| スロット背景 | srgba(0.2, 0.18, 0.15, 0.95) | srgba(0.4, 0.35, 0.3, 1.0) | WHITE |
| アイテム名 | - | - | srgba(0.8, 0.8, 0.7, 1.0) |
| 納品ボタン | srgba(0.2, 0.4, 0.2, 0.9) | srgba(0.3, 0.6, 0.3, 1.0) | WHITE |
| ホバー時 | srgba(0.3, 0.6, 0.3, 0.95) | srgba(0.4, 0.8, 0.4, 1.0) | - |
