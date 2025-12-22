# Bevy ECS Design Skill

Bevy ECSアーキテクチャの設計・実装を支援するスキルです。

## 参照ドキュメント

- `.specify/specs/bevy-ecs-best-practices.md`
- `.specify/specs/rust-gamedev-best-practices.md`

---

## コンポーネント設計

### 小さく保つ

```rust
// Good: 単一責任
#[derive(Component)]
struct Position(Vec3);

#[derive(Component)]
struct Velocity(Vec3);

#[derive(Component)]
struct Health(f32);

// Bad: 巨大コンポーネント
#[derive(Component)]
struct Entity {
    position: Vec3,
    velocity: Vec3,
    health: f32,
    inventory: Vec<Item>,
    // ... 多すぎる
}
```

### マーカーコンポーネント

```rust
#[derive(Component)]
struct Player;

#[derive(Component)]
struct Enemy;

#[derive(Component)]
struct NeedsUpdate;
```

### Bundle活用

```rust
#[derive(Bundle)]
struct MachineBundle {
    machine: MachineInstance,
    power: PowerConsumer,
    inventory: MachineInventory,
    transform: Transform,
    visibility: Visibility,
}
```

---

## システム設計

### B1. 単一責任

```rust
// Good: 一つのことだけ
fn move_entities(mut query: Query<(&mut Transform, &Velocity)>) {
    for (mut transform, velocity) in &mut query {
        transform.translation += velocity.0;
    }
}

// Bad: 複数の責任
fn update_everything(/* ... */) {
    // 移動処理
    // 衝突判定
    // ダメージ計算
    // UI更新
}
```

### B2. システム順序

```rust
app.add_systems(Update, (
    read_input,
    apply_movement,
    check_collisions,
    apply_damage,
    update_ui,
).chain());

// または明示的な順序
app.add_systems(Update, apply_movement.after(read_input));
app.add_systems(Update, check_collisions.after(apply_movement));
```

### B3. 変更検出

```rust
fn update_on_change(
    query: Query<&Health, Changed<Health>>,
) {
    for health in &query {
        // Healthが変更されたエンティティのみ処理
    }
}

fn handle_new_entities(
    query: Query<&Machine, Added<Machine>>,
) {
    for machine in &query {
        // 新規追加されたMachineのみ処理
    }
}
```

---

## クエリパターン

### フィルタリング

```rust
// With/Without
fn update_active_machines(
    query: Query<&mut Machine, (With<Active>, Without<Broken>)>,
) { }

// Option
fn update_with_optional(
    query: Query<(&Transform, Option<&Velocity>)>,
) {
    for (transform, maybe_velocity) in &query {
        if let Some(velocity) = maybe_velocity {
            // ...
        }
    }
}
```

### 複数クエリ

```rust
fn interact(
    players: Query<&Transform, With<Player>>,
    machines: Query<(&Transform, &Machine), Without<Player>>,
) {
    for player_transform in &players {
        for (machine_transform, machine) in &machines {
            // ...
        }
    }
}
```

---

## リソース vs コンポーネント

| 使用場面 | 選択 |
|----------|------|
| グローバルに1つ | Resource |
| エンティティに紐づく | Component |
| 設定値 | Resource |
| 状態 | 両方可能 |

```rust
// Resource
#[derive(Resource)]
struct GameSettings {
    volume: f32,
    difficulty: Difficulty,
}

// Component
#[derive(Component)]
struct PlayerSettings {
    sensitivity: f32,
}
```

---

## イベント活用

```rust
#[derive(Event)]
struct DamageEvent {
    target: Entity,
    amount: f32,
}

fn deal_damage(mut events: EventWriter<DamageEvent>) {
    events.send(DamageEvent { target, amount: 10.0 });
}

fn receive_damage(
    mut events: EventReader<DamageEvent>,
    mut query: Query<&mut Health>,
) {
    for event in events.read() {
        if let Ok(mut health) = query.get_mut(event.target) {
            health.0 -= event.amount;
        }
    }
}
```

---

## パフォーマンス最適化

### 並列実行

```rust
// 自動並列（データ依存がなければ）
app.add_systems(Update, (
    system_a,  // Query<&A>
    system_b,  // Query<&B>
    system_c,  // Query<&C>
));  // 並列実行される

// 順序依存
app.add_systems(Update, (
    system_a,
    system_b.after(system_a),
));
```

### run_if条件

```rust
fn expensive_system() { }

app.add_systems(Update,
    expensive_system.run_if(should_run)
);

fn should_run(state: Res<GameState>) -> bool {
    state.is_active()
}
```

---

## 工場ゲーム向けパターン

### マシングリッド

```rust
#[derive(Resource)]
struct MachineGrid {
    grid: HashMap<IVec3, Entity>,
}

fn place_machine(
    mut grid: ResMut<MachineGrid>,
    mut commands: Commands,
) {
    let entity = commands.spawn(MachineBundle::default()).id();
    grid.grid.insert(position, entity);
}
```

### コンベア最適化

```rust
// バッチ処理
fn tick_conveyors(
    mut conveyors: Query<&mut Conveyor>,
    time: Res<Time>,
) {
    // 固定タイムステップで処理
    if time.elapsed_secs() % TICK_INTERVAL < time.delta_secs() {
        for mut conveyor in &mut conveyors {
            conveyor.tick();
        }
    }
}
```

---

## チェックリスト

- [ ] コンポーネントは小さく単一責任か
- [ ] システム順序は明示されているか
- [ ] Changed/Addedを活用しているか
- [ ] 並列実行の機会を活かしているか
- [ ] 不要なクエリ走査を避けているか
- [ ] イベントでシステム間通信しているか

---

*このスキルはBevy ECS設計の品質を確保するためのガイドです。*
