# スキルインデックス

全スキルの要点集約。実装時に参照。

---

## クイックリファレンス

### 機能実装時のチェック

|実装内容|確認パターン|必須チェック|
|--------|-----------|-----------|
|マルチプレイ|N1-N4|サーバー権威、予測、差分圧縮|
|セキュリティ|C1-C3|入力検証、セーブ暗号化|
|UI|U1-U4,E1-E6|情報階層、操作FB、Undo|
|サウンド|S1-S4|ミキシング、バリエーション|
|グラフィック|G1-G4|LOD、インスタンシング、カリング|
|ECS設計|B1-B3|コンポーネント分割、システム順序|
|Rust|RS1-RS3|所有権、アロケーション、エラー処理|
|アクセシビリティ|A1-A3,I1-I2|色覚、字幕、リマップ|
|MOD|M1-M3|ライフサイクル、サンドボックス|
|レベルデザイン|L1-L3,P1-P4|進行フェーズ、探索報酬|

---

## コードスニペット

### サーバー権威(N1)
```rust
fn validate_input(input: &Input) -> Result<(), Error> {
    if input.speed > MAX_SPEED { return Err(SpeedHack); }
    if !bounds.contains(input.pos) { return Err(OutOfBounds); }
    Ok(())
}
```

### クライアント予測(N2)
```rust
fn predict(input: &Input) {
    apply_locally(input);
    pending.push((seq++, input.clone()));
}
fn reconcile(server: &State, ack: u64) {
    state = server.clone();
    for (s, i) in &pending { if *s > ack { apply(i); } }
}
```

### セーブ暗号化(C3)
```rust
use aes_gcm::{Aes256Gcm, KeyInit};
fn encrypt(data: &[u8], key: &[u8;32]) -> Vec<u8> {
    Aes256Gcm::new_from_slice(key).unwrap()
        .encrypt(&nonce, data).unwrap()
}
```

### Bevy ECS(B1-B3)
```rust
// 小コンポーネント
#[derive(Component)] struct Pos(Vec3);
#[derive(Component)] struct Vel(Vec3);

// システム順序
app.add_systems(Update, (input, move_sys, collide).chain());

// 変更検出
fn on_change(q: Query<&Health, Changed<Health>>) { }
```

### サウンド(S2)
```rust
fn play_varied(clips: &[Clip]) {
    let clip = clips.choose(&mut rng);
    let pitch = rng.gen_range(0.9..1.1);
    play(clip, pitch);
}
```

### LOD(G1)
```rust
fn update_lod(dist: f32) -> Lod {
    match dist {
        d if d < 64.0 => Full,
        d if d < 128.0 => Med,
        d if d < 256.0 => Low,
        _ => Icon,
    }
}
```

---

## 評価チェックリスト

### ECS
- [ ] コンポーネント単一責任
- [ ] システム順序明示
- [ ] Changed/Added活用
- [ ] 並列実行考慮

### パフォーマンス
- [ ] 60FPS維持
- [ ] 描画コール500以下
- [ ] インスタンシング使用
- [ ] 不要clone排除

### UI/UX
- [ ] 全操作にFB
- [ ] ツールチップあり
- [ ] Undo/Redo
- [ ] 情報階層適切

### アクセシビリティ
- [ ] 色覚モード
- [ ] 字幕オプション
- [ ] キーリマップ可
- [ ] コントラスト4.5:1+

### セキュリティ(MP)
- [ ] サーバー権威
- [ ] 入力バリデーション
- [ ] レート制限
- [ ] セーブ暗号化

### ネットワーク(MP)
- [ ] クライアント予測
- [ ] 差分圧縮
- [ ] 再接続処理

---

---

## 3Dモデリング

### クイックリファレンス

| 複雑さ | 戦略 |
|--------|------|
| シンプル | Hyper3D一発生成 |
| 中程度 | Hyper3D + Blender調整 |
| 複雑な機械 | **パーツ分割生成** |
| 高品質必須 | Sketchfab検索 |

### プロンプト必須要素

```
[オブジェクト], [スタイル] style, [素材] materials,
[構造], [用途], single object
```

### AI生成の得意・不得意

| カテゴリ | 品質 |
|----------|------|
| シンプルな小物 | ⭐⭐⭐⭐ |
| スタイライズドキャラ | ⭐⭐⭐ |
| 産業機械・メカ | ⭐⭐ |
| 有機的形状 | ⭐ |

**詳細**: `.claude/skills/modeling/` 参照

---

## 詳細参照

必要時のみ元ファイル参照:
- `.specify/memory/patterns-compact.md` - パターン詳細
- `.specify/specs/index-compact.md` - 仕様詳細
- `.claude/skills/modeling/` - 3Dモデリング詳細
