# v0.2 レビュー依頼

## 概要
v0.2「全体在庫システム」を実装しました。以下の観点でレビューをお願いします：

1. **アーキテクチャ**: GlobalInventoryの設計は適切か
2. **コード品質**: unwrap使用、エラーハンドリング、命名規則
3. **UI/UX**: 各UIがダサくないか、Minecraft風UIとして適切か

## 実装内容

### 1. GlobalInventory (src/player/global_inventory.rs)
- HashMap<BlockType, u32>で機械を一元管理
- add_item, remove_item, get_count, clear, iter メソッド

### 2. バイオーム採掘 (src/world/biome.rs)
- BiomeType: Iron, Copper, Coal, Stone, Mixed, Unmailable
- 座標ベースハッシュ関数でバイオーム判定
- スポーン地点保証（半径15ブロック）

### 3. UI追加
- GlobalInventory表示パネル（インベントリUI内）
- クエスト納品ボタン

### 4. 機械facing
- Miner, Furnace, Crusherにfacing: Direction追加
- 設置時にプレイヤーの向きを反映

## レビュー対象ファイル
- src/player/global_inventory.rs (新規)
- src/world/biome.rs (新規)
- src/setup/ui/inventory_ui.rs (GlobalInventoryパネル追加)
- src/setup/ui/mod.rs (クエスト納品ボタン追加)
- src/systems/quest.rs (納品ボタン処理)
- src/components/machines.rs (facing追加)

## 質問
1. GlobalInventoryの設計に問題はありますか？
2. バイオームシステムの実装は適切ですか？
3. UIのスタイリング（色、サイズ、レイアウト）はMinecraft風として適切ですか？ダサくないですか？
4. 改善提案があればお願いします
