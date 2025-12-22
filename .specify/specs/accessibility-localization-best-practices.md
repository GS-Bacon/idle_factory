# ゲーム アクセシビリティ・ローカライズ ベストプラクティス

**作成日**: 2025-12-22
**目的**: すべてのプレイヤーがゲームを楽しめるようにするための実装指針

---

## 1. アクセシビリティの基本

### 1.1 主要なガイドライン

```
Game Accessibility Guidelines (GAG):
  - 基本、中級、上級の3レベル
  - 運動、視覚、聴覚、認知の各障害に対応

Xbox Accessibility Guidelines (XAG):
  - Microsoftが策定
  - 詳細なチェックリスト付き

WCAG（Web Content Accessibility Guidelines）:
  - コントラスト比など数値基準
  - ゲームUIにも適用可能
```

### 1.2 障害カテゴリ別の考慮

| カテゴリ | 影響 | 主な対策 |
|----------|------|----------|
| 視覚 | 見えにくい/見えない | 色覚モード、高コントラスト、拡大 |
| 聴覚 | 聞こえにくい/聞こえない | 字幕、視覚的効果音 |
| 運動 | 操作が困難 | キーリマップ、長押し不要 |
| 認知 | 理解・記憶が困難 | シンプルUI、ヒント、一時停止 |

---

## 2. 視覚アクセシビリティ

### 2.1 色覚サポート

```rust
#[derive(Clone, Copy)]
enum ColorblindMode {
    Normal,
    Deuteranopia,  // 緑色覚異常（最も一般的）
    Protanopia,    // 赤色覚異常
    Tritanopia,    // 青色覚異常
}

struct ColorPalette {
    success: Color,
    warning: Color,
    error: Color,
    info: Color,
}

fn get_palette(mode: ColorblindMode) -> ColorPalette {
    match mode {
        ColorblindMode::Normal => ColorPalette {
            success: Color::GREEN,
            warning: Color::YELLOW,
            error: Color::RED,
            info: Color::BLUE,
        },
        ColorblindMode::Deuteranopia => ColorPalette {
            success: Color::CYAN,       // 緑→シアン
            warning: Color::ORANGE,
            error: Color::MAGENTA,      // 赤→マゼンタ
            info: Color::BLUE,
        },
        // 他のモードも定義
    }
}
```

### 2.2 色だけに頼らない

```rust
// 悪い例: 色だけで状態を示す
fn bad_status_indicator(health: f32) -> Color {
    if health > 0.5 { Color::GREEN } else { Color::RED }
}

// 良い例: 色 + 形状/アイコン
struct StatusIndicator {
    color: Color,
    icon: Icon,      // ハート、警告アイコンなど
    label: String,   // テキストラベル
}

fn good_status_indicator(health: f32) -> StatusIndicator {
    StatusIndicator {
        color: health_color(health),
        icon: if health > 0.5 { Icon::Heart } else { Icon::Warning },
        label: format!("HP: {:.0}%", health * 100.0),
    }
}
```

### 2.3 コントラスト

```rust
// WCAG準拠のコントラスト比
const MIN_CONTRAST_TEXT: f32 = 4.5;      // 通常テキスト
const MIN_CONTRAST_LARGE: f32 = 3.0;     // 大きいテキスト（18pt+）
const MIN_CONTRAST_UI: f32 = 3.0;        // UI要素

fn check_contrast(foreground: Color, background: Color) -> bool {
    let ratio = calculate_contrast_ratio(foreground, background);
    ratio >= MIN_CONTRAST_TEXT
}

// 背景に応じてテキスト色を調整
fn adaptive_text_color(background: Color) -> Color {
    let luminance = background.luminance();
    if luminance > 0.5 {
        Color::BLACK
    } else {
        Color::WHITE
    }
}
```

### 2.4 テキストサイズ

```rust
struct TextSettings {
    // 推奨最小サイズ（1080pで）
    min_size_console: f32,  // 26px
    min_size_pc: f32,       // 18px

    // ユーザー設定
    scale: f32,  // 1.0-2.0
}

fn calculate_font_size(base: f32, settings: &TextSettings) -> f32 {
    base * settings.scale
}
```

---

## 3. 聴覚アクセシビリティ

### 3.1 字幕

```rust
struct SubtitleSettings {
    enabled: bool,
    size: SubtitleSize,           // Small, Medium, Large
    background_opacity: f32,      // 0.0-1.0
    speaker_names: bool,          // "ジョン: こんにちは"
    sound_effects: bool,          // "[爆発音]"
}

enum SubtitleSize {
    Small,   // 26px @ 1080p
    Medium,  // 32px @ 1080p
    Large,   // 40px @ 1080p
}

// 効果音の字幕
fn format_sound_subtitle(sound: &SoundEvent) -> String {
    match sound.sound_type {
        SoundType::Explosion => "[爆発音]",
        SoundType::Footsteps => "[足音]",
        SoundType::Alert => "[警報]",
        // ...
    }.to_string()
}
```

### 3.2 視覚的音響表現

```rust
// 重要な音を視覚的に表現
struct VisualSoundIndicator {
    direction: Option<Vec2>,  // 音の方向（レーダー風表示）
    intensity: f32,           // 音の大きさ
    icon: Icon,               // 音の種類
}

fn show_sound_indicator(
    sound: &SoundEvent,
    listener: &Listener,
) -> VisualSoundIndicator {
    let direction = (sound.position - listener.position).normalize();
    VisualSoundIndicator {
        direction: Some(Vec2::new(direction.x, direction.z)),
        intensity: sound.volume,
        icon: sound_to_icon(sound),
    }
}
```

---

## 4. 運動アクセシビリティ

### 4.1 コントロールリマッピング

```rust
struct ControlSettings {
    // 全キー/ボタンがリマップ可能
    bindings: HashMap<Action, Vec<Input>>,

    // オプション
    toggle_sprint: bool,      // 長押し不要
    toggle_crouch: bool,
    toggle_aim: bool,

    hold_duration: Duration,  // 長押し判定時間
}

impl ControlSettings {
    fn rebind(&mut self, action: Action, new_input: Input) {
        self.bindings.insert(action, vec![new_input]);
    }

    fn add_alternate(&mut self, action: Action, alt_input: Input) {
        self.bindings.entry(action).or_default().push(alt_input);
    }
}
```

### 4.2 入力アシスト

```rust
struct InputAssist {
    // エイムアシスト
    aim_assist: AimAssistLevel,

    // QTE難易度
    qte_timing_window: f32,  // 1.0 = 通常、2.0 = 2倍の猶予

    // 連打不要
    mash_to_hold: bool,      // 連打→長押しに変換

    // ワンハンドモード
    one_handed_mode: bool,
}

enum AimAssistLevel {
    Off,
    Low,
    Medium,
    High,
}
```

---

## 5. 認知アクセシビリティ

### 5.1 難易度調整

```rust
struct CognitiveAssist {
    // ゲームスピード
    game_speed: f32,  // 0.5-1.0

    // ヒントシステム
    hints_enabled: bool,
    hint_frequency: HintFrequency,

    // UIシンプル化
    simplified_ui: bool,

    // 一時停止
    pause_anytime: bool,
}

enum HintFrequency {
    Never,
    OnRequest,   // プレイヤーが要求時
    AfterDelay,  // 一定時間後に自動
    Always,      // 常にヒント表示
}
```

### 5.2 情報の明確化

```rust
// 目標の明確な表示
struct ObjectiveDisplay {
    current_objective: String,
    next_steps: Vec<String>,
    marker: Option<WorldMarker>,  // 3D空間でのマーカー
}

// 進行状況の可視化
struct ProgressTracker {
    total_steps: u32,
    completed_steps: u32,
    visual_progress_bar: bool,
}
```

---

## 6. ローカライズ

### 6.1 技術的基盤

```rust
// 文字列の外部化
// strings/ja.json
// {
//   "menu.start": "ゲーム開始",
//   "menu.options": "設定",
//   "item.iron_ore": "鉄鉱石"
// }

use fluent::{FluentBundle, FluentResource};

struct LocalizationManager {
    current_locale: String,
    bundles: HashMap<String, FluentBundle<FluentResource>>,
}

impl LocalizationManager {
    fn get(&self, key: &str) -> String {
        let bundle = &self.bundles[&self.current_locale];
        bundle.format_pattern(key, None)
            .unwrap_or_else(|| key.to_string())
    }

    fn get_with_args(&self, key: &str, args: &HashMap<&str, FluentValue>) -> String {
        // プレースホルダー付き翻訳
        // "items_count": "{$count}個のアイテム"
    }
}
```

### 6.2 文字列設計

```rust
// 悪い例: 文字列連結
let message = "You have " + count.to_string() + " items";

// 良い例: プレースホルダー
// en.json: "items_count": "You have {count} items"
// ja.json: "items_count": "アイテムが{count}個あります"
let message = localize("items_count", &[("count", count)]);

// 悪い例: 文法的仮定
let message = item_name + "s";  // 複数形

// 良い例: 複数形を別キーに
// en.json: "items_one": "{count} item", "items_other": "{count} items"
// ja.json: "items_one": "{count}個", "items_other": "{count}個"
```

### 6.3 レイアウト対応

```rust
struct LocalizedUI {
    // テキスト拡張に対応
    text_container_min_width: f32,
    text_container_growth: f32,  // 最大150%まで拡張

    // RTL（右から左）対応
    direction: TextDirection,

    // フォント
    font_family: Vec<String>,  // フォールバック付き
}

enum TextDirection {
    LeftToRight,  // 英語、日本語など
    RightToLeft,  // アラビア語、ヘブライ語
}
```

### 6.4 日付・数値

```rust
use chrono::prelude::*;
use num_format::{Locale, ToFormattedString};

fn format_date(date: DateTime<Utc>, locale: &str) -> String {
    match locale {
        "ja" => date.format("%Y年%m月%d日").to_string(),
        "en-US" => date.format("%m/%d/%Y").to_string(),
        "en-GB" => date.format("%d/%m/%Y").to_string(),
        _ => date.to_rfc3339(),
    }
}

fn format_number(number: i64, locale: &str) -> String {
    match locale {
        "ja" => number.to_formatted_string(&Locale::ja),
        "en" => number.to_formatted_string(&Locale::en),
        _ => number.to_string(),
    }
}
```

---

## 7. 実装優先順位

### 7.1 必須（最低限）

```
□ 字幕オプション
□ コントロールリマッピング
□ 音量スライダー（カテゴリ別）
□ テキストサイズ調整
□ 一時停止機能
```

### 7.2 推奨

```
□ 色覚モード（3種類）
□ 高コントラストモード
□ トグルオプション（長押し→切り替え）
□ UIスケール調整
□ 難易度カスタマイズ
```

### 7.3 上級

```
□ スクリーンリーダー対応
□ 完全キーボード操作
□ 視覚的音響表示
□ ワンハンドモード
□ 詳細なアシストオプション
```

---

## 8. チェックリスト

### 視覚
- [ ] 色覚モードがあるか
- [ ] コントラスト比は4.5:1以上か
- [ ] テキストサイズは調整可能か
- [ ] 色だけに頼っていないか

### 聴覚
- [ ] 字幕があるか
- [ ] 字幕サイズは調整可能か
- [ ] 効果音にも字幕があるか

### 運動
- [ ] 全キーがリマップ可能か
- [ ] 長押しをトグルに変更できるか
- [ ] QTEは省略/簡易化できるか

### 認知
- [ ] ゲームを一時停止できるか
- [ ] ヒントシステムがあるか
- [ ] 目標が明確に表示されるか

### ローカライズ
- [ ] 文字列が外部化されているか
- [ ] プレースホルダーを使用しているか
- [ ] テキスト拡張に対応しているか
- [ ] フォントフォールバックがあるか

---

## 参考文献

- [Game Accessibility Guidelines](https://gameaccessibilityguidelines.com/)
- [Xbox Accessibility Guidelines](https://learn.microsoft.com/en-us/gaming/accessibility/xbox-accessibility-guidelines/112)
- [WCAG Contrast Requirements](https://webaim.org/articles/contrast/)
- [Video Game Accessibility Testing - TestDevLab](https://www.testdevlab.com/blog/accessibility-testing-in-video-games)
- [Game Localization Best Practices - IGDA](https://igda-website.s3.us-east-2.amazonaws.com/wp-content/uploads/2021/04/09142137/Best-Practices-for-Game-Localization-v22.pdf)

---

*このレポートはゲームアクセシビリティとローカライズのベストプラクティス調査に基づいています。*
