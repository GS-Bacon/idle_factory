# ゲームUI/HUD実装ベストプラクティス

**作成日**: 2025-12-22
**目的**: 視認性が高く使いやすいゲームUIを設計するための指針

---

## 1. 基本原則

### 1.1 UIの種類

| 種類 | 説明 | 例 |
|------|------|-----|
| Diegetic | ゲーム世界内に存在 | Dead Spaceの背中HP |
| Non-diegetic | 画面オーバーレイ | 通常のHP/MP表示 |
| Spatial | 3D空間に浮かぶ | 敵の頭上HP、名前タグ |
| Meta | 画面効果で表現 | 被ダメージで赤くなる |

### 1.2 情報階層（Information Hierarchy）

```
優先度1（常時表示）:
  - HP/リソース
  - 緊急警告

優先度2（状況に応じて）:
  - ミニマップ
  - クエスト進行
  - ホットバー

優先度3（要求時のみ）:
  - 詳細ステータス
  - インベントリ
  - 設定メニュー
```

---

## 2. HUDデザイン

### 2.1 レイアウト原則

```
┌─────────────────────────────────────┐
│ [HP]                    [ミニマップ] │
│                                     │
│                                     │
│                                     │
│                                     │
│                                     │
│ [クエスト]                          │
│                                     │
│           [ホットバー/ツール]        │
└─────────────────────────────────────┘

原則:
  - 重要情報は四隅に配置
  - 中央は空けてゲームプレイに集中
  - 関連情報はグループ化
```

### 2.2 要素の優先順位

```rust
struct HudElement {
    priority: HudPriority,
    visibility: Visibility,
    position: HudPosition,
}

enum HudPriority {
    Critical,   // HP、警告（常に最前面）
    Primary,    // ホットバー、ミニマップ
    Secondary,  // クエスト、通知
    Tertiary,   // 詳細情報、ヒント
}

// 優先度に応じてZオーダーと透明度を調整
fn calculate_visibility(priority: HudPriority, context: &GameContext) -> f32 {
    match priority {
        HudPriority::Critical => 1.0,
        HudPriority::Primary => if context.in_combat { 1.0 } else { 0.8 },
        HudPriority::Secondary => if context.in_menu { 1.0 } else { 0.6 },
        HudPriority::Tertiary => if context.hovering { 1.0 } else { 0.0 },
    }
}
```

### 2.3 動的HUD

```rust
// 状況に応じてHUDを表示/非表示
fn update_hud_visibility(
    context: &GameContext,
    mut visibility_query: Query<(&HudElement, &mut Visibility)>,
) {
    for (element, mut vis) in visibility_query.iter_mut() {
        *vis = match (context.state, element.element_type) {
            // 戦闘中はHP強調
            (GameState::Combat, HudElementType::Health) => Visibility::Visible,

            // 探索中はミニマップ表示
            (GameState::Exploring, HudElementType::Minimap) => Visibility::Visible,

            // 建設中はツールバー表示
            (GameState::Building, HudElementType::Toolbar) => Visibility::Visible,

            // デフォルトは状況判断
            _ => calculate_default_visibility(element),
        };
    }
}
```

---

## 3. メニュー/インベントリ設計

### 3.1 Minecraft風インベントリ

```
┌─────────────────────────────────────────┐
│ [装備]   [メインインベントリ 4×9]       │
│  [頭]    ┌─┬─┬─┬─┬─┬─┬─┬─┬─┐           │
│  [胴]    ├─┼─┼─┼─┼─┼─┼─┼─┼─┤           │
│  [脚]    ├─┼─┼─┼─┼─┼─┼─┼─┼─┤           │
│  [足]    ├─┼─┼─┼─┼─┼─┼─┼─┼─┤           │
│          └─┴─┴─┴─┴─┴─┴─┴─┴─┘           │
│          [ホットバー 1×9]               │
│          ┌─┬─┬─┬─┬─┬─┬─┬─┬─┐           │
│          └─┴─┴─┴─┴─┴─┴─┴─┴─┘           │
└─────────────────────────────────────────┘
```

### 3.2 ナビゲーション

```rust
// キーボード/コントローラー対応
fn handle_menu_navigation(
    input: &Input,
    mut selected: ResMut<SelectedSlot>,
    grid: &InventoryGrid,
) {
    if input.just_pressed(Key::Up) {
        selected.move_up(grid);
    }
    if input.just_pressed(Key::Down) {
        selected.move_down(grid);
    }
    // ... 左右も同様

    // Tab/Shift+Tabでセクション間移動
    if input.just_pressed(Key::Tab) {
        selected.next_section(grid);
    }
}
```

---

## 4. ツールチップ

### 4.1 ツールチップ設計

```rust
struct Tooltip {
    title: String,
    description: String,
    stats: Vec<StatLine>,
    hint: Option<String>,
}

// 表示遅延とフェード
const TOOLTIP_DELAY: Duration = Duration::from_millis(500);
const TOOLTIP_FADE_IN: Duration = Duration::from_millis(150);

fn show_tooltip(
    hovered: &HoveredElement,
    mut tooltip: ResMut<TooltipState>,
    time: Res<Time>,
) {
    if hovered.duration > TOOLTIP_DELAY {
        tooltip.opacity = (tooltip.opacity + time.delta_seconds() / TOOLTIP_FADE_IN.as_secs_f32())
            .min(1.0);
        tooltip.visible = true;
    }
}
```

### 4.2 コンテキスト認識

```rust
// 状況に応じた情報表示
fn generate_tooltip(item: &Item, context: &PlayerContext) -> Tooltip {
    let mut tooltip = Tooltip::new(&item.name, &item.description);

    // 現在の装備と比較
    if let Some(equipped) = context.get_equipped(item.slot) {
        tooltip.add_comparison(item, equipped);
    }

    // 使用可能かどうか
    if !item.meets_requirements(context) {
        tooltip.add_warning("要件を満たしていません");
    }

    tooltip
}
```

---

## 5. フィードバック設計

### 5.1 視覚フィードバック

```rust
// ボタン状態
enum ButtonState {
    Normal,
    Hovered,    // 明るく
    Pressed,    // 暗く、縮小
    Disabled,   // 灰色、透明
}

fn apply_button_style(state: ButtonState) -> Style {
    match state {
        ButtonState::Normal => Style {
            brightness: 1.0,
            scale: 1.0,
            ..default()
        },
        ButtonState::Hovered => Style {
            brightness: 1.2,
            scale: 1.02,
            ..default()
        },
        ButtonState::Pressed => Style {
            brightness: 0.8,
            scale: 0.98,
            ..default()
        },
        ButtonState::Disabled => Style {
            brightness: 0.5,
            opacity: 0.6,
            ..default()
        },
    }
}
```

### 5.2 アニメーション

```rust
// UIアニメーション原則
struct UiAnimation {
    duration: Duration,
    easing: EasingFunction,
}

// 推奨値
const BUTTON_HOVER: UiAnimation = UiAnimation {
    duration: Duration::from_millis(100),
    easing: EasingFunction::EaseOut,
};

const MENU_OPEN: UiAnimation = UiAnimation {
    duration: Duration::from_millis(200),
    easing: EasingFunction::EaseOutCubic,
};

const NOTIFICATION_APPEAR: UiAnimation = UiAnimation {
    duration: Duration::from_millis(300),
    easing: EasingFunction::EaseOutBack,
};
```

---

## 6. 工場ゲーム特有のUI

### 6.1 情報密度の管理

```
問題: 工場ゲームは大量の情報を表示する必要がある

解決策:
  1. ズームレベルで情報量を調整
  2. ホバーで詳細を表示
  3. 色で状態を即座に把握
```

```rust
// ズームレベル別の表示
fn get_machine_display(machine: &Machine, zoom: f32) -> MachineDisplay {
    if zoom < 0.5 {
        // 遠距離: 色付きアイコンのみ
        MachineDisplay::Icon { color: machine.status_color() }
    } else if zoom < 1.5 {
        // 中距離: 基本情報
        MachineDisplay::Basic {
            icon: machine.icon(),
            status: machine.status_icon(),
        }
    } else {
        // 近距離: 詳細情報
        MachineDisplay::Detailed {
            name: machine.name.clone(),
            progress: machine.progress,
            input_items: machine.inputs.clone(),
            output_items: machine.outputs.clone(),
        }
    }
}
```

### 6.2 フロー表示

```rust
// コンベアのアイテムフロー表示
struct FlowIndicator {
    items_per_minute: f32,
    direction: Vec2,
    color: Color,
}

// 色で流量を表現
fn flow_color(items_per_minute: f32, capacity: f32) -> Color {
    let ratio = items_per_minute / capacity;
    if ratio < 0.3 {
        Color::RED      // 不足
    } else if ratio < 0.8 {
        Color::YELLOW   // 警告
    } else {
        Color::GREEN    // 正常
    }
}
```

### 6.3 警告システム

```rust
struct AlertSystem {
    alerts: Vec<Alert>,
    display_limit: usize,
}

struct Alert {
    priority: AlertPriority,
    message: String,
    location: Option<Vec3>,
    timestamp: Instant,
}

enum AlertPriority {
    Critical,   // 即座に表示、音あり
    Warning,    // 表示、音なし
    Info,       // キューに追加
}

fn display_alerts(system: &AlertSystem) {
    // 優先度順にソート
    let sorted = system.alerts.iter()
        .sorted_by_key(|a| a.priority)
        .take(system.display_limit);

    for alert in sorted {
        show_alert_ui(alert);
    }
}
```

---

## 7. アクセシビリティ

### 7.1 テキスト

| 項目 | 推奨値 |
|------|--------|
| 最小フォントサイズ | 18px @ 1080p |
| コントラスト比 | 4.5:1 以上 |
| 行間 | 1.5em |
| スケール範囲 | 100%-200% |

### 7.2 色覚サポート

```rust
struct ColorScheme {
    success: Color,
    warning: Color,
    error: Color,
}

fn get_color_scheme(mode: ColorblindMode) -> ColorScheme {
    match mode {
        ColorblindMode::Normal => ColorScheme {
            success: Color::GREEN,
            warning: Color::YELLOW,
            error: Color::RED,
        },
        ColorblindMode::Deuteranopia => ColorScheme {
            success: Color::BLUE,
            warning: Color::ORANGE,
            error: Color::MAGENTA,
        },
        // ... 他のモード
    }
}
```

---

## 8. チェックリスト

### HUD
- [ ] 重要情報は常に視認可能か
- [ ] 中央は空いているか
- [ ] 情報過多になっていないか

### メニュー
- [ ] キーボードナビゲーション対応か
- [ ] 戻る操作が一貫しているか
- [ ] ツールチップがあるか

### フィードバック
- [ ] 全操作に視覚フィードバックがあるか
- [ ] アニメーションは適切な速度か
- [ ] 状態変化が明確か

### アクセシビリティ
- [ ] テキストサイズは調整可能か
- [ ] コントラストは十分か
- [ ] 色覚モードがあるか

---

## 参考文献

- [7 obvious beginner mistakes with your game's HUD - UX Bootcamp](https://bootcamp.uxdesign.cc/7-obvious-beginner-mistakes-with-your-games-hud-from-a-ui-ux-art-director-d852e255184a)
- [UX and UI in game design - Medium](https://medium.com/@brdelfino.work/ux-and-ui-in-game-design-exploring-hud-inventory-and-menus-5d8c189deb65)
- [Types of UI in Gaming - Medium](https://medium.com/@lorenzoardeni/types-of-ui-in-gaming-diegetic-non-diegetic-spatial-and-meta-5024ce6362d0)
- [Game UI Database](https://www.gameuidatabase.com/)

---

*このレポートはゲームUI/UXのベストプラクティス調査に基づいています。*
