# v0.2 実装計画: 全体在庫システム

**作成日**: 2026-01-01
**ステータス**: 計画完了、実装待ち

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

### Phase 1: 全体在庫システム基盤

#### 1.1 GlobalInventoryリソース作成
- [ ] `src/player/global_inventory.rs` 新規作成
- [ ] `GlobalInventory` リソース定義
  - `items: HashMap<BlockType, u32>`
  - `add_item()`, `remove_item()`, `get_count()` メソッド
- [ ] 初期化システム（game_spec::INITIAL_EQUIPMENT を追加）

#### 1.2 納品PFのアイテム受け入れ→全体在庫追加
- [ ] `DeliveryPlatform` にアイテムが入った時の処理変更
- [ ] `delivered` HashMap → `GlobalInventory` に追加
- [ ] クエスト進捗は `GlobalInventory` から計算

---

### Phase 2: 機械設置/撤去

#### 2.1 機械設置時に全体在庫から消費
- [ ] `block_placement` 関数を変更
- [ ] 設置前に `GlobalInventory.remove_item()` で在庫チェック
- [ ] 在庫不足時は設置不可（UIフィードバック）

#### 2.2 機械撤去時に全体在庫へ戻す
- [ ] `block_break` 関数を変更
- [ ] 機械ブロック破壊時に `GlobalInventory.add_item()` で戻す

---

### Phase 3: 既存システム置き換え

#### 3.1 既存Inventoryを全体在庫に置き換え
- [ ] `quest_claim_rewards` → `GlobalInventory` に報酬追加
- [ ] 初期装備付与 → `GlobalInventory` に追加
- [ ] 既存 `Inventory` リソースの使用箇所を全て置き換え
- [ ] 旧 `Inventory` を削除 or deprecated に

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

#### 6.2 テスト・動作確認
- [ ] 既存テストの更新（Inventory → GlobalInventory）
- [ ] E2Eテストで全体在庫動作確認
- [ ] クエスト納品フロー確認

---

### Phase 7: バイオーム採掘システム

#### 7.1 BiomeType定義とマップ生成
- [ ] `src/world/biome.rs` 新規作成
- [ ] `BiomeType` enum定義（Iron, Copper, Coal, Stone, Mixed）
- [ ] バイオームマップ生成（Perlinノイズ or Voronoi）
- [ ] 座標→バイオーム判定関数

#### 7.2 スポーン地点保証
- [ ] 納品PF周辺（半径15ブロック）に鉄・石炭・銅バイオームを強制配置
- [ ] シード値に関係なく初期エリアは固定配置
- [ ] バイオーム境界の可視化（デバッグ用）

#### 7.3 採掘機のバイオーム対応
- [ ] `miner_system` を変更
- [ ] 真下ブロック参照 → バイオーム確率テーブル参照
- [ ] `game_spec::biome_mining_spec` から確率取得
- [ ] 確率に基づいてアイテム生成

#### 7.4 採掘不可バイオーム対応
- [ ] ocean, lava, void バイオームでは採掘機が動作しない
- [ ] 設置時に警告表示
- [ ] 既存採掘機は停止状態に

#### 7.5 テスト・動作確認
- [ ] バイオーム判定の単体テスト
- [ ] 確率テーブルの合計100%検証
- [ ] スポーン保証のE2Eテスト
- [ ] 採掘アイテム分布の統計テスト

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
2. **セーブデータ互換性** - 旧セーブの Inventory データの移行
3. **UI実装工数** - マイクラ風UIは既存UIより複雑

---

## 承認待ち

「実装開始」の指示で Phase 1 から着手。
