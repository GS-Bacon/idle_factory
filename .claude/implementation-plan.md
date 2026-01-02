# 統合実装計画 (2026-01-02)

## 現状サマリー

| 項目 | 値 |
|------|-----|
| コード行数 | **11,500行** (目標12,500行以下 ✅) |
| テスト | **200件** 通過 (unit:87, e2e:113) |
| unwrap() | **17箇所** (全てテストコード内) |
| Clippy警告 | **0件** |

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
| Vec::contains()→HashSet化 | chunk.rs, conveyor.rs | 中 | [ ] |
| 不要なclone削除 | conveyor.rs:220,265 | 低 | [ ] |
| クエストデータ変換キャッシュ | quest.rs:20-46 | 低 | [ ] |

---

## Phase 2: セキュリティ・エラー処理

**並行実行可能**

| タスク | ファイル | 状態 |
|--------|----------|------|
| main.rsのunwrap削除 (5箇所) | main.rs:239-295 | [ ] |
| save.rsのunwrap削除 (5箇所) | save.rs | [ ] |
| game_spec.rsのunwrap削除 (7箇所) | game_spec.rs | [ ] |
| 配列インデックス範囲チェック | world/mod.rs:71-73 | [ ] |
| 座標キーの範囲チェック | save.rs:184-194 | [ ] |
| コマンドパス走査防止 | command系 | [ ] |
| コマンドパース検証強化 | /tp, /give等 | [ ] |
| NaN/Infinity処理 | f32::is_finite()追加 | [ ] |

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
- [ ] 機械間直接接続
- [ ] レシピシステム統合 (Single Source of Truth)
- ~~視覚フィードバック~~ → 削除
- [ ] デバッグコマンド (/debug_machine, /debug_connection)

---

## Phase 4: テスト強化

| タスク | 詳細 | 状態 |
|--------|------|------|
| カバレッジ計測 | `cargo tarpaulin`で現状把握 | [ ] |
| コンベア統合テスト | 複数連結・分岐・合流 | [ ] |
| セーブ/ロード往復テスト | 保存→読込→比較 | [ ] |
| UIインタラクションテスト | ボタン押下→状態変化 | [ ] |
| インベントリUIテスト | 650行あるがテストなし | [ ] |
| システム間連携テスト | Miner→Conveyor→Furnace→Delivery | [ ] |
| エッジケーステスト | 満杯時、空時、境界値 | [ ] |
| 負荷テスト | 大量アイテム・大量機械 | [ ] |
| SSIM比較 (Rust版) | `image-compare` crateで構造類似度 | [ ] |
| cargo-fuzz導入 | セーブ/ロードのパース部分をfuzz | [ ] |

**目標**: カバレッジ70%以上

---

## Phase 5: コードダイエット

**順序依存あり**

| タスク | 削減量 | 依存 | 状態 |
|--------|--------|------|------|
| machine_interact統合 | -190行 | なし | [ ] |
| machine_output統合 | -110行 | ↑完了後 | [ ] |
| machine_ui_input統合 | -145行 | ↑完了後 | [ ] |
| Color定数化 | -80行 | なし | [ ] |
| raycast共通化 | -45行 | なし | [ ] |

---

## Phase 6: アーキテクチャ改善 (中優先度)

| タスク | 詳細 | 状態 |
|--------|------|------|
| InputContext統合 | 6リソース→1構造体 | [ ] |
| ワイルドカードインポート削除 | `use components::*`を明示化 | [ ] |
| DebugHudState重複修正 | plugins/debug.rs + plugins/ui.rs | [ ] |
| MachineInteraction統合 | 3つのInteracting*→1つのenum | [ ] |
| UIState統合 | 7つのUIリソース→1つの構造体 | [ ] |
| イベント駆動への移行 | 直接Resource変更→Event発行 | [ ] |

---

## Phase 7: インフラ・最適化 (低優先度)

**並行実行可能、他タスクと競合しにくい**

| タスク | 詳細 | 状態 |
|--------|------|------|
| CI/CD改善 | clippy警告をエラー扱い、WASM自動ビルド | [ ] |
| Cargo.toml最適化 | 不要依存削除、feature整理 | [ ] |
| フォント最適化 | サブセット化 (16MB→軽量化) | [ ] |
| WASMサイズ最適化 | wasm-opt、LTO見直し | [ ] |
| バイナリサイズ削減 | strip symbols、codegen-units=1 | [ ] |
| 依存関係棚卸し | 409依存の必要性調査 | [ ] |

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

## 次のアクション

1. **Phase 1 開始**: highlight.rs + conveyor.rs を並行で改善
2. **Phase 2 並行**: unwrap()削除を進める
3. 完了後 **Phase 3** (v0.2機能実装) へ
