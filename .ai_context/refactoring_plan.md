# リファクタリング計画書

**作成日**: 2026-01-01
**作成者**: Claude + Gemini 連携

---

## Gemini の提案サマリー

### 理想的なmain.rs構造
```rust
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(GameCorePlugin)   // 共通設定、ログ
        .add_plugins(PlayerPlugin)     // 移動、視点、インベントリ
        .add_plugins(WorldPlugin)      // チャンク生成、メッシュ化
        .add_plugins(InteractionPlugin)// ブロック破壊・設置、レイキャスト
        .add_plugins(MachinePlugin)    // ベルトコンベア、マシン ← 既存
        .add_plugins(UiPlugin)         // ← 既存
        .run();
}
```

### SystemSet導入
```rust
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum GameSystemSet {
    Input,
    Simulation,
    Rendering,
}

app.configure_sets(Update, (
    GameSystemSet::Input,
    GameSystemSet::Simulation,
    GameSystemSet::Rendering
).chain());
```

---

## 実行計画

### Phase 1: PlayerPlugin 作成

**移動するシステム**:
- `toggle_cursor_lock` (Input)
- `player_look` (Input)
- `player_move` (Input)
- `tick_action_timers` (Simulation)
- `tutorial_dismiss` (Input)

**移動するリソース**:
- `CursorLockState`
- `ContinuousActionTimer`

**ファイル**: `src/plugins/player.rs`

---

### Phase 2: WorldPlugin 作成

**移動するシステム**:
- `spawn_chunk_tasks` (Simulation)
- `receive_chunk_meshes` (Simulation)
- `unload_distant_chunks` (Simulation)

**移動するリソース**:
- `WorldData`
- `ChunkMeshTasks`

**ファイル**: `src/plugins/world.rs`

---

### Phase 3: InteractionPlugin 作成

**移動するシステム**:
- `update_target_block` (Simulation)
- `block_break` (Simulation, after target_block)
- `block_place` (Simulation, after target_block)
- `select_block_type` (Input)
- `update_target_highlight` (Rendering, after target_block)
- `rotate_conveyor_placement` (Input)
- `update_guide_markers` (Rendering)

**ファイル**: `src/plugins/interaction.rs`

---

### Phase 4: QuestPlugin 作成

**移動するシステム**:
- `update_conveyor_shapes` (Simulation)
- `quest_progress_check` (Simulation)
- `quest_claim_rewards` (Simulation)
- `update_delivery_ui` (Rendering)
- `update_quest_ui` (Rendering)
- `setup_delivery_platform` (Startup)
- `load_machine_models` (Startup)

**移動するリソース**:
- `CurrentQuest`

**ファイル**: `src/plugins/quest.rs`

---

### Phase 5: SystemSet 導入

**定義場所**: `src/plugins/mod.rs`

**各Pluginでの適用**:
```rust
// PlayerPlugin
app.add_systems(Update, (
    toggle_cursor_lock,
    player_look,
    player_move,
).in_set(GameSystemSet::Input));

// InteractionPlugin
app.add_systems(Update, (
    block_break,
    block_place,
).in_set(GameSystemSet::Simulation).after(update_target_block));
```

---

## 変更後のmain.rs (目標: 約100行)

```rust
fn main() {
    let mut app = App::new();

    // Platform setup (WASM/Native)
    #[cfg(not(target_arch = "wasm32"))]
    { /* Native plugins */ }
    #[cfg(target_arch = "wasm32")]
    { /* WASM plugins */ }

    app
        .add_plugins(GameEventsPlugin)
        .add_plugins(PlayerPlugin)
        .add_plugins(WorldPlugin)
        .add_plugins(InteractionPlugin)
        .add_plugins(QuestPlugin)
        .add_plugins(MachineSystemsPlugin)
        .add_plugins(UIPlugin)
        .add_plugins(SavePlugin)
        .init_resource::<Inventory>()
        .init_resource::<CreativeMode>()
        .init_resource::<GameFont>()
        .init_resource::<E2EExportConfig>()
        .add_systems(Update, (
            export_e2e_state,
            handle_teleport_event,
            handle_look_event,
            handle_setblock_event,
            handle_spawn_machine_event,
            handle_debug_conveyor_event,
        ))
        .run();
}
```

---

## 優先順位

| 順位 | タスク | 効果 | 工数 |
|------|--------|------|------|
| 1 | PlayerPlugin | main.rs 20行削減 | 15分 |
| 2 | WorldPlugin | main.rs 15行削減 | 10分 |
| 3 | InteractionPlugin | main.rs 25行削減 | 20分 |
| 4 | QuestPlugin | main.rs 20行削減 | 15分 |
| 5 | SystemSet導入 | 順序制御の明確化 | 30分 |

**合計工数**: 約1.5時間
**期待効果**: main.rs 507行 → 約100行

---

## 注意事項

1. **テスト**: 各Phase後に `cargo test && cargo clippy` 実行
2. **順序依存**: `update_target_block` は `block_break/block_place` の前に実行必須
3. **リソース初期化**: 各PluginでリソースをStates付きで初期化

---

## 承認待ち

この計画でよければ「実行」と言ってください。
修正が必要な場合はフィードバックをお願いします。
