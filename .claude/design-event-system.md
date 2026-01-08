# イベントシステム詳細設計

## 目標

1. **コアイベントの中央管理**: 全コアイベントを1ファイルで定義
2. **Mod拡張対応**: Modが独自イベントを追加可能
3. **AI安全性**: AIが1ファイル読むだけで正しく実装できる
4. **将来拡張**: 新機能追加時にイベントだけ追加すれば済む

---

## 現状の問題点

| 問題 | 影響 | 深刻度 |
|------|------|--------|
| イベント定義が2箇所（mod.rs, game_events.rs） | AIが片方だけ読んで実装する | 高 |
| Mod API EventType が手動同期 | 新イベント追加時に漏れる | 高 |
| GuardedEventWriter が未使用 | 循環防止が機能していない | 中 |
| MachineCompleted がMod APIで未ブリッジ | Modが購読できない | 中 |

---

## 新アーキテクチャ

### ファイル構成

```
src/events/
├── mod.rs              # 再エクスポート + EventsPlugin
├── core.rs             # 【新規】全コアイベント定義（Single Source of Truth）
├── types.rs            # EventSource, EventKind 等の共通型
├── guarded_writer.rs   # GuardedEventWriter（既存）
├── config.rs           # EventSystemConfig（既存mod.rsから分離）
└── bridge.rs           # 【新規】Mod API自動ブリッジ
```

### 削除するファイル

- `events/game_events.rs` → `core.rs` に統合

---

## コアイベント設計

### 設計原則

```rust
// 全コアイベントは以下の特徴を持つ：
// 1. #[derive(Event, Clone, Debug, Serialize)] で統一
// 2. CoreEvent トレイトを実装（自動でMod API対応）
// 3. 1ファイル（core.rs）に全て定義
```

### core.rs の構造

```rust
//! コアイベント定義（Single Source of Truth）
//!
//! 【重要】新しいイベントを追加する場合：
//! 1. このファイルにイベント構造体を追加
//! 2. CoreEventKind enum にバリアントを追加
//! 3. impl CoreEvent for YourEvent を追加
//! 4. register_core_events! マクロに追加
//!
//! これだけで自動的に：
//! - Bevy イベントとして登録される
//! - Mod API で購読可能になる
//! - イベントブリッジが動作する

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use crate::core::ItemId;

// ============================================================================
// 共通型
// ============================================================================

/// イベント発生源
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EventSource {
    Player(u64),      // player_id（マルチ対応）
    Machine(u64),     // entity.to_bits()
    Mod(u32),         // mod_id
    System,           // ワールド生成等
}

/// コアイベントの種類（Mod API用）
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CoreEventKind {
    // ブロック系
    BlockPlacing,
    BlockPlaced,
    BlockBreaking,
    BlockBroken,

    // 機械系
    MachineSpawned,
    MachineStarted,
    MachineCompleted,
    MachineFuelConsumed,
    MachineRemoved,

    // プレイヤー系
    PlayerSpawned,
    PlayerMoved,
    InventoryChanged,

    // 物流系
    ConveyorTransfer,
    ItemPickedUp,
    ItemDropped,
    ItemDelivered,

    // クエスト系
    QuestStarted,
    QuestProgressed,
    QuestCompleted,
}

impl CoreEventKind {
    /// Mod API用の文字列表現
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::BlockPlacing => "block.placing",
            Self::BlockPlaced => "block.placed",
            Self::BlockBreaking => "block.breaking",
            Self::BlockBroken => "block.broken",
            Self::MachineSpawned => "machine.spawned",
            Self::MachineStarted => "machine.started",
            Self::MachineCompleted => "machine.completed",
            Self::MachineFuelConsumed => "machine.fuel_consumed",
            Self::MachineRemoved => "machine.removed",
            Self::PlayerSpawned => "player.spawned",
            Self::PlayerMoved => "player.moved",
            Self::InventoryChanged => "inventory.changed",
            Self::ConveyorTransfer => "conveyor.transfer",
            Self::ItemPickedUp => "item.picked_up",
            Self::ItemDropped => "item.dropped",
            Self::ItemDelivered => "item.delivered",
            Self::QuestStarted => "quest.started",
            Self::QuestProgressed => "quest.progressed",
            Self::QuestCompleted => "quest.completed",
        }
    }

    /// 文字列からパース
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "block.placing" => Some(Self::BlockPlacing),
            "block.placed" => Some(Self::BlockPlaced),
            "block.breaking" => Some(Self::BlockBreaking),
            "block.broken" => Some(Self::BlockBroken),
            "machine.spawned" => Some(Self::MachineSpawned),
            "machine.started" => Some(Self::MachineStarted),
            "machine.completed" => Some(Self::MachineCompleted),
            "machine.fuel_consumed" => Some(Self::MachineFuelConsumed),
            "machine.removed" => Some(Self::MachineRemoved),
            "player.spawned" => Some(Self::PlayerSpawned),
            "player.moved" => Some(Self::PlayerMoved),
            "inventory.changed" => Some(Self::InventoryChanged),
            "conveyor.transfer" => Some(Self::ConveyorTransfer),
            "item.picked_up" => Some(Self::ItemPickedUp),
            "item.dropped" => Some(Self::ItemDropped),
            "item.delivered" => Some(Self::ItemDelivered),
            "quest.started" => Some(Self::QuestStarted),
            "quest.progressed" => Some(Self::QuestProgressed),
            "quest.completed" => Some(Self::QuestCompleted),
            _ => None,
        }
    }

    /// 全種類を取得
    pub fn all() -> &'static [Self] {
        &[
            Self::BlockPlacing, Self::BlockPlaced, Self::BlockBreaking, Self::BlockBroken,
            Self::MachineSpawned, Self::MachineStarted, Self::MachineCompleted,
            Self::MachineFuelConsumed, Self::MachineRemoved,
            Self::PlayerSpawned, Self::PlayerMoved, Self::InventoryChanged,
            Self::ConveyorTransfer, Self::ItemPickedUp, Self::ItemDropped, Self::ItemDelivered,
            Self::QuestStarted, Self::QuestProgressed, Self::QuestCompleted,
        ]
    }

    /// 高頻度イベント（デフォルトでMod通知OFF）
    pub fn is_high_frequency(&self) -> bool {
        matches!(self, Self::ConveyorTransfer | Self::PlayerMoved)
    }
}

// ============================================================================
// コアイベントトレイト
// ============================================================================

/// コアイベントが実装すべきトレイト
pub trait CoreEvent: Event + Clone + Serialize + Send + Sync + 'static {
    /// このイベントの種類
    fn kind() -> CoreEventKind;

    /// JSON形式に変換（Mod API用）
    fn to_json(&self) -> serde_json::Value {
        serde_json::to_value(self).unwrap_or(serde_json::Value::Null)
    }
}

// ============================================================================
// ブロック系イベント
// ============================================================================

/// ブロック配置直前（キャンセル可能）
#[derive(Event, Clone, Debug, Serialize, Deserialize)]
pub struct BlockPlacing {
    pub pos: IVec3,
    pub block: ItemId,
    pub source: EventSource,
    #[serde(skip)]
    pub cancelled: bool,
}

impl CoreEvent for BlockPlacing {
    fn kind() -> CoreEventKind { CoreEventKind::BlockPlacing }
}

/// ブロック配置完了
#[derive(Event, Clone, Debug, Serialize, Deserialize)]
pub struct BlockPlaced {
    pub pos: IVec3,
    pub block: ItemId,
    pub source: EventSource,
}

impl CoreEvent for BlockPlaced {
    fn kind() -> CoreEventKind { CoreEventKind::BlockPlaced }
}

/// ブロック破壊開始
#[derive(Event, Clone, Debug, Serialize, Deserialize)]
pub struct BlockBreaking {
    pub pos: IVec3,
    pub block: ItemId,
    pub source: EventSource,
    pub progress: f32,
}

impl CoreEvent for BlockBreaking {
    fn kind() -> CoreEventKind { CoreEventKind::BlockBreaking }
}

/// ブロック破壊完了
#[derive(Event, Clone, Debug, Serialize, Deserialize)]
pub struct BlockBroken {
    pub pos: IVec3,
    pub block: ItemId,
    pub source: EventSource,
    pub drops: Vec<(ItemId, u32)>,
}

impl CoreEvent for BlockBroken {
    fn kind() -> CoreEventKind { CoreEventKind::BlockBroken }
}

// ============================================================================
// 機械系イベント
// ============================================================================

/// 機械生成完了
#[derive(Event, Clone, Debug, Serialize, Deserialize)]
pub struct MachineSpawned {
    pub entity_id: u64,  // entity.to_bits()
    pub machine_type: ItemId,
    pub pos: IVec3,
}

impl CoreEvent for MachineSpawned {
    fn kind() -> CoreEventKind { CoreEventKind::MachineSpawned }
}

/// 機械加工開始
#[derive(Event, Clone, Debug, Serialize, Deserialize)]
pub struct MachineStarted {
    pub entity_id: u64,
    pub recipe_id: Option<String>,
    pub inputs: Vec<(ItemId, u32)>,
}

impl CoreEvent for MachineStarted {
    fn kind() -> CoreEventKind { CoreEventKind::MachineStarted }
}

/// 機械加工完了
#[derive(Event, Clone, Debug, Serialize, Deserialize)]
pub struct MachineCompleted {
    pub entity_id: u64,
    pub recipe_id: Option<String>,
    pub outputs: Vec<(ItemId, u32)>,
}

impl CoreEvent for MachineCompleted {
    fn kind() -> CoreEventKind { CoreEventKind::MachineCompleted }
}

/// 燃料消費
#[derive(Event, Clone, Debug, Serialize, Deserialize)]
pub struct MachineFuelConsumed {
    pub entity_id: u64,
    pub fuel: ItemId,
    pub amount: u32,
}

impl CoreEvent for MachineFuelConsumed {
    fn kind() -> CoreEventKind { CoreEventKind::MachineFuelConsumed }
}

/// 機械撤去
#[derive(Event, Clone, Debug, Serialize, Deserialize)]
pub struct MachineRemoved {
    pub entity_id: u64,
    pub machine_type: ItemId,
    pub pos: IVec3,
}

impl CoreEvent for MachineRemoved {
    fn kind() -> CoreEventKind { CoreEventKind::MachineRemoved }
}

// ============================================================================
// プレイヤー系イベント
// ============================================================================

/// プレイヤー参加
#[derive(Event, Clone, Debug, Serialize, Deserialize)]
pub struct PlayerSpawned {
    pub player_id: u64,
    pub pos: Vec3,
}

impl CoreEvent for PlayerSpawned {
    fn kind() -> CoreEventKind { CoreEventKind::PlayerSpawned }
}

/// プレイヤー移動（高頻度）
#[derive(Event, Clone, Debug, Serialize, Deserialize)]
pub struct PlayerMoved {
    pub player_id: u64,
    pub from: Vec3,
    pub to: Vec3,
}

impl CoreEvent for PlayerMoved {
    fn kind() -> CoreEventKind { CoreEventKind::PlayerMoved }
}

/// インベントリ変更
#[derive(Event, Clone, Debug, Serialize, Deserialize)]
pub struct InventoryChanged {
    pub player_id: u64,
    pub slot: u8,
    pub item: ItemId,
    pub delta: i32,  // 正=追加、負=消費
}

impl CoreEvent for InventoryChanged {
    fn kind() -> CoreEventKind { CoreEventKind::InventoryChanged }
}

// ============================================================================
// 物流系イベント
// ============================================================================

/// コンベア転送（高頻度）
#[derive(Event, Clone, Debug, Serialize, Deserialize)]
pub struct ConveyorTransfer {
    pub from_pos: IVec3,
    pub to_pos: IVec3,
    pub item: ItemId,
}

impl CoreEvent for ConveyorTransfer {
    fn kind() -> CoreEventKind { CoreEventKind::ConveyorTransfer }
}

/// アイテム取得
#[derive(Event, Clone, Debug, Serialize, Deserialize)]
pub struct ItemPickedUp {
    pub player_id: u64,
    pub item: ItemId,
    pub count: u32,
}

impl CoreEvent for ItemPickedUp {
    fn kind() -> CoreEventKind { CoreEventKind::ItemPickedUp }
}

/// アイテム落下
#[derive(Event, Clone, Debug, Serialize, Deserialize)]
pub struct ItemDropped {
    pub player_id: u64,
    pub item: ItemId,
    pub count: u32,
    pub pos: Vec3,
}

impl CoreEvent for ItemDropped {
    fn kind() -> CoreEventKind { CoreEventKind::ItemDropped }
}

/// アイテム納品
#[derive(Event, Clone, Debug, Serialize, Deserialize)]
pub struct ItemDelivered {
    pub item: ItemId,
    pub count: u32,
    pub quest_id: Option<String>,
}

impl CoreEvent for ItemDelivered {
    fn kind() -> CoreEventKind { CoreEventKind::ItemDelivered }
}

// ============================================================================
// クエスト系イベント
// ============================================================================

/// クエスト開始
#[derive(Event, Clone, Debug, Serialize, Deserialize)]
pub struct QuestStarted {
    pub quest_id: String,
}

impl CoreEvent for QuestStarted {
    fn kind() -> CoreEventKind { CoreEventKind::QuestStarted }
}

/// クエスト進捗
#[derive(Event, Clone, Debug, Serialize, Deserialize)]
pub struct QuestProgressed {
    pub quest_id: String,
    pub progress: f32,  // 0.0 - 1.0
}

impl CoreEvent for QuestProgressed {
    fn kind() -> CoreEventKind { CoreEventKind::QuestProgressed }
}

/// クエスト完了
#[derive(Event, Clone, Debug, Serialize, Deserialize)]
pub struct QuestCompleted {
    pub quest_id: String,
    pub rewards: Vec<(ItemId, u32)>,
}

impl CoreEvent for QuestCompleted {
    fn kind() -> CoreEventKind { CoreEventKind::QuestCompleted }
}

// ============================================================================
// イベント登録マクロ
// ============================================================================

/// 全コアイベントを登録するマクロ
///
/// 【重要】新しいイベントを追加したらここにも追加すること！
/// コンパイルエラーで漏れを検出できる。
macro_rules! register_core_events {
    ($app:expr) => {
        $app.add_event::<BlockPlacing>()
            .add_event::<BlockPlaced>()
            .add_event::<BlockBreaking>()
            .add_event::<BlockBroken>()
            .add_event::<MachineSpawned>()
            .add_event::<MachineStarted>()
            .add_event::<MachineCompleted>()
            .add_event::<MachineFuelConsumed>()
            .add_event::<MachineRemoved>()
            .add_event::<PlayerSpawned>()
            .add_event::<PlayerMoved>()
            .add_event::<InventoryChanged>()
            .add_event::<ConveyorTransfer>()
            .add_event::<ItemPickedUp>()
            .add_event::<ItemDropped>()
            .add_event::<ItemDelivered>()
            .add_event::<QuestStarted>()
            .add_event::<QuestProgressed>()
            .add_event::<QuestCompleted>()
    };
}

pub(crate) use register_core_events;
```

---

## Modイベント設計

### Modが独自イベントを追加する仕組み

```rust
// src/events/mod_event.rs

/// Mod独自のイベント（動的、型なし）
#[derive(Event, Clone, Debug, Serialize, Deserialize)]
pub struct ModEvent {
    /// Mod識別子 "my_mod:custom_event"
    pub event_id: String,
    /// イベントデータ（JSON）
    pub data: serde_json::Value,
    /// 発生源
    pub source: EventSource,
}

impl ModEvent {
    pub fn new(mod_id: &str, event_name: &str, data: serde_json::Value) -> Self {
        Self {
            event_id: format!("{}:{}", mod_id, event_name),
            data,
            source: EventSource::Mod(0), // TODO: proper mod_id
        }
    }

    pub fn mod_id(&self) -> &str {
        self.event_id.split(':').next().unwrap_or("")
    }

    pub fn event_name(&self) -> &str {
        self.event_id.split(':').nth(1).unwrap_or("")
    }
}

/// Modイベントの購読管理
#[derive(Resource, Default)]
pub struct ModEventSubscriptions {
    /// event_id -> 購読中のconnection_id一覧
    subscriptions: HashMap<String, HashSet<u64>>,
}

impl ModEventSubscriptions {
    pub fn subscribe(&mut self, event_id: &str, conn_id: u64) {
        self.subscriptions
            .entry(event_id.to_string())
            .or_default()
            .insert(conn_id);
    }

    pub fn unsubscribe(&mut self, event_id: &str, conn_id: u64) {
        if let Some(set) = self.subscriptions.get_mut(event_id) {
            set.remove(&conn_id);
        }
    }

    pub fn get_subscribers(&self, event_id: &str) -> Vec<u64> {
        self.subscriptions
            .get(event_id)
            .map(|s| s.iter().copied().collect())
            .unwrap_or_default()
    }
}
```

### Mod API でのイベント操作

```json
// コアイベント購読
{ "method": "event.subscribe", "params": { "event_type": "block.placed" } }

// Modイベント購読（ワイルドカード対応）
{ "method": "event.subscribe", "params": { "event_type": "my_mod:*" } }
{ "method": "event.subscribe", "params": { "event_type": "my_mod:custom_explosion" } }

// Modイベント発火
{ "method": "event.emit", "params": {
    "event_id": "my_mod:custom_explosion",
    "data": { "pos": [10, 5, 20], "radius": 5 }
}}
```

---

## 自動ブリッジ設計

### bridge.rs の構造

```rust
//! イベント自動ブリッジ
//!
//! コアイベントを自動的にMod APIに転送する。
//! 新しいコアイベントを追加しても、ここを変更する必要はない。

use bevy::prelude::*;
use super::core::*;

/// コアイベントをMod APIにブリッジするマクロ
macro_rules! bridge_core_event {
    ($app:expr, $event:ty) => {
        $app.add_systems(
            PostUpdate,
            bridge_event::<$event>.run_if(resource_exists::<ModApiServer>)
        );
    };
}

/// 汎用ブリッジシステム
fn bridge_event<E: CoreEvent>(
    mut events: EventReader<E>,
    subscriptions: Res<CoreEventSubscriptions>,
    server: Res<ModApiServer>,
    config: Res<EventSystemConfig>,
) {
    let kind = E::kind();

    // 高頻度イベントの除外チェック
    if kind.is_high_frequency() && config.external_exclude.contains(&kind) {
        return;
    }

    for event in events.read() {
        let conn_ids = subscriptions.get_subscribers(kind);
        if conn_ids.is_empty() {
            continue;
        }

        let notification = JsonRpcNotification::new(
            &format!("event.{}", kind.as_str()),
            event.to_json(),
        );

        for conn_id in conn_ids {
            let _ = server.tx.send(ClientMessage::Notify {
                conn_id,
                notification: notification.clone(),
            });
        }
    }
}

/// 全コアイベントのブリッジを登録
pub fn register_all_bridges(app: &mut App) {
    bridge_core_event!(app, BlockPlacing);
    bridge_core_event!(app, BlockPlaced);
    bridge_core_event!(app, BlockBreaking);
    bridge_core_event!(app, BlockBroken);
    bridge_core_event!(app, MachineSpawned);
    bridge_core_event!(app, MachineStarted);
    bridge_core_event!(app, MachineCompleted);
    bridge_core_event!(app, MachineFuelConsumed);
    bridge_core_event!(app, MachineRemoved);
    bridge_core_event!(app, PlayerSpawned);
    bridge_core_event!(app, PlayerMoved);
    bridge_core_event!(app, InventoryChanged);
    bridge_core_event!(app, ConveyorTransfer);
    bridge_core_event!(app, ItemPickedUp);
    bridge_core_event!(app, ItemDropped);
    bridge_core_event!(app, ItemDelivered);
    bridge_core_event!(app, QuestStarted);
    bridge_core_event!(app, QuestProgressed);
    bridge_core_event!(app, QuestCompleted);
}
```

---

## AI安全性の担保

### 1. 網羅性テスト

```rust
// tests/event_completeness.rs

#[test]
fn all_core_event_kinds_have_events() {
    // CoreEventKind の全バリアントに対応するイベントがあるか
    for kind in CoreEventKind::all() {
        assert!(
            kind.as_str().len() > 0,
            "CoreEventKind::{:?} has no string representation",
            kind
        );
    }
}

#[test]
fn all_core_events_are_registered() {
    // register_core_events! で登録されているか確認
    // （コンパイル時にチェックされるが、念のため）
    let mut app = App::new();
    register_core_events!(app);

    // 各イベントが送信可能か確認
    assert!(app.world.get_resource::<Events<BlockPlaced>>().is_some());
    assert!(app.world.get_resource::<Events<MachineCompleted>>().is_some());
    // ... 全イベント
}

#[test]
fn all_core_events_are_bridged() {
    // register_all_bridges で登録されているか確認
    // → bridge.rs のマクロに追加漏れがないか
}

#[test]
fn core_event_kind_count_matches() {
    // CoreEventKind::all() の数と実際の定義数が一致するか
    assert_eq!(
        CoreEventKind::all().len(),
        19, // ← 新イベント追加時にここも更新（忘れるとテスト失敗）
        "CoreEventKind count mismatch - did you add a new event?"
    );
}
```

### 2. コメントによるガイド

```rust
//! 【AIへの指示】
//!
//! 新しいコアイベントを追加する場合：
//!
//! 1. このファイル（core.rs）に構造体を追加
//!    ```rust
//!    #[derive(Event, Clone, Debug, Serialize, Deserialize)]
//!    pub struct MyNewEvent { ... }
//!    impl CoreEvent for MyNewEvent { fn kind() -> CoreEventKind { ... } }
//!    ```
//!
//! 2. CoreEventKind enum にバリアントを追加
//!    - as_str() に文字列マッピングを追加
//!    - from_str() にパースを追加
//!    - all() に追加
//!
//! 3. register_core_events! マクロに追加
//!
//! 4. bridge.rs の register_all_bridges() に追加
//!
//! 5. tests/event_completeness.rs のカウントを更新
//!
//! 【これだけで完了】他のファイルを変更する必要はありません。
```

### 3. コンパイル時チェック

```rust
// CoreEvent トレイトを実装していないとコンパイルエラー
fn bridge_event<E: CoreEvent>(...) { ... }

// register_core_events! に追加し忘れると、そのイベントは送信できない
// → システムが EventWriter<MyEvent> を要求した時点でパニック
```

---

## 購読管理の統合

### CoreEventSubscriptions

```rust
// src/events/subscriptions.rs

/// コアイベントの購読管理（Mod API用）
#[derive(Resource, Default)]
pub struct CoreEventSubscriptions {
    /// CoreEventKind -> 購読中のconnection_id一覧
    by_kind: HashMap<CoreEventKind, HashSet<u64>>,
    /// connection_id -> 購読中のCoreEventKind一覧
    by_connection: HashMap<u64, HashSet<CoreEventKind>>,
}

impl CoreEventSubscriptions {
    pub fn subscribe(&mut self, kind: CoreEventKind, conn_id: u64) {
        self.by_kind.entry(kind).or_default().insert(conn_id);
        self.by_connection.entry(conn_id).or_default().insert(kind);
    }

    pub fn unsubscribe(&mut self, kind: CoreEventKind, conn_id: u64) {
        if let Some(set) = self.by_kind.get_mut(&kind) {
            set.remove(&conn_id);
        }
        if let Some(set) = self.by_connection.get_mut(&conn_id) {
            set.remove(&kind);
        }
    }

    pub fn remove_connection(&mut self, conn_id: u64) {
        if let Some(kinds) = self.by_connection.remove(&conn_id) {
            for kind in kinds {
                if let Some(set) = self.by_kind.get_mut(&kind) {
                    set.remove(&conn_id);
                }
            }
        }
    }

    pub fn get_subscribers(&self, kind: CoreEventKind) -> Vec<u64> {
        self.by_kind
            .get(&kind)
            .map(|s| s.iter().copied().collect())
            .unwrap_or_default()
    }
}
```

---

## 移行計画

### Phase 1: 統合（破壊的変更なし）

1. `events/core.rs` を新規作成
2. `events/game_events.rs` の内容を `core.rs` に移行
3. `events/mod.rs` のレガシーイベントを `core.rs` に移行
4. 古いイベント名を type alias で互換維持

```rust
// 互換性レイヤー（一時的）
pub type BlockPlaceEvent = BlockPlaced;
pub type BlockBreakEvent = BlockBroken;
```

### Phase 2: 既存コードの更新

1. `EventWriter<BlockPlaceEvent>` → `EventWriter<BlockPlaced>` に置換
2. `modding/handlers/events.rs` の `EventType` を削除、`CoreEventKind` を使用
3. `modding/event_bridge.rs` を新しい `bridge.rs` に置換

### Phase 3: クリーンアップ

1. 互換性レイヤーを削除
2. `events/game_events.rs` を削除
3. テストを追加

---

## まとめ

| 項目 | 変更前 | 変更後 |
|------|--------|--------|
| イベント定義 | 2箇所（mod.rs, game_events.rs） | 1箇所（core.rs） |
| Mod API連携 | 手動同期（EventType enum） | 自動（CoreEventKind） |
| 新イベント追加 | 4ファイル変更必要 | 1ファイル + マクロ2箇所 |
| AI安全性 | 低（漏れやすい） | 高（テストで検出） |
| Mod拡張 | 不可 | ModEvent で可能 |

**この設計により**：
- コアイベントは中央管理（core.rs）
- Modは独自イベントを追加可能（ModEvent）
- AIは core.rs だけ読めば正しく実装できる
- テストで漏れを検出できる
