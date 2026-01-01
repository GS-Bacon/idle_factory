# v0.2 実装計画: 全体在庫システム

**作成日**: 2026-01-01
**ステータス**: 基本実装完了

---

## 概要

プレイヤー個人インベントリを廃止し、「全体在庫」で一元管理するシステムを実装する。
納品プラットフォームが倉庫とクエスト納品の二役を担う。

## 仕様書

`src/game_spec.rs` に定義済み:
- `global_inventory_spec` - 全体在庫仕様
- `delivery_platform_spec` - 納品PF仕様
- `assembler_spec` - 組立機仕様（将来）
- `quest_system_spec` - クエストシステム仕様
- `ui_spec` - UI仕様
- `biome_mining_spec` - バイオーム採掘仕様 ✅ 追加済み

## 初期装備（更新済み）

```rust
INITIAL_EQUIPMENT = [
    (MinerBlock, 2),      // 2台で鉄+石炭を同時採掘
    (ConveyorBlock, 30),  // 往復ライン×3本分
    (FurnaceBlock, 1),
];
```

---

## 実装フェーズ

### Phase 0: 事前準備（Geminiレビュー反映）✅ 完了

#### 0.1 Direction型の統一
- [x] `src/components/machines.rs` に既存 `Direction` enum を使用
- [x] コンベアと機械で同じ `Direction` 型を使用
- [x] `game_spec::machine_io_spec::MachineFacing` は未使用のため削除不要

#### 0.2 noiseクレート追加
- [x] 座標ベースの疑似乱数ハッシュ関数を実装（`world/biome.rs`）

---

### Phase 1: 全体在庫システム基盤 ✅ 完了

#### 1.1 GlobalInventoryリソース作成
- [x] `src/player/global_inventory.rs` 新規作成
- [x] `GlobalInventory` リソース定義
  - `items: HashMap<BlockType, u32>`
  - `add_item()`, `remove_item()`, `get_count()` メソッド
  - `try_consume(items: &[(BlockType, u32)]) -> bool` アトミック消費メソッド
- [x] `#[derive(Serialize, Deserialize)]` を追加
- [x] 初期化システム（main.rs で INITIAL_EQUIPMENT から初期化）

#### 1.2 納品PFのアイテム受け入れ→全体在庫追加
- [ ] 後回し: DeliveryPlatform変更は未実装
- [ ] クエスト進捗は既存の `delivered` HashMap から計算

---

### Phase 2: 機械設置/撤去 ✅ 完了

#### 2.1 機械設置時に全体在庫から消費
- [x] `block_placement.rs` 関数を変更
- [x] 設置前に `GlobalInventory.remove_item()` で在庫チェック
- [x] 在庫不足時は設置不可（return）

#### 2.2 機械撤去時に全体在庫へ戻す
- [x] `breaking.rs` 関数を変更
- [x] 機械ブロック破壊時に `GlobalInventory.add_item()` で戻す

---

### Phase 3: 既存システム置き換え（段階的移行）✅ 部分完了

#### 3.1 GlobalInventory を追加（旧Inventoryと並行稼働）
- [x] `GlobalInventory` リソースを追加（旧 `Inventory` は残す）
- [x] 機械関連は `GlobalInventory` を使用
- [ ] 旧 `Inventory` は素材用に維持（スロットベースUI）

#### 3.2 使用箇所を順次移行
- [x] `quest_claim_rewards` → `GlobalInventory` に報酬追加
- [x] 初期装備付与 → `GlobalInventory` に追加
- [x] 機械設置/撤去 → `GlobalInventory` を使用
- [x] 各移行後にテスト実行で動作確認（106テスト通過）

#### 3.3 旧Inventory削除（後回し）
- [ ] 旧 `Inventory` は素材管理用に維持
- [ ] UIとの互換性のため残す

---

### Phase 4: UI改修

#### 4.1 納品PF UIをマイクラ風グリッドに改修
- [ ] `src/ui/storage_ui.rs` 新規作成
- [ ] 8列グリッドレイアウト
- [ ] スロットサイズ 54x54px（既存と統一）
- [ ] ページネーション（4行/ページ、計32スロット表示）
- [ ] アイテムアイコン + 数量表示

#### 4.2 カテゴリタブ・検索機能追加
- [ ] タブボタン: 全て / 素材 / 機械 / 部品
- [ ] 検索ボックス（アイテム名フィルタ）
- [ ] タブ/検索によるフィルタリング処理

---

### Phase 5: クエスト機能拡張

#### 5.1 クエスト納品ボタン実装（手動納品）
- [ ] UI上に「納品」ボタン追加
- [ ] クリック時に `GlobalInventory` から消費
- [ ] 報酬を `GlobalInventory` に追加
- [ ] 達成可能時のみボタンをアクティブ化

#### 5.2 サブクエスト表示・複数同時進行対応
- [ ] `ActiveSubQuests` リソース追加
- [ ] サブクエスト一覧UI
- [ ] 複数同時進行（最大5個）
- [ ] 自動納品オプション（サブのみ）

---

### Phase 6: 仕上げ

#### 6.1 初期支給を全体在庫に変更
- [ ] ゲーム開始時に `game_spec::INITIAL_EQUIPMENT` を `GlobalInventory` へ
- [ ] 納品PFは最初からワールドに配置済み

#### 6.2 セーブデータ移行
- [ ] 旧セーブデータ検出（`Inventory` フィールドの存在確認）
- [ ] 旧 `Inventory` → `GlobalInventory` への自動移行ロジック
- [ ] 移行失敗時のフォールバック（初期装備で開始）
- [ ] 移行完了後に旧フィールドを削除（クリーンアップ）

#### 6.3 テスト・動作確認
- [ ] 既存テストの更新（Inventory → GlobalInventory）
- [ ] E2Eテストで全体在庫動作確認
- [ ] クエスト納品フロー確認
- [ ] セーブデータ移行テスト（旧→新）

---

### Phase 7: バイオーム採掘システム ✅ 完了

#### 7.1 BiomeType定義とマップ生成
- [x] `src/world/biome.rs` 新規作成
- [x] `BiomeType` enum定義（Iron, Copper, Coal, Stone, Mixed, Unmailable）
- [x] バイオームマップ生成（座標ベースハッシュ関数）
- [x] 座標→バイオーム判定関数 `BiomeMap::get_biome()`

#### 7.2 スポーン地点保証
- [x] 納品PF周辺（半径15ブロック）に鉄・石炭・銅バイオームを強制配置
- [x] セクター分割で初期エリアは固定配置（angle-based）
- [ ] バイオーム境界の可視化（後回し）

#### 7.3 採掘機のバイオーム対応
- [x] `miner_mining` システムを変更
- [x] バイオーム確率テーブル参照
- [x] `game_spec::biome_mining_spec` から確率取得
- [x] `BiomeType::sample_resource()` で確率に基づいてアイテム生成
- [x] `Miner` に `tick_count` フィールド追加（乱数用）

#### 7.4 採掘不可バイオーム対応
- [x] `Unmailable` バイオームでは採掘機が動作しない
- [x] `BiomeMap::can_mine()` でチェック
- [ ] 設置時警告は後回し

#### 7.5 テスト・動作確認
- [x] バイオーム判定の単体テスト
- [x] 確率テーブルの合計100%検証
- [x] スポーン保証テスト
- [x] 全106テスト通過

---

## 影響範囲

### 変更が必要なファイル

| ファイル | 変更内容 |
|----------|----------|
| `src/player/mod.rs` | GlobalInventory 追加 |
| `src/player/inventory.rs` | deprecated or 削除 |
| `src/systems/quest.rs` | 報酬を GlobalInventory へ |
| `src/systems/machines/block_operations/` | 設置/撤去で在庫消費/返却 |
| `src/ui/inventory_ui.rs` | 全体在庫ビューアに変更 |
| `src/main.rs` | GlobalInventory リソース初期化 |
| `src/world/biome.rs` | **新規** バイオーム定義・マップ生成 |
| `src/world/mod.rs` | biome モジュール追加 |
| `src/systems/machines/miner.rs` | バイオーム確率テーブル参照 |

### 削除予定

- `PlayerInventory` コンポーネント（個人インベントリ）
- 手動アイテム拾い機能
- 採掘機の「真下ブロック参照」ロジック

---

## 依存関係

```
Phase 1 → Phase 2 → Phase 3 → Phase 4 → Phase 5 → Phase 6
  ↓                                                   ↓
GlobalInventory が全ての基盤                    Phase 7
                                                   ↓
                                          バイオーム採掘（独立実装可）
```

**Note**: Phase 7（バイオーム採掘）は Phase 1-6 と並行して実装可能。
ただし Phase 6 完了後に統合テストを行う。

---

## リスク・注意点

1. **既存テストの大量修正** - Inventory を使うテストが多い
2. **セーブデータ互換性** - 旧セーブの Inventory データの移行 → **Phase 6.2 で対策済み**
3. **UI実装工数** - マイクラ風UIは既存UIより複雑

---

## 影響範囲調査（実装前に確認）

### Inventory 使用箇所（2026-01-01 調査）

| カテゴリ | ファイル数 | 参照数 | 対応方針 |
|----------|-----------|--------|----------|
| **UIコンポーネント** | 2 | 約20 | `InventoryOpen` は残す（UI状態用） |
| **player/inventory.rs** | 1 | 約50 | `GlobalInventory` に置き換え |
| **main.rs** | 1 | 約10 | リソース初期化を変更 |
| **plugins/ui.rs** | 1 | 約5 | `InventoryOpen` 初期化は残す |
| **テスト** | 3 | 約15 | テストケース更新 |
| **合計** | - | 約132行 | - |

### 注意: `InventoryOpen` と `Inventory` の区別

- `InventoryOpen`: UI状態（開閉フラグ）→ **削除しない**
- `Inventory`: アイテム管理リソース → **`GlobalInventory` に置き換え**
- `InventoryUI`, `InventorySlotUI`: UIコンポーネント → **名前変更不要**

---

---

### Phase 8: 機械入出力システム

**仕様書**: `src/game_spec.rs` の `machine_io_spec`, `machine_spec`, `recipe_spec` に定義済み

#### 8.1 機械コンポーネントに facing 追加
- [ ] `Miner`, `Furnace`, `Crusher` に `facing: Direction` フィールド追加
- [ ] 設置時にプレイヤーの向きから facing を決定
- [ ] セーブ/ロード対応
- [ ] **ヘルパー関数**: `get_port_world_pos(machine_pos: IVec3, facing: Direction, port: IoPort) -> IVec3` を実装

#### 8.2 入力ポートからのアイテム受け取り
- [ ] コンベア終端が機械の入力ポート（Back）に接続時、アイテムを受け取る
- [ ] `game_spec::machine_spec` から入力ポート位置を取得
- [ ] `get_port_world_pos()` を使用してワールド座標に変換

#### 8.3 出力ポートへのアイテム送出
- [ ] 機械の出力ポート（Front）に隣接するコンベアにアイテムを流す
- [ ] 現在の「全方向探索」を「出力ポート方向のみ」に変更
- [ ] facing に基づいてワールド座標に変換

#### 8.4 機械間直接接続
- [ ] 機械の出力ポートと別機械の入力ポートが隣接時、直接受け渡し
- [ ] 例: `[採掘機]Front → Back[精錬炉]`
- [ ] `game_spec::machine_connection_spec` の設定を使用

#### 8.5 レシピシステム統合（Single Source of Truth）
- [ ] `Furnace::get_smelt_output()` を `game_spec::recipe_spec::find_recipe()` に置き換え
- [ ] `BlockType::smelt_result()` を削除（重複ロジック排除）
- [ ] `Crusher` の2倍出力ロジックを `recipe_spec` から取得
- [ ] 燃料要件を `recipe_spec` から参照
- [ ] 確率副産物の処理（乱数で判定）

#### 8.6 機械のブロック描画（オプション）
- [ ] 機械を1x1x1キューブとして描画
- [ ] 向きに応じたテクスチャ回転
- [ ] 入出力ポートの視覚的表示

#### 8.7 テスト・動作確認
- [ ] 入出力方向の単体テスト
- [ ] 機械間直接接続のテスト
- [ ] レシピ検索・適用のテスト
- [ ] E2Eテスト: 採掘機→精錬炉→コンベア
- [ ] facing変換の境界値テスト（全4方向×全ポート位置）
- [ ] 確率出力の統計テスト（1000回実行で期待値±10%）

#### 8.7.1 デバッグコマンド追加
- [ ] `/debug_machine <pos>` - 機械の状態表示（facing, buffer, ports）
- [ ] `/debug_connection` - 全機械の接続状態をログ出力
- [ ] `/spawn_production_line` - テスト用に採掘機→精錬炉→納品PFラインを自動配置

#### 8.7.2 視覚的デバッグ（F3モード）
- [ ] 機械の入出力ポート方向を矢印で表示
- [ ] 接続状態を色分け（緑=接続OK、赤=不正接続、灰=未接続）
- [ ] 機械間直接接続を点線で可視化

#### 8.7.3 統計・モニタリング
- [ ] `/stats` コマンド - 各機械のスループット表示（個/分）
- [ ] 機械UIに処理速度表示（現在の処理速度 / 理論最大値）

#### 8.8 視覚フィードバック実装
- [ ] `miner_visual_feedback`を削除（パルスアニメーション廃止）
- [ ] 機械状態（Idle/Working）判定システム追加
- [ ] Working状態で発光マテリアル適用（機械別の色）
- [ ] Furnace/Crusherに煙パーティクル追加
- [ ] `game_spec::machine_rendering_spec` の設定を使用

#### 8.9 コンベア接続の方向判定
- [ ] 機械の入力ポートに接続できるコンベアの向きを検証
- [ ] 機械の出力ポートから出たアイテムの流れる方向を検証
- [ ] 不正接続時のUI警告（オプション）

---

## 依存関係（更新）

```
Phase 1 → Phase 2 → Phase 3 → Phase 4 → Phase 5 → Phase 6
  ↓                                                   ↓
GlobalInventory が全ての基盤                    Phase 7 (バイオーム)
                                                      ↓
                                                Phase 8 (機械入出力)
                                                      ↓
                                                  機械が仕様通りに動作
```

**Note**: Phase 8 は Phase 6 完了後に実装。ただし Phase 7 と並行可能。

---

## ユーザー確認チェックリスト

Phase 8 実装後にユーザーが確認すべき項目。

### 基本動作
- [ ] 機械設置時に向きが正しく反映されるか
- [ ] 採掘機→コンベア→精錬炉の基本ラインが動作するか
- [ ] 機械間直接接続（採掘機→精錬炉）が動作するか
- [ ] 燃料切れ時に精錬炉が停止するか

### 視覚フィードバック
- [ ] 稼働中の機械が発光しているか（マイクラ風）
- [ ] 精錬炉/粉砕機から煙が出ているか
- [ ] パルスアニメーションが削除されているか

### 入出力方向
- [ ] 機械の向きと入出力方向が直感的か
- [ ] 不正接続時に警告が表示されるか（オプション）
- [ ] F3デバッグモードで接続状態が確認できるか

### UI/UX
- [ ] 機械UIにスループットが表示されているか
- [ ] `/stats` コマンドで生産量が確認できるか
- [ ] `/debug_machine` で機械状態が確認できるか

### ゲームプレイ体験
- [ ] 機械の配置・接続が分かりやすいか
- [ ] 工場ライン構築が楽しいか
- [ ] 詰まり・ボトルネックの原因が分かりやすいか

---

## 承認待ち

「実装開始」の指示で Phase 1 から着手。
