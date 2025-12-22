# ゲームサウンド実装ベストプラクティス

**作成日**: 2025-12-22
**目的**: プレイヤーに快適なオーディオ体験を提供するための実装指針

---

## 1. オーディオミキシングの基本原則

### 1.1 音量バランスの優先順位

| 優先度 | カテゴリ | 目安レベル |
|--------|----------|-----------|
| 1 | ダイアログ/ボイス | 最も明瞭に |
| 2 | 重要なSE（警告、フィードバック） | ダイアログと被らない帯域 |
| 3 | ゲームプレイSE（機械、アクション） | -3〜6dB from ダイアログ |
| 4 | 環境音/アンビエント | -10〜15dB from ダイアログ |
| 5 | BGM | 最も控えめ |

### 1.2 業界標準ラウドネス

> Sony, Nintendo, Microsoftの推奨値（Game Audio Network Guild経由）

| プラットフォーム | 推奨ラウドネス |
|-----------------|---------------|
| コンソール/PC | -24 LUFS |
| ポータブル | -16 LUFS |

**Wwise標準**: -23dB LUFS（Loudness Normalization機能）

---

## 2. ダッキングシステム

### 2.1 ダッキングとは

重要な音声（ダイアログ等）の再生時に、他のオーディオを自動的に減衰させる技術。

### 2.2 推奨パラメータ

```
ダイアログ時のダッキング:
  対象: BGM、環境音、一般SE
  減衰量: 10〜15dB
  アタック: 50〜100ms（急すぎない）
  リリース: 500ms〜2s（ポンピング防止）

一般SE時のダッキング:
  対象: BGM
  減衰量: 3〜6dB
  リリース: 300〜500ms
```

### 2.3 Bevy実装例

```rust
#[derive(Resource)]
struct AudioMixer {
    master_volume: f32,
    music_volume: f32,
    sfx_volume: f32,
    voice_volume: f32,
    ducking_amount: f32,
}

impl AudioMixer {
    fn apply_ducking(&mut self, duck_target: DuckTarget, amount: f32) {
        match duck_target {
            DuckTarget::Music => self.music_volume *= (1.0 - amount),
            DuckTarget::Sfx => self.sfx_volume *= (1.0 - amount),
            DuckTarget::All => {
                self.music_volume *= (1.0 - amount);
                self.sfx_volume *= (1.0 - amount * 0.5);
            }
        }
    }
}
```

---

## 3. 優先度システム

### 3.1 音の優先度分類

```
Priority 1 (最高): ダイアログ、重大警告
Priority 2: プレイヤーアクション音
Priority 3: 近距離の敵/機械音
Priority 4: 環境SE
Priority 5 (最低): 遠距離アンビエント
```

### 3.2 同時発音数の管理

```rust
const MAX_SIMULTANEOUS_SOUNDS: usize = 32;
const MAX_PER_CATEGORY: usize = 8;

#[derive(Component)]
struct SoundPriority(u8);

fn manage_sound_priority(
    mut sounds: Query<(&SoundPriority, &mut AudioSink)>,
) {
    // 優先度でソート、上限超過時は低優先度を停止
    let mut sorted: Vec<_> = sounds.iter_mut().collect();
    sorted.sort_by_key(|(p, _)| p.0);

    for (i, (_, mut sink)) in sorted.iter_mut().enumerate() {
        if i >= MAX_SIMULTANEOUS_SOUNDS {
            sink.stop();
        }
    }
}
```

---

## 4. 反復疲労の防止

### 4.1 バリエーションシステム

> 「同じ音の繰り返しはプレイヤーを疲弊させる」

```rust
#[derive(Resource)]
struct SoundVariants {
    footsteps: Vec<Handle<AudioSource>>,
    clicks: Vec<Handle<AudioSource>>,
    machine_hum: Vec<Handle<AudioSource>>,
}

fn play_with_variation(
    variants: &[Handle<AudioSource>],
    commands: &mut Commands,
    asset_server: &AssetServer,
) {
    use rand::Rng;
    let mut rng = rand::thread_rng();

    // ランダムなバリエーション選択
    let sound = &variants[rng.gen_range(0..variants.len())];

    // ピッチのランダム化（±5%）
    let pitch = 0.95 + rng.gen::<f32>() * 0.10;

    // ボリュームの微調整（±10%）
    let volume = 0.90 + rng.gen::<f32>() * 0.20;

    commands.spawn(AudioBundle {
        source: sound.clone(),
        settings: PlaybackSettings::DESPAWN
            .with_speed(pitch)
            .with_volume(Volume::new(volume)),
    });
}
```

### 4.2 Minecraftの例

> 「ブロックを何度も叩いても毎回ピッチが違うので快適」

| テクニック | 効果 |
|-----------|------|
| ピッチ変化 | ±5〜15%のランダム化 |
| 複数バリエーション | 3〜5種類の音素材 |
| ボリューム変化 | ±10%のランダム化 |
| 間引き | 連続時は一部再生をスキップ |

---

## 5. 空間オーディオ

### 5.1 基本設定

```rust
fn setup_spatial_audio(
    mut commands: Commands,
) {
    commands.spawn((
        SpatialBundle::default(),
        AudioBundle {
            source: sound,
            settings: PlaybackSettings::LOOP,
        },
        SpatialSettings {
            min_distance: 1.0,   // フル音量の距離
            max_distance: 50.0,  // 聞こえなくなる距離
            rolloff: RolloffMode::Linear,
        },
    ));
}
```

### 5.2 減衰カーブ

| モード | 用途 |
|--------|------|
| Linear | 一般的なSE |
| Logarithmic | リアルな物理挙動 |
| Custom | 特殊な演出 |

### 5.3 距離による音質変化

```
近距離 (0-5m): フル帯域、高明瞭度
中距離 (5-20m): 高音減衰、リバーブ増加
遠距離 (20m+): ローパス、リバーブ強
```

---

## 6. UIオーディオのベストプラクティス

### 6.1 基本原則

| 原則 | 説明 |
|------|------|
| **反復耐性** | 何度聞いても疲れない |
| **ソフトアタック** | 急激な立ち上がりを避ける（10ms以上のフェードイン） |
| **感情的意味** | 成功=上昇音、失敗=下降音 |
| **一貫性** | 同じ操作は同じ音 |

### 6.2 UIサウンドの分類

```
ポジティブ（成功、確認）:
  - 協和音程（長3度、完全5度）
  - 上昇する音程
  - 明るい音色

ネガティブ（エラー、キャンセル）:
  - 不協和音程
  - 下降する音程
  - ダークな音色

ニュートラル（ホバー、移動）:
  - 短い、控えめ
  - 邪魔にならない
```

### 6.3 省略すべき場面

| 場面 | 理由 |
|------|------|
| 高頻度テキストチャット | 通知音が煩わしい |
| 常時表示UI | 不要な音は省く |
| 連続入力中 | 全キーに音は不要 |

---

## 7. アダプティブミュージック

### 7.1 Factorioのアプローチ

> 「音楽はアクションを説明するのではなく、風景・雰囲気を表現する」

```
プレイヤーの集中を妨げない:
  - 大きな感情の波を避ける
  - ニュートラルだが豊かで動的
  - 設計・物流を考える時間を邪魔しない
```

### 7.2 レイヤリング技法

```
基本レイヤー構成:
  Layer 1: アンビエントベッド（常時）
  Layer 2: ハーモニックパッド（状況に応じて）
  Layer 3: リズム要素（テンション時）
  Layer 4: メロディ（クライマックス時）

クロスフェード:
  - ズームレベルで切り替え
  - ゲーム状態で切り替え
  - 時間経過で切り替え
```

---

## 8. アクセシビリティ

### 8.1 視覚障害者向け

| 機能 | 実装 |
|------|------|
| **音声UI** | 全メニュー項目を読み上げ |
| **空間手がかり** | 方向を示す音響定位 |
| **ソナー機能** | Pingで周囲を探索 |
| **イベント音声** | 重要イベントを音声通知 |

### 8.2 聴覚障害者向け

| 機能 | 実装 |
|------|------|
| **字幕** | 全ダイアログ＋話者名 |
| **効果音字幕** | [爆発音] [足音] |
| **視覚的キュー** | 音に対応する画面エフェクト |
| **振動フィードバック** | ゲームパッド対応 |

---

## 9. ボリュームスライダー設計

### 9.1 推奨スライダー構成

```
必須:
  - マスター
  - BGM
  - SE
  - ボイス/ダイアログ

推奨追加:
  - 環境音
  - UI音
  - 通知音
```

### 9.2 実装例

```rust
#[derive(Resource, Serialize, Deserialize)]
struct AudioSettings {
    master: f32,      // 0.0 - 1.0
    music: f32,
    sfx: f32,
    voice: f32,
    ambient: f32,
    ui: f32,
}

impl AudioSettings {
    fn get_final_volume(&self, category: AudioCategory) -> f32 {
        let category_vol = match category {
            AudioCategory::Music => self.music,
            AudioCategory::Sfx => self.sfx,
            AudioCategory::Voice => self.voice,
            AudioCategory::Ambient => self.ambient,
            AudioCategory::Ui => self.ui,
        };
        self.master * category_vol
    }
}
```

---

## 10. 本プロジェクトへの適用

### 10.1 工場ゲーム特有の考慮点

| 要素 | 対策 |
|------|------|
| 多数の機械音 | 優先度システム必須 |
| 長時間プレイ | 疲労防止最優先 |
| 複雑な操作 | UIフィードバック明確に |
| 集中が必要 | BGMは控えめに |

### 10.2 推奨サウンドカテゴリ

```
Category 1: Player (歩行、アクション)
Category 2: Machine (機械稼働音)
Category 3: Production (生産完了、警告)
Category 4: UI (クリック、ホバー、通知)
Category 5: Ambient (環境音)
Category 6: Music (BGM)
```

### 10.3 優先実装順

1. **カテゴリ別ボリューム制御**
2. **ダッキングシステム**
3. **機械音のバリエーション化**
4. **空間オーディオ（機械の位置）**
5. **アダプティブBGM**

---

## 参考文献

- [5 Audio Pitfalls Every Game Developer Should Know](https://www.thegameaudioco.com/5-audio-pitfalls-every-game-developer-should-know)
- [Tips and Techniques for Balancing Game Audio](https://www.linkedin.com/advice/3/how-do-you-balance-game-audio-other-sound-elements)
- [Wwise HDR Overview](https://www.audiokinetic.com/en/blog/wwise-hdr-overview-and-best-practices-for-game-mixing/)
- [Game Audio Theory: Ducking](https://www.gamedeveloper.com/audio/game-audio-theory-ducking)
- [How to Maintain Immersion in Game Audio](https://www.asoundeffect.com/game-audio-immersion/)
- [Ross Tregenza's 3 Essential Ingredients to Great Game UI Sound Design](https://www.asoundeffect.com/game-ui-sound-design/)
- [Factorio Friday Facts #396 - Sound Improvements](https://www.factorio.com/blog/post/fff-396)
- [Blind Accessibility in Interactive Entertainment](https://www.audiokinetic.com/en/blog/blind-accessibility-in-interactive-entertainment/)

---

*このレポートはゲームオーディオ実装のベストプラクティス調査に基づいています。*
