# Security & Anti-Cheat Skill

セキュリティとチート対策の設計・実装を支援するスキルです。

## 参照ドキュメント

- `.specify/specs/security-anticheat-best-practices.md`
- `.specify/specs/security-anticheat-antipatterns.md`

---

## 多層防御アーキテクチャ

```
Layer 1: サーバー権威（基盤）
    ↓
Layer 2: 入力バリデーション
    ↓
Layer 3: 統計的異常検知
    ↓
Layer 4: ログ・監査
```

---

## 必須チェックリスト

### サーバー権威

- [ ] プレイヤー位置はサーバーが計算
- [ ] HPはサーバーのみ変更可能
- [ ] インベントリはサーバー管理
- [ ] ダメージ計算はサーバー側

### 入力バリデーション

```rust
fn validate_input(input: &PlayerInput) -> Result<(), ValidationError> {
    // 移動速度チェック
    if input.movement.length() > MAX_SPEED {
        return Err(ValidationError::SpeedHack);
    }

    // レート制限
    if input.actions_per_second > MAX_ACTIONS {
        return Err(ValidationError::RateLimitExceeded);
    }

    // 範囲チェック
    if !is_within_bounds(input.target_position) {
        return Err(ValidationError::OutOfBounds);
    }

    Ok(())
}
```

### セーブデータ保護

```rust
use aes_gcm::{Aes256Gcm, KeyInit};

fn encrypt_save(data: &[u8], key: &[u8; 32]) -> Vec<u8> {
    let cipher = Aes256Gcm::new_from_slice(key).unwrap();
    let nonce = generate_random_nonce();
    cipher.encrypt(&nonce, data).unwrap()
}

fn decrypt_save(encrypted: &[u8], key: &[u8; 32]) -> Result<Vec<u8>, DecryptError> {
    let cipher = Aes256Gcm::new_from_slice(key).unwrap();
    cipher.decrypt(&extract_nonce(encrypted), encrypted)
}
```

---

## 実装パターン

### C1. 多層防御

| レイヤー | 対策 |
|----------|------|
| 1. サーバー権威 | 全重要ロジックをサーバー実行 |
| 2. 入力検証 | 速度、範囲、レート制限 |
| 3. 異常検知 | 統計分析、パターン検出 |
| 4. 監査ログ | 詳細ログ、リプレイ保存 |

### C2. 入力バリデーション

```rust
struct InputValidator {
    max_speed: f32,
    max_actions_per_second: u32,
    world_bounds: AABB,
}

impl InputValidator {
    fn validate(&self, input: &Input) -> ValidationResult {
        // 物理的に不可能な移動を検出
        // クールダウン無視を検出
        // 範囲外アクセスを検出
    }
}
```

### C3. セーブ保護

| 手法 | 目的 |
|------|------|
| AES-256-GCM | 暗号化 + 整合性 |
| ハッシュチェック | 改ざん検出 |
| バージョン管理 | 古いセーブの移行 |

---

## アンチパターン回避

| 絶対に避ける | 対策 |
|-------------|------|
| クライアント信頼 | サーバー権威 |
| 平文セーブ | 暗号化 |
| 難読化依存 | 多層防御 |
| 検証なしアクション | 全入力検証 |

---

## インディーゲーム向け戦略

### 最小限の効果的対策

1. **サーバー権威**（必須）
2. **セーブ暗号化**（オフラインゲーム）
3. **基本的な入力検証**
4. **詳細ログ**

### 避けるべき過剰対策

- カーネルレベルアンチチート（リソース不足）
- リアルタイム機械学習（複雑すぎる）
- 常時オンライン必須（UX悪化）

---

## 段階的BAN対応

```rust
enum ViolationResponse {
    Warning,           // 初回：警告
    ShadowBan,         // 複数回：シャドウBAN
    TemporaryBan(u32), // 継続：一時BAN（日数）
    PermanentBan,      // 悪質：永久BAN
}

fn handle_violation(player: &Player, severity: u32) -> ViolationResponse {
    match player.violation_count {
        0 => ViolationResponse::Warning,
        1..=3 => ViolationResponse::ShadowBan,
        4..=10 => ViolationResponse::TemporaryBan(severity * 7),
        _ => ViolationResponse::PermanentBan,
    }
}
```

---

*このスキルはセキュリティ実装の品質を確保するためのガイドです。*
