# マルチプレイヤーネットワーク実装ベストプラクティス

**作成日**: 2025-12-22
**目的**: 安定したマルチプレイヤー体験を提供するためのネットワーク実装指針

---

## 1. 基本アーキテクチャ

### 1.1 権威サーバーモデル（Authoritative Server）

> ゲームバランス、競争公平性、チート対策に関わるすべてのデータはサーバーが権威を持つ

**サーバーが管理すべきデータ**:
```
必須（サーバー権威）:
  ✓ プレイヤー位置
  ✓ HP/体力
  ✓ 弾薬数
  ✓ クールダウン
  ✓ ダメージ計算
  ✓ ゲーム進行状態

クライアント権威可（セキュリティリスク低）:
  ✓ コスメティック効果
  ✓ ローカルアニメーション
  ✓ UIフィードバック
```

**実装原則**:
```rust
// 良い例: サーバーが全ゲームロジックを実行
fn server_process_input(input: PlayerInput) -> GameState {
    validate_input(&input)?;
    let new_state = apply_physics(input);
    broadcast_state(new_state);
}

// 悪い例: クライアントを信頼
fn client_authority() {
    // クライアントから「プレイヤーAを倒した」を受信
    // → 偽造可能！
}
```

---

### 1.2 クライアント予測（Client-Side Prediction）

> 入力からサーバー応答までの遅延をなくし、即座に反応するように見せる

**問題**:
- 入力 → サーバー → 応答 で往復100-200ms
- この遅延で操作が重く感じる

**解決策**:
```rust
struct ClientPrediction {
    pending_inputs: Vec<(u64, Input)>,  // (シーケンス番号, 入力)
    predicted_state: GameState,
}

fn predict(input: Input) {
    // 1. 入力を即座にローカルで適用
    predicted_state = apply_input(predicted_state, input);

    // 2. 入力をサーバーに送信（シーケンス番号付き）
    send_to_server(seq_num, input);
    pending_inputs.push((seq_num, input));
}
```

**効果**: 操作のレスポンスが即座になる

---

### 1.3 サーバー再調整（Server Reconciliation）

> サーバーからの権威状態とクライアント予測のズレを修正

**仕組み**:
```rust
fn reconcile(server_state: GameState, last_processed_seq: u64) {
    // 1. サーバーの権威状態を受け入れ
    current_state = server_state;

    // 2. まだ処理されていない入力を再適用
    for (seq, input) in pending_inputs.iter() {
        if *seq > last_processed_seq {
            current_state = apply_input(current_state, *input);
        }
    }

    // 3. 処理済み入力を削除
    pending_inputs.retain(|(seq, _)| *seq > last_processed_seq);
}
```

**効果**: 予測ミスによる「巻き戻り」を最小化

---

## 2. 遅延対策

### 2.1 補間（Interpolation）

> 他プレイヤーの動きを滑らかに表示

```rust
// 過去2つの状態間を補間
fn interpolate(t: f32, state_old: &State, state_new: &State) -> State {
    State {
        position: state_old.position.lerp(state_new.position, t),
        rotation: state_old.rotation.slerp(state_new.rotation, t),
    }
}

// 表示は100ms過去の状態
const INTERPOLATION_DELAY: f32 = 0.1;
```

**トレードオフ**:
| 遅延 | スムーズさ | 正確さ |
|------|-----------|--------|
| 50ms | 低 | 高 |
| 100ms | 中 | 中 |
| 200ms | 高 | 低 |

### 2.2 外挿（Extrapolation）

> 次のパケットが届くまで現在の動きを予測

```rust
fn extrapolate(last_state: &State, velocity: Vec3, dt: f32) -> State {
    State {
        position: last_state.position + velocity * dt,
        ..last_state.clone()
    }
}
```

**注意**: 急な方向転換で「瞬間移動」に見える

### 2.3 ラグ補正（Lag Compensation）

> シューターで「撃った時に見えていた位置」でヒット判定

```rust
fn lag_compensated_hit_check(
    shooter_time: f32,
    target_id: EntityId,
    ray: Ray,
) -> bool {
    // 1. 射撃者の視点での時刻に巻き戻し
    let historical_state = rewind_state(shooter_time);

    // 2. その時点のターゲット位置でヒット判定
    let target_hitbox = historical_state.get_hitbox(target_id);
    ray.intersects(target_hitbox)
}
```

**問題点**: 「カバーに隠れたのに撃たれた」現象

---

## 3. ロールバックネットコード

### 3.1 概要

> 格闘ゲーム等で使用される予測＋巻き戻し方式

```
フレームN: 入力を予測して実行
フレームN+3: 実際の入力到着
  → 予測が間違っていたら
  → 状態をフレームNに巻き戻し
  → 正しい入力で再シミュレーション
```

### 3.2 Bevy向け実装

```rust
// bevy_ggrsを使用
use bevy_ggrs::*;

fn rollback_system(
    mut commands: Commands,
    inputs: Res<PlayerInputs<GGRSConfig>>,
) {
    // GGRSが自動でロールバック管理
    for (handle, input) in inputs.iter() {
        // 入力に基づいて状態更新
    }
}
```

### 3.3 適用シーン

| ゲームタイプ | ロールバック | サーバー権威 |
|-------------|-------------|-------------|
| 格闘ゲーム | ◎ | △ |
| FPS | △ | ◎ |
| RTS | △ | ◎ |
| 工場建設 | △ | ◎ |

---

## 4. Bevy向けネットワークライブラリ

### 4.1 Lightyear

> 予測・ロールバック・WebTransport対応の高機能ライブラリ

**特徴**:
- サーバー権威アーキテクチャ
- クライアント予測＋ロールバック内蔵
- WebTransport/WebSocket対応（WASM可）
- 帯域制限・Interest Management

```rust
// Lightyearの基本設定
use lightyear::prelude::*;

#[derive(Component, Serialize, Deserialize)]
struct Position(Vec2);

#[derive(Message, Serialize, Deserialize)]
struct PlayerInput {
    direction: Vec2,
}

// Replicateバンドルで自動同期
commands.spawn((
    Position(Vec2::ZERO),
    Replicate::default(),
));
```

### 4.2 Renet / bevy_renet

> シンプルで高速なUDPベースライブラリ

**特徴**:
- 認証・接続管理
- チャンネルベースのメッセージング
- 低レベルで柔軟

```rust
use bevy_renet::*;
use renet::*;

fn send_message(mut client: ResMut<RenetClient>) {
    let message = bincode::serialize(&GameEvent::Move(Vec2::X)).unwrap();
    client.send_message(CHANNEL_RELIABLE, message);
}
```

### 4.3 選択ガイド

| 要件 | Lightyear | Renet |
|------|-----------|-------|
| 予測/ロールバック | 内蔵 | 自作 |
| WASM対応 | ◎ | △ |
| 学習コスト | 中 | 低 |
| 柔軟性 | 中 | 高 |

---

## 5. 帯域最適化

### 5.1 差分圧縮（Delta Compression）

```rust
// 変更があった値のみ送信
struct DeltaState {
    changed_mask: u32,
    position: Option<Vec3>,    // 変更時のみSome
    rotation: Option<Quat>,
    health: Option<i32>,
}
```

### 5.2 Interest Management

```rust
// プレイヤーに近いエンティティのみ同期
fn filter_entities(player_pos: Vec3, entities: &[Entity]) -> Vec<Entity> {
    entities.iter()
        .filter(|e| e.position.distance(player_pos) < SYNC_RADIUS)
        .collect()
}
```

### 5.3 送信頻度調整

| カテゴリ | 送信頻度 | 重要度 |
|----------|---------|--------|
| プレイヤー位置 | 20-60Hz | 高 |
| AI位置 | 10-20Hz | 中 |
| 静的オブジェクト | イベント時 | 低 |
| チャット | イベント時 | 低 |

---

## 6. 工場ゲーム特有の考慮

### 6.1 大量エンティティの同期

```rust
// 問題: 1000台の機械を60Hzで同期 → 帯域爆発

// 解決策1: バッチ更新
struct FactoryBatchUpdate {
    tick: u64,
    machine_states: HashMap<EntityId, MachineState>,
}

// 解決策2: 差分のみ送信
struct MachineEvent {
    id: EntityId,
    event_type: EventType,  // Started, Stopped, Produced
}
```

### 6.2 ボクセル変更の同期

```rust
// チャンク単位で変更をまとめる
struct ChunkUpdate {
    chunk_pos: IVec3,
    changes: Vec<(IVec3, BlockType)>,  // ローカル座標, ブロック
}

// 変更履歴を保持してクライアント再接続に対応
struct WorldHistory {
    changes: VecDeque<(u64, ChunkUpdate)>,
}
```

### 6.3 決定的シミュレーション

```rust
// 入力のみ同期、各クライアントが同じ計算
// → 帯域大幅削減

struct FactorySimulation {
    tick: u64,
    rng_seed: u64,  // 乱数シード固定
}

fn simulate(inputs: Vec<PlayerInput>) -> FactoryState {
    // 全クライアントで同じ結果になる
}
```

---

## 7. チェックリスト

### アーキテクチャ
- [ ] サーバー権威モデルか
- [ ] クライアント予測があるか
- [ ] サーバー再調整があるか
- [ ] ラグ補正が必要か（FPS等）

### パフォーマンス
- [ ] 差分圧縮を使用しているか
- [ ] Interest Managementがあるか
- [ ] 適切な送信頻度か

### 安定性
- [ ] 再接続処理があるか
- [ ] 状態同期エラーの回復方法があるか
- [ ] タイムアウト処理があるか

---

## 参考文献

- [Client-Side Prediction and Server Reconciliation - Gabriel Gambetta](https://www.gabrielgambetta.com/client-side-prediction-server-reconciliation.html)
- [Source Multiplayer Networking - Valve](https://developer.valvesoftware.com/wiki/Source_Multiplayer_Networking)
- [Lightyear - GitHub](https://github.com/cBournhonesque/lightyear)
- [Renet - GitHub](https://github.com/lucaspoffo/renet)
- [Game Networking Demystified - Ruoyu Sun](https://ruoyusun.com/2019/09/21/game-networking-5.html)

---

*このレポートはネットワークプログラミングのベストプラクティス調査に基づいています。*
