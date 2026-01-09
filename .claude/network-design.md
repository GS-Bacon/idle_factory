# リソースネットワーク基盤設計 (N.1-N.5)

## 概要

電力・液体・信号を統一的に扱う汎用ネットワーク基盤を実装する。

**設計方針**: Factorio Fluids 2.0 方式（セグメント統合）
- パイプ/電線を1つのセグメントとして扱い、即座に伝播
- セグメント内は「流れ」の概念なし（シンプル・高性能）

---

## データ構造

### リソース種別 (N.1) - 動的ID方式

```rust
/// ネットワーク種別ID（Mod拡張可能）
pub type NetworkTypeId = Id<NetworkTypeCategory>;

/// ネットワーク種別の仕様定義
#[derive(Clone, Debug)]
pub struct NetworkTypeSpec {
    pub id: NetworkTypeId,
    pub name: String,
    /// 貯蔵可能か（バッテリー、タンク）
    pub has_storage: bool,
    /// 値の型
    pub value_type: NetworkValueType,
    /// 伝播方式
    pub propagation: PropagationType,
    /// 導管の互換性グループ（同じグループなら接続可能）
    pub conduit_group: Option<String>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum NetworkValueType {
    Float,      // f32: 電力(W)、液体(mB)
    Discrete,   // u8: 信号(0-15)
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PropagationType {
    Instant,    // 即時伝播（電力、信号）
    Segment,    // セグメント内で均一化（液体）
    Distance,   // 距離減衰（Mod用: 熱など）
}

// base で定義される組み込みネットワーク種別
// mods/base/network_types.toml
```

**base の組み込み定義 (TOML)**:
```toml
# mods/base/network_types.toml

[[network_types]]
id = "base:power"
name = "電力"
has_storage = true
value_type = "Float"
propagation = "Instant"

[[network_types]]
id = "base:fluid"
name = "液体"
has_storage = true
value_type = "Float"
propagation = "Segment"
conduit_group = "pipe"

[[network_types]]
id = "base:gas"
name = "気体"
has_storage = true
value_type = "Float"
propagation = "Segment"
conduit_group = "pipe"  # 液体とパイプ共用可能

[[network_types]]
id = "base:signal"
name = "信号"
has_storage = false
value_type = "Discrete"
propagation = "Instant"
```

**Mod での拡張例**:
```toml
# mods/magic_mod/network_types.toml

[[network_types]]
id = "magic:mana"
name = "魔力"
has_storage = true
value_type = "Float"
propagation = "Distance"  # 距離で減衰
conduit_group = "mana_crystal"  # 専用導管
```

| 特性 | 電力 | 液体 | 信号 |
|------|------|------|------|
| 値の型 | f32 (W) | f32 (mB) | u8 (0-15) |
| 保存 | なし | あり（タンク） | なし |
| 分岐時 | 優先度順 | 容量比 | コピー |
| 減衰 | なし | なし | **Modで選択可** |

### 信号システム設計（ハイブリッド方式）

**base（デフォルト）**: セグメント方式、減衰なし
- シンプルな ON/OFF 的使い方
- セグメント内は即時伝播

**Mod拡張**: 減衰・比較器を追加可能
- `DecayWire`: 1ブロックごとに強度-1（マイクラ風）
- `Repeater`: 信号を15に増幅
- `Comparator`: 2つの信号を比較、差分出力

```rust
#[derive(Component)]
pub struct SignalNode {
    pub strength: u8,           // 0-15
    pub decay_per_block: u8,    // デフォルト0、Modで1に設定可
}

// Mod用: 減衰ワイヤー
#[derive(Component)]
pub struct DecayWire {
    pub decay_rate: u8,  // 通常1
}

// Mod用: リピーター（信号増幅）
#[derive(Component)]
pub struct SignalRepeater {
    pub delay_ticks: u8,  // 1-4
    pub output_strength: u8,  // 常に15
}

// Mod用: コンパレーター（比較器）
#[derive(Component)]
pub struct SignalComparator {
    pub mode: ComparatorMode,  // Compare or Subtract
}
```

**伝播アルゴリズム**:
1. セグメント内の最大強度を取得
2. 各ノードに伝播（DecayWire がある場合は距離計算）
3. Comparator/Repeater は別システムで処理

### セグメント構造 (N.2)

```rust
#[derive(Component)]
pub struct NetworkSegment {
    pub id: SegmentId,
    pub network_type: NetworkType,
    pub supply: f32,      // 供給量
    pub demand: f32,      // 需要量
    pub capacity: f32,    // 総容量（液体用）
    pub amount: f32,      // 現在量（液体用）
    pub nodes: Vec<Entity>,
    pub node_positions: HashMap<IVec3, Entity>,
}
```

### ノード種別 (N.3)

```rust
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum NodeRole {
    Producer,  // 発電機、ポンプ
    Consumer,  // 機械
    Storage,   // バッテリー、タンク
    Conduit,   // 電線、パイプ
}

#[derive(Component)]
pub struct NetworkPort {
    pub side: PortSide,
    pub network_type: NetworkType,
    pub role: NodeRole,
    pub segment_id: Option<SegmentId>,
}

#[derive(Component)]
pub struct PowerNode {
    pub role: NodeRole,
    pub power_watts: f32,
    pub satisfaction: f32,  // 0.0-1.0
    pub priority: i8,       // -10〜10
}
```

---

## アルゴリズム (N.4)

### セグメント検出（Flood Fill）

1. ブロック配置/破壊イベント発生
2. 変更位置の隣接セグメントを収集
3. 0個 → 新規セグメント、1個 → 追加、複数 → マージ
4. 破壊時は分割判定（再Flood Fill）

### 電力分配

```rust
fn distribute_power(segment: &mut NetworkSegment, nodes: &mut [PowerNode]) {
    let satisfaction = (segment.supply / segment.demand).clamp(0.0, 1.0);

    // 優先度順にソート → 高優先度から満たす
    nodes.sort_by_key(|n| -n.priority);
    let mut remaining = segment.supply;
    for node in nodes.iter_mut().filter(|n| n.role == Consumer) {
        if node.power_watts <= remaining {
            node.satisfaction = 1.0;
            remaining -= node.power_watts;
        } else {
            node.satisfaction = remaining / node.power_watts;
            remaining = 0.0;
        }
    }
}
```

### 液体分配（Factorio方式）

- Producer: セグメントに無制限プッシュ
- Consumer: 充填率 × 定格流量 でプル
- 長いパイプでも即時伝播（遅延なし）

---

## ファイル構成

### 新規ファイル

```
src/logistics/network/
├── mod.rs              # モジュール定義、NetworkPlugin
├── types.rs            # NetworkType, NodeRole, SegmentId
├── segment.rs          # NetworkSegment
├── node.rs             # NetworkPort, PowerNode, FluidNode, SignalNode
├── conduit.rs          # Wire, Pipe, SignalWire
├── registry.rs         # SegmentRegistry
├── detector.rs         # セグメント検出（Flood Fill）
└── distribution.rs     # 分配アルゴリズム
```

### 変更ファイル

| ファイル | 変更内容 |
|---------|---------|
| `src/game_spec/machines.rs` | IoPort に PortType 追加 |
| `src/machines/generic.rs` | 電力チェック追加 |
| `src/plugins/game.rs` | NetworkPlugin 登録 |
| `src/modding/handlers/mod.rs` | network API 追加 |

---

## 実装順序

### Phase 1: 型定義（1時間）

| ファイル | 内容 |
|---------|------|
| `network/mod.rs` | モジュール構造 |
| `network/types.rs` | NetworkType, NodeRole, SegmentId |
| `network/segment.rs` | NetworkSegment |
| `network/node.rs` | NetworkPort, PowerNode |

### Phase 2: セグメント管理（2時間）

| ファイル | 内容 |
|---------|------|
| `network/registry.rs` | SegmentRegistry |
| `network/detector.rs` | Flood Fill 実装 |
| `network/conduit.rs` | Wire コンポーネント |

### Phase 3: 分配アルゴリズム（2時間）

| ファイル | 内容 |
|---------|------|
| `network/distribution.rs` | 電力分配システム |
| テスト追加 | 分配ロジックの単体テスト |

### Phase 4: 統合（1時間）

| ファイル | 内容 |
|---------|------|
| `game_spec/machines.rs` | PortType 追加 |
| `machines/generic.rs` | 電力チェック |
| `plugins/game.rs` | Plugin 登録 |

### Phase 5: Mod API (N.5)（1時間）

| ファイル | 内容 |
|---------|------|
| `modding/handlers/network.rs` | WebSocket API |

---

## Mod 拡張ポイント (N.5)

### 設計方針: API統一

内部実装は分離（コンベア vs リソースネットワーク）だが、Mod APIは統一インターフェース。

| 内部システム | 対象 | 特徴 |
|-------------|------|------|
| コンベア (`logistics/conveyor.rs`) | アイテム | 方向性あり、位置あり、衝突判定 |
| リソースネットワーク (`logistics/network/`) | 電力/液体/信号/気体 | セグメント統合、即時伝播 |

### 統一 Logistics API

```json
// === アイテム輸送（コンベア系） ===

// アイテムをテレポート（Mod用：ワープパイプなど）
{ "method": "logistics.teleport_item", "params": {
    "from_entity": 12345,
    "to_entity": 67890,
    "item_id": "base:iron_ingot",
    "count": 10
}}

// コンベア上のアイテム一覧取得
{ "method": "logistics.get_conveyor_items", "params": { "entity": 12345 } }

// アイテムをコンベアに投入
{ "method": "logistics.insert_item", "params": {
    "entity": 12345,
    "item_id": "base:iron_ingot",
    "count": 1
}}

// === リソースネットワーク（電力/液体/信号） ===

// ネットワーク種別登録（気体、魔力など）
{ "method": "network.register_type", "params": { "id": "mymod:mana", ... } }

// セグメント情報取得
{ "method": "network.get_segment", "params": { "segment_id": 42 } }

// 電力状態取得
{ "method": "network.get_power_status", "params": { "entity": 12345 } }

// 仮想接続の登録（無線給電など用）
{ "method": "network.add_virtual_link", "params": {
    "from_pos": [10, 5, 20],
    "to_pos": [50, 5, 20],
    "network_type": "base:power",
    "bidirectional": true
}}

// 仮想接続の削除
{ "method": "network.remove_virtual_link", "params": { "link_id": 123 } }
```

### 仮想接続API（Mod用）

base は物理接続のみだが、Mod が無線給電などを実装できるよう API を用意：

```rust
/// 仮想接続（物理的に離れたノードを接続）
#[derive(Clone, Debug)]
pub struct VirtualLink {
    pub id: VirtualLinkId,
    pub from_pos: IVec3,
    pub to_pos: IVec3,
    pub network_type: NetworkTypeId,
    pub bidirectional: bool,
}

#[derive(Resource, Default)]
pub struct VirtualLinkRegistry {
    pub links: HashMap<VirtualLinkId, VirtualLink>,
}
```

**セグメント検出での扱い**:
- Flood Fill 時に `VirtualLinkRegistry` も参照
- 仮想接続先も隣接ノードとして扱う
- Mod が WebSocket API で登録/削除

**Mod実装例（無線給電）**:
1. Mod が送信機/受信機ブロックを追加
2. 配置時に WebSocket で `network.add_virtual_link` 呼び出し
3. 破壊時に `network.remove_virtual_link` 呼び出し

### イベント

- `SegmentFormed` - セグメント形成
- `SegmentBroken` - セグメント分割/消滅
- `PowerShortage` - 電力不足発生
- `VirtualLinkAdded` - 仮想接続追加
- `VirtualLinkRemoved` - 仮想接続削除

---

## テスト計画

### 単体テスト

- `test_segment_flood_fill` - 直線状電線 → 1セグメント
- `test_segment_merge` - 2セグメント接続 → マージ
- `test_segment_split` - 中央破壊 → 2セグメント
- `test_power_distribution_priority` - 優先度順分配

### シナリオテスト

```toml
# tests/scenarios/power_grid_basic.toml
name = "電力グリッド基本"
[[steps]]
action = "spawn_machine"
params = { type = "generator", position = [5, 8, 5] }
# ... 電線接続 ...
[[steps]]
action = "assert"
params = { condition = "machine_powered(7,8,5) == true" }
```

---

## 見積もり

| Phase | 時間 |
|-------|------|
| Phase 1: 型定義 | 1h |
| Phase 2: セグメント管理 | 2h |
| Phase 3: 分配アルゴリズム | 2h |
| Phase 4: 統合 | 1h |
| Phase 5: Mod API | 1h |
| **合計** | **7h** |

---

## 次のステップ（M3: 電力システム）

N.1-N.5 完了後:
1. 電線ブロック追加（BlockType、モデル）
2. 発電機追加（水車、石炭発電）
3. 電力消費機械の設定（MachineSpec.power_consumption）
4. 電力UI（セグメント状態表示）
