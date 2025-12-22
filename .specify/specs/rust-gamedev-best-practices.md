# Rust ゲーム開発 ベストプラクティス

**作成日**: 2025-12-22
**目的**: Rustの特性を活かしたゲーム開発の実装指針

---

## 1. メモリ管理

### 1.1 所有権とボローイング

```rust
// 良い例: 明確な所有権
struct Player {
    position: Vec3,
    inventory: Inventory,  // Playerが所有
}

impl Player {
    // 借用で参照を渡す
    fn position(&self) -> &Vec3 {
        &self.position
    }

    // 可変借用で変更を許可
    fn move_to(&mut self, new_pos: Vec3) {
        self.position = new_pos;
    }
}

// 悪い例: 不必要なクローン
fn process_items(inventory: Inventory) {  // 所有権を奪う
    // ...
}
// 呼び出し側でclone()が必要

// 良い例: 参照で渡す
fn process_items(inventory: &Inventory) {
    // ...
}
```

### 1.2 スマートポインタの使い分け

```rust
// Box: 単一所有権、ヒープ割り当て
let boxed: Box<LargeData> = Box::new(LargeData::default());

// Rc: 単一スレッド、共有所有権
use std::rc::Rc;
let shared: Rc<GameAsset> = Rc::new(load_asset("texture.png"));
let shared2 = Rc::clone(&shared);  // 参照カウント増加

// Arc: マルチスレッド、共有所有権
use std::sync::Arc;
let thread_safe: Arc<GameState> = Arc::new(GameState::new());

// RefCell: 実行時借用チェック（単一スレッド）
use std::cell::RefCell;
let mutable: RefCell<Player> = RefCell::new(Player::new());
mutable.borrow_mut().health -= 10;
```

### 1.3 アロケーション最小化

```rust
// 良い例: 事前割り当て
struct ParticleSystem {
    particles: Vec<Particle>,
}

impl ParticleSystem {
    fn new(capacity: usize) -> Self {
        Self {
            particles: Vec::with_capacity(capacity),  // 事前割り当て
        }
    }

    fn spawn(&mut self, particle: Particle) {
        // 容量内ならアロケーションなし
        self.particles.push(particle);
    }
}

// 悪い例: 毎フレーム新規Vec
fn bad_update() {
    let entities = Vec::new();  // 毎フレームアロケーション
    // ...
}

// 良い例: 再利用
struct GameState {
    temp_buffer: Vec<Entity>,  // 再利用
}

fn good_update(state: &mut GameState) {
    state.temp_buffer.clear();  // 容量は維持
    // ...
}
```

---

## 2. パフォーマンス最適化

### 2.1 スタック vs ヒープ

```rust
// スタック: 高速だがサイズ固定
struct StackData {
    position: [f32; 3],  // 12バイト、スタック
    rotation: [f32; 4],  // 16バイト、スタック
}

// ヒープ: 柔軟だが遅い
struct HeapData {
    items: Vec<Item>,       // ヒープ
    name: String,           // ヒープ
}

// 小さなデータはスタックに
#[derive(Clone, Copy)]  // Copy可能 = スタックに適している
struct Position(Vec3);
```

### 2.2 イテレータの活用

```rust
// 良い例: イテレータチェーン（遅延評価）
let sum: f32 = entities.iter()
    .filter(|e| e.is_active())
    .map(|e| e.value())
    .sum();  // 一度のイテレーションで完了

// 悪い例: 中間コレクション
let active: Vec<_> = entities.iter()
    .filter(|e| e.is_active())
    .collect();  // 中間Vecを作成
let sum: f32 = active.iter()
    .map(|e| e.value())
    .sum();
```

### 2.3 SIMD活用

```rust
// glam（Bevyの数学ライブラリ）はSIMD対応
use glam::Vec3;

fn update_positions(positions: &mut [Vec3], velocity: Vec3, dt: f32) {
    let delta = velocity * dt;  // SIMD演算
    for pos in positions {
        *pos += delta;  // SIMD加算
    }
}

// 並列処理にはrayonを使用
use rayon::prelude::*;

fn parallel_update(entities: &mut [Entity]) {
    entities.par_iter_mut().for_each(|entity| {
        entity.update();
    });
}
```

---

## 3. エラーハンドリング

### 3.1 Result と Option

```rust
// ゲーム操作にはResultを使用
fn load_save(path: &Path) -> Result<GameSave, SaveError> {
    let data = std::fs::read(path)?;
    let save: GameSave = bincode::deserialize(&data)?;
    Ok(save)
}

#[derive(Debug, thiserror::Error)]
enum SaveError {
    #[error("ファイルが見つかりません: {0}")]
    NotFound(PathBuf),
    #[error("データが破損しています")]
    Corrupted,
    #[error("バージョン非互換: {0}")]
    IncompatibleVersion(u32),
}

// 存在しない可能性があるものにはOption
fn get_item(inventory: &Inventory, slot: usize) -> Option<&Item> {
    inventory.slots.get(slot)
}
```

### 3.2 パニック回避

```rust
// 悪い例: 配列外アクセスでパニック
fn bad_access(items: &[Item], index: usize) -> &Item {
    &items[index]  // 範囲外でパニック！
}

// 良い例: 安全なアクセス
fn safe_access(items: &[Item], index: usize) -> Option<&Item> {
    items.get(index)
}

// ゲームロジックでは unwrap/expect を避ける
// ただし、初期化時など確実な場合はOK
fn setup() {
    let config = load_config().expect("設定ファイルは必須");
}
```

### 3.3 ログとデバッグ

```rust
use tracing::{info, warn, error, debug, trace};

fn game_update() {
    trace!("フレーム開始");

    if player.health < 20 {
        warn!(health = player.health, "プレイヤーの体力が低い");
    }

    match spawn_enemy() {
        Ok(enemy) => info!(?enemy, "敵をスポーン"),
        Err(e) => error!(?e, "敵のスポーンに失敗"),
    }
}
```

---

## 4. 並行性

### 4.1 マルチスレッド

```rust
use std::thread;
use std::sync::{Arc, Mutex};

// スレッドセーフなゲーム状態
struct SharedState {
    data: Arc<Mutex<GameData>>,
}

impl SharedState {
    fn update(&self) {
        let mut data = self.data.lock().unwrap();
        data.tick += 1;
    }
}

// チャネルでスレッド間通信
use std::sync::mpsc;

fn audio_thread() {
    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        while let Ok(sound) = rx.recv() {
            play_sound(sound);
        }
    });

    // メインスレッドからサウンドを送信
    tx.send(Sound::Explosion).unwrap();
}
```

### 4.2 async/await

```rust
use tokio;

// ネットワーク操作に async
async fn fetch_leaderboard() -> Result<Leaderboard, NetworkError> {
    let response = reqwest::get("https://api.game.com/leaderboard")
        .await?;
    let data = response.json().await?;
    Ok(data)
}

// Bevyでの非同期タスク
fn spawn_async_task(runtime: Res<TokioRuntime>) {
    runtime.spawn(async {
        let leaderboard = fetch_leaderboard().await;
        // ...
    });
}
```

---

## 5. データ構造

### 5.1 効率的なコレクション

```rust
// Vec: 順序付き、高速イテレーション
let entities: Vec<Entity> = Vec::new();

// HashMap: O(1)ルックアップ
use std::collections::HashMap;
let items: HashMap<ItemId, Item> = HashMap::new();

// HashSet: 重複なし
use std::collections::HashSet;
let active_ids: HashSet<EntityId> = HashSet::new();

// BTreeMap: ソート済み
use std::collections::BTreeMap;
let sorted_scores: BTreeMap<u32, PlayerId> = BTreeMap::new();
```

### 5.2 スロットマップ

```rust
// 高速なエンティティ管理にslotmap
use slotmap::{SlotMap, new_key_type};

new_key_type! {
    pub struct EntityKey;
}

struct EntityManager {
    entities: SlotMap<EntityKey, Entity>,
}

impl EntityManager {
    fn spawn(&mut self, entity: Entity) -> EntityKey {
        self.entities.insert(entity)
    }

    fn get(&self, key: EntityKey) -> Option<&Entity> {
        self.entities.get(key)
    }

    fn remove(&mut self, key: EntityKey) -> Option<Entity> {
        self.entities.remove(key)
    }
}
```

### 5.3 Arena Allocation

```rust
// 同じライフタイムのオブジェクトをまとめて管理
use bumpalo::Bump;

fn frame_update(arena: &Bump) {
    // フレーム中のみ有効なアロケーション
    let temp_data = arena.alloc_slice_fill_default::<f32>(1000);

    // フレーム終了時に一括解放
}
```

---

## 6. シリアライズ

### 6.1 serde活用

```rust
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct GameSave {
    version: u32,
    player: PlayerData,
    world: WorldData,
    #[serde(default)]  // 後方互換性
    new_field: Option<NewData>,
}

// バイナリ形式（高速、コンパクト）
fn save_binary(save: &GameSave) -> Result<Vec<u8>, Error> {
    bincode::serialize(save)
}

// JSON形式（デバッグ用）
fn save_json(save: &GameSave) -> Result<String, Error> {
    serde_json::to_string_pretty(save)
}
```

### 6.2 バージョン管理

```rust
#[derive(Serialize, Deserialize)]
struct SaveFile {
    version: u32,
    #[serde(flatten)]
    data: SaveData,
}

fn load_save(bytes: &[u8]) -> Result<SaveData, Error> {
    let file: SaveFile = bincode::deserialize(bytes)?;

    match file.version {
        1 => migrate_v1_to_v2(file.data),
        2 => Ok(file.data),
        v => Err(Error::UnsupportedVersion(v)),
    }
}
```

---

## 7. テスト

### 7.1 ユニットテスト

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_damage_calculation() {
        let attacker = Entity { attack: 10 };
        let defender = Entity { defense: 3 };

        let damage = calculate_damage(&attacker, &defender);

        assert_eq!(damage, 7);
    }

    #[test]
    fn test_inventory_add() {
        let mut inv = Inventory::new(10);

        assert!(inv.add(Item::IronOre, 5).is_ok());
        assert_eq!(inv.count(Item::IronOre), 5);
    }
}
```

### 7.2 プロパティベーステスト

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn inventory_never_negative(
        items in prop::collection::vec(any::<Item>(), 0..100)
    ) {
        let mut inv = Inventory::new(100);

        for item in items {
            let _ = inv.add(item, 1);
        }

        for count in inv.all_counts() {
            prop_assert!(count >= 0);
        }
    }
}
```

---

## 8. チェックリスト

### メモリ管理
- [ ] 不必要なクローンを避けているか
- [ ] 適切なスマートポインタを使用しているか
- [ ] アロケーションを最小化しているか

### パフォーマンス
- [ ] ホットパスでヒープ割り当てを避けているか
- [ ] イテレータを活用しているか
- [ ] 並列処理の機会を活かしているか

### 安全性
- [ ] unwrap/expectを避けているか
- [ ] 適切なエラー型を定義しているか
- [ ] ログを適切に出力しているか

### コード品質
- [ ] 所有権が明確か
- [ ] テストがあるか
- [ ] ドキュメントがあるか

---

## 参考文献

- [Rust Game Development - Rapid Innovation](https://www.rapidinnovation.io/post/rust-game-engines-the-complete-guide-for-modern-game-development)
- [Memory Management for High-Performance Games in Rust](https://www.javacodegeeks.com/2024/11/memory-management-techniques-for-high-performance-games-in-rust.html)
- [A Guide to Safe and Concurrent Game Development](https://30dayscoding.com/blog/game-development-with-rust-safe-and-concurrent-game-programming)

---

*このレポートはRustゲーム開発のベストプラクティス調査に基づいています。*
