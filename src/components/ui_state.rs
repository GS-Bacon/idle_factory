//! UI State Management
//!
//! Stack-based UI state management for proper ESC/back navigation
//! and input capture control.

use bevy::prelude::*;

/// UIのコンテキスト（どの画面が開いているか）
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UIContext {
    /// 通常のゲームプレイ（スタックが空の状態）
    Gameplay,
    /// インベントリ画面 (E key)
    Inventory,
    /// グローバルインベントリ / 倉庫 (Tab key)
    GlobalInventory,
    /// コマンド入力 (T or / key)
    CommandInput,
    /// ポーズメニュー (ESC when stack is empty)
    PauseMenu,
    /// 設定画面
    Settings,
    /// マシンUI（汎用化、Entityで特定）
    Machine(Entity),
}

/// UIの状態を管理するリソース
/// スタック構造により「戻る」動作を自然に表現
#[derive(Resource, Debug)]
pub struct UIState {
    /// 画面のスタック。末尾が現在アクティブな画面。
    /// 空の場合は Gameplay 状態とみなす。
    stack: Vec<UIContext>,
}

impl Default for UIState {
    fn default() -> Self {
        // 起動時はポーズメニューから始まる
        bevy::log::info!("[UIState] Creating default with PauseMenu stack");
        Self {
            stack: vec![UIContext::PauseMenu],
        }
    }
}

impl UIState {
    /// Create an empty UIState (for testing)
    #[cfg(test)]
    pub fn new_empty() -> Self {
        Self { stack: vec![] }
    }

    /// Get current UI context (top of stack, or Gameplay if empty)
    pub fn current(&self) -> UIContext {
        self.stack.last().cloned().unwrap_or(UIContext::Gameplay)
    }

    /// Check if in normal gameplay mode (no UI open)
    pub fn is_gameplay(&self) -> bool {
        self.stack.is_empty()
    }

    /// Check if a specific context is currently active
    pub fn is_active(&self, context: &UIContext) -> bool {
        self.current() == *context
    }

    /// Check if any UI is open (not in gameplay)
    pub fn has_ui_open(&self) -> bool {
        !self.stack.is_empty()
    }

    /// Push a new UI context onto the stack
    pub fn push(&mut self, context: UIContext) {
        // Don't push if already at top
        if self.current() != context {
            self.stack.push(context);
        }
    }

    /// Pop the current UI context (ESC/back action)
    pub fn pop(&mut self) -> Option<UIContext> {
        self.stack.pop()
    }

    /// Clear all UI and return to gameplay
    pub fn clear(&mut self) {
        self.stack.clear();
    }

    /// Replace current context (for tab switching within same UI level)
    pub fn replace(&mut self, context: UIContext) {
        if self.stack.is_empty() {
            self.stack.push(context);
        } else {
            *self.stack.last_mut().unwrap() = context;
        }
    }

    /// Check if a machine UI is open for a specific entity
    pub fn is_machine_open(&self, entity: Entity) -> bool {
        matches!(self.current(), UIContext::Machine(e) if e == entity)
    }

    /// Get the machine entity if a machine UI is open
    pub fn get_open_machine(&self) -> Option<Entity> {
        match self.current() {
            UIContext::Machine(entity) => Some(entity),
            _ => None,
        }
    }

    /// Get stack depth (for testing)
    pub fn stack_depth(&self) -> usize {
        self.stack.len()
    }

    /// Get stack as string slice (for testing)
    pub fn stack_as_strings(&self) -> Vec<String> {
        self.stack
            .iter()
            .map(|ctx| match ctx {
                UIContext::Gameplay => "Gameplay".to_string(),
                UIContext::Inventory => "Inventory".to_string(),
                UIContext::GlobalInventory => "GlobalInventory".to_string(),
                UIContext::CommandInput => "Command".to_string(),
                UIContext::PauseMenu => "PauseMenu".to_string(),
                UIContext::Settings => "Settings".to_string(),
                UIContext::Machine(_) => "MachineUI".to_string(),
            })
            .collect()
    }

    /// Check if stack contains a specific context string (for testing)
    pub fn stack_contains(&self, ctx_str: &str) -> bool {
        self.stack_as_strings().iter().any(|s| s == ctx_str)
    }
}

/// UI操作イベント
#[derive(Event, Debug, Clone)]
pub enum UIAction {
    /// 新しい画面を開く（スタックに積む）
    Push(UIContext),
    /// 現在の画面を閉じる（スタックから降ろす）
    Pop,
    /// 全てのUIを閉じてゲームに戻る
    Clear,
    /// 現在の画面を置き換える（例: インベントリから別のタブへ）
    Replace(UIContext),
    /// Toggle a context (push if not active, pop if active)
    Toggle(UIContext),
}

// === System Conditions (run_if helpers) ===

/// System runs only when in gameplay mode (no UI open)
pub fn in_gameplay(ui_state: Res<UIState>) -> bool {
    ui_state.is_gameplay()
}

/// System runs only when any UI is open
pub fn has_ui_open(ui_state: Res<UIState>) -> bool {
    ui_state.has_ui_open()
}

/// Create a condition that checks if a specific UI context is active
pub fn in_ui_context(context: UIContext) -> impl Fn(Res<UIState>) -> bool + Clone {
    move |ui_state: Res<UIState>| ui_state.is_active(&context)
}

/// System runs only when inventory is open
pub fn in_inventory(ui_state: Res<UIState>) -> bool {
    ui_state.is_active(&UIContext::Inventory)
}

/// System runs only when pause menu is open
pub fn in_pause_menu(ui_state: Res<UIState>) -> bool {
    ui_state.is_active(&UIContext::PauseMenu)
}

/// System runs only when command input is open
pub fn in_command_input(ui_state: Res<UIState>) -> bool {
    ui_state.is_active(&UIContext::CommandInput)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ui_state_default() {
        // 起動時はPauseMenuから始まる
        let state = UIState::default();
        assert!(!state.is_gameplay());
        assert_eq!(state.current(), UIContext::PauseMenu);
    }

    #[test]
    fn test_ui_state_push_pop() {
        let mut state = UIState::new_empty();

        state.push(UIContext::Inventory);
        assert!(!state.is_gameplay());
        assert_eq!(state.current(), UIContext::Inventory);

        state.push(UIContext::PauseMenu);
        assert_eq!(state.current(), UIContext::PauseMenu);

        let popped = state.pop();
        assert_eq!(popped, Some(UIContext::PauseMenu));
        assert_eq!(state.current(), UIContext::Inventory);

        state.pop();
        assert!(state.is_gameplay());
    }

    #[test]
    fn test_ui_state_clear() {
        let mut state = UIState::new_empty();
        state.push(UIContext::Inventory);
        state.push(UIContext::PauseMenu);

        state.clear();
        assert!(state.is_gameplay());
    }

    #[test]
    fn test_ui_state_replace() {
        let mut state = UIState::new_empty();
        state.push(UIContext::Inventory);
        state.replace(UIContext::GlobalInventory);

        assert_eq!(state.current(), UIContext::GlobalInventory);
        assert_eq!(state.stack.len(), 1);
    }

    #[test]
    fn test_ui_state_machine() {
        let mut state = UIState::new_empty();
        let entity = Entity::from_raw(42);

        state.push(UIContext::Machine(entity));
        assert!(state.is_machine_open(entity));
        assert_eq!(state.get_open_machine(), Some(entity));

        let other_entity = Entity::from_raw(99);
        assert!(!state.is_machine_open(other_entity));
    }

    #[test]
    fn test_ui_state_no_duplicate_push() {
        let mut state = UIState::new_empty();
        state.push(UIContext::Inventory);
        state.push(UIContext::Inventory); // Should not add duplicate

        assert_eq!(state.stack.len(), 1);
    }
}
