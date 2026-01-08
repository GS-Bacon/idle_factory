# 骨格設計：将来機能の拡張ポイント

## 設計プロセスのチェックリスト（2026-01-07 追加）

**教訓**: D.2（動的ID）設計時、「基盤の型定義」は詳細に書いたが「移行中の安全性」を見落とし、Modアイテムでパニックする時限爆弾を埋め込んでしまった。

### 新機能設計時の必須チェック

| # | チェック項目 | 質問例 |
|---|-------------|--------|
| 1 | **境界条件** | 内部↔外部、base game↔Mod、信頼できる↔信頼できないデータの境界は？ |
| 2 | **移行中の安全性** | 旧型と新型が共存する期間、どちらかが欠けたらどうなる？ |
| 3 | **失敗ケース** | 不正入力、欠落データ、未知のIDが来たらどうなる？ |
| 4 | **パニック監査** | この機能で `expect`, `unwrap`, `panic!` を使う箇所は？本当に必要？ |
| 5 | **フォールバック** | 失敗時のデフォルト動作は？ログ出力？スキップ？エラー表示？ |

### 設計書に含めるべき項目

```markdown
## 新機能: XXX

### 正常系
（従来通りの設計）

### 境界・エラー処理 ← 追加必須
- 外部入力の検証方法
- 不明なIDのフォールバック
- 移行中の互換性保証

### パニック箇所 ← 追加必須
- なし / または理由付きで列挙
```

### アンチパターン

| やりがち | 問題 | 対策 |
|----------|------|------|
| 「基盤作ったから完了」 | 移行中に爆弾が残る | 移行完了条件を明記 |
| 「段階的に移行」だけ | 境界の安全性が曖昧 | 境界での失敗処理を先に書く |
| 「内部なのでunwrap」 | Modが内部に入ってくる | Mod経路を洗い出す |

---

## 機能リスト（全17機能）

| # | 機能 | 概要 | 既存影響度 |
|---|------|------|-----------|
| **基盤システム** | | | |
| 1 | 電力システム | 発電機→電線→機械の電力供給 | 中 |
| 2 | 液体・気体 | パイプ、タンク、流体処理機械 | 中 |
| 3 | 信号制御 | レッドストーン的な論理回路 | 中 |
| 4 | クラフトシステム | プレイヤー手動クラフト | 中 |
| **物流・輸送** | | | |
| 5 | 線路機能 | 列車による長距離輸送 | 中 |
| 6 | ストレージ | 大容量倉庫、フィルター、自動整理 | 中 |
| **自動化・AI** | | | |
| 7 | ロボット | データ駆動プログラム（飛行含む） | 低〜中 |
| 8 | Mob追加 | 敵/友好NPC | 中 |
| **UI・UX** | | | |
| 9 | マップ機能 | ワールド俯瞰表示 | 低 |
| 10 | ブループリント | 配置パターンの保存・再利用 | 低 |
| 11 | 統計・分析 | 生産量グラフ、ボトルネック表示 | 低 |
| 12 | サウンド | BGM、効果音、環境音 | 低 |
| 13 | 実績システム | Steam実績連携 | 低 |
| **カスタマイズ** | | | |
| 14 | プレイヤースキン | マイクラ風の見た目変更 | 低 |
| 15 | モデルインポート | Blockbench直接インポート | 中 |
| **拡張性** | | | |
| 16 | Modding対応 | フルアクセス（Factorioレベル） | 高 |
| 17 | マルチプレイ基盤 | 複数プレイヤーの同期 | 高 |

**設計方針**:
- 研究ツリーはクエストと統合（レシピは基本アンロック済み、任意ロック可能）
- 資源枯渇なし（Modで対応可能に）
- 天候・昼夜は将来拡張可能な設計に（今は実装しない）

---

## 確定した設計判断（2026-01-08 更新）

以下の根本的な設計判断は**確定**。変更する場合は十分な理由が必要。

| 判断 | 決定 | 理由 |
|------|------|------|
| **ID方式** | 動的ID + Phantom Type | 型安全 + Mod対応 |
| **Mod構成** | ハイブリッド（WASM + WebSocket + TOML） | ロジック拡張 + 言語自由 + データ定義 |
| **base** | 本体に内蔵（Engine + base 一体） | シンプル、開発速度優先 |
| **マルチ** | 確定実装、今すぐComponent化 | 後からは困難 |
| **イベント** | 全フック（設計付き） | マルチ・Mod・デバッグ全てに必要 |

### Modレイヤー詳細

| レイヤー | 実行方式 | 言語 | 用途 | 優先度 |
|---------|---------|------|------|--------|
| **Data Mod** | TOML読み込み | - | アイテム/機械/レシピ | ✅ 実装済み |
| **Script Mod** | WebSocket | Python, Lua, JS | イベントフック | 基盤あり |
| **Core Mod** | WASM（Wasmtime） | Rust, C++ | ロジック追加・置換 | 将来 |

---

## 設計原則（2026-01-07 更新）

### 1. 動的ID + Phantom Type（型安全な動的ID）

**BlockType enum を廃止し、動的IDに移行する。ただし型安全性は維持。**

```rust
use std::marker::PhantomData;

/// 型安全な動的ID
/// - カテゴリ混同をコンパイル時に防止（Phantom Type）
/// - 存在保証はRegistry経由でのみ生成することで担保
/// - 高速比較（内部u32）
/// - 可読性（Interned String で "namespace:id" 形式対応）
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct Id<Category> {
    raw: u32,
    _marker: PhantomData<Category>,
}

// カテゴリマーカー（ゼロサイズ型）
pub struct ItemCategory;
pub struct MachineCategory;
pub struct RecipeCategory;
pub struct FluidCategory;

// 型エイリアス
pub type ItemId = Id<ItemCategory>;
pub type MachineId = Id<MachineCategory>;
pub type RecipeId = Id<RecipeCategory>;
pub type FluidId = Id<FluidCategory>;

impl<C> Id<C> {
    /// Registry経由でのみ生成可能（pub(crate)）
    ///
    /// **設計意図**: ModはRegistryの`register()`戻り値でのみIDを取得する。
    /// 直接`Id::new()`を呼べないことで、存在しないIDの作成を防止。
    pub(crate) fn new(raw: u32) -> Self {
        Self { raw, _marker: PhantomData }
    }

    pub fn raw(&self) -> u32 { self.raw }
}

// Interned String で文字列ID対応
// Note: マルチプレイ時はRwLock<StringInterner>でスレッドセーフに
pub struct StringInterner {
    to_id: HashMap<String, u32>,
    to_str: Vec<String>,
}

impl StringInterner {
    pub fn get_or_intern(&mut self, s: &str) -> u32 { ... }
    pub fn resolve(&self, id: u32) -> &str { ... }
}

// スレッドセーフ版（マルチプレイ用）
pub type SharedInterner = Arc<RwLock<StringInterner>>;
```

**Registry経由でのみID生成**:
```rust
impl ItemRegistry {
    /// 新アイテム登録（IDを返す）
    pub fn register(&mut self, name: &str, desc: ItemDescriptor) -> ItemId {
        let raw = self.interner.get_or_intern(name);
        self.items.insert(raw, desc);
        ItemId::new(raw)
    }

    /// ID → Descriptor
    pub fn get(&self, id: ItemId) -> Option<&ItemDescriptor> {
        self.items.get(&id.raw())
    }

    /// 文字列 → ID（存在しない場合None）
    pub fn lookup(&self, name: &str) -> Option<ItemId> {
        self.interner.get(name).map(ItemId::new)
    }
}
```

**型安全性の保証**:
| 保証 | 達成方法 |
|------|----------|
| カテゴリ混同防止 | Phantom Type（`Id<ItemCategory>` と `Id<MachineCategory>` は別型）|
| 存在保証 | Registry経由でのみ生成（`new` は `pub(crate)`）|
| 高速比較 | 内部u32（`==` は整数比較）|
| 可読性 | Interned String（`"mymod:super_ingot"` 形式）|
| Mod対応 | 実行時に追加可能（enumではない）|

**移行計画**:
1. `Id<T>` と Registry を実装
2. 組み込みアイテムを登録（起動時）
3. 既存コードを段階的に移行（`BlockType` → `ItemId`）
4. Mod読み込み時に追加アイテムを登録

---

### 1.1 セーブデータ互換性（動的IDの永続化）

**問題**: ランタイム生成される`u32`をそのまま保存すると、Modの読み込み順が変わった瞬間にIDがズレてセーブデータが壊れる。

**解決策**: セーブデータには**文字列ID**を保存する。

```rust
// セーブデータ構造
#[derive(Serialize, Deserialize)]
pub struct SaveData {
    pub version: u32,
    pub world: WorldSaveData,
    pub player: PlayerSaveData,
}

// ブロックは文字列IDで保存
#[derive(Serialize, Deserialize)]
pub struct BlockSaveData {
    pub id: String,  // "base:iron_ore", "mymod:super_block"
    pub pos: IVec3,
    pub metadata: Option<serde_json::Value>,
}

// ロード時に再マッピング
impl SaveData {
    pub fn load(path: &Path, registry: &ItemRegistry) -> Result<Self, LoadError> {
        let data: SaveData = serde_json::from_reader(File::open(path)?)?;

        // 各ブロックのIDを文字列→ItemIdに変換
        for block in &data.world.blocks {
            match registry.lookup(&block.id) {
                Some(id) => { /* OK */ },
                None => {
                    // Modが削除された場合のフォールバック
                    warn!("Unknown block ID: {}, replacing with air", block.id);
                    // → 空気ブロックに置換、またはエラー
                }
            }
        }
        Ok(data)
    }
}
```

**Mod削除時のフォールバック**:
| ケース | 対応 |
|--------|------|
| 不明なブロック | 空気に置換 + 警告ログ |
| 不明なアイテム | インベントリから削除 + 警告ログ |
| 不明な機械 | 基本ブロックに置換 + アイテムドロップ |

---

### 2. マルチプレイ対応の原則

**マルチプレイは確定実装。PlayerInventory Component化は最優先。**

| 原則 | 内容 | 対応状況 |
|------|------|----------|
| **No Singletons for Player Data** | プレイヤー固有データはComponentにする | ⚠️ **最優先で変更** |
| **NetworkId層** | Entity参照はNetworkId経由にする | 未実装 |
| **サーバー権威モデル** | 全状態変更はイベント経由 | イベントシステムで対応 |

```rust
// ❌ 現在（シングルトン）- 廃止予定
#[derive(Resource)]
pub struct PlayerInventory { ... }

// ✅ 将来（コンポーネント）
#[derive(Component)]
pub struct Inventory { ... }

// プレイヤーはEntityになる
commands.spawn((Player, Inventory::default(), NetworkId(uuid)));
```

#### NetworkId と Entity のマッピング

**問題**: `Entity` はサーバーとクライアントで異なる値になる。

```rust
/// ネットワーク上で一意なID（UUID）
#[derive(Component, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NetworkId(pub u64);

/// サーバー/クライアント間のEntityマッピング
#[derive(Resource, Default)]
pub struct EntityMap {
    network_to_entity: HashMap<NetworkId, Entity>,
    entity_to_network: HashMap<Entity, NetworkId>,
}

impl EntityMap {
    /// NetworkId → Entity（ローカル）
    pub fn get_entity(&self, network_id: NetworkId) -> Option<Entity> {
        self.network_to_entity.get(&network_id).copied()
    }

    /// Entity → NetworkId
    pub fn get_network_id(&self, entity: Entity) -> Option<NetworkId> {
        self.entity_to_network.get(&entity).copied()
    }

    /// 新規登録（サーバー側でEntity生成時）
    pub fn register(&mut self, entity: Entity, network_id: NetworkId) {
        self.network_to_entity.insert(network_id, entity);
        self.entity_to_network.insert(entity, network_id);
    }

    /// 削除（Entity despawn時）
    pub fn unregister(&mut self, entity: Entity) {
        if let Some(network_id) = self.entity_to_network.remove(&entity) {
            self.network_to_entity.remove(&network_id);
        }
    }
}

/// NetworkIdを自動生成（サーバー側）
#[derive(Resource)]
pub struct NetworkIdGenerator {
    next_id: u64,
}

impl NetworkIdGenerator {
    pub fn next(&mut self) -> NetworkId {
        let id = NetworkId(self.next_id);
        self.next_id += 1;
        id
    }
}
```

**使用パターン**:
```rust
// ネットワークメッセージでは NetworkId を使用
struct SpawnMachineMessage {
    network_id: NetworkId,
    machine_type: MachineId,
    pos: IVec3,
}

// ローカル処理では EntityMap で変換
fn handle_spawn_machine(
    msg: SpawnMachineMessage,
    mut commands: Commands,
    mut entity_map: ResMut<EntityMap>,
) {
    let entity = commands.spawn(MachineBundle::new(...)).id();
    entity_map.register(entity, msg.network_id);
}
```

**移行戦略**:
```rust
// Step 1: 互換レイヤー（一時的）
#[derive(Resource)]
pub struct LocalPlayer(pub Entity);  // ローカルプレイヤーのEntity

// Step 2: 新コードはQuery使用
fn system(
    local_player: Res<LocalPlayer>,
    query: Query<&mut Inventory>,
) {
    if let Ok(mut inv) = query.get_mut(local_player.0) { ... }
}

// Step 3: 既存コード段階的移行
// Step 4: PlayerInventory Resource 削除
```

### Modding の原則（2026-01-08 確定）

**ハイブリッド方式：WASM + WebSocket + TOML**

```
┌─────────────────────────────────────────────┐
│  Engine + base (Rust)                       │
│  ├─ 描画、ECS、アセット                      │
│  ├─ バニラコンテンツ（本体に内蔵）            │
│  ├─ WASM ローダー（Wasmtime）               │
│  └─ WebSocket サーバー                      │
├─────────────────────────────────────────────┤
│  Core Mod (WASM)          ← 同一プロセス     │
│  └─ ロジック追加・置換（高速）               │
├─────────────────────────────────────────────┤
│  Script Mod (WebSocket)   ← 別プロセス       │
│  └─ イベントフック、設定変更（言語自由）      │
├─────────────────────────────────────────────┤
│  Data Mod (TOML)          ← ファイル読み込み │
│  └─ アイテム/機械/レシピ定義                 │
└─────────────────────────────────────────────┘
```

| レイヤー | 実行方式 | 言語 | 用途 |
|---------|---------|------|------|
| **Core Mod** | WASM（同一プロセス） | Rust, C++ 等 | 新ロジック、システム置換 |
| **Script Mod** | WebSocket（別プロセス） | Python, Lua, JS 等 | イベントフック、設定変更 |
| **Data Mod** | TOML読み込み | - | アイテム/機械/レシピ定義 |

**設計判断（2026-01-08）**:
- base は本体に残す（Engine + base 一体）
- Core Mod で新ロジック定義 → Data Mod で別ユーザーがアイテム追加可能
- Script Mod で Python/Lua ユーザーにも門戸を開く

**Modでできること**:
| やりたいこと | 使うレイヤー |
|-------------|-------------|
| 新アイテム追加 | Data Mod |
| 既存ロジックで新機械 | Data Mod |
| 新しい機械ロジック | Core Mod |
| イベントに反応 | Script Mod / Core Mod |
| 設定値変更 | Script Mod |
| システム置換 | Core Mod |

**階層的Mod拡張の例**:
```
[Core Mod] electric_machines.wasm
    │  → 「電力消費して加工」ロジックを定義
    │
    ├─[Data Mod] electric_machines/machines.toml
    │      → 電気炉、電気粉砕機（Mod作者）
    │
    └─[Data Mod] my_addon/machines.toml
           → 超電気炉（別ユーザー、上のロジックを使用）
```

### 3. イベントシステム（全フック設計）

**全ての状態変更をイベント経由にする。** マルチプレイ同期・Modフック・デバッグ全てに必要。

#### 基本設計

```rust
// Bevy Observer（0.14+）を基盤として使用
#[derive(Event)]
pub struct BlockPlaced {
    pub pos: IVec3,
    pub block: ItemId,      // 動的ID
    pub player: Entity,
    pub source: EventSource,
}

#[derive(Clone, Copy)]
pub enum EventSource {
    Player(Entity),
    Machine(Entity),
    Mod(ModId),
    System,  // ワールド生成など
}

// 登録
app.add_observer(on_block_placed);

fn on_block_placed(trigger: Trigger<BlockPlaced>, ...) {
    // 処理
}
```

#### イベントカタログ（網羅的）

**ブロック系**:
| イベント | 発火タイミング | データ |
|----------|----------------|--------|
| `BlockPlacing` | 配置直前（キャンセル可能） | pos, block, player |
| `BlockPlaced` | 配置完了後 | pos, block, player |
| `BlockBreaking` | 破壊開始時 | pos, block, player, progress |
| `BlockBroken` | 破壊完了後 | pos, block, player, drops |

**機械系**:
| イベント | 発火タイミング | データ |
|----------|----------------|--------|
| `MachineSpawned` | 機械Entity生成後 | entity, machine_id, pos |
| `MachineStarted` | 加工開始時 | entity, recipe_id, inputs |
| `MachineCompleted` | 加工完了時 | entity, recipe_id, outputs |
| `MachineFuelConsumed` | 燃料消費時 | entity, fuel_id, amount |
| `MachineRemoved` | 機械撤去時 | entity, machine_id, pos |

**プレイヤー系**:
| イベント | 発火タイミング | データ |
|----------|----------------|--------|
| `PlayerSpawned` | プレイヤー参加時 | entity, network_id |
| `PlayerMoved` | 位置変更時 | entity, from, to |
| `InventoryChanged` | インベントリ変更時 | entity, slot, item_id, delta |
| `PlayerDamaged` | ダメージ時（将来） | entity, amount, source |
| `PlayerDied` | 死亡時（将来） | entity, cause |

**物流系**:
| イベント | 発火タイミング | データ |
|----------|----------------|--------|
| `ConveyorTransfer` | アイテム移動時 | from_pos, to_pos, item_id |
| `ItemPickedUp` | アイテム取得時 | entity, item_id, count |
| `ItemDropped` | アイテム落下時 | entity, item_id, count, pos |
| `ItemDelivered` | 納品時 | item_id, count, quest_id |

**クエスト系**:
| イベント | 発火タイミング | データ |
|----------|----------------|--------|
| `QuestStarted` | クエスト開始時 | quest_id |
| `QuestProgressed` | 進捗更新時 | quest_id, progress |
| `QuestCompleted` | クエスト完了時 | quest_id, rewards |

#### 循環防止・パフォーマンス対策

```rust
/// イベントシステム設定
#[derive(Resource)]
pub struct EventSystemConfig {
    /// 最大連鎖深さ（循環防止）
    pub max_depth: u8,  // デフォルト: 16

    /// デバッグログ
    pub log_enabled: bool,

    /// 外部通知を除外するイベント（パフォーマンス用）
    pub external_exclude: HashSet<EventKind>,
}

/// 現在の連鎖深さを追跡（システム内部用）
#[derive(Resource, Default)]
pub struct EventDepth(pub u8);

/// 高頻度イベントはバッチ化
pub struct BatchedEvents<E: Event> {
    events: Vec<E>,
    flush_interval: f32,  // 秒
}
```

#### 循環防止の実装詳細

**誰がチェックするか**: イベント送信用のラッパーAPIで強制。

```rust
/// 安全なイベント送信（深さチェック付き）
pub struct GuardedEventWriter<'w, E: Event> {
    writer: EventWriter<'w, E>,
    depth: ResMut<'w, EventDepth>,
    config: Res<'w, EventSystemConfig>,
}

impl<E: Event> GuardedEventWriter<'_, E> {
    /// 深さチェック付きイベント送信
    pub fn send(&mut self, event: E) -> Result<(), EventError> {
        if self.depth.0 >= self.config.max_depth {
            error!("Event depth exceeded: {:?}", std::any::type_name::<E>());
            return Err(EventError::MaxDepthExceeded);
        }
        self.depth.0 += 1;
        self.writer.send(event);
        Ok(())
    }
}

/// フレーム開始時にリセット
fn reset_event_depth(mut depth: ResMut<EventDepth>) {
    depth.0 = 0;
}

// First system in schedule
app.add_systems(First, reset_event_depth);
```

**強制方法**: 通常の `EventWriter` を直接使わず、`GuardedEventWriter` のみを公開。

#### 高頻度イベントの除外リスト

以下のイベントは**デフォルトで外部Mod通知をOFF**（内部処理のみ）:

| イベント | 理由 | 代替手段 |
|----------|------|----------|
| `ConveyorTransfer` | 毎フレーム大量発生（数千/秒） | バッチ化して統計イベントで通知 |
| `PlayerMoved` | 毎フレーム発生 | 位置変化が閾値超えた時のみ通知 |

**設定可能**:
```rust
// Mod側でオプトイン可能
{ "action": "subscribe", "events": ["conveyor_transfer"], "force": true }
// → 警告付きで許可（パフォーマンス影響を理解した上で）
```

#### 外部Mod通知

```rust
/// イベントを外部Modに通知
pub struct ModEventBridge {
    subscribers: HashMap<EventKind, Vec<ModConnection>>,
}

impl ModEventBridge {
    /// イベント発火時に呼ばれる
    fn on_event<E: Event + Serialize>(&self, event: &E) {
        let kind = E::kind();
        if let Some(subscribers) = self.subscribers.get(&kind) {
            let json = serde_json::to_string(event).unwrap();
            for conn in subscribers {
                conn.send(&json);  // WebSocket送信
            }
        }
    }
}
```

**理由**:
- Bevy組み込みなのでメンテ不要
- ECS哲学に沿った設計
- システムスケジューリングで実行順序制御
- 全フックにより：マルチ同期、Mod対応、デバッグログ、undo/redo が可能

---

### 4. スクリプト方針（WASM + WebSocket ハイブリッド）

**Core Mod（WASM）と Script Mod（WebSocket）の2層構成。**

```
[Engine + base]
    ├─ WASM ローダー ←── [Core Mod] ロジック追加・置換
    └─ WebSocket ←────── [Script Mod] イベントフック
```

#### 実装フェーズ

| Phase | 方式 | 内容 | 状態 |
|-------|------|------|------|
| **Phase 1** | Data Mod（TOML） | アイテム/機械/レシピ定義 | ✅ 実装済み |
| **Phase 2** | WebSocket API | イベント通知、Script Mod | 基盤あり |
| **Phase 3** | WASM ローダー | Core Mod、ロジック拡張 | 未着手 |

#### 固定 Tick システム（必須）

**固定 Tick は WebSocket だけでなく、複数の理由で必須。**

| 用途 | なぜ必要 |
|------|---------|
| **マルチプレイ** | クライアント間の状態同期を一定間隔で |
| **大規模工場** | 毎フレーム処理は重い（1000機械×60fps=6万回/秒） |
| **決定論的再現** | デバッグ、リプレイ、バグ再現 |
| **Script Mod** | WebSocket レイテンシ吸収 |
| **省電力** | バックグラウンド時に Tick 頻度を下げる |

**設計**: Minecraft同様の**固定Tick**（1秒=20tick=50ms/tick）を採用。

```rust
// Bevy FixedUpdate を使用
app.insert_resource(Time::<Fixed>::from_hz(20.0)); // 20 tick/秒

// ゲームロジックは FixedUpdate で実行（20回/秒）
app.add_systems(FixedUpdate, (
    generic_machine_tick,
    conveyor_transfer,
    signal_tick,
    process_mod_commands,  // Script Mod からのコマンド
));

// 描画は Update で実行（60fps、補間で滑らかに）
app.add_systems(Update, (
    interpolate_conveyor_items,
    interpolate_positions,
    render_ui,
));
```

**パフォーマンス比較**:
| 方式 | 機械1000台の処理回数/秒 |
|------|------------------------|
| 毎フレーム（60fps） | 60,000 回 |
| 固定Tick（20tick） | 20,000 回（3倍軽い） |

**レイテンシ許容範囲**:
| 操作 | 許容遅延 | WebSocket対応 |
|------|----------|---------------|
| ブロック配置 | 1-2 tick (50-100ms) | ✅ 問題なし |
| 機械制御 | 1-2 tick | ✅ 問題なし |
| ロボット指示 | 2-3 tick | ✅ 問題なし |
| 戦闘（将来） | 0-1 tick | ✅ ゲーム本体で処理、Modは結果を受け取るのみ |

**結論**: 固定Tick採用により、**外部API方式で十分な精度を確保可能**。

#### Script Mod の制限事項

| 制限 | 内容 | 理由 |
|------|------|------|
| **非同期のみ** | ゲームをブロックしない | フリーズ防止 |
| **読み取り優先** | 状態変更はリクエスト（ゲームが検証・適用） | 不正防止 |
| **タイムアウト** | 100ms 応答なければ無視 | ハング防止 |
| **レート制限** | 1秒あたり最大100リクエスト | DoS防止 |
| **高頻度イベント除外** | ConveyorTransfer, PlayerMoved 等はデフォルトOFF | 帯域節約 |

#### Core Mod vs Script Mod の使い分け

| やりたいこと | 推奨 | 理由 |
|-------------|------|------|
| 毎 Tick 処理 | Core Mod | レイテンシなし |
| たまにイベント反応 | Script Mod | 言語自由 |
| 状態の直接変更 | Core Mod | 検証なしで高速 |
| 状態の問い合わせ | どちらでも | - |
| プロトタイピング | Script Mod | ホットリロード容易 |
| 本番運用 | Core Mod | パフォーマンス |

#### Mod API Server

```rust
pub struct ModApiServer {
    listener: TcpListener,  // WebSocket
    connections: Vec<ModConnection>,
}

// API エンドポイント
enum ModAction {
    // 購読
    Subscribe { events: Vec<EventKind> },
    Unsubscribe { events: Vec<EventKind> },

    // 読み取り
    GetRegistry,  // アイテム/機械/レシピ一覧
    GetEntity { id: NetworkId },
    GetWorld { chunk: IVec2 },

    // 書き込み（権限チェック付き）
    RegisterItem { descriptor: ItemDescriptor },
    SpawnEntity { kind: EntityKind, pos: Vec3 },
    ModifyEntity { id: NetworkId, changes: EntityChanges },
}
```

#### Mod API バージョニング

**問題**: API変更時に既存Modが壊れる。

```rust
/// APIバージョン（セマンティックバージョニング）
pub const MOD_API_VERSION: &str = "1.0.0";

/// 接続時のハンドシェイク
#[derive(Serialize, Deserialize)]
pub struct HandshakeRequest {
    /// Modが要求するAPIバージョン
    pub api_version: String,
    /// Mod識別子
    pub mod_id: String,
    /// Mod名（表示用）
    pub mod_name: String,
}

#[derive(Serialize, Deserialize)]
pub struct HandshakeResponse {
    /// サーバーのAPIバージョン
    pub api_version: String,
    /// 互換性ステータス
    pub compatibility: Compatibility,
    /// 非推奨警告（あれば）
    pub deprecation_warnings: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub enum Compatibility {
    /// 完全互換
    Full,
    /// 後方互換（一部機能が使えない）
    Backward { missing_features: Vec<String> },
    /// 非互換（接続拒否）
    Incompatible { reason: String },
}
```

**バージョニングルール**:
| 変更内容 | バージョン | 対応 |
|----------|-----------|------|
| 新機能追加 | minor (1.1.0) | 後方互換 |
| バグ修正 | patch (1.0.1) | 後方互換 |
| 破壊的変更 | major (2.0.0) | 非互換警告 |

**メッセージ形式**:
```json
{
    "api_version": "1.0",
    "action": "subscribe",
    "events": ["block_placed", "machine_processed"]
}
```

---

## 現在のアーキテクチャ（参照用）

```
components/     ← ECSコンポーネント（データ）
    machines.rs     Machine, Conveyor, MachineSlot

game_spec/      ← 仕様定義（Single Source of Truth）
    machines.rs     MachineSpec, IoPort
    recipes.rs      RecipeSpec

machines/       ← 機械処理ロジック
    generic.rs      generic_machine_tick()

logistics/      ← 物流ロジック
    conveyor.rs     conveyor_transfer()

world/          ← チャンク・ワールド管理
    mod.rs          ChunkData, WorldData
```

---

## 各機能の設計骨格

### 1. 電力システム

**データ構造**
```rust
// components/power.rs
pub struct PowerConsumer {
    pub required_watts: u32,
    pub is_powered: bool,
}

pub struct PowerProducer {
    pub output_watts: u32,
    pub fuel_slot: Option<MachineSlot>,
}

// グリッドは自動計算（接続されたノードをグラフ探索）
```

**拡張ポイント**
- `MachineSpec` に `power_consumption: Option<u32>` 追加
- `generic_machine_tick()` で `is_powered` チェック追加
- 新モジュール: `logistics/power_grid.rs`

**既存変更**
- `IoPort` に `PortType::Power` 追加

---

### 2. 液体・気体

**データ構造**
```rust
// components/fluids.rs
pub struct FluidSlot {
    pub fluid_type: Option<FluidType>,
    pub amount_mb: u32,      // ミリバケット
    pub capacity_mb: u32,
}

pub struct Pipe {
    pub position: IVec3,
    pub fluid: Option<FluidType>,
    pub pressure: f32,
}

// game_spec/fluids.rs
pub struct FluidSpec {
    pub id: &'static str,
    pub viscosity: f32,      // 流速に影響
    pub temperature: f32,    // 熱処理用
}
```

**拡張ポイント**
- 新モジュール: `logistics/pipe.rs`（conveyor.rsと並列）
- `IoPort` に `PortType::Fluid` 追加
- `MachineSpec.slots` に `SlotType::Fluid` 追加

---

### 3. 信号制御（レッドストーン的）

**データ構造**
```rust
// components/signals.rs
pub struct SignalEmitter {
    pub signal_strength: u8,  // 0-15
    pub condition: SignalCondition,
}

pub struct SignalReceiver {
    pub threshold: u8,
    pub is_active: bool,
}

pub enum SignalCondition {
    Always,
    InventoryFull,
    InventoryEmpty,
    HasItem(ItemId),  // 動的ID
    PowerLow,
    Timer { interval_secs: f32 },
}

// 信号線
pub struct SignalWire {
    pub position: IVec3,
    pub strength: u8,
}
```

**拡張ポイント**
- 新モジュール: `logistics/signals.rs`
- 機械に `signal_input: bool` で動作ON/OFF
- 論理ゲート: AND, OR, NOT, XOR ブロック

**信号の伝播**
```
センサー（在庫検知等）
    ↓ signal_strength: 15
信号線（減衰: 1/ブロック）
    ↓ strength: 12
論理ゲート（AND等）
    ↓
機械（signal_input=true で動作）
```

---

### 4. クラフトシステム

**データ構造**
```rust
// components/crafting.rs
pub struct CraftingGrid {
    pub slots: [Option<(BlockType, u32)>; 9],  // 3x3
    pub result: Option<(BlockType, u32)>,
}

// game_spec/recipes.rs に追加
pub struct CraftingRecipe {
    pub pattern: &'static [&'static str],  // "III", " S ", " S " など
    pub ingredients: &'static [(char, BlockType)],
    pub result: (BlockType, u32),
    pub unlocked: bool,  // デフォルトtrue、任意ロック可能
}
```

**拡張ポイント**
- `ui/crafting.rs` でクラフトUI
- クエスト報酬で `unlocked` を変更可能
- Modでレシピ追加/ロック変更可能

---

### 5. 線路機能

**データ構造**
```rust
// components/trains.rs
pub struct Rail {
    pub position: IVec3,
    pub rail_type: RailType,  // Straight, Curve, Switch
    pub signal_block_id: u32, // 閉塞区間
}

pub struct Train {
    pub cars: Vec<TrainCar>,
    pub speed: f32,
    pub route: Vec<IVec3>,
}

pub struct TrainStation {
    pub position: IVec3,
    pub schedule: Vec<ScheduleEntry>,
}
```

**拡張ポイント**
- 新モジュール: `logistics/rail.rs`
- 信号制御との連携（閉塞信号）
- 列車はEntityの集合（機関車 + 貨車）

---

### 6. ストレージ

**データ構造**
```rust
// components/storage.rs
pub struct StorageBlock {
    pub capacity: u32,
    pub slots: Vec<MachineSlot>,
    pub filter: Option<Vec<BlockType>>,  // 許可アイテムリスト
    pub priority: i32,  // 入出力優先度
}

pub struct StorageNetwork {
    pub storages: Vec<Entity>,
    pub total_capacity: u32,
    pub auto_sort: bool,
}
```

**拡張ポイント**
- 倉庫ブロック（小/中/大）
- フィルター設定UI
- コンベアとの接続
- 自動整理機能

**依存関係**: ストレージはコンベアシステム（`logistics/conveyor.rs`）に依存。コンベアのIoPort接続ロジックを再利用。

---

### 7. ロボット（データ駆動）

**設計方針**: プリセット動作をデータ駆動で定義。Mod APIで動的に追加・変更可能。

**データ構造**
```rust
// components/robots.rs
pub struct Robot {
    pub position: Vec3,  // 浮動小数点（飛行対応）
    pub inventory: Vec<MachineSlot>,
    pub program_id: RobotProgramId,  // データ駆動
    pub program_params: RobotProgramParams,
    pub state: RobotState,
    pub can_fly: bool,
    pub fuel: f32,
}

// game_spec/robots.rs
pub struct RobotProgramSpec {
    pub id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub required_params: &'static [ParamDef],
}

/// パラメータ定義（UI自動生成用）
pub struct ParamDef {
    pub name: &'static str,
    pub param_type: ParamType,  // Position, Entity, ItemFilter, etc.
}

/// 実行時パラメータ
pub struct RobotProgramParams {
    pub values: HashMap<String, ParamValue>,
}

pub enum ParamValue {
    Position(IVec3),
    Entity(Entity),
    ItemFilter(Vec<ItemId>),
    Number(f32),
}

pub enum RobotState {
    Idle,
    Moving { target: Vec3 },
    Mining { target: IVec3, progress: f32 },
    Crafting,
    Flying { target: Vec3, altitude: f32 },
}
```

**組み込みプログラム**
```rust
// game_spec/robots.rs
pub const ROBOT_PROGRAMS: &[RobotProgramSpec] = &[
    RobotProgramSpec {
        id: "mine_and_deliver",
        name: "採掘→納品",
        required_params: &[
            ParamDef { name: "target", param_type: ParamType::Position },
            ParamDef { name: "delivery", param_type: ParamType::Entity },
        ],
    },
    RobotProgramSpec {
        id: "patrol_route",
        name: "巡回",
        required_params: &[
            ParamDef { name: "waypoints", param_type: ParamType::PositionList },
        ],
    },
    RobotProgramSpec {
        id: "filter_transfer",
        name: "フィルター転送",
        required_params: &[
            ParamDef { name: "from", param_type: ParamType::Entity },
            ParamDef { name: "to", param_type: ParamType::Entity },
            ParamDef { name: "filter", param_type: ParamType::ItemFilter },
        ],
    },
];
```

**Mod APIでの拡張**
```json
// 新プログラム登録
{
    "action": "register_robot_program",
    "program": {
        "id": "mymod:smart_miner",
        "name": "スマート採掘",
        "required_params": [
            { "name": "area", "type": "PositionRange" },
            { "name": "priority", "type": "ItemFilter" }
        ]
    }
}

// ロボットにプログラム設定
{
    "action": "set_robot_program",
    "robot_id": "uuid",
    "program_id": "mymod:smart_miner",
    "params": {
        "area": { "min": [0, 0, 0], "max": [10, 10, 10] },
        "priority": ["base:diamond_ore", "base:iron_ore"]
    }
}
```

**拡張ポイント**
- 新モジュール: `systems/robot.rs`
- `game_spec/robots.rs` でプログラム定義
- UIはパラメータ定義から自動生成
- Mod APIで新プログラム追加可能

---

### 8. Mob追加

**データ構造**
```rust
// components/mobs.rs
pub struct Mob {
    pub mob_type: MobType,
    pub health: f32,
    pub ai_state: AIState,
}

pub enum AIState {
    Idle,
    Wandering,
    Chasing(Entity),
    Fleeing,
    Working,  // 友好NPCの作業
}

// game_spec/mobs.rs
pub struct MobSpec {
    pub id: &'static str,
    pub max_health: f32,
    pub speed: f32,
    pub hostile: bool,
    pub drops: &'static [(BlockType, u32, f32)], // item, count, probability
}
```

**拡張ポイント**
- 新モジュール: `mobs/`
- 衝突システム拡張（player/ に戦闘追加）
- スポーン条件（バイオーム、時間帯）

---

### 9. マップ機能

**データ構造**
```rust
// components/map.rs
pub struct MapMarker {
    pub position: IVec3,
    pub marker_type: MarkerType,
    pub label: Option<String>,
}

pub enum MarkerType {
    Machine(BlockType),
    Player,
    Custom,
}

// リソース
pub struct MapData {
    pub explored_chunks: HashSet<IVec2>,
    pub markers: Vec<MapMarker>,
}
```

**拡張ポイント**
- 新モジュール: `ui/map.rs`
- チャンクの「探索済み」フラグ
- ミニマップ + フルスクリーンマップ

---

### 10. ブループリント

**データ構造**
```rust
// save/blueprint.rs
pub struct Blueprint {
    pub name: String,
    pub size: IVec3,
    pub blocks: Vec<BlueprintBlock>,
}

pub struct BlueprintBlock {
    pub offset: IVec3,
    pub block_type: BlockType,
    pub rotation: u8,
    pub machine_config: Option<MachineConfig>,
}
```

**拡張ポイント**
- 保存: `save/blueprint.rs`
- UI: 選択範囲 → 保存、プレビュー表示
- 配置: ゴースト表示 → 一括配置

---

### 11. 統計・分析

**データ構造**
```rust
// components/statistics.rs
pub struct ProductionStats {
    pub item_produced: HashMap<BlockType, TimeSeries>,
    pub item_consumed: HashMap<BlockType, TimeSeries>,
    pub power_usage: TimeSeries,
}

pub struct TimeSeries {
    pub samples: VecDeque<(f64, f32)>,  // (timestamp, value)
    pub resolution: f32,  // サンプリング間隔（秒）
}

pub struct BottleneckAnalysis {
    pub slow_machines: Vec<(Entity, f32)>,  // 稼働率が低い機械
    pub full_outputs: Vec<Entity>,  // 出力詰まり
    pub empty_inputs: Vec<Entity>,  // 入力待ち
}
```

**拡張ポイント**
- 新モジュール: `ui/statistics.rs`
- グラフ描画（bevy_egui等）
- ボトルネック自動検出

---

### 12. サウンド

**データ構造**
```rust
// components/audio.rs
pub struct SoundEmitter {
    pub sound_id: &'static str,
    pub volume: f32,
    pub loop_: bool,
}

// game_spec/sounds.rs
pub struct SoundSpec {
    pub id: &'static str,
    pub path: &'static str,
    pub category: SoundCategory,  // BGM, SFX, Ambient
    pub default_volume: f32,
}
```

**拡張ポイント**
- `bevy_audio` または `bevy_kira_audio`
- カテゴリ別音量設定
- 3D空間サウンド（機械の動作音）

---

### 13. 実績システム

**データ構造**
```rust
// components/achievements.rs
pub struct Achievement {
    pub id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub condition: AchievementCondition,
    pub unlocked: bool,
}

pub enum AchievementCondition {
    ProduceItem { item: BlockType, count: u32 },
    PlaceMachines { count: u32 },
    CompleteQuest { quest_id: &'static str },
    PlayTime { minutes: u32 },
    // ...
}
```

**拡張ポイント**
- Steamworks SDK連携
- ローカル実績（オフライン用）
- 通知UI

---

### 14. プレイヤースキン

**データ構造**
```rust
// components/player_skin.rs
pub struct PlayerSkin {
    pub texture: Handle<Image>,
    pub model: PlayerModel,  // Steve/Alex 体型
}

pub enum PlayerModel {
    Classic,  // 4px腕
    Slim,     // 3px腕
}

pub struct SkinLoader {
    pub cache: HashMap<String, Handle<Image>>,
}
```

**拡張ポイント**
- マイクラ形式のスキンPNG読み込み（64x64）
- スキン選択UI
- オンラインスキンサーバー対応（将来）

---

### 15. モデルインポート（Blockbench対応）

**参考ツール**:
- [Blockbench Import Library](https://github.com/tomalbrc/blockbench-import-library) - Fabric用ライブラリ
- [GeckoLib](https://github.com/bernie-g/geckolib) - アニメーションエンジン
- [Chameleon](https://github.com/mchorse/chameleon) - Blockbenchモデル読み込み

**データ構造**
```rust
// assets/model_loader.rs
pub struct BlockbenchModel {
    pub meshes: Vec<Mesh>,
    pub textures: Vec<Handle<Image>>,
    pub animations: HashMap<String, Animation>,
    pub bones: Vec<Bone>,
}

pub struct Animation {
    pub name: String,
    pub duration: f32,
    pub keyframes: Vec<Keyframe>,
    pub loop_mode: LoopMode,
}

// .bbmodel (JSON) パーサー
pub fn load_bbmodel(path: &str) -> Result<BlockbenchModel, Error>;
```

**対応フォーマット**
- `.bbmodel` (Blockbench Generic)
- `.ajmodel` (Animated Java)
- glTF/GLB（既存のBevy対応）

**拡張ポイント**
- 新モジュール: `assets/blockbench.rs`
- アニメーション再生システム
- Modからのモデル読み込み
- ホットリロード対応

---

### 16. Modding対応（フルアクセス）

**目標: Factorioレベルの深いMod対応**

**設計方針（2026-01-08 確定）**:
- **ハイブリッド方式**: WASM（ロジック）+ WebSocket（スクリプト）+ TOML（データ）
- base は本体に内蔵（Engine + base 一体）
- Core Mod で新ロジック定義 → Data Mod で別ユーザーが拡張可能

**Modレイヤー構成**
| レイヤー | 実行方式 | 言語 | 用途 |
|---------|---------|------|------|
| **Core Mod** | WASM（同一プロセス） | Rust, C++ | ロジック追加・置換 |
| **Script Mod** | WebSocket（別プロセス） | Python, Lua, JS | イベントフック |
| **Data Mod** | TOML読み込み | - | アイテム/機械/レシピ |

**Modで変更可能にする範囲**
| レイヤー | 内容 | 実現方法 |
|----------|------|----------|
| コンテンツ | ブロック、アイテム、機械、レシピ | Data Mod（TOML） |
| ロジック | 機械動作、物流ルール | Core Mod（WASM） |
| イベント | フック、条件分岐 | Script Mod（WebSocket）/ Core Mod |
| UI | カスタムUI、HUD | Core Mod |
| レンダリング | シェーダー、エフェクト | カスタムシェーダー |
| モデル | 機械・Mobの3Dモデル | Blockbench直接インポート |
| サウンド | BGM、効果音 | OGG/WAVファイル |
| スキン | プレイヤー見た目 | PNG画像 |

**アーキテクチャ**

```rust
// 動的ID（Mod追加アイテム用）
pub struct ItemId(u32);

pub struct ModRegistry {
    pub items: HashMap<ItemId, ItemDescriptor>,
    pub machines: HashMap<MachineId, MachineDescriptor>,
    pub recipes: HashMap<RecipeId, RecipeDescriptor>,
    pub models: HashMap<String, BlockbenchModel>,
    pub sounds: HashMap<String, SoundSpec>,
}

// 外部Mod API
pub struct ModApiServer {
    // WebSocket or JSON-RPC
    pub connections: Vec<ModConnection>,
}
```

**Mod API（外部プロセスから呼び出し）**
```json
// WebSocket経由でイベント購読
{ "action": "subscribe", "events": ["block_placed", "machine_processed"] }

// イベント通知（ゲーム→Mod）
{ "event": "block_placed", "pos": [10, 5, 20], "block": "iron_ore" }

// アクション実行（Mod→ゲーム）
{ "action": "register_item", "id": "mymod:super_ingot", "descriptor": {...} }
{ "action": "spawn_entity", "type": "machine", "pos": [10, 5, 20] }
```

**Data Mod（TOML形式）**
```toml
# mods/my_mod/items.toml
[[items]]
id = "mymod:super_ingot"
name = "超合金インゴット"
color = [0.8, 0.8, 0.2]
stack_size = 64

[[recipes]]
machine = "furnace"
inputs = [{ id = "mymod:rare_ore", count = 2 }]
outputs = [{ id = "mymod:super_ingot", count = 1 }]
```

**必要な基盤**
1. **イベントシステム**: Bevy Observer + 外部通知 ✅ 基盤完了
2. **動的レジストリ**: enum廃止、ID化 ✅ 基盤完了
3. **Data Mod ローダー**: TOML読み込み ✅ 実装済み
4. **WebSocket API**: Script Mod 用（基盤あり）
5. **WASM ローダー**: Wasmtime 統合（将来）
6. **アセットローダー**: Mod用アセットパス対応
7. **Blockbenchローダー**: .bbmodel パーサー

**実装順序**
1. Phase C: 内部でDescriptor化 ✅完了
2. イベントシステム ✅完了
3. 動的ID基盤 ✅完了（移行中）
4. Data Mod ローダー ✅完了
5. WebSocket API 起動・安定化
6. **WASM ローダー（Wasmtime）** ← 新規
7. Blockbenchローダー実装
8. Mod API仕様公開

---

### 17. マルチプレイ基盤

**必要な変更（大規模）**
```rust
// components/network.rs
pub struct NetworkId(u64);
pub struct Authority {
    pub owner: PlayerId,
}

// 同期対象
- プレイヤー位置
- ブロック変更
- 機械状態
- インベントリ
- チャンク読み込み
```

**拡張ポイント**
- 新モジュール: `networking/`
- 全システムに同期ポイント追加
- サーバー権威モデル

---

## 依存関係グラフ（2026-01-07 更新）

```
                    ┌─────────────────────────────────┐
                    │   Bevy Observer イベント基盤    │
                    │   （フックポイント）            │
                    └─────────────────────────────────┘
                                   │
            ┌──────────────────────┼──────────────────────┐
            │                      │                      │
            ↓                      ↓                      ↓
    ┌───────────────┐    ┌─────────────────┐    ┌─────────────┐
    │データ駆動Mod  │    │  動的レジストリ  │    │  Mod API    │
    │(TOML/JSON)    │    │  (enum廃止)     │    │ (WebSocket) │
    └───────────────┘    └─────────────────┘    └─────────────┘
            │                      │                      │
            └──────────────────────┼──────────────────────┘
                                   │
                    ┌──────────────┴──────────────┐
                    │    フルアクセスModding完成   │
                    │  （外部プロセスから制御可能）│
                    └─────────────────────────────┘
                                   │
    ┌──────────────────────────────┼──────────────────────────────┐
    │              │               │               │              │
    ↓              ↓               ↓               ↓              ↓
┌──────┐    ┌──────────┐    ┌──────────┐    ┌─────────┐    ┌──────┐
│ 電力 │    │液体・気体│    │ 信号制御 │    │  線路   │    │ Mob  │
└──────┘    └──────────┘    └──────────┘    └─────────┘    └──────┘


            独立機能（いつでも実装可能）
            ════════════════════════════

    ┌─────────┐  ┌─────────────┐  ┌──────────┐  ┌─────────┐
    │ マップ  │  │ブループリント│  │ 統計分析 │  │ サウンド│
    └─────────┘  └─────────────┘  └──────────┘  └─────────┘

    ┌─────────┐  ┌─────────────┐  ┌──────────┐  ┌───────────┐
    │ 実績    │  │プレイヤー   │  │ストレージ│  │Blockbench │
    │         │  │スキン       │  │          │  │インポート │
    └─────────┘  └─────────────┘  └──────────┘  └───────────┘

    ┌──────────┐  ┌──────────┐
    │クラフト  │  │ロボット  │ ← プリセット動作
    │システム  │  │          │   Mod APIで拡張可能
    └──────────┘  └──────────┘


            最後に実装
            ══════════

                    ┌─────────────────┐
                    │ マルチプレイ基盤│ ← PlayerInventory Component化が前提
                    └─────────────────┘
```

**設計方針**:
- **ゲーム本体にLua等のスクリプト言語は統合しない**
- Bevy ObserverでイベントAPIを提供
- 外部ModはWebSocket/JSON-RPC経由で連携
- データ駆動Modding（TOML/JSON）で大半のMod対応が可能
- ロジック変更は外部プロセスから

---

## 推奨実装順序（2026-01-07 更新）

### 最優先（基盤）

**これらは機能追加より先にやる。後からは困難。**

| Phase | 機能 | 理由 | 状態 |
|-------|------|------|------|
| **D.0** | **PlayerInventory Component化 + MachineBundle + 安全API** | マルチ確定、後から困難 | ⚠️ 最優先 |
| **D.1** | **イベントシステム（全フック）** | マルチ同期・Mod・デバッグ | 未着手 |
| **D.2** | **動的ID + Phantom Type + newtype** | Mod追加アイテム、型安全 | 未着手 |

### 基盤フェーズ（D.0-D.2 完了後）

| Phase | 機能 | 理由 |
|-------|------|------|
| D.3 | Mod API Server | WebSocket/JSON-RPC基盤 |
| D.4 | データ駆動Modding | TOML/JSONでコンテンツ追加 |
| D.5 | Blockbenchローダー | モデル追加の基盤 |

### 独立機能（並行可）

| Phase | 機能 | 理由 |
|-------|------|------|
| D.6 | マップ機能 | 独立、QoL向上 |
| D.7 | ブループリント | 独立、大規模建築効率化 |
| D.8 | クラフトシステム | 手動クラフト |
| D.9 | ストレージ | 倉庫ブロック |
| D.10 | 統計・分析 | 生産可視化 |
| D.11 | サウンド | BGM、効果音 |
| D.12 | 実績システム | Steam連携 |
| D.13 | プレイヤースキン | 見た目変更 |
| D.14 | ロボット（データ駆動） | Mod APIで拡張可能 |

### Mod対応しながら機能追加

| Phase | 機能 | 理由 |
|-------|------|------|
| D.15 | 電力システム + Mod API | 最初のMod対応機能として |
| D.16 | 液体・気体 + Mod API | 電力と同パターン |
| D.17 | 信号制御 + Mod API | 電力・流体の条件分岐 |
| D.18 | 線路機能 + Mod API | 信号制御で閉塞管理 |
| D.19 | Mob + Mod API | 敵/NPCのMod追加可能に |

### 最後

| Phase | 機能 | 理由 |
|-------|------|------|
| D.20 | マルチプレイ本実装 | 全機能の同期設計（D.0-D.2が前提） |

**ポイント**:
- **D.0-D.2 は機能追加より優先**（後から変更は困難）
- **ゲーム本体にLua/WASM等は統合しない** → 外部Modに任せる
- 各機能追加時にMod APIも同時設計
- ロボットはデータ駆動、Mod APIで外部から制御拡張可能

---

## 共通拡張ポイント（先に整備すべき）

### 1. IoPortの拡張
```rust
pub struct IoPort {
    pub side: PortSide,
    pub port_type: PortType,  // 追加: Item, Fluid, Power, Signal
    pub slot_id: u8,
}

pub enum PortType {
    Item,
    Fluid,
    Power,
    Signal,
}
```

### 2. ネットワーク（グラフ）基盤
```rust
// 電力グリッド、信号網、パイプ網で共通
pub trait NetworkNode {
    fn position(&self) -> IVec3;
    fn connections(&self) -> Vec<IVec3>;
}

pub struct Network<T: NetworkNode> {
    pub nodes: HashMap<IVec3, Entity>,
    pub graph: Graph<Entity>,  // petgraphなど
}
```

### 3. Descriptor統一
```rust
// 全てのゲームオブジェクトを同じパターンで定義
pub trait GameDescriptor {
    fn id(&self) -> &'static str;
    fn name(&self) -> &'static str;
}
```

### 4. 天候・昼夜の拡張ポイント（将来用）
```rust
// 今は実装しないが、拡張可能な設計にしておく
pub struct TimeOfDay {
    pub hour: f32,  // 0.0-24.0
}

pub struct Weather {
    pub weather_type: WeatherType,
    pub intensity: f32,
}

pub enum WeatherType {
    Clear,
    Rain,
    Storm,
    // Modで追加可能
}
```

---

## AI安全基盤（コンパイラ/テストによる保護）

**目標**: AIが一部のファイルだけ読んで機能追加しても、全体が壊れない状態を作る。

### 保護の原則

| 原則 | 内容 |
|------|------|
| **コンパイラが守る** | 型システムで不整合を検出 |
| **テストが守る** | 壊れたら即座にわかる |
| **Bundleが守る** | 必須コンポーネントをBundleで強制 |
| **APIが守る** | 直接操作を禁止、安全なAPIのみ公開 |

### D.0と同時に実装すべき保護策

#### 1. MachineBundle導入

```rust
#[derive(Bundle)]
pub struct MachineBundle {
    pub machine: Machine,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub visibility: Visibility,
    // 必須コンポーネントを全て含む
}

// AIはこれだけ書けばOK
commands.spawn(MachineBundle::new(&MINER, pos, Direction::North));
```

#### 2. 安全なワールド操作API

```rust
// WorldData の直接操作を禁止
impl WorldData {
    /// 内部専用（外部から呼べない）
    pub(crate) fn set_block_internal(&mut self, pos: IVec3, block: BlockType) { ... }
}

// ✅ 安全なAPI（イベント自動発火）
pub struct WorldCommands<'w> { ... }

impl WorldCommands<'_> {
    /// ブロック配置（BlockPlacedイベント自動発火）
    pub fn place_block(&mut self, pos: IVec3, block: ItemId, source: EventSource) {
        self.world.set_block_internal(pos, block);
        self.events.send(BlockPlaced { pos, block, source });
    }
}
```

**強制方法**: `set_block` を `pub(crate)` にして外部から直接呼べなくする

#### 3. 網羅性テスト

```rust
#[test]
fn all_recipe_items_exist_in_registry() {
    for recipe in ALL_RECIPES {
        for (input, _) in recipe.inputs {
            assert!(ITEM_DESCRIPTORS.iter().any(|(bt, _)| *bt == *input));
        }
    }
}

#[test]
fn machine_ports_match_ui_slots() {
    for machine in ALL_MACHINES {
        for port in machine.ports {
            assert!(machine.ui_slots.iter().any(|s| s.slot_id == port.slot_id));
        }
    }
}

#[test]
fn all_recipe_machines_have_recipes() {
    for machine in ALL_MACHINES {
        if let ProcessType::Recipe(mt) = machine.process_type {
            assert!(
                get_recipes_for_machine(mt).len() > 0,
                "Machine {} has Recipe process type but no recipes",
                machine.id
            );
        }
    }
}

#[test]
fn all_placeable_blocks_have_model_or_fallback() {
    for (block_type, desc) in ITEM_DESCRIPTORS {
        if desc.is_placeable {
            let glb = format!("assets/models/{}.glb", block_type.model_name());
            let vox = format!("assets/models/{}.vox", block_type.model_name());
            assert!(
                Path::new(&glb).exists() || Path::new(&vox).exists() || block_type.has_fallback(),
                "BlockType::{:?} has no model file",
                block_type
            );
        }
    }
}
```

#### 4. デバッグダンプ機能（F12）

```rust
// AIがゲーム状態を把握できるようにする
pub fn dump_game_state(...) -> String {
    serde_json::to_string_pretty(&GameStateDump {
        machines: ...,
        conveyors: ...,
        inventory: ...,
    }).unwrap()
}
```

### newtype パターン（Phase D.2で実装）

```rust
// 型混同をコンパイルエラーに
pub struct InputSlotId(pub u8);
pub struct OutputSlotId(pub u8);
pub struct FuelSlotId(pub u8);

pub enum SlotRef {
    Input(InputSlotId),
    Output(OutputSlotId),
    Fuel(FuelSlotId),
}
```

### 変更影響マップ

#### BlockType を追加した場合
- ✅ 必須: `game_spec/registry.rs` の `ITEM_DESCRIPTORS` に追加
- ✅ 必須: `cargo test test_all_block_types_registered` が通る
- 可能性: レシピ、地形生成、3Dモデル

#### MachineSpec を追加した場合
- ✅ 必須: `game_spec/machines.rs` の `ALL_MACHINES` に追加
- ✅ 必須: `game_spec/recipes.rs` に対応レシピ追加
- ✅ 必須: `block_type.rs` に対応する `BlockType` 追加
- 自動: UI生成、tick処理、コンベア接続

---

## パニック防止戦略（2026-01-07 策定）

**目標**: Modアイテム追加・削除でゲームがクラッシュしない設計

### パニックの分類と対処方針

| カテゴリ | 発生源 | 対処方針 | 優先度 |
|----------|--------|----------|--------|
| **Mod境界** | `BlockType↔ItemId`変換 | **フォールバック必須** | 🔴 最優先 |
| **外部入力** | セーブデータ、TOML | **Result返却 + ログ** | 🔴 高 |
| **アセット** | Blockbench等 | **エラー画像/メッシュ置換** | 🟡 中 |
| **内部ロジック** | 「起こり得ない」状態 | **debug_assertのみ** | 🟢 低 |

### 現状のパニック箇所（2026-01-07時点）

```
unwrap(): 92箇所
expect(): 72箇所
panic!:   15箇所
合計:     179箇所
```

### 致命的なパニック箇所（Modアイテム経路）

| ファイル | 行 | 問題 | 対策 |
|----------|-----|------|------|
| machines.rs:93 | ConveyorItem::new | Modアイテムがコンベアに乗ると即死 | ItemId直接保持 |
| machines.rs:119 | ConveyorItem::set_item_id | 同上 | 同上 |
| machines.rs:375 | MachineSlot::new | Modアイテムがスロットに入ると即死 | 同上 |
| machines.rs:383 | MachineSlot設定 | 同上 | 同上 |
| mod.rs:356 | Quest::new | Modアイテムがクエスト要求に入ると即死 | 同上 |
| mod.rs:363 | Quest報酬 | 同上 | 同上 |
| craft/mod.rs:131,150 | レシピ | Modアイテムのレシピで即死 | 同上 |

### その他の危険なパニック箇所

| 優先度 | ファイル | 問題 | 対策 |
|--------|----------|------|------|
| 🔴 P0 | core/id.rs:191 | `from_block_type_static` が無条件パニック | Result返却に変更 |
| 🔴 P1 | save/format.rs:1660 | 不明ItemIDでパニック | フォールバック（空気に置換） |
| 🟡 P2 | blockbench.rs:907,945,997等 | JSON/Base64パース失敗でパニック | Resultチェーン |
| 🟡 P2 | blockbench.rs:1235,1269,1285等 | 不正bone構造でパニック | フォールバックメッシュ |

### 設計と実装の乖離

| 設計書の記述 | 現状 | 差異 |
|------------|------|------|
| パニック防止戦略 P.1-P.5 | ConveyorItem, MachineSlot, Quest | ❌ 完全未着手 |
| Mod境界でのフォールバック | save/format.rs | ❌ パニック直結 |
| ValidItemId型 | 未実装 | ❌ ItemIdは未検証で使用 |
| GuardedEventWriter | events/guarded_writer.rs | ✅ 実装済み、だが未使用 |
| EntityMap | 未実装 | ❌ LocalPlayerがEntity直接参照 |

### 境界条件の見落とし

#### 1. Base Game ↔ Mod

| 経路 | 危険な操作 | 現状 | 対策 |
|------|-----------|------|------|
| コンベア | ItemId → BlockType | パニック | ItemId直接保持 |
| マシン | ItemId → BlockType | パニック | ItemId直接保持 |
| クエスト | Modアイテム要求 | パニック | ItemId対応 |
| セーブ | 不明ItemId | パニック | フォールバック |
| レシピ | Modレシピ | パニック | Mod専用API |

#### 2. 内部データ ↔ 外部入力

| 入力元 | バリデーション | リスク |
|--------|--------------|--------|
| セーブファイル | 部分的 | Mod削除時に不明ID→パニック |
| Blockbench | なし | 不正bone構造→パニック |
| TOML/Mod定義 | なし | 未実装 |

#### 3. Single ↔ Multi Player

| データ | 現状 | 問題 |
|--------|------|------|
| LocalPlayer | Entity直接参照 | マルチでEntity異なる |
| Machine State | Entity + Position | 同期機構なし |

### 不足しているテスト

```rust
// ❌ 未実装：Modアイテムがコンベアを生き残るか
#[test]
fn mod_item_on_conveyor_no_panic() {
    let mod_item = ItemId::from_string("test_mod:custom_ore");
    let conveyor = ConveyorItem::new(mod_item);  // 現在はパニック
}

// ❌ 未実装：Mod削除後のセーブ再ロード
#[test]
fn save_with_unknown_item_has_fallback() {
    // 不明アイテムは空気に置換されるべき
}

// ❌ 未実装：不正Blockbenchファイル
#[test]
fn malformed_blockbench_returns_error() {
    let result = load_blockbench("{ invalid }");
    assert!(result.is_err());  // 現在はパニック
}
```

### 対策パターン

#### パターンA: フォールバックアイテム（Mod境界用）

```rust
// 不明なアイテムを表す特別なItemId
pub const UNKNOWN_ITEM: ItemId = ItemId::from_raw(0); // "base:unknown"

// ConveyorItemの安全な実装
impl ConveyorItem {
    pub fn new(item_id: ItemId) -> Self {
        Self {
            item_id,  // BlockTypeを経由しない！
            progress: 0.0,
            visual_entity: None,
        }
    }

    /// 描画時のみBlockType参照（失敗したらフォールバック表示）
    pub fn visual_block_type(&self) -> BlockType {
        self.item_id.to_block_type()
            .unwrap_or(BlockType::Stone) // 見た目だけなので石でOK
    }
}
```

#### パターンB: Result型での伝播（セーブ/ロード用）

```rust
pub fn load_game(path: &Path) -> Result<SaveData, LoadError> {
    // 不明アイテムはログ出力して除外
    let items: Vec<_> = save.inventory
        .into_iter()
        .filter_map(|item| {
            if registry.exists(item.id) {
                Some(item)
            } else {
                warn!("Unknown item '{}' removed from save", item.id);
                None
            }
        })
        .collect();
    Ok(SaveData { inventory: items, .. })
}
```

#### パターンC: 型による制約（ValidItemId）

```rust
/// Registryに存在することが保証されたItemId
pub struct ValidItemId(ItemId);

impl GameRegistry {
    /// 未検証のIDを検証
    pub fn validate(&self, id: ItemId) -> Option<ValidItemId> {
        if self.items.contains(id.raw()) {
            Some(ValidItemId(id))
        } else {
            None
        }
    }
}
```

### 安全性レベル定義

| レベル | 定義 | 必要作業 |
|--------|------|----------|
| **L1** | Mod対応として最低限 | P.0-P.3 |
| **L2** | 外部入力全般に堅牢 | L1 + P.4 + GuardedEventWriter使用 |
| **L3** | 将来拡張も安全 | L2 + P.5 + EntityMap + StringInterner |

### 修正フェーズ

| Phase | 対象 | 内容 | レベル | 状態 |
|-------|------|------|--------|------|
| **P.0** | core/id.rs | `from_block_type_static`をResult返却 | L1 | ❌ 未着手 |
| **P.1** | ConveyorItem, MachineSlot | BlockType廃止→ItemId直接保持 | L1 | ❌ 未着手 |
| **P.2** | Quest, Craft | 同上 | L1 | ❌ 未着手 |
| **P.3** | セーブ/ロード | 不明アイテムフィルタリング | L1 | ❌ 未着手 |
| **P.4** | Blockbench | フォールバックメッシュ | L2 | ❌ 未着手 |
| **P.5** | ValidItemId | 型安全強化 | L3 | ❌ 未着手 |

### 追加タスク（L2-L3用）

| タスク | 内容 | レベル | 状態 |
|--------|------|--------|------|
| **GuardedEventWriter使用** | 全イベント送信箇所で使用開始 | L2 | ❌ 未着手 |
| **EntityMap実装** | NetworkId ↔ Entity マッピング | L3 | ❌ 未着手 |
| **StringInterner安全化** | Arc<RwLock>でスレッドセーフに | L3 | ❌ 未着手 |

### 作業順序

```
今すぐ（L1達成）
├── P.0: core/id.rs Result返却
├── P.1: ConveyorItem, MachineSlot
├── P.2: Quest, Craft
└── P.3: セーブ/ロード フォールバック
    ↓
D.15着手前（L2達成）
├── P.4: Blockbench エラーハンドリング
└── GuardedEventWriter使用開始
    ↓
D.15-D.19と並行（L3達成）
├── P.5: ValidItemId導入
└── EntityMap実装
    ↓
D.20（マルチ）前
└── StringInternerスレッドセーフ化
```

### 検証テスト（P.1完了後に追加）

```rust
#[test]
fn mod_item_survives_conveyor_flow() {
    // Modアイテム（BlockTypeに存在しない）を作成
    let mod_item = ItemId::from_string("test_mod:super_ingot");

    // コンベアに乗せる → パニックしない
    let conveyor_item = ConveyorItem::new(mod_item);

    // 機械スロットに入れる → パニックしない
    let slot = MachineSlot::new(mod_item, 1);

    // 描画用BlockType取得 → フォールバック
    assert!(conveyor_item.visual_block_type().is_some());
}
```

---

## 次のアクション

### 完了済み
- ✅ Phase C完了（BlockDescriptor, ItemDescriptor, GameRegistry）
- ✅ Phase D.0-D.14 基盤実装完了

### Phase D: 基盤強化（実装状況）

| 順序 | タスク | 状態 | 備考 |
|------|--------|------|------|
| **D.0** | PlayerInventory Component化 + MachineBundle | ✅ 完了 | |
| D.1 | イベントシステム | ✅ 完了 | 7イベント送信、7箇所購読 |
| D.2 | 動的ID基盤 | ✅ 基盤完了 | 移行10%、段階的に継続 |
| D.3 | Mod API Server | ✅ 基盤完了 | WebSocket未起動 |
| D.4 | データ駆動Modding | ✅ 完了 | 起動時ロード実装済み |
| D.5-D.14 | 各機能Plugin | ✅ 完了 | 全登録済み |

### 次の優先タスク

1. **パニック防止 P.1-P.2**: ConveyorItem, MachineSlot, QuestのBlockType依存除去
2. **Phase D.15**: 電力システム（新アーキ実証）
3. **BlockType移行**: 新機能実装時に段階的に

### 独立機能（Phase D完了後、並行可）
- マップ機能
- ブループリント
- クラフトシステム
- ストレージ
- サウンド

### 基盤システム（Mod API対応しながら）
- 電力 → 液体 → 信号 → 線路 の順
