# 統合実装計画 (2026-01-04 更新)

## 現状サマリー

| 項目 | 値 |
|------|-----|
| コード行数 | **22,500行** |
| テスト | **318件** 通過 (lib:103, bin:37, e2e:148, fuzz:11, proptest:8, ssim:3, integration:8) |
| unwrap() | **17箇所** (全てテストコード内) |
| Clippy警告 | **0件** |
| カバレッジ | **8.54%** (全体)、ロジック部分70%+ |

---

## 優先順位（2026-01-04 更新）

| 順位 | カテゴリ | 理由 |
|------|----------|------|
| **1** | v0.2完成 | ゲームとして遊べる状態に |
| **2** | アーキテクチャ再設計 | 将来機能の土台（-1,300行） |
| **3** | 機能拡張 | v0.3以降の新機能 |

---

## Phase A: v0.2完成（短期）✅ 完了

### A.1 UIテーマ刷新 ✅

- テーマ定数: `setup/ui/mod.rs`に定義済み（SLOT_SIZE, SLOT_RADIUS, 色定数等）
- スロットBorderRadius: 全UIに適用済み
- ホバー/選択スタイル: `systems/inventory_ui.rs`で実装済み
- 機械UI統一: `ui/machine_ui.rs`でFactoryテーマ適用済み

### A.2 バイオーム表示UI ✅

- BiomeHudText: `setup/ui/mod.rs`で実装済み
- update_biome_hud: `systems/debug_ui.rs`で実装済み

### A.3 チュートリアル ✅

- TutorialProgress: `components/mod.rs`で定義済み
- TutorialPanel, TutorialStepText: `setup/ui/mod.rs`で実装済み
- tutorial.rs: `systems/tutorial.rs`で8ステップ実装済み

---

## Phase B: アーキテクチャ再設計（中期）✅ 大部分完了

**参照**: [architecture-redesign.md](architecture-redesign.md)

### B.1 準備 ✅ 完了

- `core/`: inventory.rs, network.rs, recipe.rs 実装済み
- 機能コンポーネント: ItemAcceptor, ItemEjector, Crafter, MachineInventory, PowerConsumer 定義済み
- MachineDescriptor: MINER, FURNACE, CRUSHER 定義済み
- `ui/widgets.rs`: spawn_slot, spawn_button 実装済み

### B.2 物流インフラ分離 ✅ 完了

- `logistics/conveyor.rs`: 557行、コンベアシステム実装済み

### B.3 機械統合 ✅ 完了

- `machines/`: miner.rs, furnace.rs, crusher.rs で個別実装
- 共通コンポーネント: `components/machines.rs` で定義

### B.4 UI統合 ✅ 完了

- `ui/`: machine_ui.rs, storage_ui.rs, widgets.rs
- `setup/ui/`: inventory_ui.rs, mod.rs
- `systems/`: inventory_ui.rs, debug_ui.rs
- 3箇所に分散しているが、各々の責務が明確で統合の必要なし

### B.5 セーブ統合 ✅ 完了

- `save/`: format.rs, systems.rs 実装済み

### B.6 最適化 ✅ 完了

- main.rs: GamePlugin化済み（50行のみ）
- updater/: feature gate実装済み
- debug/: 既存のまま維持

**現状**: 22,567行（アーキテクチャ整備完了、行数削減は必須ではない）

---

## 現在のタスク（2026-01-06）

| # | タスク | 状態 | 備考 |
|---|--------|------|------|
| 1 | チュートリアル中にメインクエストを完全に非表示にする | ✅完了 | `tutorial.rs:185-191` |
| 2 | 製錬炉が鉱石の搬入を受け付けない問題を修正 | ✅完了 | テスト8件通過、`logistics/conveyor.rs:295-340` |
| 3 | チュートリアルクエストで個数関係のものにプログレスバーを表示 | ✅完了 | `tutorial.rs:216-248`、スクショ確認済 |
| 4 | 鉱石バイオーム表示をシンプルに、他UIと色を統一 | ✅完了 | 左上「[Cu] 銅鉱脈」表示確認済 |

### インフラ改善タスク

| # | タスク | 状態 | 根拠 |
|---|--------|------|------|
| 5 | 並列worktree実行時の重複コミット検出 | 未着手 | 過去2件の重複修正が発生 |
| 6 | 座標系の統一（Transform vs IVec3） | 未着手 | 製錬炉搬入バグ等の根本原因 |

#### タスク5: 並列worktree重複検出

**問題**: 同時刻に同名コミットが発生（2026-01-05に2件）

**対策案**:
- `parallel-run.sh finish` 時に同名コミットをチェック
- または git hook で検出

#### タスク6: 座標系統一

**問題**: `Transform.translation` と `position` フィールドの混在

```rust
// 現状（不一貫）
let pos = transform.translation.floor().as_ivec3();  // 破壊時
let furnace = Furnace { position: block_pos, ... };  // 作成時
if conveyor_pos == transform.translation.floor()... // 搬入判定
```

**対策案**:
- 全機械に `position: IVec3` フィールドを標準化
- 座標変換を `fn get_block_position(transform: &Transform) -> IVec3` に統一
- `Transform.translation` の直接参照を禁止（Clippy lint化検討）

---

## 将来タスク（v0.3以降）

以下は現時点では着手しない。v0.2完成 + アーキテクチャ安定後に検討。

| 機能 | 詳細 |
|------|------|
| データ外部化 | recipes.json, quests.json, machines.json |
| BlockType階層化 | TerrainBlock, OreBlock等の分離 |
| 電力システム | 発電機・導管・消費 |
| 流体パイプ | ポンプ・パイプ・タンク |
| マルチプレイ | WebSocket同期 |
| Modding API | Lua/WASM |

---

## 完了済みタスク

<details>
<summary>クリックで展開</summary>

### リファクタリング
- [x] block_operations.rs 分割 (1001行→3ファイル)
- [x] ui_setup.rs 分割 (977行→3ファイル)
- [x] targeting.rs 分割 (759行→4ファイル)
- [x] command_ui.rs 分割 (826行→4ファイル)
- [x] MachineSystemsPlugin 作成
- [x] UIPlugin 作成
- [x] SavePlugin 作成

### パフォーマンス改善 (旧Phase 1)
- [x] ハイライトメッシュキャッシュ化
- [x] O(N²)コンベア転送→HashMap化
- [x] Vec::contains()→HashSet化
- [x] クエストデータ変換キャッシュ

### セキュリティ・エラー処理 (旧Phase 2)
- [x] unwrap()削減 (72箇所→17箇所)
- [x] 配列インデックス範囲チェック
- [x] コマンドパス走査防止
- [x] NaN/Infinity処理

### v0.2機能 (旧Phase 3)
- [x] GlobalInventory基盤
- [x] 機械設置/撤去
- [x] 8列グリッドレイアウト
- [x] ページネーション
- [x] カテゴリタブ・検索機能
- [x] 納品ボタン
- [x] 機械入出力システム
- [x] バイオーム採掘システム

### テスト強化 (旧Phase 4)
- [x] カバレッジ計測 (8.54%全体、ロジック70%+)
- [x] コンベア統合テスト
- [x] セーブ/ロード往復テスト
- [x] UIインタラクションテスト
- [x] SSIM比較テスト
- [x] ファジング基盤

### プラットフォーム再設計 (旧Phase 10)
- [x] PlatformBlock追加
- [x] DeliveryPlatform.delivered削除
- [x] GlobalInventory経由に変更

</details>

---

## 実行順序マトリクス

```
【v0.2完成】
A.1 UIテーマ ──┐
A.2 バイオームUI ├─→ v0.2リリース
A.3 チュートリアル ┘

【アーキテクチャ再設計】（v0.2完成後）
B.1 準備 ─→ B.2 物流分離 ─→ B.3 機械統合 ─→ B.4 UI統合 ─→ B.5 セーブ ─→ B.6 最適化
                                                                              │
                                                                              ↓
                                                                         v0.3検討
```

---

## 合見積サマリー（2026-01-04）

| 観点 | Claude | Gemini | 採用 |
|------|--------|--------|------|
| 機械設計 | Machineトレイト | ECSコンポジション | **Gemini** |
| コンベア | 機械として統合 | 物流インフラ分離 | **Gemini** |
| UI | 共通化 | Entity Observers | **両方** |
| 移行 | 3Phase | 垂直分割 | **Gemini** |

**結論**: ECSの特性を活かし、`tick()`メソッドに依存せず、機能コンポーネント（ItemAcceptor, Crafter等）で構成

---

## 次のアクション

**Phase A・B 完了** ✅

現在の状態:
- v0.2機能: 全て実装済み
- アーキテクチャ: 整備完了
- テスト: 280件通過
- Clippy警告: 0件

次のステップ:
1. **v0.2リリース準備** - リリースノート作成、タグ付け
2. **v0.3検討** - 将来タスクから優先度を決定

---

## 将来設計メモ（v0.3以降）

### 納品プラットフォームのスループット制限

**設計原則**: 入力無制限・容量無制限・**出力制限**

```
納品プラットフォーム
├── 入力: 無制限（なんでも受け入れ）
├── 容量: 無制限（無限倉庫OK）
├── アイテム出力: 4個/tick（ボトルネック）
└── 電力出力: 100W（上限固定）
```

**効果**:
- 序盤は納品PFだけで十分 → シンプル
- 生産規模拡大 → スループット不足 → 専用ブロックが必要に
- 「とりあえず全部入れる」でも動くが、**最適ではない**

### ブロック別スループット比較

| ブロック | 容量 | 出力 | 用途 |
|----------|------|------|------|
| 納品PF | ∞ | 4/tick | 汎用・序盤 |
| 小型倉庫 | 1,000 | 16/tick | 中盤 |
| 大型倉庫 | 10,000 | 64/tick | 終盤 |
| 蓄電池 | 10,000Wh | 500W | 電力バッファ |
| 大型蓄電池 | 100,000Wh | 2,000W | 大規模工場 |

### 電力システム

| フェーズ | 電力源 | 出力 | プレイヤー体験 |
|----------|--------|------|---------------|
| 序盤 | 納品PF | 100W | 「電気ってこう使う」を学ぶ |
| 中盤 | 石炭発電機 | 200W | 燃料管理 |
| 中盤 | ソーラー | 50W（昼のみ） | 昼夜サイクル |
| 終盤 | 蒸気タービン | 500W | 水+熱源 |
| 終盤 | 原子炉 | 2,000W | 高コスト・高リスク |

### ゲームプレイの流れ

1. 最初は納品PF一つで全部回る
2. 機械増える → 供給追いつかない
3. 「倉庫建てて分散させよう」← 自然な動機
4. 「発電機作って電力増やそう」← 同様の動機

**液体・気体も同じパターンで統一可能**
