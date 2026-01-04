# 統合実装計画 (2026-01-02)

## 現状サマリー

| 項目 | 値 |
|------|-----|
| コード行数 | **11,500行** (目標12,500行以下 ✅) |
| テスト | **280件** 通過 (lib:74, bin:38, e2e:146, fuzz:11, proptest:8, ssim:3) |
| unwrap() | **17箇所** (全てテストコード内) |
| Clippy警告 | **0件** |
| カバレッジ | **8.54%** (全体)、ロジック部分70%+ |

---

## 完了済みタスク (参照用)

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

### バグ修正・クリーンアップ
- [x] unwrap()削減 (72箇所→17箇所)
- [x] 未使用コード削除
- [x] Clippy警告修正 (0件達成)

### E2E・テスト改善
- [x] /test コマンド (production, stress)
- [x] /assert コマンド (inventory, slot検証)
- [x] /spawn_line コマンド
- [x] /debug_conveyor コマンド
- [x] スモークテスト (scripts/smoke_test.sh)
- [x] ビジュアル回帰テスト基盤 (scripts/visual_regression.sh)
- [x] ファジング基盤 (scripts/fuzz_test.sh)
- [x] シナリオテスト (scripts/scenario_test.sh)

### ログ改善
- [x] 構造化ログ形式 (EventLogger)
- [x] チャンク生成ログのノイズ削減
- [x] ログサマリー生成 (scripts/summarize_log.sh)
- [x] 異常検出ルール (scripts/detect_anomalies.sh)

### v0.2 実装
- [x] Phase 0: 事前準備 (Direction型統一)
- [x] Phase 1: GlobalInventory基盤
- [x] Phase 2: 機械設置/撤去
- [x] Phase 3: 既存システム置き換え
- [x] Phase 7: バイオーム採掘システム

</details>

---

## 優先順位

| 順位 | カテゴリ | 理由 |
|------|----------|------|
| **1** | パフォーマンス改善 | FPS低下は体験を直接損なう |
| **2** | セキュリティ・エラー処理 | クラッシュ防止 |
| **3** | v0.2機能実装 | ゲームの完成度向上 |
| **4** | テスト強化 | バグ早期発見、回帰防止 |
| **5** | コードダイエット | 将来の機能追加を楽に |

---

## Phase 1: パフォーマンス改善

**並行実行可能**

| タスク | ファイル | 影響度 | 状態 |
|--------|----------|--------|------|
| ハイライトメッシュキャッシュ化 | highlight.rs | 高 | [x] ✅ |
| O(N²)コンベア転送→HashMap化 | conveyor.rs | 高 | [x] ✅ (既存実装がHashMap使用) |
| Vec::contains()→HashSet化 | chunk.rs, conveyor.rs | 中 | [x] ✅ |
| 不要なclone削除 | conveyor.rs:220,265 | 低 | [x] ✅ (分析済み、必要なclone) |
| クエストデータ変換キャッシュ | quest.rs:20-46 | 低 | [x] ✅ |

---

## Phase 2: セキュリティ・エラー処理

**並行実行可能**

| タスク | ファイル | 状態 |
|--------|----------|------|
| main.rsのunwrap削除 (5箇所) | main.rs:239-295 | [x] ✅ (全てテストコード内) |
| save.rsのunwrap削除 (5箇所) | save.rs | [x] ✅ (全てテストコード内) |
| game_spec.rsのunwrap削除 (7箇所) | game_spec.rs | [x] ✅ (全てテストコード内) |
| 配列インデックス範囲チェック | world/mod.rs:71-73 | [x] ✅ (debug_assert + pos_to_index_checked) |
| 座標キーの範囲チェック | save.rs:184-194 | [x] ✅ (エッジケーステスト追加) |
| コマンドパス走査防止 | command系 | [x] ✅ |
| コマンドパース検証強化 | /tp, /give等 | [x] ✅ (NaN/Infinity検証追加) |
| NaN/Infinity処理 | f32::is_finite()追加 | [x] ✅ |

---

## Phase 3: v0.2 機能実装

**順序依存あり**

### 3.1 UI改修 (Phase 4 from v02_plan)
- [x] `src/ui/storage_ui.rs` 新規作成 ✅
- [x] 8列グリッドレイアウト ✅
- [x] ページネーション (4行/ページ、32スロット表示) ✅
- [x] カテゴリタブ・検索機能 ✅

### 3.2 クエスト機能拡張 (Phase 5 from v02_plan)
- [x] 「納品」ボタン (GlobalInventoryから消費) ✅
- [x] サブクエスト対応準備 ✅ (ActiveSubQuestsリソース追加)
- ~~自動納品オプション~~ → 削除

### 3.3 仕上げ (Phase 6 from v02_plan)
- [x] 初期支給をGlobalInventoryに変更 ✅ (main.rsで実装済み)
- ~~セーブデータ移行~~ → 不要（既存プレイヤーなし）
- ~~移行テスト~~ → 不要

### 3.4 機械入出力システム (Phase 8 from v02_plan)
- [x] 機械にfacing追加 (設置時にプレイヤー向きから決定) ✅ (既存)
- [x] 入力ポートからのアイテム受け取り ✅ (背面からのみ受け取り)
- [x] 出力ポートへのアイテム送出 ✅ (facing方向のみ)
- [x] 機械間直接接続 ✅ (Miner→Furnace/Crusher, Crusher→Furnace)
- [x] レシピシステム統合 (Single Source of Truth) ✅ (Furnace/Crusher→recipe_spec)
- ~~視覚フィードバック~~ → 削除
- [x] デバッグコマンド ✅ (/debug_conveyor, /debug_machine, /debug_connection)

---

## Phase 4: テスト強化

| タスク | 詳細 | 状態 |
|--------|------|------|
| カバレッジ計測 | `cargo tarpaulin`で現状把握 | [x] ✅ (8.54%全体、ロジック70%+) |
| コンベア統合テスト | 複数連結・分岐・合流 | [x] ✅ (8テスト追加) |
| セーブ/ロード往復テスト | 保存→読込→比較 | [x] ✅ (全機械タイプ対応) |
| UIインタラクションテスト | ボタン押下→状態変化 | [x] ✅ (4テスト追加) |
| インベントリUIテスト | 650行あるがテストなし | [x] ✅ (HeldItem, Swap, Trashテスト) |
| システム間連携テスト | Miner→Conveyor→Furnace→Delivery | [x] ✅ (機械facing/inputテスト) |
| エッジケーステスト | 満杯時、空時、境界値 | [x] ✅ (key_to_pos, 境界値テスト) |
| 負荷テスト | 大量アイテム・大量機械 | [x] ✅ (5テスト追加) |
| SSIM比較 (Rust版) | `image-compare` crateで構造類似度 | [x] ✅ (tests/ssim_test.rs) |
| cargo-fuzz導入 | セーブ/ロードのパース部分をfuzz | [x] ✅ (proptest版: tests/fuzz_save_load.rs) |

### 4.1 E2Eテスト改善 (VLMテスト完了後)

**短期 (1-2時間)** - バグ発見効率が高い順

| タスク | 詳細 | 効果 | 状態 |
|--------|------|------|------|
| ウィンドウ指定撮影 | `scrot`→`import -window`で対象ウィンドウのみ | デスクトップ誤撮影防止 | [x] ✅ |
| panic検出追加 | E2E中もログ監視、panic時即終了 | クラッシュ見逃し防止 | [x] ✅ |
| 撮影前ウィンドウ確認 | `xdotool windowactivate --sync`追加 | フォーカス外れ防止 | [x] ✅ |

**中期 (半日)** - 操作の信頼性向上

| タスク | 詳細 | 効果 | 状態 |
|--------|------|------|------|
| ゲーム状態JSON出力 | `TestState`構造体→`e2e_state.json` | 操作成功を確認可能 | [x] ✅ (既存) |
| 操作後の状態検証 | `wait_for '.creative_mode == true'`パターン | 盲目的操作を排除 | [x] ✅ |
| コマンド実行確認 | `/creative`等の結果をフィードバック | コマンド失敗検出 | [x] ✅ |

**長期 (数日)** - 根本的な改善

| タスク | 詳細 | 効果 | 状態 |
|--------|------|------|------|
| ゲーム内スクショ | `ScreenshotManager`使用 | 最も正確な画面取得 | [x] ✅ (/screenshotコマンド) |
| VLMテスト統合 | test_all.shにレベル自動選択で組込 | 視覚バグ自動検出 | [x] ✅ (quick/full自動選択) |
| ランダムプレイテスト | 1000回ランダム操作→異常検出 | 予想外バグ発見 | [x] ✅ (random_play_test.sh) |
| テスト結果JSON化 | 実行時間・カバレッジ・トレンド | 可視化・追跡可能 | [x] ✅ (test_json_report.sh) |

**目標**: ロジック部分カバレッジ70%以上 ✅
**現状**: 280テスト通過 (lib:74, bin:38, e2e:146, fuzz:11, proptest:8, ssim:3)
**カバレッジ**: 8.54%全体 (Bevy ECSシステム関数は0%、ロジック部分70%+達成)
- `world/mod.rs`: 75%、`world/biome.rs`: 88%
- `player/inventory.rs`: 72%、`player/global_inventory.rs`: 83%

---

## Phase 5: コードダイエット

**順序依存あり**

| タスク | 削減量 | 依存 | 状態 |
|--------|--------|------|------|
| machine_interact統合 | -190行 | なし | [-] (複雑、リスク大) |
| machine_output統合 | -110行 | ↑完了後 | [-] (機械間接続で差異化) |
| machine_ui_input統合 | -145行 | ↑完了後 | [-] (各機械で異なる動作、統合リスク大) |
| Color定数化 | -80行 | なし | [x] ✅ (ui_colorsモジュール追加) |
| raycast共通化 | -45行 | なし | [x] ✅ (dda_raycast関数追加) |

---

## Phase 6: アーキテクチャ改善 (中優先度)

| タスク | 詳細 | 状態 |
|--------|------|------|
| InputContext統合 | 6リソース→1構造体 | [-] (大規模、慎重に) |
| ワイルドカードインポート削除 | `use components::*`を明示化 | [-] (13ファイル、リスク大) |
| DebugHudState重複修正 | plugins/debug.rs + plugins/ui.rs | [x] ✅ (UIPluginから削除) |
| MachineInteraction統合 | 3つのInteracting*→1つのenum | [-] (67箇所・17ファイル、慎重に) |
| UIState統合 | 7つのUIリソース→1つの構造体 | [-] (大規模、慎重に) |
| イベント駆動への移行 | 直接Resource変更→Event発行 | [-] (根本的変更、v0.3以降) |

---

## Phase 7: インフラ・最適化 (低優先度)

**並行実行可能、他タスクと競合しにくい**

| タスク | 詳細 | 状態 |
|--------|------|------|
| CI/CD改善 | clippy警告をエラー扱い、WASM自動ビルド | [x] ✅ (既に設定済み) |
| Cargo.toml最適化 | 不要依存削除、feature整理 | [x] ✅ (確認済み、全依存使用中) |
| フォント最適化 | サブセット化 (16MB→軽量化) | [ ] |
| WASMサイズ最適化 | wasm-opt、LTO見直し | [ ] |
| バイナリサイズ削減 | strip symbols、codegen-units=1 | [x] ✅ (profile.releaseで設定済み) |
| 依存関係棚卸し | 409依存の必要性調査 | [x] ✅ (全依存使用確認) |

---

## Phase 8: UIテーマ刷新 (Factoryテーマ)

**概要**: オレンジ基調の工場ゲームらしいUIに統一

| タスク | 詳細 | 状態 |
|--------|------|------|
| テーマ定数追加 | `ui/mod.rs`にスロット/クエスト色定義 | [ ] |
| スロットBorderRadius追加 | インベントリ・ホットバー | [ ] |
| ホバー/選択スタイル | Interaction監視で色変更 | [ ] |
| クエストパネル更新 | BorderRadius + テーマ色 | [ ] |
| 機械UI統一 | 共通スタイル関数抽出 | [ ] |

**テーマ値**: `tools/ui_tuner.html` で調整済み
- スロット: 48px, 角丸6px, ボーダー2px
- 色: `#ff8800`(オレンジ), `#ffcc00`(黄色選択), `#2d2d2d`(背景)

---

## 将来機能 (v0.3以降)

- リプレイシステム: 操作記録・巻き戻し・早送り
- ブループリント: 範囲選択保存・ペースト
- 電力システム: 発電機・電線・電力消費
- 流体パイプ: ポンプ・パイプ・タンク
- マルチプレイ基盤: WebSocket同期
- Modding API: Lua/WASM埋め込み
- ビジュアルプログラミング: ノードグラフ

---

## 並行実行マトリクス

```
Phase 1 (パフォーマンス) → 並行OK
Phase 2 (セキュリティ)   → 並行OK
Phase 1 と Phase 2       → 一部並行OK (ファイル重複なければ)
Phase 3 (v0.2機能)       → 順序あり (3.1→3.2→3.3→3.4)
Phase 5 (ダイエット)     → machine系は順序必須
Phase 7 (インフラ)       → 他と並行OK
```

---

## Phase 9: チュートリアルクエスト

**概要**: 報酬なしのチュートリアルクエストで基礎操作を自然に学べるようにする

### 設計方針

- **別トラック方式**: `TUTORIAL_QUESTS` を `MAIN_QUESTS` とは別に管理
- **報酬なし**: `rewards: &[]` で報酬を設定しない
- **アクション達成判定**: アイテム納品ではなく、特定アクションの実行を検知

### 9.1 データ構造追加

| タスク | 詳細 | 状態 |
|--------|------|------|
| `QuestType::Tutorial` 追加 | game_spec/mod.rs | [ ] |
| `TUTORIAL_QUESTS` 定義 | game_spec/mod.rs | [ ] |
| `TutorialProgress` リソース | components/mod.rs | [ ] |
| `TutorialAction` enum | components/mod.rs | [ ] |

### 9.2 チュートリアルクエスト一覧

| 順番 | ID | 説明 | 達成条件 |
|------|-----|------|---------|
| 0 | tut_move | WASDで移動しよう | 移動距離50ブロック |
| 1 | tut_break | ブロックを掘ろう | 任意のブロック1個採掘 |
| 2 | tut_inventory | Eでインベントリを開こう | インベントリ開閉1回 |
| 3 | tut_place_miner | 採掘機を設置しよう | 採掘機1台設置 |
| 4 | tut_place_conveyor | コンベアを3個繋げよう | コンベア3個連続設置 |
| 5 | tut_place_furnace | 精錬炉を設置しよう | 精錬炉1台設置 |
| 6 | tut_connect_line | 採掘→コンベア→精錬を繋げよう | 有効な接続1本 |
| 7 | tut_first_ingot | インゴットができるまで待とう | インゴット1個生成 |

### 9.3 システム実装

| タスク | 詳細 | 状態 |
|--------|------|------|
| `tutorial_action_tracker` | 各アクションを検知してTutorialProgressに反映 | [ ] |
| `tutorial_progress_check` | 進捗判定システム | [ ] |
| `tutorial_advance` | 次のチュートリアルへ進む処理 | [ ] |
| チュートリアルUI | 専用パネル（メインクエストUIとは別） | [ ] |

### 9.4 UI表示

| タスク | 詳細 | 状態 |
|--------|------|------|
| チュートリアルパネル追加 | 画面上部中央に目標表示 | [ ] |
| 進捗インジケータ | 「チュートリアル 3/8」形式 | [ ] |
| 完了演出 | チュートリアル完了時のフェードアウト | [ ] |
| スキップボタン | 設定でオン/オフ切り替え | [ ] |

### 9.5 統合・テスト

| タスク | 詳細 | 状態 |
|--------|------|------|
| チュートリアル完了後にメインクエスト開始 | CurrentQuest.index=0 | [ ] |
| セーブ/ロード対応 | TutorialProgress永続化 | [ ] |
| E2Eテスト | チュートリアル全ステップ通過テスト | [ ] |
| 新規ゲーム開始時の初期化 | TutorialProgress::default() | [ ] |

### 実装順序

```
9.1 データ構造 → 9.2 クエスト定義 → 9.3 システム → 9.4 UI → 9.5 統合
```

---

## Phase 10: プラットフォーム再設計

**概要**: 納品プラットフォームを「8×8固定サイズの巨大倉庫」として再設計

### 新仕様

- **サイズ**: 8×8ブロック固定、高さ1ブロック
- **収納ポート**: 各辺6個（四隅除外）、合計24ポート/個
- **インベントリ**: 無限容量、全プラットフォームで共有（= GlobalInventory）
- **増設**: 横並べ・積み上げ可能

### 用語定義

- **収納**: コンベア経由でプラットフォームにアイテムが入ること
- **納品**: クエストのためにインベントリからアイテムを消費すること

### 10.1 定数・BlockType追加

| タスク | ファイル | 状態 |
|--------|----------|------|
| PLATFORM_SIZE=8 | src/constants.rs | [x] ✅ (既存定義あり) |
| PlatformBlock追加 | src/block_type.rs | [x] ✅ |

### 10.2 コンポーネント簡素化

| タスク | ファイル | 状態 |
|--------|----------|------|
| DeliveryPlatform.delivered削除 | src/components/mod.rs | [x] ✅ |
| position: IVec3追加 | src/components/mod.rs | [x] ✅ |

### 10.3 納品ロジック変更

| タスク | ファイル | 状態 |
|--------|----------|------|
| platform.delivered参照を全削除 | src/systems/quest.rs | [x] ✅ |
| platform_query削除 | src/systems/quest.rs | [x] ✅ |
| can_deliver簡略化 | src/systems/quest.rs | [x] ✅ |
| quest_deliver_button簡略化 | src/systems/quest.rs | [x] ✅ |
| update_quest_ui進捗表示変更 | src/systems/quest.rs | [x] ✅ |

### 10.4 収納ロジック変更（HashSet方式）

| タスク | ファイル | 状態 |
|--------|----------|------|
| build_valid_ports関数追加 | src/systems/conveyor.rs | [x] ✅ (setup_delivery_platformで動的生成) |
| 四隅を収納ポートから除外 | src/systems/conveyor.rs | [x] ✅ |
| 複数プラットフォーム対応 | src/systems/conveyor.rs | [x] ✅ (GlobalInventory経由) |

### 10.5 配置システム変更

| タスク | ファイル | 状態 |
|--------|----------|------|
| setup_delivery_platform削除 | src/systems/quest.rs | [ ] 将来タスク |
| PlatformBlock配置対応 | src/systems/block_operations/placement.rs | [ ] 将来タスク |

### 10.6 初期アイテム・モデル

| タスク | ファイル | 状態 |
|--------|----------|------|
| PlatformBlock×1初期支給 | src/main.rs | [ ] 将来タスク |
| VOXモデル作成 | assets/models/machines/platform.vox | [ ] 将来タスク |

### 10.7 セーブデータ・テスト

| タスク | ファイル | 状態 |
|--------|----------|------|
| delivered削除対応 | src/save.rs | [x] ✅ |
| 関連テスト更新 | tests/*.rs | [x] ✅ |

### 実装順序

```
10.1 定数・BlockType → 10.2 コンポーネント → 10.3 納品ロジック
→ 10.4 収納ロジック → 10.5 配置システム → 10.6 初期アイテム・モデル
→ 10.7 セーブ・テスト
```

### 削除したコード ✅

- ~~`setup_delivery_platform()` 関数~~ → 将来タスク
- `DeliveryPlatform.delivered` フィールド ✅
- `delivered` を参照する全てのロジック ✅
- `can_deliver_from_global_inventory()` の platform 引数 ✅
- セーブ/ロードの`delivery.delivered`操作 ✅ → GlobalInventory経由に変更

---

## Phase 11: ゲームデータ外部化

**概要**: 頻繁に調整するゲームデータをJSONに外出しし、再コンパイルなしでバランス調整可能に

### 設計方針

- **外出し対象**: 変更頻度が高いデータのみ
- **型定義はRust側に残す**: 型安全性を維持
- **JSON Schema**: VSCodeでバリデーション・補完を効かせる

### 11.1 外出し対象

| データ | 現在の場所 | 変更頻度 | 外出し |
|--------|-----------|---------|--------|
| レシピ | `game_spec.rs` | 高 | ✅ `recipes.json` |
| クエスト | `game_spec.rs` | 高 | ✅ `quests.json` |
| 機械パラメータ | `game_spec.rs` | 中 | ✅ `machines.json` |
| 物理定数 | `constants.rs` | 低 | ❌ |
| UIサイズ・色 | 各所 | 低 | ❌ |
| キーバインド | `input.rs` | 低 | ❌ |
| BlockType | `block_type.rs` | 低 | ❌ |

### 11.2 ファイル構成

```
assets/data/
  recipes.json      # レシピ定義（精錬、粉砕など）
  quests.json       # メインクエスト・サブクエスト
  machines.json     # 処理時間、燃料効率、入出力ポート
  schema/
    recipes.schema.json
    quests.schema.json
    machines.schema.json
```

### 11.3 実装タスク

| タスク | 詳細 | 状態 |
|--------|------|------|
| `assets/data/` ディレクトリ作成 | | [ ] |
| `recipes.json` 作成 | 現在の `FURNACE_RECIPES`, `CRUSHER_RECIPES` を移行 | [ ] |
| `quests.json` 作成 | `MAIN_QUESTS`, `SUB_QUESTS` を移行 | [ ] |
| `machines.json` 作成 | 処理時間、燃料消費量、ポート設定 | [ ] |
| JSON Schema作成 | VSCode補完・バリデーション用 | [ ] |
| ローダー実装 | `game_spec.rs` でJSON読み込み | [ ] |
| エラーハンドリング | ファイル不在・パースエラー時のフォールバック | [ ] |
| テスト追加 | JSON読み込みテスト | [ ] |

### 11.4 将来拡張（v0.3以降）

- Web GUIエディタ（プレイヤーフィードバックが増えた時）
- Modding対応（ユーザーがレシピを追加）

---

## Phase 12: 機械システムのコンポーネント化 (v0.3)

**概要**: Bevy ECSパターンに従い、機械の共通機能をコンポーネントに分離

**注意**: Geminiレビューにより、Phase 12（機械）を先に、Phase 13（BlockType）を後に実施することを推奨。
理由: 機械システムを先に抽象化すれば、BlockType変更時の修正箇所が「汎用機械システム」に集約される。

### 設計方針

**現状の問題**:
- Miner(355行), Furnace(~300行), Crusher(328行) が似たパターンを個別実装
- 新機械追加時に大量のボイラープレートが必要

**Bevy的アプローチ**: 巨大Structではなく、機能をComponentに分割

### 12.1 共通コンポーネント定義

```rust
// 共通コンポーネント
#[derive(Component)]
struct WorldPosition(IVec3);

#[derive(Component)]
struct Facing(Direction);

#[derive(Component)]
struct InputSlot {
    item: Option<BlockType>,
    count: u32,
    max: u32,
    filter: Option<BlockCategory>, // 入力制限
}

#[derive(Component)]
struct OutputSlot {
    item: Option<BlockType>,
    count: u32,
}

#[derive(Component)]
struct ProcessingState {
    progress: f32,
    speed: f32,
}

#[derive(Component)]
struct MachineConfig {
    machine_type: MachineType, // Recipe検索用
    energy_consumption: u32,
}

// バンドル定義
#[derive(Bundle)]
struct StandardMachineBundle {
    pos: WorldPosition,
    facing: Facing,
    input: InputSlot,
    output: OutputSlot,
    process: ProcessingState,
    config: MachineConfig,
}
```

### 12.2 汎用処理システム

```rust
fn machine_processing_system(
    time: Res<Time>,
    mut query: Query<(&mut InputSlot, &mut OutputSlot, &mut ProcessingState, &MachineConfig)>
) {
    for (input, output, process, config) in query.iter_mut() {
        // 共通の「入力消費 -> 進捗 -> 出力生成」ロジック
        // レシピは config.machine_type を使って検索
    }
}
```

### 12.3 実装タスク

| タスク | 詳細 | 状態 |
|--------|------|------|
| 共通コンポーネント作成 | InputSlot, OutputSlot, ProcessingState | [ ] |
| MachineConfig作成 | machine_type, energy_consumption | [ ] |
| 汎用処理システム実装 | machine_processing_system | [ ] |
| Assemblerを新方式で実装 | 新システムの実証 | [ ] |
| Furnaceリファクタ | 新コンポーネント構成に置き換え | [ ] |
| Crusherリファクタ | 同上 | [ ] |
| Minerリファクタ | InputSlotなしの特殊ケース | [ ] |
| 旧コード削除 | 旧Miner, Furnace, Crusher struct | [ ] |
| テスト更新 | 新コンポーネント対応 | [ ] |

### 12.4 移行戦略

1. **新しい機械から作る**: 既存コードをいじらず、Assemblerを新システムで実装
2. **新システムの実証**: Assemblerが動くことを確認
3. **既存機械の置き換え**: Furnace → Crusher → Miner の順で移行
4. **旧コードの削除**: 最後に旧struct群を削除

---

## Phase 13: BlockType階層化 (v0.3)

**概要**: BlockType enumを役割別に分離し、新アイテム追加時の変更範囲を局所化

**注意**: Phase 12（機械システム）完了後に実施。先に実施すると702箇所の変更が必要。

### 設計方針

**現状の問題**:
- 15種類が1つのenumに混在（地形・鉱石・機械・アイテム・ツール）
- 702箇所、31ファイルで使用
- is_machine(), is_ore() 等のヘルパーに新種追加し忘れるリスク

**段階的移行アプローチ**: 一気に702箇所を変更せず、段階的に移行

### 13.1 新Enum定義

```rust
enum TerrainBlock { Stone, Grass }
enum OreBlock { IronOre, CopperOre, Coal }
enum MachineBlock { Miner, Conveyor, Furnace, Crusher, Assembler, Platform }
enum ProcessedItem { IronIngot, CopperIngot, IronDust, CopperDust }
enum Tool { StonePickaxe }

enum GameItem {
    Terrain(TerrainBlock),
    Ore(OreBlock),
    Machine(MachineBlock),
    Processed(ProcessedItem),
    Tool(Tool),
}
```

### 13.2 移行戦略（3段階）

**Phase 13-A: 新Enum定義と相互変換**
- 新しいenum群を定義
- BlockTypeは削除せず「マスターID」として残す
- `BlockType::as_game_item()` と `From<OreBlock> for BlockType` を実装

**Phase 13-B: ロジックの移行（データはそのまま）**
- BlockTypeを保存・通信用の軽量ID (Copy可能) として維持
- 計算・判定を行う場所でのみ `let item = block.as_game_item();` してパターンマッチ
- データ構造（Vec<BlockType>やComponent）はまだ書き換えない

**Phase 13-C: 完全移行（必要な場合のみ）**
- パフォーマンスや型安全性が必要な場合のみ、ストレージ層もGameItemに置き換え
- BevyのComponentとしてはフラットなenum (BlockType) の方が扱いやすい場合もあり、13-Bで止めるのも現実的

### 13.3 実装タスク

| タスク | 詳細 | 状態 |
|--------|------|------|
| **Phase 13-A** | | |
| 新enum群定義 | TerrainBlock, OreBlock, MachineBlock, ProcessedItem, Tool | [ ] |
| GameItem enum定義 | 統合enum | [ ] |
| 相互変換実装 | as_game_item(), From/Into traits | [ ] |
| **Phase 13-B** | | |
| machine_processing_system更新 | GameItemベースの判定に変更 | [ ] |
| レシピシステム更新 | GameItem対応 | [ ] |
| UI更新 | カテゴリ表示対応 | [ ] |
| **Phase 13-C（オプション）** | | |
| Component定義更新 | 必要に応じてGameItemに変更 | [ ] |
| 全システム移行 | BlockType参照を完全削除 | [ ] |

---

## アーキテクチャレビュー結果サマリー (2026-01-04)

### Claude分析

| 問題点 | 影響度 | 改善案 |
|--------|--------|--------|
| BlockType密結合 | 高 | Phase 13で階層化 |
| 機械システム重複 | 中 | Phase 12でコンポーネント化 |
| UIコード肥大化 | 中 | ロジック/UI分離（将来） |
| InputState分散 | 低 | Phase 6で統合 |

### Gemini分析

| 評価 | 詳細 |
|------|------|
| ✅ Plugin構成 | 適切に分離 |
| ✅ ECS準拠 | Bevyパターンに従っている |
| ✅ イベント駆動 | システム間が疎結合 |
| ✅ game_spec | Single Source of Truth |

### 合見積結論

**実施順序**: Phase 12 (機械) → Phase 13 (BlockType)

| Phase | タスク | 推定工数 | トリガー |
|-------|--------|----------|----------|
| 12 | 機械コンポーネント化 | 2-3日 | 次の新機械追加時 |
| 13-A | 新Enum定義 | 半日 | Phase 12完了後 |
| 13-B | ロジック移行 | 1-2日 | 13-A完了後 |
| 13-C | 完全移行 | 1-2日 | 必要に応じて |

---

## 次のアクション

**Phase 1-7, 10 (基本部分) 完了** ✅

### 短期（v0.2完成）
1. **Phase 9**: チュートリアルクエスト実装
2. **Phase 8**: UIテーマ刷新（Factoryテーマ適用）

### 中期（拡張性改善 - トリガー駆動）
3. **Phase 11**: ゲームデータ外部化 → バランス調整が増えたら
4. **Phase 12**: 機械コンポーネント化 → **次の新機械追加時**
5. **Phase 13**: BlockType階層化 → Phase 12完了後

### 長期（オプション）
6. **Phase 5/6**: 残りのコードダイエット・アーキテクチャ改善
7. **v0.3機能**: 電力システム、流体パイプ、マルチプレイ等
