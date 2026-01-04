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

## 次のアクション

**Phase 1-7 はほぼ完了** ✅

1. **Phase 9**: チュートリアルクエスト実装
2. **Phase 8**: UIテーマ刷新（Factoryテーマ適用）
3. **Phase 5/6**: 残りのコードダイエット・アーキテクチャ改善（オプション）
4. **v0.3機能**: 将来機能リストから選択して実装
