# Accessibility & Localization Skill

アクセシビリティとローカライズの設計・実装を支援するスキルです。

## 参照ドキュメント

- `.specify/specs/accessibility-localization-best-practices.md`

---

## 視覚アクセシビリティ

### 色覚対応

```rust
enum ColorBlindMode {
    Normal,
    Protanopia,    // 赤色覚異常
    Deuteranopia,  // 緑色覚異常
    Tritanopia,    // 青色覚異常
}

fn apply_colorblind_filter(color: Color, mode: ColorBlindMode) -> Color {
    match mode {
        ColorBlindMode::Normal => color,
        ColorBlindMode::Protanopia => simulate_protanopia(color),
        ColorBlindMode::Deuteranopia => simulate_deuteranopia(color),
        ColorBlindMode::Tritanopia => simulate_tritanopia(color),
    }
}
```

### 色に依存しない情報伝達

| 情報 | 色 | 追加表現 |
|------|-----|----------|
| 成功 | 緑 | チェックマーク |
| エラー | 赤 | X マーク |
| 警告 | 黄 | 三角形アイコン |
| 情報 | 青 | i アイコン |

### コントラスト

```
最小コントラスト比: 4.5:1（WCAG AA）
推奨コントラスト比: 7:1（WCAG AAA）

テキスト vs 背景のチェックツール:
- WebAIM Contrast Checker
- Colour Contrast Analyser
```

---

## 聴覚アクセシビリティ

### 字幕システム

```rust
#[derive(Component)]
struct Subtitle {
    text: String,
    speaker: Option<String>,
    duration: f32,
    style: SubtitleStyle,
}

struct SubtitleStyle {
    font_size: f32,
    background_opacity: f32,
    position: SubtitlePosition,
}

// 設定可能な項目
struct SubtitleSettings {
    enabled: bool,
    size: FontSize,      // Small, Medium, Large
    background: bool,
    speaker_labels: bool,
}
```

### 視覚的音響表示

| 音タイプ | 視覚表示 |
|----------|----------|
| 環境音 | 方向インジケーター |
| 警告音 | 画面フラッシュ |
| 機械音 | 振動パターン |
| 音声 | 字幕 |

---

## 運動アクセシビリティ

### キーリマップ

```rust
struct InputBindings {
    bindings: HashMap<Action, Vec<KeyCode>>,
}

impl InputBindings {
    fn rebind(&mut self, action: Action, new_key: KeyCode) {
        self.bindings.insert(action, vec![new_key]);
    }

    fn add_alternative(&mut self, action: Action, key: KeyCode) {
        self.bindings.get_mut(&action).unwrap().push(key);
    }
}
```

### 入力補助

| 機能 | 説明 |
|------|------|
| ホールド/トグル切替 | 長押し → 切り替え |
| 入力感度調整 | マウス感度、デッドゾーン |
| オートエイム | 対象自動補正 |
| ワンボタンモード | 複合入力を単一化 |

### 時間制限緩和

```rust
struct AccessibilitySettings {
    disable_time_limits: bool,
    extended_timers: f32,  // 倍率
    pause_anywhere: bool,
}

fn apply_timer_modifier(base_time: f32, settings: &AccessibilitySettings) -> f32 {
    if settings.disable_time_limits {
        f32::INFINITY
    } else {
        base_time * settings.extended_timers
    }
}
```

---

## 認知アクセシビリティ

### ヒントシステム

```rust
struct HintSettings {
    show_hints: bool,
    hint_frequency: HintFrequency,
    progressive_hints: bool,  // 時間経過で詳細化
}

enum HintFrequency {
    OnRequest,     // 要求時のみ
    Occasional,    // たまに
    Frequent,      // 頻繁に
}
```

### 難易度調整

| 設定 | 説明 |
|------|------|
| 自動セーブ頻度 | 1分〜無効 |
| 失敗ペナルティ | なし〜厳格 |
| チュートリアルスキップ | 可/不可 |
| ゲーム速度 | 0.5x〜2x |

---

## ローカライズ

### 文字列外部化

```rust
// fluent形式
// locales/ja/main.ftl
// inventory-title = インベントリ
// items-count = { $count } 個

fn localize(key: &str, args: &HashMap<&str, FluentValue>) -> String {
    BUNDLE.format_value(key, Some(args))
}

// 使用例
let text = localize("items-count", &hashmap!{ "count" => 42 });
```

### レイアウト適応

```rust
// テキスト拡張を考慮したUI
struct LocalizedText {
    key: String,
    max_expansion: f32,  // 1.5 = 50%拡張可能
}

fn calculate_text_width(text: &str, font_size: f32, language: Language) -> f32 {
    let base_width = measure_text(text, font_size);

    // 言語による拡張率
    let expansion = match language {
        Language::German => 1.3,
        Language::Japanese => 0.8,
        Language::English => 1.0,
        _ => 1.2,
    };

    base_width * expansion
}
```

### RTL対応

```rust
enum TextDirection {
    LTR,  // 左から右（英語、日本語など）
    RTL,  // 右から左（アラビア語、ヘブライ語）
}

fn layout_ui(direction: TextDirection) {
    match direction {
        TextDirection::LTR => layout_left_to_right(),
        TextDirection::RTL => layout_right_to_left(),
    }
}
```

---

## 設定UI構成

```
アクセシビリティ設定
├── 視覚
│   ├── 色覚モード: [通常/P型/D型/T型]
│   ├── UIスケール: [75%-200%]
│   ├── フォントサイズ: [小/中/大]
│   └── ハイコントラスト: [ON/OFF]
├── 聴覚
│   ├── 字幕: [ON/OFF]
│   ├── 字幕サイズ: [小/中/大]
│   ├── 話者ラベル: [ON/OFF]
│   └── 視覚的音響表示: [ON/OFF]
├── 操作
│   ├── キーリマップ: [設定...]
│   ├── マウス感度: [スライダー]
│   ├── ホールド/トグル: [設定...]
│   └── ワンハンドモード: [ON/OFF]
└── 認知
    ├── チュートリアルヒント: [頻度]
    ├── ゲーム速度: [0.5x-2x]
    └── 時間制限緩和: [ON/OFF]
```

---

## チェックリスト

### 視覚

- [ ] 色覚モードがあるか
- [ ] コントラスト比4.5:1以上か
- [ ] 色だけに頼っていないか
- [ ] UIスケール調整可能か
- [ ] フォントサイズ変更可能か

### 聴覚

- [ ] 字幕オプションがあるか
- [ ] 字幕サイズ調整可能か
- [ ] 視覚的音響表示があるか

### 操作

- [ ] 全キーがリマップ可能か
- [ ] ホールド/トグル切替可能か
- [ ] 感度調整可能か
- [ ] ワンハンドモードがあるか

### 認知

- [ ] ヒントシステムがあるか
- [ ] ポーズがいつでも可能か
- [ ] 難易度調整可能か

### ローカライズ

- [ ] 文字列が外部化されているか
- [ ] テキスト拡張を考慮したレイアウトか
- [ ] RTL言語対応可能か

---

*このスキルはアクセシビリティとローカライズの品質を確保するためのガイドです。*
