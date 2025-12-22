# ゲームセキュリティ・チート対策ベストプラクティス

**作成日**: 2025-12-22
**目的**: 公平なゲーム体験を守るためのセキュリティ実装指針

---

## 1. 基本原則

### 1.1 多層防御（Defense in Depth）

> 単一の対策に依存せず、複数の防御層を設ける

```
層1: サーバー権威（最重要）
  ↓
層2: 入力バリデーション
  ↓
層3: 異常検知・統計分析
  ↓
層4: クライアント保護（補助）
  ↓
層5: 行動分析・機械学習
```

### 1.2 サーバー権威（Server Authority）

> クライアントサイドの対策だけでは不十分

```rust
// 原則: クライアントは信頼できない
fn process_client_action(client_id: ClientId, action: Action) {
    // 1. 入力のバリデーション
    if !validate_action(&action) {
        log_suspicious(client_id, "invalid_action");
        return;
    }

    // 2. 権限チェック
    if !can_perform_action(client_id, &action) {
        log_suspicious(client_id, "unauthorized_action");
        return;
    }

    // 3. サーバー側で結果を計算
    let result = calculate_action_result(&action);

    // 4. 結果をクライアントに通知
    send_result(client_id, result);
}
```

---

## 2. サーバーサイド対策

### 2.1 入力バリデーション

```rust
fn validate_player_input(input: &PlayerInput) -> Result<(), ValidationError> {
    // 移動速度チェック
    if input.velocity.length() > MAX_PLAYER_SPEED {
        return Err(ValidationError::SpeedHack);
    }

    // 位置の連続性チェック
    if input.position.distance(last_position) > MAX_MOVEMENT_PER_TICK {
        return Err(ValidationError::Teleport);
    }

    // アクションのクールダウンチェック
    if input.action.is_some() && !cooldown_expired(input.action) {
        return Err(ValidationError::CooldownBypass);
    }

    Ok(())
}
```

### 2.2 統計的異常検知

```rust
struct PlayerStats {
    kills: u32,
    deaths: u32,
    headshot_rate: f32,
    average_reaction_time: Duration,
    // ...
}

fn detect_statistical_anomaly(stats: &PlayerStats) -> bool {
    // 異常に高いヘッドショット率
    if stats.headshot_rate > 0.9 && stats.kills > 50 {
        return true;
    }

    // 人間離れした反応速度
    if stats.average_reaction_time < Duration::from_millis(50) {
        return true;
    }

    // 統計的にありえないK/D比
    if stats.kills as f32 / stats.deaths.max(1) as f32 > 20.0 {
        return true;
    }

    false
}
```

### 2.3 レートリミティング

```rust
struct RateLimiter {
    actions: HashMap<ActionType, VecDeque<Instant>>,
    limits: HashMap<ActionType, (u32, Duration)>,  // (回数, 期間)
}

impl RateLimiter {
    fn check(&mut self, action: ActionType) -> bool {
        let (limit, window) = self.limits[&action];
        let history = self.actions.entry(action).or_default();

        // 期限切れのエントリを削除
        let cutoff = Instant::now() - window;
        while history.front().map(|t| *t < cutoff).unwrap_or(false) {
            history.pop_front();
        }

        if history.len() >= limit as usize {
            return false;  // レート制限超過
        }

        history.push_back(Instant::now());
        true
    }
}
```

---

## 3. クライアントサイド保護

### 3.1 コード難読化

```rust
// IL2CPP（Unity）またはネイティブコンパイル（Rust）を使用
// → 逆コンパイルを困難に

// 追加の難読化
// - 変数名のランダム化
// - 制御フローの複雑化
// - 文字列の暗号化
```

### 3.2 メモリ保護

```rust
// 重要な値を暗号化して保持
struct ProtectedValue<T: Copy> {
    encrypted: u64,
    key: u64,
}

impl<T: Copy> ProtectedValue<T> {
    fn get(&self) -> T {
        let decrypted = self.encrypted ^ self.key;
        unsafe { std::mem::transmute_copy(&decrypted) }
    }

    fn set(&mut self, value: T) {
        let raw: u64 = unsafe { std::mem::transmute_copy(&value) };
        self.key = rand::random();
        self.encrypted = raw ^ self.key;
    }
}
```

### 3.3 整合性チェック

```rust
fn verify_game_files() -> bool {
    let expected_hashes = load_expected_hashes();

    for (path, expected_hash) in expected_hashes {
        let actual_hash = calculate_hash(&path);
        if actual_hash != expected_hash {
            return false;  // ファイル改ざん検出
        }
    }

    true
}
```

---

## 4. セーブデータ保護

### 4.1 暗号化

```rust
use aes_gcm::{Aes256Gcm, Key, Nonce};

fn encrypt_save(save_data: &[u8], key: &[u8; 32]) -> Vec<u8> {
    let cipher = Aes256Gcm::new(Key::from_slice(key));
    let nonce = Nonce::from_slice(&generate_nonce());

    cipher.encrypt(nonce, save_data)
        .expect("encryption failure")
}

fn decrypt_save(encrypted: &[u8], key: &[u8; 32]) -> Result<Vec<u8>, Error> {
    let cipher = Aes256Gcm::new(Key::from_slice(key));
    // nonceはファイルに含める
    cipher.decrypt(nonce, encrypted)
}
```

### 4.2 署名検証

```rust
use hmac::{Hmac, Mac};
use sha2::Sha256;

fn sign_save(save_data: &[u8], secret: &[u8]) -> Vec<u8> {
    let mut mac = Hmac::<Sha256>::new_from_slice(secret).unwrap();
    mac.update(save_data);
    mac.finalize().into_bytes().to_vec()
}

fn verify_save(save_data: &[u8], signature: &[u8], secret: &[u8]) -> bool {
    let mut mac = Hmac::<Sha256>::new_from_slice(secret).unwrap();
    mac.update(save_data);
    mac.verify_slice(signature).is_ok()
}
```

---

## 5. インディー向け戦略

### 5.1 優先順位

| 優先度 | 対策 | 効果 | コスト |
|--------|------|------|--------|
| 必須 | サーバー権威 | 高 | 中 |
| 必須 | 入力バリデーション | 高 | 低 |
| 推奨 | 統計的異常検知 | 中 | 中 |
| 推奨 | セーブデータ暗号化 | 中 | 低 |
| 任意 | クライアント難読化 | 低 | 中 |

### 5.2 シャドウバン（Invisible Ban）

> インディーならではの強力な対策

```rust
enum BanType {
    Hard,      // 明示的にアクセス拒否
    Shadow,    // 気づかれないようにペナルティ
}

fn apply_shadow_ban(player_id: PlayerId) {
    // チーターをチーター同士でマッチング
    set_matchmaking_pool(player_id, Pool::Cheaters);

    // または: 統計を記録するが報酬を与えない
    // または: アイテムドロップ率を0に
}
```

### 5.3 コスト効率

```
インディーゲームの優先事項:
  1. サーバー権威アーキテクチャ（必須）
  2. 基本的な入力バリデーション
  3. セーブデータ暗号化（シングルプレイ）
  4. 簡易統計分析

避けるべき:
  - 高価なアンチチートソリューション
  - カーネルレベルの保護（過剰）
  - 複雑な機械学習システム
```

---

## 6. 外部ソリューション

### 6.1 Easy Anti-Cheat (EAC)

- **提供**: Epic Games
- **価格**: 基本無料
- **特徴**:
  - エンジン・プラットフォーム非依存
  - Steam/Epic両対応
  - カーネルレベル保護

### 6.2 Anybrain

- **特徴**:
  - サーバーサイド中心
  - 行動分析（100+のバイオメトリクス）
  - 導入・メンテナンス容易

### 6.3 FairFight

- **特徴**:
  - 非侵入型
  - サーバーサイドのみ
  - リアルタイム分析

---

## 7. 工場ゲーム特有の対策

### 7.1 リソース改ざん対策

```rust
// サーバーが全リソースを管理
struct PlayerResources {
    owner: PlayerId,
    items: HashMap<ItemId, u64>,
    last_verified: Instant,
}

fn transfer_item(from: PlayerId, to: PlayerId, item: ItemId, amount: u64) {
    // サーバーで残高確認・転送
    let from_resources = get_resources(from);
    if from_resources.items.get(&item).unwrap_or(&0) < &amount {
        log_cheat_attempt(from, "insufficient_items");
        return;
    }

    // 転送実行
    from_resources.items.entry(item).and_modify(|n| *n -= amount);
    get_resources(to).items.entry(item).and_modify(|n| *n += amount);
}
```

### 7.2 レシピ・進行改ざん対策

```rust
// 解放されたレシピはサーバーが管理
fn can_craft(player: PlayerId, recipe: RecipeId) -> bool {
    let unlocked = get_unlocked_recipes(player);
    unlocked.contains(&recipe)
}

fn unlock_recipe(player: PlayerId, recipe: RecipeId) {
    // 条件を満たしているかサーバーが確認
    if meets_unlock_requirements(player, recipe) {
        add_unlocked_recipe(player, recipe);
    }
}
```

---

## 8. チェックリスト

### 必須
- [ ] サーバー権威アーキテクチャか
- [ ] 全ての入力にバリデーションがあるか
- [ ] レートリミティングがあるか

### 推奨
- [ ] 統計的異常検知があるか
- [ ] セーブデータが暗号化されているか
- [ ] ログが記録されているか

### マルチプレイ
- [ ] リソース操作はサーバー側か
- [ ] 進行状態はサーバー管理か
- [ ] 不正検知時の対応が定義されているか

---

## 参考文献

- [The 4 best anti-cheat solutions for indie games in 2024 - Getgud.io](https://www.getgud.io/blog/the-4-best-anti-cheat-solutions-for-indie-games-in-2023/)
- [Securing Game Code in 2025 - Medium](https://medium.com/@lzysoul/securing-game-code-in-2025-modern-anti-cheat-techniques-and-best-practices-e2e0f6f14173)
- [An indie guide to anti-cheat software - Medium](https://medium.com/teeny-tiny-game-dev-essays/an-indie-guide-to-anti-cheat-software-51bb770a90a0)
- [Best Security Practices in Game Development - Microsoft](https://learn.microsoft.com/en-us/windows/win32/dxtecharts/best-security-practices-in-game-development)

---

*このレポートはゲームセキュリティのベストプラクティス調査に基づいています。*
