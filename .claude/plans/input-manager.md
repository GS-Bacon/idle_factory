# InputManager導入計画

## 概要

**目的**: 入力処理を一元化し、テスト用仮想入力とキーバインド設定を可能にする

**背景**:
- キー入力を読む箇所が24箇所に分散
- テスト用仮想入力を注入するには全箇所に `|| virtual_input` を追加する必要がある
- キーバインド変更も困難

## 現状分析

### 入力読み取り箇所: 24箇所

| ファイル | キー | 用途 |
|----------|------|------|
| player.rs | WASD, Space, Shift, Arrow | 移動、ジャンプ、カメラ |
| ui_navigation.rs | E, ESC, Tab | UI開閉 |
| hotbar.rs | 1-9 | ホットバー選択 |
| quest.rs | Q | クエストUI |
| command/ui.rs | T, /, Enter, Tab, ESC, Backspace | コマンド入力 |
| generic.rs (machines) | E, ESC | 機械UI |
| storage_ui.rs | Backspace, 1-9, Space | ストレージUI |
| inventory_ui.rs | Shift | 一括移動 |
| targeting/conveyor.rs | R | コンベア回転 |
| debug_ui.rs | F3 | デバッグ表示 |
| breaking.rs | MouseLeft | ブロック破壊 |
| placement.rs | MouseRight | ブロック設置 |

### 既存の抽象化

- `InputState` enum: 状態管理（Gameplay/Inventory/MachineUI/Command/Paused）
- `InputStateResourcesWithCursor`: SystemParam で状態取得
- `allows_movement()`, `allows_block_actions()` など: 状態別の許可判定

## 設計

### 1. GameAction enum（セマンティックアクション）

```rust
/// ゲーム操作のセマンティックアクション
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GameAction {
    // === 移動 ===
    MoveForward,
    MoveBackward,
    MoveLeft,
    MoveRight,
    Jump,
    Descend,        // クリエイティブモード下降

    // === カメラ ===
    LookUp,
    LookDown,
    LookLeft,
    LookRight,

    // === UI ===
    ToggleInventory,    // E
    TogglePause,        // ESC
    ToggleGlobalInventory, // Tab
    ToggleQuest,        // Q
    OpenCommand,        // T, /
    CloseUI,            // ESC (UI内)
    Confirm,            // Enter
    Cancel,             // ESC

    // === ホットバー ===
    Hotbar1, Hotbar2, Hotbar3, Hotbar4, Hotbar5,
    Hotbar6, Hotbar7, Hotbar8, Hotbar9,

    // === ブロック操作 ===
    PrimaryAction,      // 左クリック（破壊）
    SecondaryAction,    // 右クリック（設置/操作）
    RotateBlock,        // R

    // === 修飾キー ===
    ModifierShift,      // Shift（一括操作）

    // === デバッグ ===
    ToggleDebug,        // F3
}
```

### 2. InputManager リソース

```rust
#[derive(Resource)]
pub struct InputManager {
    /// キーバインド設定: Action → KeyCode/MouseButton のマッピング
    bindings: HashMap<GameAction, Vec<InputBinding>>,

    /// 今フレームで押されたアクション
    just_pressed: HashSet<GameAction>,

    /// 現在押されているアクション
    pressed: HashSet<GameAction>,

    /// 今フレームで離されたアクション
    just_released: HashSet<GameAction>,

    /// 仮想入力（テスト用）
    virtual_just_pressed: HashSet<GameAction>,
    virtual_pressed: HashSet<GameAction>,
}

pub enum InputBinding {
    Key(KeyCode),
    Mouse(MouseButton),
}

impl InputManager {
    pub fn just_pressed(&self, action: GameAction) -> bool {
        self.just_pressed.contains(&action)
            || self.virtual_just_pressed.contains(&action)
    }

    pub fn pressed(&self, action: GameAction) -> bool {
        self.pressed.contains(&action)
            || self.virtual_pressed.contains(&action)
    }

    /// テスト用: 仮想入力を注入
    pub fn inject_press(&mut self, action: GameAction) {
        self.virtual_just_pressed.insert(action);
        self.virtual_pressed.insert(action);
    }

    /// テスト用: 仮想入力をクリア
    pub fn clear_virtual(&mut self) {
        self.virtual_just_pressed.clear();
        self.virtual_pressed.clear();
    }
}
```

### 3. 更新システム

```rust
/// PreUpdate で物理入力を読み取り、InputManager を更新
fn update_input_manager(
    mut input_manager: ResMut<InputManager>,
    key_input: Res<ButtonInput<KeyCode>>,
    mouse_input: Res<ButtonInput<MouseButton>>,
) {
    input_manager.just_pressed.clear();
    input_manager.just_released.clear();
    input_manager.pressed.clear();

    for (action, bindings) in &input_manager.bindings {
        for binding in bindings {
            match binding {
                InputBinding::Key(key) => {
                    if key_input.just_pressed(*key) {
                        input_manager.just_pressed.insert(*action);
                    }
                    if key_input.pressed(*key) {
                        input_manager.pressed.insert(*action);
                    }
                    if key_input.just_released(*key) {
                        input_manager.just_released.insert(*action);
                    }
                }
                InputBinding::Mouse(button) => {
                    // 同様
                }
            }
        }
    }
}

/// フレーム終了時に仮想入力をクリア
fn clear_virtual_input(mut input_manager: ResMut<InputManager>) {
    input_manager.virtual_just_pressed.clear();
}
```

### 4. デフォルトキーバインド

```rust
impl Default for InputManager {
    fn default() -> Self {
        let mut bindings = HashMap::new();

        // 移動
        bindings.insert(GameAction::MoveForward, vec![InputBinding::Key(KeyCode::KeyW)]);
        bindings.insert(GameAction::MoveBackward, vec![InputBinding::Key(KeyCode::KeyS)]);
        bindings.insert(GameAction::MoveLeft, vec![InputBinding::Key(KeyCode::KeyA)]);
        bindings.insert(GameAction::MoveRight, vec![InputBinding::Key(KeyCode::KeyD)]);
        bindings.insert(GameAction::Jump, vec![InputBinding::Key(KeyCode::Space)]);

        // UI
        bindings.insert(GameAction::ToggleInventory, vec![InputBinding::Key(KeyCode::KeyE)]);
        bindings.insert(GameAction::TogglePause, vec![InputBinding::Key(KeyCode::Escape)]);
        // ...

        Self {
            bindings,
            just_pressed: HashSet::new(),
            pressed: HashSet::new(),
            just_released: HashSet::new(),
            virtual_just_pressed: HashSet::new(),
            virtual_pressed: HashSet::new(),
        }
    }
}
```

### 5. 移行後のシステム例

```rust
// Before
pub fn player_move(
    key_input: Res<ButtonInput<KeyCode>>,
    // ...
) {
    if key_input.pressed(KeyCode::KeyW) {
        direction += forward;
    }
}

// After
pub fn player_move(
    input: Res<InputManager>,
    // ...
) {
    if input.pressed(GameAction::MoveForward) {
        direction += forward;
    }
}
```

### 6. WebSocket API追加

```rust
// test.send_input ハンドラー
"test.send_input" => {
    // params: { "action": "ToggleInventory" }
    let action: GameAction = params.action.parse()?;
    input_manager.inject_press(action);
    Ok(json!({ "success": true }))
}

// test.get_state ハンドラー
"test.get_state" => {
    Ok(json!({
        "ui_state": format!("{:?}", ui_state.current()),
        "player_position": [pos.x, pos.y, pos.z],
        "cursor_locked": cursor_state.locked,
        "pressed_actions": input_manager.pressed.iter().map(|a| format!("{:?}", a)).collect::<Vec<_>>(),
    }))
}
```

## 実装フェーズ

### Phase 1: 基盤（壊さず追加）

| タスク | ファイル | 内容 |
|--------|----------|------|
| 1.1 | `src/input/mod.rs` | GameAction, InputManager 定義 |
| 1.2 | `src/input/mod.rs` | update_input_manager システム |
| 1.3 | `src/plugins/game.rs` | InputManager を Resource 登録、システム追加 |
| 1.4 | テスト | InputManager 単体テスト |

**完了条件**: 既存コード変更なし、InputManager が動作

### Phase 2: 移行（1ファイルずつ）

| タスク | ファイル | 変更箇所 |
|--------|----------|----------|
| 2.1 | ui_navigation.rs | E, ESC, Tab → GameAction |
| 2.2 | player.rs | WASD, Space, Arrow → GameAction |
| 2.3 | hotbar.rs | 1-9 → GameAction |
| 2.4 | quest.rs | Q → GameAction |
| 2.5 | command/ui.rs | T, /, Enter, Tab, ESC → GameAction |
| 2.6 | machines/generic.rs | E, ESC → GameAction |
| 2.7 | block_operations/*.rs | Mouse → GameAction |
| 2.8 | その他 | R, F3, Shift など |

**移行ルール**:
- 1ファイルごとにテスト実行
- `ButtonInput<KeyCode>` → `InputManager` に置換
- 動作確認してから次へ

### Phase 3: API追加

| タスク | 内容 |
|--------|------|
| 3.1 | `test.send_input` ハンドラー追加 |
| 3.2 | `test.get_state` ハンドラー追加 |
| 3.3 | WebSocket API テスト |

### Phase 4: シナリオテスト

| タスク | 内容 |
|--------|------|
| 4.1 | シナリオTOML形式定義 |
| 4.2 | テストランナー (Node.js or Rust) |
| 4.3 | サンプルシナリオ作成 |
| 4.4 | CI統合 |

## リスクと対策

| リスク | 対策 |
|--------|------|
| 移行中に既存機能が壊れる | Phase 1で追加のみ、Phase 2で1ファイルずつ移行 |
| キーバインドの漏れ | 移行前に全キー入力をリスト化（上記表） |
| InputState との整合性 | InputManager は入力読み取りのみ、InputState は状態判定で役割分離 |
| パフォーマンス | HashSet は O(1)、問題なし |

## 将来の拡張

- **キーバインド設定UI**: InputManager.bindings を設定画面から変更
- **ゲームパッド対応**: InputBinding に Gamepad 追加
- **マクロ/連射**: InputManager でフィルタリング

## InputState との役割分離

```
InputManager: 「何が押されたか」を抽象化
  - 物理入力 → セマンティックアクション変換
  - 仮想入力の注入
  - キーバインド設定

InputState: 「今何ができるか」を判定
  - UI状態に基づく入力許可/禁止
  - allows_movement(), allows_block_actions() など
  - 変更なし（既存のまま）
```

## 作成日

2026-01-09
