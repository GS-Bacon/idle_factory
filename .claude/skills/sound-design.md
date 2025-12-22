# Sound Design Skill

サウンド設計・実装を支援するスキルです。

## 参照ドキュメント

- `.specify/specs/sound-implementation-best-practices.md`
- `.specify/specs/sound-implementation-antipatterns.md`

---

## ミキシング階層

```
Master
├── Music (0.7)
├── SFX (1.0)
│   ├── UI (0.9)
│   ├── Gameplay (1.0)
│   └── Ambient (0.6)
└── Voice (1.0)
```

### 優先度システム

| 優先度 | カテゴリ | 例 |
|--------|----------|-----|
| 1 (最高) | クリティカル | アラート、警告 |
| 2 | プレイヤーアクション | 設置、クラフト |
| 3 | 環境 | 機械稼働音 |
| 4 (最低) | アンビエント | BGM、環境音 |

---

## 反復疲労防止

### バリエーション

```rust
struct SoundVariation {
    clips: Vec<AudioClip>,
    pitch_range: (f32, f32),  // e.g., (0.9, 1.1)
    volume_range: (f32, f32), // e.g., (0.8, 1.0)
}

fn play_with_variation(sound: &SoundVariation) {
    let clip = sound.clips.choose(&mut rng).unwrap();
    let pitch = rng.gen_range(sound.pitch_range.0..sound.pitch_range.1);
    let volume = rng.gen_range(sound.volume_range.0..sound.volume_range.1);
    play(clip, pitch, volume);
}
```

### 頻出音の対策

| 音タイプ | バリエーション数 | ピッチ変化 |
|----------|------------------|------------|
| 足音 | 4-6 | ±10% |
| 設置音 | 3-4 | ±5% |
| クラフト完了 | 2-3 | ±3% |
| UI操作 | 2 | ±5% |

---

## 空間オーディオ

### 減衰設定

```rust
struct SpatialAudio {
    min_distance: f32,  // 最大音量距離
    max_distance: f32,  // 無音距離
    rolloff: Rolloff,   // Linear, Logarithmic
}

// 工場ゲーム推奨値
const MACHINE_AUDIO: SpatialAudio = SpatialAudio {
    min_distance: 2.0,
    max_distance: 30.0,
    rolloff: Rolloff::Logarithmic,
};
```

### 多数音源の管理

```rust
// 近い順にN個のみ再生
fn cull_distant_sounds(
    listener: Vec3,
    sources: &mut Vec<AudioSource>,
    max_simultaneous: usize,
) {
    sources.sort_by(|a, b| {
        a.distance_to(listener).partial_cmp(&b.distance_to(listener)).unwrap()
    });
    sources.truncate(max_simultaneous);
}
```

---

## 工場ゲーム向けサウンド

### 機械音設計

| 状態 | サウンド | ボリューム |
|------|----------|------------|
| アイドル | 低いハム音 | 0.3 |
| 稼働中 | リズミカルな動作音 | 0.6 |
| 過負荷 | 警告音混じり | 0.8 |
| 停止 | 減速音 | 0.5→0 |

### コンベア音

```rust
// ベルト速度に応じた音
fn conveyor_sound(speed: f32) -> AudioParams {
    AudioParams {
        pitch: 0.8 + (speed / MAX_SPEED) * 0.4,
        volume: 0.2 + (speed / MAX_SPEED) * 0.3,
    }
}
```

---

## カテゴリ別ボリューム

### 設定UI

```rust
struct AudioSettings {
    master: f32,      // 0.0-1.0
    music: f32,
    sfx: f32,
    voice: f32,
    ambient: f32,
}

// 最終ボリューム計算
fn final_volume(category: Category, settings: &AudioSettings) -> f32 {
    settings.master * match category {
        Category::Music => settings.music,
        Category::SFX => settings.sfx,
        Category::Voice => settings.voice,
        Category::Ambient => settings.ambient,
    }
}
```

---

## アクセシビリティ

### 字幕

```rust
struct Subtitle {
    speaker: Option<String>,
    text: String,
    duration: f32,
    position: SubtitlePosition,
}

enum SubtitlePosition {
    Speaker(Entity),  // 話者の近く
    Bottom,           // 画面下部
    Top,              // 画面上部
}
```

### 視覚的音響表示

| 音タイプ | 視覚表示 |
|----------|----------|
| 警告 | 画面フラッシュ |
| 方向音 | インジケーター |
| 環境音 | 波紋エフェクト |

---

## チェックリスト

- [ ] カテゴリ別ボリュームスライダーがあるか
- [ ] 頻出音にバリエーションがあるか（3個以上）
- [ ] ピッチ/ボリューム変化があるか
- [ ] 空間オーディオが適切か
- [ ] 多数音源のカリングがあるか
- [ ] 字幕オプションがあるか
- [ ] 視覚的音響表示オプションがあるか

---

## アンチパターン

| 避ける | 対策 |
|--------|------|
| 単一サンプル繰り返し | バリエーション追加 |
| 全音同ボリューム | 優先度システム |
| 遠距離でも同音量 | 減衰カーブ設定 |
| 無限同時再生 | 音源数上限 |
| ミュート時もロード | 再生時ロード |

---

*このスキルはサウンド実装の品質を確保するためのガイドです。*
