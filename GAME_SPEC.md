# Infinite Voxel Factory - ゲーム仕様書

## 概要
Factorio、Satisfactory、Minecraft(Create Mod)の影響を受けた3Dボクセル工場シミュレーションゲーム。
敵やサバイバル要素なし、純粋な「自動化」と「建築」に特化。

**技術スタック:** Rust / Bevy Engine 0.15

---

## プレイヤー操作

### 移動
| 操作 | キー |
|------|------|
| 前進 | W |
| 後退 | S |
| 左移動 | A |
| 右移動 | D |
| 上昇 | Space |
| 下降 | Shift |
| ダッシュ | Ctrl |

### 視点操作
- マウス移動で視点回転
- 左クリックでカーソルロック
- Escapeでカーソル解放

### 設定値
- 歩行速度: 5.0 units/s
- 走行速度: 10.0 units/s
- マウス感度: 0.003
- 初期位置: (0, 5, 0)
- 目の高さ: 1.8 units

---

## 建築システム

### 機械の設置
- **1キー:** コンベア選択
- **2キー:** 採掘機選択
- **3キー:** 組立機選択
- **左クリック:** 設置
- **右クリック:** インタラクション

### 設置方向
機械はプレイヤーの向いている方向と逆向きに設置される（出力がプレイヤー側を向く）

---

## 機械一覧

### 1. コンベアベルト (Conveyor)
アイテムを一方向に搬送する。

| パラメータ | 値 |
|-----------|-----|
| 搬送速度 | 1.0 units/s |
| 最大アイテム数 | 4個 |
| 当たり判定 | 1.0 x 0.2 x 1.0 |

**動作:**
- アイテムは progress (0.0～1.0) で位置を管理
- progress >= 1.0 で次の機械へ転送
- 同一コンベア上のアイテム間隔: 0.25 (1/4)
- 右クリックで raw_ore を追加（デバッグ用）

### 2. 採掘機 (Miner)
無限に raw_ore を生成し搬出する。

| パラメータ | 値 |
|-----------|-----|
| 採掘速度 | 1.0 個/秒 |
| 出力アイテム | raw_ore |
| 当たり判定 | 1.0 x 1.0 x 1.0 |

**動作:**
- progress が 1.0 に達すると採掘完了
- 前方のコンベアにアイテムを自動搬出
- 搬出先が満杯の場合は待機

### 3. 組立機 (Assembler)
レシピに従ってアイテムを加工する。

| パラメータ | 値 |
|-----------|-----|
| 入力インベントリ | 最大10スロット |
| 出力インベントリ | 最大10スロット |
| 当たり判定 | 1.0 x 1.0 x 1.0 |

**動作:**
- 前面からアイテムを受け取り
- 背面へ完成品を排出
- 右クリックでUIを開きレシピを選択

---

## レシピシステム

### 定義形式 (YAML)
```yaml
- id: "ore_to_ingot"
  name: "Iron Ingot"
  inputs:
    - item: "raw_ore"
      count: 1
  outputs:
    - item: "ingot"
      count: 1
  craft_time: 2.0
```

### 現在のレシピ
| ID | 名前 | 入力 | 出力 | 加工時間 |
|----|------|------|------|---------|
| ore_to_ingot | Iron Ingot | raw_ore x1 | ingot x1 | 2.0秒 |

---

## アイテムシステム

### ItemSlot 構造
```
- item_id: アイテム識別子 (String)
- count: 個数 (u32)
- progress: コンベア上の位置 (f32, 0.0-1.0)
- unique_id: 一意識別子 (u64)
- from_direction: 搬入元方向 (Option<Direction>)
```

### 現在のアイテム
| ID | 説明 |
|----|------|
| raw_ore | 採掘機から産出される鉱石 |
| ingot | raw_ore を精錬したインゴット |

---

## UIシステム

### 機械UI (MachineUiPlugin)
- **開く:** 組立機を右クリック
- **閉じる:** Escapeキー または 閉じるボタン

### UI機能
- 現在のレシピ表示
- レシピ選択ボタン
- 入力/出力インベントリ表示（リアルタイム更新）

---

## 電力システム (Power Network)

### コンポーネント
| コンポーネント | 役割 |
|---------------|------|
| PowerSource | 電力供給 (capacity, current_speed) |
| PowerConsumer | 電力消費 (stress_impact, is_active) |
| Shaft | 動力伝達 (stress_resistance) |
| PowerNode | ネットワークノード (id, group_id) |

### 動作
1. 隣接する機械をグラフ構造で管理
2. 接続されたノードをグループ化 (BFS)
3. グループごとに応力計算
4. `total_stress > total_capacity` で過負荷
5. 過負荷時: consumer が停止、speed = 0

---

## マルチブロックシステム

### パターン定義
```
MultiblockPattern {
  id: String,
  name: String,
  size: [x, y, z],
  blocks: HashMap<"x,y,z", block_id>,
  master_offset: [x, y, z]
}
```

### コンポーネント
| コンポーネント | 役割 |
|---------------|------|
| MultiblockMaster | マスターブロック (pattern_id, slave_positions) |
| MultiblockSlave | スレーブブロック (master_pos) |
| FormedMultiblocks | 形成済み構造の管理 |

### イベント
- `MultiblockFormedEvent`: 構造形成時
- `MultiblockBrokenEvent`: 構造破壊時
- `ValidateStructureEvent`: 検証要求

---

## ブロック定義

### 形式 (YAML)
```yaml
- id: conveyor
  name: Conveyor Belt
  is_solid: true
  texture: "none"
  collision: [0.0, 0.0, 0.0, 1.0, 0.2, 1.0]
```

### 現在のブロック
| ID | 名前 | 当たり判定 |
|----|------|-----------|
| dirt | Dirt Block | 1x1x1 |
| stone | Stone Block | 1x1x1 |
| air | Air | なし |
| conveyor | Conveyor Belt | 1x0.2x1 |
| miner | Mining Drill | 1x1x1 |
| assembler | Assembler | 1x1x1 |

---

## ワールドシステム

### チャンク
- サイズ: 32 x 32 x 32 ブロック
- Greedy Meshing によるメッシュ最適化

### 座標系
- X: 東西 (+X = 東)
- Y: 上下 (+Y = 上)
- Z: 南北 (+Z = 南)

### 方向 (Direction)
| 方向 | ベクトル |
|------|---------|
| North | (0, 0, -1) |
| South | (0, 0, +1) |
| East | (+1, 0, 0) |
| West | (-1, 0, 0) |

---

## シミュレーション

### Fixed Timestep
- 更新間隔: 20TPS (50ms)
- 決定論的計算でマルチプレイ同期対応

### 処理順序
1. 入力処理
2. 建築処理
3. 機械シミュレーション (Conveyor → Miner → Assembler)
4. 電力計算
5. マルチブロック検証
6. レンダリング

---

## ファイル構成

```
src/
├── core/           # 設定、レジストリ、デバッグ
├── gameplay/       # ゲームロジック
│   ├── grid.rs     # グリッド管理
│   ├── building.rs # 建築
│   ├── player.rs   # プレイヤー
│   ├── power.rs    # 電力
│   ├── multiblock.rs # マルチブロック
│   └── machines/   # 機械
├── rendering/      # レンダリング
├── ui/             # UI
└── network/        # ネットワーク (スタブ)

assets/
├── data/
│   ├── blocks/     # ブロック定義
│   └── recipes/    # レシピ定義
└── textures/       # テクスチャ
```

---

## 今後の実装予定

### Phase 4: スクリプティング
- Lua VM (mlua) 統合
- サンドボックスAPI
- 信号システム

### Phase 5: 最適化
- マルチスレッド化
- LOD (Level of Detail)
- Modding SDK
