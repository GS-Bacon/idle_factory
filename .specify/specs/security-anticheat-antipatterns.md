# ゲームセキュリティ・チート対策アンチパターン

**作成日**: 2025-12-22
**目的**: セキュリティ対策の失敗例と回避策

---

## 1. 致命的アンチパターン

### 1.1 クライアント信頼（Client Trust）

> 「クライアントサイドの対策だけでは、熟練チーターには無力」

**症状**:
- チートツールで値を書き換え放題
- 無敵、無限リソース、瞬間移動
- ゲームバランス崩壊

**実例**:
```rust
// 悪い例: クライアントが「持っている」と言えば信じる
fn on_craft_request(player: PlayerId, recipe: RecipeId) {
    // クライアントから「材料あります」→ 信じる
    complete_craft(player, recipe);  // ❌
}

// 良い例: サーバーが確認
fn on_craft_request(player: PlayerId, recipe: RecipeId) {
    let inventory = get_server_inventory(player);
    if has_materials(inventory, recipe) {
        consume_materials(inventory, recipe);
        add_item(inventory, recipe.output);
    }
}
```

---

### 1.2 セキュリティ through Obscurity

> 「隠せば安全」は幻想

**症状**:
- 逆コンパイルで全て露出
- ハードコードした秘密鍵が漏洩
- 暗号化キーがクライアントに含まれる

**実例**:
```rust
// 悪い例: 秘密鍵をクライアントに埋め込む
const SECRET_KEY: &str = "super_secret_123";  // ❌ 誰でも見つけられる

fn encrypt_save(data: &[u8]) -> Vec<u8> {
    xor_encrypt(data, SECRET_KEY.as_bytes())  // 解読容易
}
```

**対策**:
```rust
// 良い例: サーバー側で処理、またはユーザー固有キー
fn get_encryption_key(user_id: UserId) -> Key {
    // サーバーから取得、または
    // ユーザーパスワードから派生
    derive_key_from_user_data(user_id)
}
```

---

### 1.3 平文セーブデータ

**症状**:
- JSONやXMLで保存 → 直接編集
- お金、アイテム、進行度を自由に変更

**対策**:
```rust
// 暗号化 + 署名
fn save_game(data: &GameData) -> Result<(), Error> {
    let serialized = bincode::serialize(data)?;
    let encrypted = encrypt(&serialized);
    let signature = sign(&encrypted);

    write_file(Path::new("save.dat"), &[encrypted, signature].concat())
}

fn load_game() -> Result<GameData, Error> {
    let file = read_file(Path::new("save.dat"))?;
    let (encrypted, signature) = split_at_signature(&file);

    if !verify_signature(&encrypted, &signature) {
        return Err(Error::TamperedSave);
    }

    let decrypted = decrypt(&encrypted)?;
    bincode::deserialize(&decrypted)
}
```

---

### 1.4 バリデーション不足

**症状**:
- 範囲外の値でクラッシュ
- 負の値で無限リソース
- 異常な速度で移動

**実例**:
```rust
// 悪い例: 入力をそのまま使用
fn set_player_speed(speed: f32) {
    player.speed = speed;  // 1000000.0 でも受け入れる
}

// 良い例: バリデーション
fn set_player_speed(speed: f32) -> Result<(), Error> {
    if speed < 0.0 || speed > MAX_SPEED {
        return Err(Error::InvalidSpeed);
    }
    player.speed = speed;
    Ok(())
}
```

---

## 2. パフォーマンスアンチパターン

### 2.1 過剰なクライアントチェック

**症状**:
- FPS低下
- ラグ増加
- ユーザー体験悪化

**実例**:
```rust
// 悪い例: 毎フレーム全メモリスキャン
fn anti_cheat_update() {
    scan_all_memory();           // 重い
    verify_all_files();          // 重い
    check_running_processes();   // 重い
}
```

**対策**:
```rust
// 良い例: サンプリングと分散
fn anti_cheat_update(frame: u64) {
    match frame % 1000 {
        0 => verify_critical_files(),
        500 => sample_memory_check(),
        _ => {}  // ほとんどのフレームは何もしない
    }
}
```

---

### 2.2 全プレイヤー全アクション検証

**症状**:
- サーバー負荷爆発
- レスポンス遅延
- スケールしない

**対策**:
```rust
// 重要度に応じた検証
fn validate_action(action: &Action) {
    match action.importance {
        Importance::Critical => full_validation(action),
        Importance::Normal => basic_validation(action),
        Importance::Cosmetic => skip_validation(),
    }
}
```

---

## 3. 誤警報アンチパターン

### 3.1 閾値の設定ミス

**症状**:
- 正規プレイヤーが誤BAN
- 上手いプレイヤーほど検出される
- コミュニティの不満

**実例**:
```rust
// 悪い例: 固定閾値
if headshot_rate > 0.5 {
    ban_player(player);  // 上手い人を誤BAN
}

// 良い例: 文脈を考慮
fn evaluate_player(stats: &PlayerStats) -> SuspicionLevel {
    let adjusted_threshold = calculate_threshold(
        stats.play_time,
        stats.skill_rating,
        stats.weapon_used,
    );

    if stats.headshot_rate > adjusted_threshold {
        SuspicionLevel::Review  // 即BANではなくレビュー
    } else {
        SuspicionLevel::Normal
    }
}
```

---

### 3.2 単一指標依存

**症状**:
- 一つの異常で即判定
- 偶然の一致で誤BAN
- 検出回避が容易

**対策**:
```rust
// 複合指標で判定
struct SuspicionScore {
    speed_anomaly: f32,
    accuracy_anomaly: f32,
    timing_anomaly: f32,
    pattern_anomaly: f32,
}

fn calculate_final_score(scores: &SuspicionScore) -> f32 {
    // 複数の指標を重み付けして合算
    scores.speed_anomaly * 0.3
        + scores.accuracy_anomaly * 0.3
        + scores.timing_anomaly * 0.2
        + scores.pattern_anomaly * 0.2
}
```

---

## 4. 運用アンチパターン

### 4.1 ログ不足

**症状**:
- 何が起きたかわからない
- 不正の証拠がない
- 誤BANの弁明ができない

**対策**:
```rust
// 重要なアクションはすべてログ
fn log_action(player: PlayerId, action: &Action, context: &Context) {
    let log_entry = ActionLog {
        timestamp: Instant::now(),
        player,
        action: action.clone(),
        position: context.player_position,
        server_tick: context.tick,
        client_input_seq: context.input_seq,
    };

    append_log(log_entry);
}
```

---

### 4.2 即時BAN

**症状**:
- 誤BANの修正が困難
- 正規プレイヤーの離脱
- 訴訟リスク

**対策**:
```rust
enum Consequence {
    Monitor,        // 監視強化
    Warn,           // 警告
    TemporaryBan,   // 一時BAN
    PermanentBan,   // 永久BAN
}

fn handle_detection(player: PlayerId, confidence: f32) {
    match confidence {
        c if c < 0.5 => apply_consequence(player, Consequence::Monitor),
        c if c < 0.8 => apply_consequence(player, Consequence::Warn),
        c if c < 0.95 => apply_consequence(player, Consequence::TemporaryBan),
        _ => {
            // 人間によるレビュー
            queue_for_review(player);
        }
    }
}
```

---

### 4.3 更新の怠慢

**症状**:
- 既知のチートが横行
- 対策がすぐに迂回される
- チーターコミュニティに先を越される

**対策**:
```
継続的な対策更新:
  - 週次でチート手法の監視
  - 月次で検出ロジック更新
  - 四半期でアーキテクチャレビュー
```

---

## 5. シングルプレイ特有のアンチパターン

### 5.1 過剰な制限

> 「シングルプレイでもオンラインBANする」

**症状**:
- オフラインプレイ不可
- 常時接続必須
- プレイヤーの反発

**対策**:
```
シングルプレイでの方針:
  - チートは許容（プレイヤーの自由）
  - オンライン機能にのみ制限
  - 実績/ランキングは別管理
```

---

### 5.2 MOD敵視

**症状**:
- MODがチート扱いでBAN
- MODコミュニティの離脱
- ゲームの寿命短縮

**対策**:
```rust
// MODを明示的に許可
fn detect_modification(file_hash: Hash) -> ModificationType {
    if is_approved_mod(file_hash) {
        return ModificationType::ApprovedMod;
    }
    if is_known_cheat(file_hash) {
        return ModificationType::Cheat;
    }
    ModificationType::UnknownMod  // 警告のみ
}
```

---

## 6. アンチパターン速見表

### 絶対に避ける

| パターン | 症状 | 対策 |
|----------|------|------|
| クライアント信頼 | チート蔓延 | サーバー権威 |
| 平文セーブ | データ改ざん | 暗号化+署名 |
| バリデーション不足 | 無限リソース | 全入力を検証 |
| 即時BAN | 誤BAN | 段階的対応 |

### 強く避ける

| パターン | 症状 | 対策 |
|----------|------|------|
| Obscurity依存 | キー漏洩 | 適切な暗号化 |
| 単一指標依存 | 誤検出 | 複合判定 |
| ログ不足 | 証拠なし | 詳細ログ |
| 過剰チェック | FPS低下 | サンプリング |

### 注意

| パターン | 症状 | 対策 |
|----------|------|------|
| 閾値ミス | 誤BAN | 文脈考慮 |
| 更新怠慢 | チート横行 | 継続更新 |
| MOD敵視 | コミュニティ離脱 | MOD許可 |

---

## 7. チェックリスト

### アーキテクチャ
- [ ] サーバー権威か
- [ ] クライアントを信頼していないか
- [ ] 適切な暗号化を使用しているか

### 検出
- [ ] 複合指標で判定しているか
- [ ] 閾値は適切か
- [ ] 誤検出のリスクを考慮しているか

### 運用
- [ ] 段階的な対応があるか
- [ ] 詳細なログがあるか
- [ ] 定期的な更新計画があるか

---

## 参考文献

- [How to Avoid Letting people Cheat or Hack your Games - GitHub](https://gist.github.com/gamedev44/28f2af7a8af042dfe64dfb8967cece30)
- [Protecting and Cracking Game Save files - Medium](https://medium.com/@pantelis/protecting-game-saves-and-the-case-of-unworthy-e24c8fd68e16)
- [Game Security Best Practices 2025 - Generalist Programmer](https://generalistprogrammer.com/tutorials/game-security-best-practices-complete-protection-guide-2025)

---

*このレポートはゲームセキュリティの失敗事例調査に基づいています。*
