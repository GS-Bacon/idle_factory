# Idle Factory - 現在の状態

## プロジェクト統計 (2026-01-02)

| 項目 | 値 |
|------|-----|
| 総コード行数 | 10,957行 |
| ファイル数 | 57 |
| テスト件数 | 113件 通過 |
| Clippy警告 | 0件 |
| unwrap() | 17箇所 |

## 最大ファイル (Top 10)

```
1053 src/game_spec.rs
 650 src/systems/inventory_ui.rs
 632 src/world/mod.rs
 622 src/save.rs
 599 src/systems/block_operations/placement.rs
 545 src/main.rs
 524 src/systems/save_systems.rs
 524 src/systems/conveyor.rs
 515 src/meshes.rs
 493 src/setup/ui/mod.rs
```

## タスク管理

**Single Source of Truth**: `.claude/implementation-plan.md`

削除済みファイル（重複していた）:
- `.claude/refactoring-tasks.md`
- `.ai_context/refactoring_plan.md`
- `.ai_context/execution_plan.md`

## 直近のgitコミット

```
c99f9e0 docs: 実装計画を追加
db751e8 feat: 自動バグ検出強化
9a89e79 refactor: Geminiレビュー改善タスク完了
17829b8 style: GlobalInventoryパネルを工業風ダークテーマに改善
d058300 feat: v0.2 全体在庫システム実装
```
