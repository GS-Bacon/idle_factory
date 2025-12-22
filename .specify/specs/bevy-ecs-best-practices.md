# Bevy ECS ベストプラクティス

**作成日**: 2025-12-22
**目的**: Bevy ECSを効果的に活用するための実装指針

---

## 1. ECSの基本概念

### 1.1 コアコンセプト

```
Entity（エンティティ）:
  - 一意のID
  - コンポーネントのコンテナ
  - 自体はデータを持たない

Component（コンポーネント）:
  - 純粋なデータ
  - 振る舞いを持たない
  - 小さく分割する

System（システム）:
  - ロジック/振る舞い
  - コンポーネントを操作
  - 並列実行可能

Resource（リソース）:
  - グローバルなデータ
  - ゲーム設定、入力状態など
```

### 1.2 Bevyの特徴

```rust
// シンプルなデータ型がコンポーネント
#[derive(Component)]
struct Position(Vec3);

#[derive(Component)]
struct Velocity(Vec3);

#[derive(Component)]
struct Health(i32);

// システムは通常の関数
fn movement_system(
    time: Res<Time>,
    mut query: Query<(&mut Position, &Velocity)>,
) {
    for (mut pos, vel) in &mut query {
        pos.0 += vel.0 * time.delta_secs();
    }
}
```

---

## 2. コンポーネント設計

### 2.1 小さく分割する

```rust
// 良い例: 細かく分割
#[derive(Component)]
struct Position(Vec3);

#[derive(Component)]
struct Velocity(Vec3);

#[derive(Component)]
struct Health(i32);

#[derive(Component)]
struct MaxHealth(i32);

// 悪い例: 大きなモノリシックコンポーネント
#[derive(Component)]
struct Player {
    position: Vec3,
    velocity: Vec3,
    health: i32,
    max_health: i32,
    name: String,
    inventory: Vec<Item>,
    // ... 多すぎる
}
```

**理由**:
- システムが必要なデータのみにアクセス
- 並列実行の可能性が高まる
- コードの再利用性が向上

### 2.2 マーカーコンポーネント

```rust
// タグとして使用
#[derive(Component)]
struct Player;

#[derive(Component)]
struct Enemy;

#[derive(Component)]
struct Machine;

// クエリでフィルタリング
fn player_movement(
    query: Query<&mut Position, With<Player>>,
) {
    // Playerタグを持つエンティティのみ
}
```

### 2.3 バンドル

```rust
// 関連するコンポーネントをまとめる
#[derive(Bundle)]
struct MachineBundle {
    machine: Machine,
    position: Position,
    inventory: Inventory,
    power_consumer: PowerConsumer,
    mesh: Mesh3d,
    material: MeshMaterial3d<StandardMaterial>,
}

// スポーン時に使用
fn spawn_machine(mut commands: Commands) {
    commands.spawn(MachineBundle {
        machine: Machine::new("smelter"),
        position: Position(Vec3::ZERO),
        // ...
    });
}
```

---

## 3. システム設計

### 3.1 単一責任

```rust
// 良い例: 一つのことだけを行う
fn update_velocity(mut query: Query<(&Acceleration, &mut Velocity)>) {
    for (acc, mut vel) in &mut query {
        vel.0 += acc.0;
    }
}

fn apply_velocity(
    time: Res<Time>,
    mut query: Query<(&Velocity, &mut Position)>,
) {
    for (vel, mut pos) in &mut query {
        pos.0 += vel.0 * time.delta_secs();
    }
}

// 悪い例: 複数のことを行う
fn update_movement(
    mut query: Query<(&Acceleration, &mut Velocity, &mut Position)>,
) {
    // 速度更新と位置更新を混在
}
```

### 3.2 システム順序

```rust
fn main() {
    App::new()
        .add_systems(Update, (
            input_system,
            // 順序が重要な場合は.chain()
            update_velocity.after(input_system),
            apply_velocity.after(update_velocity),
        ))
        // または明示的にチェーン
        .add_systems(Update, (
            physics_step,
            collision_detection,
            resolve_collisions,
        ).chain())
        .run();
}
```

### 3.3 条件付き実行

```rust
// ステートに応じて実行
fn main() {
    App::new()
        .init_state::<GameState>()
        .add_systems(Update, (
            player_input.run_if(in_state(GameState::Playing)),
            pause_menu.run_if(in_state(GameState::Paused)),
        ))
        .run();
}

#[derive(States, Default, Clone, Eq, PartialEq, Debug, Hash)]
enum GameState {
    #[default]
    MainMenu,
    Playing,
    Paused,
}
```

---

## 4. リソース活用

### 4.1 グローバル状態

```rust
// ゲーム設定
#[derive(Resource)]
struct GameSettings {
    master_volume: f32,
    graphics_quality: GraphicsQuality,
}

// 入力状態
#[derive(Resource)]
struct InputState {
    mouse_position: Vec2,
    selected_tool: Tool,
}

// 使用
fn apply_settings(
    settings: Res<GameSettings>,
    mut audio: ResMut<Audio>,
) {
    audio.set_volume(settings.master_volume);
}
```

### 4.2 イベント

```rust
// カスタムイベント定義
#[derive(Event)]
struct MachineProducedEvent {
    machine_id: Entity,
    item: ItemId,
    amount: u32,
}

// イベント送信
fn production_system(
    mut events: EventWriter<MachineProducedEvent>,
    machines: Query<(Entity, &Machine)>,
) {
    for (entity, machine) in &machines {
        if machine.finished_production() {
            events.send(MachineProducedEvent {
                machine_id: entity,
                item: machine.output_item,
                amount: machine.output_amount,
            });
        }
    }
}

// イベント受信
fn log_production(
    mut events: EventReader<MachineProducedEvent>,
) {
    for event in events.read() {
        info!("Machine {:?} produced {} x{}",
            event.machine_id, event.item, event.amount);
    }
}
```

---

## 5. クエリパターン

### 5.1 フィルタリング

```rust
// With: コンポーネントを持つ
fn query_with(
    query: Query<&Position, With<Player>>,
) {}

// Without: コンポーネントを持たない
fn query_without(
    query: Query<&Position, Without<Static>>,
) {}

// 複合フィルタ
fn query_complex(
    query: Query<
        (&Position, &Health),
        (With<Enemy>, Without<Dead>)
    >,
) {}
```

### 5.2 変更検出

```rust
// Changed: 変更されたコンポーネント
fn on_health_changed(
    query: Query<(&Health, &Name), Changed<Health>>,
) {
    for (health, name) in &query {
        info!("{}'s health changed to {}", name, health.0);
    }
}

// Added: 追加されたコンポーネント
fn on_enemy_spawned(
    query: Query<Entity, Added<Enemy>>,
) {
    for entity in &query {
        info!("New enemy spawned: {:?}", entity);
    }
}
```

### 5.3 オプショナルコンポーネント

```rust
fn flexible_query(
    query: Query<(&Position, Option<&Velocity>)>,
) {
    for (pos, maybe_vel) in &query {
        if let Some(vel) = maybe_vel {
            // 移動処理
        }
    }
}
```

---

## 6. パフォーマンス最適化

### 6.1 並列実行

```rust
// 自動並列化
// Bevyは可能な場合に自動でシステムを並列実行

// 明示的な並列イテレーション
fn parallel_update(
    mut query: Query<&mut Transform>,
) {
    query.par_iter_mut().for_each(|mut transform| {
        // 並列処理
    });
}
```

### 6.2 アーキタイプの最適化

```rust
// コンポーネントの追加/削除はアーキタイプ移動を引き起こす
// → 頻繁に行うとパフォーマンス低下

// 悪い例: 毎フレームコンポーネント追加/削除
fn bad_pattern(mut commands: Commands, query: Query<Entity>) {
    for entity in &query {
        commands.entity(entity).remove::<Marker>();  // 毎フレーム
        commands.entity(entity).insert(Marker);       // アーキタイプ移動
    }
}

// 良い例: フラグコンポーネントの値を変更
#[derive(Component)]
struct Enabled(bool);

fn good_pattern(mut query: Query<&mut Enabled>) {
    for mut enabled in &mut query {
        enabled.0 = !enabled.0;  // 値の変更のみ
    }
}
```

### 6.3 クエリの最適化

```rust
// 必要なデータのみ取得
fn optimized(
    // 良い: 必要なコンポーネントのみ
    query: Query<&Position, With<Moving>>,
) {}

fn not_optimized(
    // 悪い: 不要なコンポーネントも取得
    query: Query<(&Position, &Velocity, &Health, &Name)>,
) {}
```

---

## 7. 工場ゲームでのECS活用

### 7.1 機械システム

```rust
#[derive(Component)]
struct Machine {
    machine_type: MachineType,
    recipe: Option<RecipeId>,
}

#[derive(Component)]
struct ProductionProgress(f32);  // 0.0-1.0

#[derive(Component)]
struct PowerConsumer {
    required: f32,
    current: f32,
}

fn production_system(
    time: Res<Time>,
    mut machines: Query<(
        &Machine,
        &mut ProductionProgress,
        &PowerConsumer,
        &mut Inventory,
    )>,
) {
    for (machine, mut progress, power, mut inventory) in &mut machines {
        if power.current >= power.required {
            if let Some(recipe) = machine.recipe {
                progress.0 += time.delta_secs() / recipe.crafting_time;
                if progress.0 >= 1.0 {
                    complete_production(&mut inventory, recipe);
                    progress.0 = 0.0;
                }
            }
        }
    }
}
```

### 7.2 アイテム移動

```rust
#[derive(Component)]
struct ConveyorBelt {
    speed: f32,
    direction: Vec3,
}

#[derive(Component)]
struct ItemOnBelt {
    progress: f32,  // ベルト上の位置 0.0-1.0
}

fn belt_system(
    time: Res<Time>,
    belts: Query<(&ConveyorBelt, &Children)>,
    mut items: Query<&mut ItemOnBelt>,
) {
    for (belt, children) in &belts {
        for &child in children.iter() {
            if let Ok(mut item) = items.get_mut(child) {
                item.progress += belt.speed * time.delta_secs();
            }
        }
    }
}
```

### 7.3 電力ネットワーク

```rust
#[derive(Resource)]
struct PowerNetwork {
    total_supply: f32,
    total_demand: f32,
    satisfaction: f32,  // supply / demand
}

fn update_power_network(
    mut network: ResMut<PowerNetwork>,
    generators: Query<&PowerGenerator>,
    consumers: Query<&mut PowerConsumer>,
) {
    network.total_supply = generators.iter().map(|g| g.output).sum();
    network.total_demand = consumers.iter().map(|c| c.required).sum();
    network.satisfaction = (network.total_supply / network.total_demand).min(1.0);
}
```

---

## 8. チェックリスト

### コンポーネント設計
- [ ] 小さく分割されているか
- [ ] マーカーコンポーネントを活用しているか
- [ ] バンドルで関連をまとめているか

### システム設計
- [ ] 単一責任になっているか
- [ ] 順序が必要な場合は明示しているか
- [ ] 条件付き実行を活用しているか

### パフォーマンス
- [ ] 不要なコンポーネント追加/削除を避けているか
- [ ] クエリは必要最小限か
- [ ] 並列化の機会を活かしているか

---

## 参考文献

- [Bevy ECS - Official Guide](https://bevy.org/learn/quick-start/getting-started/ecs/)
- [Intro to ECS - Bevy Cheat Book](https://bevy-cheatbook.github.io/programming/ecs-intro.html)
- [bevy_ecs - Rust Docs](https://docs.rs/bevy_ecs/latest/bevy_ecs/)
- [Bevy Examples - GitHub](https://github.com/bevyengine/bevy/tree/main/examples/ecs)

---

*このレポートはBevy 0.14/0.15の公式ドキュメントおよびコミュニティベストプラクティスに基づいています。*
