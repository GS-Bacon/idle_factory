# Idle Factory - 最新コードレビュー依頼

## 正確な現状（2026-01-01時点）

### プロジェクト統計
- 総コード行数: 約12,800行（51ファイル）
- テスト: 103件通過
- Clippy警告: 0
- unwrap(): 10箇所のみ

### main.rs の実態
- **本体**: 219行（fn main + インポート）
- **テスト**: 288行（#[cfg(test)]以降）
- **合計**: 507行

### 既存Plugin（作成済み）
1. MachineSystemsPlugin - 機械系システム
2. UIPlugin - UI系システム
3. SavePlugin - セーブ/ロード

### ファイルサイズ Top 10
```
826行 src/systems/command_ui.rs      ← 最大
649行 src/systems/inventory_ui.rs
627行 src/world/mod.rs
611行 src/save.rs
557行 src/systems/block_operations/placement.rs
524行 src/systems/conveyor.rs
518行 src/systems/save_systems.rs
515行 src/meshes.rs
465行 src/setup/ui/mod.rs
```

## 依頼内容

1. **command_ui.rs（826行）の分割案**を提案してください
2. **本当に必要なリファクタリング**を優先度付きで提案してください
3. main.rsは219行なので、追加Plugin化は本当に必要か判断してください

具体的なコード例があると助かります。

## command_ui.rs の構造
```rust
//! Command input UI systems

use crate::components::*;
use crate::player::Inventory;
use crate::systems::inventory_ui::set_ui_open_state;
use crate::world::WorldData;
use crate::BlockType;
use bevy::prelude::*;
use bevy::window::CursorGrabMode;
use tracing::info;

/// E2E test command events
#[derive(Event)]
pub struct TeleportEvent {
    pub position: Vec3,
}

#[derive(Event)]
pub struct LookEvent {
    pub pitch: f32,
    pub yaw: f32,
}

#[derive(Event)]
pub struct SetBlockEvent {
    pub position: IVec3,
    pub block_type: BlockType,
}

/// Debug conveyor event (for /debug_conveyor command)
#[derive(Event)]
pub struct DebugConveyorEvent;

/// Toggle command input with T or / key
#[allow(clippy::too_many_arguments)]
pub fn command_input_toggle(
    key_input: Res<ButtonInput<KeyCode>>,
    mut command_state: ResMut<CommandInputState>,
    mut ui_query: Query<&mut Visibility, With<CommandInputUI>>,
    mut text_query: Query<&mut Text, With<CommandInputText>>,
    mut windows: Query<&mut Window>,
    interacting_furnace: Res<InteractingFurnace>,
    interacting_crusher: Res<InteractingCrusher>,
    creative_inv_open: Res<InventoryOpen>,
) {
    // Don't open if other UI is open
    if interacting_furnace.0.is_some() || interacting_crusher.0.is_some() || creative_inv_open.0 {
        return;
    }

    // T or / to open command input (when not already open)
    if !command_state.open
        && (key_input.just_pressed(KeyCode::KeyT) || key_input.just_pressed(KeyCode::Slash))
    {
        command_state.open = true;
        command_state.text.clear();
        command_state.skip_input_frame = true;  // Skip input this frame

        // Start with / if opened with slash key
        if key_input.just_pressed(KeyCode::Slash) {
            command_state.text.push('/');
        }

        // Show UI
        for mut vis in ui_query.iter_mut() {
            *vis = Visibility::Visible;
        }

        // Reset text display
        for mut text in text_query.iter_mut() {
            text.0 = format!("> {}|", command_state.text);
        }

        // Unlock cursor
        if let Ok(mut window) = windows.get_single_mut() {
            window.cursor_options.grab_mode = CursorGrabMode::None;
            window.cursor_options.visible = true;
            set_ui_open_state(true);
        }
    }
}

/// Event for spawning machines
pub use crate::events::SpawnMachineEvent;

/// Handle command input text entry
#[allow(clippy::too_many_arguments)]
pub fn command_input_handler(
    key_input: Res<ButtonInput<KeyCode>>,
    mut command_state: ResMut<CommandInputState>,
    mut ui_query: Query<&mut Visibility, With<CommandInputUI>>,
    mut text_query: Query<&mut Text, With<CommandInputText>>,
    mut windows: Query<&mut Window>,
    mut creative_mode: ResMut<CreativeMode>,
    mut inventory: ResMut<Inventory>,
    mut save_events: EventWriter<SaveGameEvent>,
    mut load_events: EventWriter<LoadGameEvent>,
    mut tp_events: EventWriter<TeleportEvent>,
    mut look_events: EventWriter<LookEvent>,
    mut setblock_events: EventWriter<SetBlockEvent>,
    mut spawn_machine_events: EventWriter<SpawnMachineEvent>,
    mut debug_conveyor_events: EventWriter<DebugConveyorEvent>,
) {
    if !command_state.open {
        return;
    }

    // ESC to close without executing
    if key_input.just_pressed(KeyCode::Escape) {
        command_state.open = false;
        command_state.text.clear();

        for mut vis in ui_query.iter_mut() {
            *vis = Visibility::Hidden;
        }

        // Lock cursor
        if let Ok(mut window) = windows.get_single_mut() {
            window.cursor_options.grab_mode = CursorGrabMode::Locked;
            window.cursor_options.visible = false;
            set_ui_open_state(false);
        }
        return;
    }

    // Enter to execute command
    if key_input.just_pressed(KeyCode::Enter) {
        let command = command_state.text.clone();
        println!(">>> ENTER pressed, command: '{}'", command);
        command_state.open = false;
        command_state.text.clear();

        for mut vis in ui_query.iter_mut() {
            *vis = Visibility::Hidden;
        }

        // Lock cursor
        if let Ok(mut window) = windows.get_single_mut() {
            window.cursor_options.grab_mode = CursorGrabMode::Locked;
            window.cursor_options.visible = false;
            set_ui_open_state(false);
        }

        // Execute command
        execute_command(
            &command,
            &mut creative_mode,
            &mut inventory,
            &mut save_events,
            &mut load_events,
            &mut tp_events,
            &mut look_events,
            &mut setblock_events,
            &mut spawn_machine_events,
            &mut debug_conveyor_events,
        );
        return;
    }

    // Backspace to delete character
    if key_input.just_pressed(KeyCode::Backspace) {
        command_state.text.pop();
    }

    // Handle character input (skip if just opened to avoid T/slash being added)
    if command_state.skip_input_frame {
        command_state.skip_input_frame = false;
    } else {
        for key in key_input.get_just_pressed() {
            if let Some(c) = keycode_to_char(*key, key_input.pressed(KeyCode::ShiftLeft) || key_input.pressed(KeyCode::ShiftRight)) {
                command_state.text.push(c);
            }
        }
    }

    // Update display text
    for mut text in text_query.iter_mut() {
        text.0 = format!("> {}|", command_state.text);
    }
}

/// Convert key code to character
fn keycode_to_char(key_code: KeyCode, shift: bool) -> Option<char> {
    match key_code {
        KeyCode::KeyA => Some(if shift { 'A' } else { 'a' }),
        KeyCode::KeyB => Some(if shift { 'B' } else { 'b' }),
        KeyCode::KeyC => Some(if shift { 'C' } else { 'c' }),
        KeyCode::KeyD => Some(if shift { 'D' } else { 'd' }),
        KeyCode::KeyE => Some(if shift { 'E' } else { 'e' }),
        KeyCode::KeyF => Some(if shift { 'F' } else { 'f' }),
        KeyCode::KeyG => Some(if shift { 'G' } else { 'g' }),
        KeyCode::KeyH => Some(if shift { 'H' } else { 'h' }),
        KeyCode::KeyI => Some(if shift { 'I' } else { 'i' }),
        KeyCode::KeyJ => Some(if shift { 'J' } else { 'j' }),
        KeyCode::KeyK => Some(if shift { 'K' } else { 'k' }),
        KeyCode::KeyL => Some(if shift { 'L' } else { 'l' }),
        KeyCode::KeyM => Some(if shift { 'M' } else { 'm' }),
        KeyCode::KeyN => Some(if shift { 'N' } else { 'n' }),
        KeyCode::KeyO => Some(if shift { 'O' } else { 'o' }),
        KeyCode::KeyP => Some(if shift { 'P' } else { 'p' }),
// ... (残り626行)
```

## command_ui.rs の関数一覧
36:pub fn command_input_toggle(
88:pub fn command_input_handler(
183:fn keycode_to_char(key_code: KeyCode, shift: bool) -> Option<char> {
231:fn execute_command(
527:pub fn handle_teleport_event(
546:pub fn handle_look_event(
573:pub fn handle_setblock_event(
584:fn parse_item_name(name: &str) -> Option<BlockType> {
608:pub fn handle_spawn_machine_event(
796:pub fn handle_debug_conveyor_event(
