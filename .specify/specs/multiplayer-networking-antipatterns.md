# マルチプレイヤーネットワーク実装アンチパターン

**作成日**: 2025-12-22
**目的**: マルチプレイヤー体験を台無しにするネットワーク実装の失敗例と対策

---

## 1. 致命的アンチパターン

### 1.1 クライアント信頼（Client Trust）

> 「クライアントから『ヒット』メッセージを受信して適用したら、チートプロキシで偽装される」
> — Valve Developer Community

**症状**:
- チーターが無敵、即死攻撃
- ゲームバランス崩壊
- 正規プレイヤー離脱

**実例**:
```rust
// 悪い例: クライアントを信頼
fn on_client_message(msg: ClientMessage) {
    match msg {
        ClientMessage::Kill { target_id } => {
            // クライアントが「倒した」と言えば適用
            kill_player(target_id);  // ❌ 偽造可能！
        }
    }
}
```

**対策**:
```rust
// 良い例: サーバーが判定
fn on_client_input(input: PlayerInput) {
    // 入力のみ受け付け、判定はサーバー
    let ray = calculate_ray(&input);
    if let Some(hit) = server_raycast(ray) {
        apply_damage(hit.entity, calculate_damage());
    }
}
```

---

### 1.2 ラグ補正の誤用（Lag Compensation Abuse）

> 「カバーに隠れたのに撃たれた」

**問題**:
```
時刻T: プレイヤーAがカバーに隠れる
時刻T+100ms: プレイヤーBがT-50msの位置を撃つ
サーバー: T-50ms時点ではAは露出していた → ヒット判定

結果: Aから見ると「隠れたのに死んだ」
```

**症状**:
- 高Pingプレイヤーに有利すぎる
- 低Pingプレイヤーの不満
- 「理不尽な死」の報告

**対策**:
```rust
// ラグ補正の上限を設ける
const MAX_LAG_COMPENSATION_MS: u32 = 100;

fn lag_compensated_check(shooter_latency: u32) {
    let compensation = shooter_latency.min(MAX_LAG_COMPENSATION_MS);
    // 100ms以上のラグは補正しない
}
```

---

### 1.3 同期ズレ放置（Desync Neglect）

**症状**:
- クライアント間で状態が異なる
- 「相手の画面では違う位置にいた」
- 累積して大きなズレに

**原因**:
```rust
// 悪い例: 浮動小数点の非決定性
fn physics_step(pos: Vec3, velocity: Vec3) -> Vec3 {
    pos + velocity * dt  // 環境によって微妙に異なる
}

// 悪い例: テーブル参照比較
if table_a == table_b {
    // マルチプレイ同期後、テーブルは新オブジェクト
    // 参照比較は失敗する
}
```

**対策**:
```rust
// 良い例: 固定小数点演算
fn physics_step_fixed(pos: FixedVec3, velocity: FixedVec3) -> FixedVec3 {
    pos + velocity * FIXED_DT
}

// 良い例: 定期的なフルステート同期
fn periodic_sync(tick: u64) {
    if tick % FULL_SYNC_INTERVAL == 0 {
        broadcast_full_state();
    }
}
```

---

### 1.4 入力バッファ暴走（Input Buffer Overflow）

> 「バッファに大量のコマンドが溜まり、プレイヤーが過去に取り残される」

**症状**:
- 入力が遅延して反映される
- 操作の「もっさり感」
- 極端なケースで320ms以上の遅延

**原因**:
```rust
// 悪い例: バッファを無制限に溜める
fn receive_input(input: Input) {
    input_buffer.push(input);  // 際限なく溜まる
}

fn process_inputs() {
    if let Some(input) = input_buffer.pop_front() {
        // 1tickに1入力しか処理しない
        // → バッファが溜まり続ける
    }
}
```

**対策**:
```rust
// 良い例: バッファ上限と追いつき処理
const MAX_BUFFER_SIZE: usize = 5;

fn receive_input(input: Input) {
    if input_buffer.len() < MAX_BUFFER_SIZE {
        input_buffer.push(input);
    }
}

fn process_inputs() {
    // 溜まっていたら複数処理して追いつく
    let to_process = input_buffer.len().min(3);
    for _ in 0..to_process {
        if let Some(input) = input_buffer.pop_front() {
            apply_input(input);
        }
    }
}
```

---

## 2. パフォーマンスアンチパターン

### 2.1 フルステート毎フレーム送信

**症状**:
- 帯域使用量が膨大
- パケットロスでスタッター
- モバイル環境で接続不安定

**原因**:
```rust
// 悪い例: 毎フレーム全データ送信
fn sync_state() {
    let full_state = serialize_entire_world();
    broadcast(full_state);  // 毎フレーム数MBを送信
}
```

**対策**:
```rust
// 良い例: 差分のみ送信
fn sync_state() {
    let delta = calculate_delta(last_sent_state, current_state);
    if !delta.is_empty() {
        broadcast(delta);
        last_sent_state = current_state.clone();
    }
}
```

---

### 2.2 全エンティティ同期（No Interest Management）

> 「1000台の機械を全クライアントに60Hzで同期 → 破綻」

**症状**:
- スケールしない
- プレイヤー増加で指数的に重くなる

**対策**:
```rust
// プレイヤーの周囲のみ同期
fn get_relevant_entities(player: &Player) -> Vec<Entity> {
    world.entities()
        .filter(|e| {
            let dist = e.position.distance(player.position);
            dist < VISIBILITY_RADIUS
        })
        .collect()
}
```

---

### 2.3 文字列シリアライズ

**症状**:
- パケットサイズ肥大
- パース時間増加
- 帯域浪費

**原因**:
```rust
// 悪い例: JSONでネットワーク通信
let message = serde_json::to_string(&game_state)?;
send(message.as_bytes());
```

**対策**:
```rust
// 良い例: バイナリシリアライズ
let message = bincode::serialize(&game_state)?;
send(&message);  // サイズ1/5〜1/10
```

---

## 3. 接続管理アンチパターン

### 3.1 再接続未考慮

**症状**:
- 一瞬の切断で進行状況消失
- 再接続するとデスポーン
- 「回線落ち = 敗北」

**対策**:
```rust
struct PlayerSession {
    id: Uuid,
    state: PlayerState,
    last_seen: Instant,
    connection: Option<Connection>,
}

fn on_disconnect(session_id: Uuid) {
    // 即座に削除せず猶予を与える
    sessions.get_mut(session_id).connection = None;
}

fn on_reconnect(session_id: Uuid, conn: Connection) {
    if let Some(session) = sessions.get_mut(session_id) {
        session.connection = Some(conn);
        send_full_state(conn, &session.state);
    }
}
```

---

### 3.2 タイムアウト固定

**症状**:
- 高レイテンシ環境で頻繁に切断
- 低レイテンシ環境で応答が遅い

**対策**:
```rust
// 適応的タイムアウト
fn calculate_timeout(rtt_history: &[Duration]) -> Duration {
    let avg_rtt = rtt_history.iter().sum() / rtt_history.len();
    let std_dev = calculate_std_dev(rtt_history);
    avg_rtt + std_dev * 4  // 平均 + 4σ
}
```

---

## 4. 工場ゲーム特有のアンチパターン

### 4.1 全機械リアルタイム同期

> 「全機械の生産状態を60Hzで同期 → 帯域爆発」

**症状**:
- 工場規模に比例して帯域増加
- 数百台で破綻

**対策**:
```rust
// イベントベース同期
enum MachineEvent {
    Started { id: EntityId },
    Stopped { id: EntityId },
    Produced { id: EntityId, item: ItemId },
}

// 状態変化時のみ送信
fn on_machine_change(event: MachineEvent) {
    broadcast_event(event);
}
```

---

### 4.2 ボクセル変更即時同期

**症状**:
- 大量ブロック配置で帯域スパイク
- ネットワーク輻輳

**対策**:
```rust
// バッチ処理
struct PendingChanges {
    changes: Vec<(IVec3, BlockType)>,
    last_sent: Instant,
}

fn flush_changes(pending: &mut PendingChanges) {
    if pending.changes.len() > 100
        || pending.last_sent.elapsed() > Duration::from_millis(50)
    {
        broadcast_chunk_update(&pending.changes);
        pending.changes.clear();
        pending.last_sent = Instant::now();
    }
}
```

---

### 4.3 非決定的シミュレーション

> 「各クライアントの浮動小数点演算結果が微妙に違う」

**症状**:
- 長時間プレイで状態がズレる
- 「私の画面では動いていた」

**対策**:
```rust
// 決定的シミュレーション
use fixed::types::I32F32;

fn simulate_fixed(state: &mut FactoryState, tick: u64) {
    // 固定小数点で計算
    for machine in &mut state.machines {
        machine.progress += I32F32::from_num(PRODUCTION_RATE);
    }
}
```

---

## 5. アンチパターン速見表

### 絶対に避ける

| パターン | 症状 | 対策 |
|----------|------|------|
| クライアント信頼 | チート蔓延 | サーバー権威 |
| 同期ズレ放置 | 状態不一致 | 定期フル同期 |
| 入力バッファ暴走 | 操作遅延 | バッファ上限 |
| 再接続未考慮 | データ消失 | セッション維持 |

### 強く避ける

| パターン | 症状 | 対策 |
|----------|------|------|
| フルステート毎フレーム | 帯域爆発 | 差分圧縮 |
| 全エンティティ同期 | スケールしない | Interest Management |
| 文字列シリアライズ | 帯域浪費 | バイナリ |
| 固定タイムアウト | 不安定 | 適応的タイムアウト |

### 注意

| パターン | 症状 | 対策 |
|----------|------|------|
| ラグ補正誤用 | 理不尽な死 | 上限設定 |
| 機械リアルタイム同期 | 帯域問題 | イベントベース |
| 非決定的シミュレーション | ズレ蓄積 | 固定小数点 |

---

## 6. チェックリスト

### セキュリティ
- [ ] クライアントを信頼していないか
- [ ] 全ての判定はサーバー側か
- [ ] 入力のバリデーションがあるか

### パフォーマンス
- [ ] 差分圧縮を使用しているか
- [ ] Interest Managementがあるか
- [ ] バイナリシリアライズか

### 安定性
- [ ] 再接続処理があるか
- [ ] 同期ズレの検出・修復があるか
- [ ] 入力バッファに上限があるか

---

## 参考文献

- [Lag Compensation - Valve Developer Community](https://developer.valvesoftware.com/wiki/Latency_Compensating_Methods_in_Client/Server_In-game_Protocol_Design_and_Optimization)
- [Source Multiplayer Networking - Valve](https://developer.valvesoftware.com/wiki/Source_Multiplayer_Networking)
- [Fast-Paced Multiplayer: Lag Compensation - Gabriel Gambetta](https://www.gabrielgambetta.com/lag-compensation.html)
- [Multiplayer Networking Resources - GitHub](https://github.com/0xFA11/MultiplayerNetworkingResources)

---

*このレポートはマルチプレイヤーゲームの問題事例調査に基づいています。*
