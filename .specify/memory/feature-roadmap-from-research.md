# 機能ロードマップ（ゲーム調査結果より）

調査日: 2025-12-25
調査対象: Minecraft, Unturned, Satisfactory, Shapez2, Factorio, パルワールド

---

## 高優先度タスク ⭐

### 1. 両側レーン付きベルト
**出典**: Factorio
**概要**: 1本のベルトで2種類のアイテムを同時搬送
**必要アイテム/ブロック**:
- なし（既存ベルトの拡張）
**必要モデル**:
- なし（既存モデルを使用）
**実装内容**:
- `ConveyorBelt`コンポーネントに`left_lane`/`right_lane`追加
- サイドローディング（横から合流時に片側レーンに挿入）
- UI: ベルト内容の左右表示

---

### 2. スマートスプリッター
**出典**: Satisfactory
**概要**: 条件付きアイテム分配
**必要アイテム**:
- `smart_splitter` - スマートスプリッター (Tier 3)
- `programmable_splitter` - プログラマブルスプリッター (Tier 4)
**必要モデル**:
- `smart_splitter.glb` - 3方向出力付きスプリッター、信号ランプ付き
- `programmable_splitter.glb` - 液晶画面付き高度スプリッター
**実装内容**:
- フィルタルール: Any, None, Overflow, Item指定
- 出力ポートごとに個別ルール設定
- GUI: ルール設定UI

---

### 3. 電力グリッド過負荷・ヒューズ
**出典**: Satisfactory
**概要**: 供給不足時の全建物停止、復旧時の電力スパイク
**必要アイテム**:
- `circuit_breaker` - 回路遮断機 (Tier 3)
- `power_switch` - 電力スイッチ (Tier 2)
**必要モデル**:
- `circuit_breaker.glb` - レバー付きボックス型
- `power_switch.glb` - 壁掛けスイッチ型
**実装内容**:
- `PowerNetwork`に`is_tripped`フラグ追加
- 過負荷検出ロジック
- 手動リセット機能
- 復旧時の電力スパイクシミュレーション

---

### 4. オーバークロック/アンダークロック
**出典**: Satisfactory
**概要**: マシン速度を1%-250%で調整
**必要アイテム**:
- `power_shard` - パワーシャード (Tier 3, レア)
- `power_shard_uncommon` - パワーシャード (Tier 2)
**必要モデル**:
- `power_shard.glb` - 発光するクリスタル型（複数色）
**実装内容**:
- `MachineInstance`に`clock_speed: f32`追加
- 電力消費の非線形計算: `power * (clock_speed ^ 1.6)`
- GUI: スライダーUI
- 250%以上にはPower Shard消費

---

### 5. ハードドライブ・代替レシピシステム
**出典**: Satisfactory
**概要**: 探索で発見→研究→代替レシピ解放
**必要アイテム**:
- `hard_drive` - ハードドライブ (Tier 3, レア)
**必要モデル**:
- `hard_drive.glb` - 破損したHDD型、発光ライト付き
**実装内容**:
- `AlternateRecipe`システム追加
- MAM風研究UI
- レシピ選択（3択から1選択）
- ハードドライブライブラリ（保存して確率調整）

---

### 6. 品質システム
**出典**: Factorio 2.0 Space Age
**概要**: 5段階の品質ティアでアイテム強化
**必要アイテム**:
- `quality_module_1` - 品質モジュール T1 (Tier 3)
- `quality_module_2` - 品質モジュール T2 (Tier 4)
- `quality_module_3` - 品質モジュール T3 (Tier 4)
**必要モデル**:
- `quality_module_1.glb` - 緑色のモジュール
- `quality_module_2.glb` - 青色のモジュール
- `quality_module_3.glb` - 紫色のモジュール
**実装内容**:
- `ItemQuality`列挙型: Normal, Uncommon(+30%), Rare(+60%), Epic(+90%), Legendary(+150%)
- クラフト時に品質モジュールで確率アップグレード
- 高品質アイテムは性能向上

---

### 7. ロジスティクスロボット
**出典**: Factorio
**概要**: 自動アイテム輸送ネットワーク
**必要アイテム**:
- `roboport` - ロボポート (Tier 3)
- `logistics_robot` - ロジスティクスロボット (Tier 3)
- `construction_robot` - 建設ロボット (Tier 3)
- `provider_chest` - プロバイダーチェスト (Tier 3)
- `requester_chest` - リクエスターチェスト (Tier 3)
- `storage_chest` - ストレージチェスト (Tier 2)
- `buffer_chest` - バッファーチェスト (Tier 3)
**必要モデル**:
- `roboport.glb` - 充電ステーション型、アンテナ付き
- `logistics_robot.glb` - 小型ドローン型
- `construction_robot.glb` - 工具付きドローン型
- `provider_chest.glb` - 黄色チェスト
- `requester_chest.glb` - 青色チェスト
- `storage_chest.glb` - 茶色チェスト
- `buffer_chest.glb` - 緑色チェスト
**実装内容**:
- `LogisticsNetwork`システム
- ロボポート範囲（50x50ロジスティクス、110x110建設）
- チェスト優先度システム
- ロボット充電・移動シミュレーション

---

### 8. ブループリント設計台
**出典**: Satisfactory/Shapez2
**概要**: 工場の一部を設計図として保存・複製
**必要アイテム**:
- `blueprint_designer_mk1` - ブループリント設計台 Mk1 (Tier 3)
- `blueprint_designer_mk2` - ブループリント設計台 Mk2 (Tier 4)
- `blueprint` - ブループリント（生成アイテム）
**必要モデル**:
- `blueprint_designer_mk1.glb` - 32m立方体サイズ表示付きテーブル
- `blueprint_designer_mk2.glb` - 大型テーブル、ホログラム表示
**実装内容**:
- 選択範囲のシリアライズ
- ブループリント保存/読み込み
- コピー&ペースト機能
- 共有機能（ファイルエクスポート）

---

### 9. AWESOME Sink報酬システム
**出典**: Satisfactory
**概要**: 余剰アイテムをポイント変換→クーポン→報酬
**必要アイテム**:
- `awesome_sink` - AWESOME Sink (Tier 2)
- `coupon` - FICSIT クーポン（通貨アイテム）
**必要モデル**:
- `awesome_sink.glb` - 大型シュレッダー/投入口型
**実装内容**:
- アイテム→ポイント変換テーブル
- ポイント→クーポン変換（指数的増加）
- クーポンショップUI
- 報酬: コスメティック、車両設計図、特殊アイテム

---

## 中優先度タスク △

### 10. ヘッドリフト・ポンプシステム
**出典**: Satisfactory
**必要アイテム**:
- `pipeline_pump` - パイプラインポンプ (Tier 2)
- `fluid_buffer` - 流体バッファ (Tier 2)
**必要モデル**:
- `pipeline_pump.glb` - インライン型ポンプ
- `fluid_buffer.glb` - 大型タンク

---

### 11. 発電機ティアシステム
**出典**: Satisfactory/Factorio
**必要アイテム**:
- `biomass_burner` - バイオマス燃焼機 (Tier 1, 30MW)
- `coal_generator` - 石炭発電機 (Tier 2, 75MW)
- `fuel_generator` - 燃料発電機 (Tier 3, 250MW)
- `nuclear_reactor` - 原子力発電機 (Tier 4, 2500MW)
- `biomass` - バイオマス燃料
- `turbofuel_bucket` - ターボ燃料
**必要モデル**:
- `biomass_burner.glb` - 小型燃焼炉
- `coal_generator.glb` - 煙突付き発電機
- `fuel_generator.glb` - 大型エンジン型
- `nuclear_reactor.glb` - 冷却塔付き大型施設

---

### 12. パル労働システム風クリーチャー自動化
**出典**: パルワールド
**必要アイテム**:
- `creature_box` - クリーチャーボックス (Tier 2)
- `work_bench` - 作業指示台 (Tier 2)
- `feed_box` - エサ箱 (Tier 1)
- `hot_spring` - 温泉 (Tier 2)
**必要モデル**:
- `creature_box.glb` - 拠点中心マーカー
- `work_bench.glb` - 作業台
- `feed_box.glb` - エサ容器
- `hot_spring.glb` - 小型温泉

---

### 13. カラールーティング列車
**出典**: Shapez2
**必要アイテム**:
- `train_station` - 列車ステーション (Tier 3)
- `train_launcher` - 列車発射台 (Tier 3)
- `locomotive` - 機関車 (各色)
- `cargo_wagon` - 貨物車両
- `rail` - レール
**必要モデル**:
- `train_station.glb` - プラットフォーム型
- `train_launcher.glb` - 発射機構付き
- `locomotive.glb` - ローポリ機関車
- `cargo_wagon.glb` - 貨物コンテナ付き車両

---

### 14. Global Transmitter/Receiver
**出典**: Shapez2
**必要アイテム**:
- `global_transmitter` - グローバル送信機 (Tier 3)
- `global_receiver` - グローバル受信機 (Tier 3)
**必要モデル**:
- `global_transmitter.glb` - アンテナ付き送信機
- `global_receiver.glb` - アンテナ付き受信機

---

## ユニーク仕様メモ（保留）

### Shapez2: 重力ルールとクリスタルの脆弱性
- シェイプに重力が適用され、サポートなしのパーツは落下
- クリスタルは隣接クリスタルと連鎖して破壊される脆弱素材
- **検討**: ブロック/アイテムの物理シミュレーション拡張として

### Satisfactory: 生産増幅器（Somersloop）
- 全スロット満杯 + 250%オーバークロック = 通常5台分の出力
- リソース消費は半分
- **検討**: エンドゲームのスーパーパワーアイテムとして

### Minecraft 1.21: 銅バルブによるTフリップフロップ
- 2ブロックでTフリップフロップ実装可能
- **検討**: 信号システムのシンプル化、新しいロジックブロック

### パルワールド: パル手術台
- 後天的にパッシブスキルを追加可能
- **検討**: クリーチャーシステム実装時に検討

### Factorio: ビーコン減衰システム
- 複数ビーコンの効果が1/√nで減衰
- **検討**: モジュール/ビーコンシステムの深み追加

---

## 必要アイテム一覧（まとめ）

### 高優先度で追加が必要なアイテム

| ID | 名前 | Tier | カテゴリ |
|----|------|------|---------|
| smart_splitter | スマートスプリッター | 3 | machine |
| programmable_splitter | プログラマブルスプリッター | 4 | machine |
| circuit_breaker | 回路遮断機 | 3 | machine |
| power_switch | 電力スイッチ | 2 | machine |
| power_shard | パワーシャード | 3 | component |
| hard_drive | ハードドライブ | 3 | component |
| quality_module_1 | 品質モジュールT1 | 3 | module |
| quality_module_2 | 品質モジュールT2 | 4 | module |
| quality_module_3 | 品質モジュールT3 | 4 | module |
| roboport | ロボポート | 3 | machine |
| logistics_robot | ロジスティクスロボット | 3 | robot |
| construction_robot | 建設ロボット | 3 | robot |
| provider_chest | プロバイダーチェスト | 3 | storage |
| requester_chest | リクエスターチェスト | 3 | storage |
| storage_chest | ストレージチェスト | 2 | storage |
| buffer_chest | バッファーチェスト | 3 | storage |
| blueprint_designer_mk1 | ブループリント設計台Mk1 | 3 | machine |
| blueprint_designer_mk2 | ブループリント設計台Mk2 | 4 | machine |
| awesome_sink | AWESOME Sink | 2 | machine |
| coupon | クーポン | 1 | currency |

### 中優先度で追加が必要なアイテム

| ID | 名前 | Tier | カテゴリ |
|----|------|------|---------|
| pipeline_pump | パイプラインポンプ | 2 | machine |
| fluid_buffer | 流体バッファ | 2 | machine |
| biomass_burner | バイオマス燃焼機 | 1 | generator |
| coal_generator | 石炭発電機 | 2 | generator |
| fuel_generator | 燃料発電機 | 3 | generator |
| nuclear_reactor | 原子力発電機 | 4 | generator |
| biomass | バイオマス | 1 | fuel |
| turbofuel_bucket | ターボ燃料 | 3 | fuel |
| creature_box | クリーチャーボックス | 2 | machine |
| work_bench | 作業指示台 | 2 | machine |
| feed_box | エサ箱 | 1 | machine |
| hot_spring | 温泉 | 2 | decoration |
| train_station | 列車ステーション | 3 | logistics |
| train_launcher | 列車発射台 | 3 | logistics |
| locomotive | 機関車 | 3 | vehicle |
| cargo_wagon | 貨物車両 | 3 | vehicle |
| rail | レール | 2 | logistics |
| global_transmitter | グローバル送信機 | 3 | signal |
| global_receiver | グローバル受信機 | 3 | signal |

---

## 必要モデル一覧（まとめ）

### 高優先度モデル

| ファイル名 | 説明 | サイズ |
|-----------|------|--------|
| smart_splitter.glb | 3方向出力、信号ランプ付き | 1x1x1 |
| programmable_splitter.glb | 液晶画面付き | 1x1x1 |
| circuit_breaker.glb | レバー付きボックス | 0.5x0.5x0.5 |
| power_switch.glb | 壁掛けスイッチ | 0.3x0.3x0.1 |
| power_shard.glb | 発光クリスタル | 0.3x0.3x0.3 |
| hard_drive.glb | 破損HDD型 | 0.3x0.3x0.1 |
| quality_module_1.glb | 緑色モジュール | 0.3x0.3x0.3 |
| quality_module_2.glb | 青色モジュール | 0.3x0.3x0.3 |
| quality_module_3.glb | 紫色モジュール | 0.3x0.3x0.3 |
| roboport.glb | 充電ステーション、アンテナ付き | 2x2x2 |
| logistics_robot.glb | 小型ドローン | 0.3x0.3x0.3 |
| construction_robot.glb | 工具付きドローン | 0.3x0.3x0.3 |
| provider_chest.glb | 黄色チェスト | 1x1x1 |
| requester_chest.glb | 青色チェスト | 1x1x1 |
| storage_chest.glb | 茶色チェスト | 1x1x1 |
| buffer_chest.glb | 緑色チェスト | 1x1x1 |
| blueprint_designer_mk1.glb | 設計テーブル | 2x1x2 |
| blueprint_designer_mk2.glb | 大型テーブル、ホログラム | 3x2x3 |
| awesome_sink.glb | 大型シュレッダー | 2x2x2 |

### 中優先度モデル

| ファイル名 | 説明 | サイズ |
|-----------|------|--------|
| pipeline_pump.glb | インライン型ポンプ | 1x1x1 |
| fluid_buffer.glb | 大型タンク | 2x3x2 |
| biomass_burner.glb | 小型燃焼炉 | 1x1x1 |
| coal_generator.glb | 煙突付き発電機 | 2x2x2 |
| fuel_generator.glb | 大型エンジン | 3x2x3 |
| nuclear_reactor.glb | 冷却塔付き大型施設 | 5x5x5 |
| creature_box.glb | 拠点中心マーカー | 1x1x1 |
| work_bench.glb | 作業台 | 1x1x1 |
| feed_box.glb | エサ容器 | 0.5x0.5x0.5 |
| hot_spring.glb | 小型温泉 | 2x1x2 |
| train_station.glb | プラットフォーム | 4x2x2 |
| train_launcher.glb | 発射機構 | 2x2x2 |
| locomotive.glb | ローポリ機関車 | 3x2x2 |
| cargo_wagon.glb | 貨物コンテナ | 3x2x2 |
| global_transmitter.glb | アンテナ付き送信機 | 1x2x1 |
| global_receiver.glb | アンテナ付き受信機 | 1x2x1 |
