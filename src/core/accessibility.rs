// src/core/accessibility.rs
//! アクセシビリティシステム (A1-A3)
//!
//! ## 視覚 (A1)
//! - 色覚モード（P型/D型/T型）
//! - コントラスト比4.5:1以上
//! - 色+形状による情報伝達
//! - UIスケール調整
//!
//! ## 聴覚 (A2)
//! - 字幕オプション
//! - 視覚的音響表示（方向インジケーター）
//! - カテゴリ別ボリューム
//!
//! ## 運動 (A3)
//! - 全キーリマップ可能（input.rsと連携）
//! - ホールド/トグル切替
//! - マウス感度調整

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// アクセシビリティプラグイン
pub struct AccessibilityPlugin;

impl Plugin for AccessibilityPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AccessibilitySettings>()
            .add_event::<AccessibilityChangedEvent>()
            .add_systems(Update, apply_accessibility_settings);
    }
}

/// 色覚モード
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ColorBlindMode {
    /// 通常
    #[default]
    Normal,
    /// P型（赤色覚異常）
    Protanopia,
    /// D型（緑色覚異常）
    Deuteranopia,
    /// T型（青色覚異常）
    Tritanopia,
    /// 高コントラスト
    HighContrast,
}

impl ColorBlindMode {
    /// 色を変換
    pub fn transform_color(&self, color: Color) -> Color {
        match self {
            ColorBlindMode::Normal => color,
            ColorBlindMode::Protanopia => self.apply_protanopia(color),
            ColorBlindMode::Deuteranopia => self.apply_deuteranopia(color),
            ColorBlindMode::Tritanopia => self.apply_tritanopia(color),
            ColorBlindMode::HighContrast => self.apply_high_contrast(color),
        }
    }

    /// P型色覚シミュレーション（赤を緑系に変換）
    fn apply_protanopia(&self, color: Color) -> Color {
        let rgba = color.to_linear();
        // 簡易変換行列（実際のシミュレーションはもっと複雑）
        let r = rgba.red * 0.567 + rgba.green * 0.433;
        let g = rgba.red * 0.558 + rgba.green * 0.442;
        let b = rgba.blue;
        Color::linear_rgba(r, g, b, rgba.alpha)
    }

    /// D型色覚シミュレーション（緑を赤系に変換）
    fn apply_deuteranopia(&self, color: Color) -> Color {
        let rgba = color.to_linear();
        let r = rgba.red * 0.625 + rgba.green * 0.375;
        let g = rgba.red * 0.700 + rgba.green * 0.300;
        let b = rgba.blue;
        Color::linear_rgba(r, g, b, rgba.alpha)
    }

    /// T型色覚シミュレーション（青を緑系に変換）
    fn apply_tritanopia(&self, color: Color) -> Color {
        let rgba = color.to_linear();
        let r = rgba.red;
        let g = rgba.green * 0.950 + rgba.blue * 0.050;
        let b = rgba.green * 0.433 + rgba.blue * 0.567;
        Color::linear_rgba(r, g, b, rgba.alpha)
    }

    /// 高コントラスト（彩度と明度を強調）
    fn apply_high_contrast(&self, color: Color) -> Color {
        let hsla = color.to_srgba();
        // 明度を強調（暗い色はより暗く、明るい色はより明るく）
        let lightness = if hsla.lightness() < 0.5 {
            hsla.lightness() * 0.5
        } else {
            1.0 - (1.0 - hsla.lightness()) * 0.5
        };
        Color::srgba(hsla.red, hsla.green, hsla.blue, hsla.alpha).with_luminance(lightness)
    }

    /// 次のモードを取得
    pub fn next(&self) -> Self {
        match self {
            ColorBlindMode::Normal => ColorBlindMode::Protanopia,
            ColorBlindMode::Protanopia => ColorBlindMode::Deuteranopia,
            ColorBlindMode::Deuteranopia => ColorBlindMode::Tritanopia,
            ColorBlindMode::Tritanopia => ColorBlindMode::HighContrast,
            ColorBlindMode::HighContrast => ColorBlindMode::Normal,
        }
    }

    /// 表示名
    pub fn display_name(&self) -> &'static str {
        match self {
            ColorBlindMode::Normal => "Normal",
            ColorBlindMode::Protanopia => "Protanopia (P-type)",
            ColorBlindMode::Deuteranopia => "Deuteranopia (D-type)",
            ColorBlindMode::Tritanopia => "Tritanopia (T-type)",
            ColorBlindMode::HighContrast => "High Contrast",
        }
    }
}

/// アクション入力モード
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum InputMode {
    /// ホールド（押している間有効）
    #[default]
    Hold,
    /// トグル（1回押すとON、もう1回押すとOFF）
    Toggle,
}

/// アクセシビリティ設定
#[derive(Resource, Clone, Serialize, Deserialize)]
pub struct AccessibilitySettings {
    // 視覚 (A1)
    /// 色覚モード
    pub color_blind_mode: ColorBlindMode,
    /// UIスケール（0.75 - 2.0）
    pub ui_scale: f32,
    /// 高コントラストUI
    pub high_contrast_ui: bool,
    /// スクリーンシェイクの強さ（0.0 - 1.0）
    pub screen_shake_intensity: f32,
    /// フラッシュエフェクト有効
    pub flash_effects_enabled: bool,

    // 聴覚 (A2)
    /// 字幕表示
    pub subtitles_enabled: bool,
    /// 字幕サイズ（1.0 = 標準）
    pub subtitle_scale: f32,
    /// 字幕背景
    pub subtitle_background: bool,
    /// 視覚的音響インジケーター
    pub visual_sound_indicators: bool,

    // 運動 (A3)
    /// スプリント入力モード
    pub sprint_mode: InputMode,
    /// しゃがみ入力モード
    pub crouch_mode: InputMode,
    /// マウス感度（0.1 - 3.0）
    pub mouse_sensitivity: f32,
    /// 反転Y軸
    pub invert_y: bool,
    /// 片手モード
    pub one_handed_mode: bool,

    // 認知
    /// ヒント表示
    pub show_hints: bool,
    /// 簡易チュートリアル
    pub simplified_tutorials: bool,
    /// 一時停止可能（カットシーン中など）
    pub pause_anywhere: bool,
}

impl Default for AccessibilitySettings {
    fn default() -> Self {
        Self {
            // 視覚
            color_blind_mode: ColorBlindMode::Normal,
            ui_scale: 1.0,
            high_contrast_ui: false,
            screen_shake_intensity: 1.0,
            flash_effects_enabled: true,

            // 聴覚
            subtitles_enabled: false,
            subtitle_scale: 1.0,
            subtitle_background: true,
            visual_sound_indicators: false,

            // 運動
            sprint_mode: InputMode::Hold,
            crouch_mode: InputMode::Hold,
            mouse_sensitivity: 1.0,
            invert_y: false,
            one_handed_mode: false,

            // 認知
            show_hints: true,
            simplified_tutorials: false,
            pause_anywhere: true,
        }
    }
}

impl AccessibilitySettings {
    /// プリセット: 視覚障害向け
    pub fn preset_visual_impaired() -> Self {
        Self {
            color_blind_mode: ColorBlindMode::HighContrast,
            ui_scale: 1.5,
            high_contrast_ui: true,
            screen_shake_intensity: 0.0,
            flash_effects_enabled: false,
            visual_sound_indicators: true,
            show_hints: true,
            ..Default::default()
        }
    }

    /// プリセット: 聴覚障害向け
    pub fn preset_hearing_impaired() -> Self {
        Self {
            subtitles_enabled: true,
            subtitle_scale: 1.25,
            subtitle_background: true,
            visual_sound_indicators: true,
            ..Default::default()
        }
    }

    /// プリセット: 運動障害向け
    pub fn preset_motor_impaired() -> Self {
        Self {
            sprint_mode: InputMode::Toggle,
            crouch_mode: InputMode::Toggle,
            mouse_sensitivity: 0.5,
            one_handed_mode: true,
            pause_anywhere: true,
            ..Default::default()
        }
    }

    /// コントラスト比を計算（WCAG 2.0準拠）
    /// 4.5:1以上が推奨
    pub fn calculate_contrast_ratio(foreground: Color, background: Color) -> f32 {
        let fg_luminance = Self::relative_luminance(foreground);
        let bg_luminance = Self::relative_luminance(background);

        let lighter = fg_luminance.max(bg_luminance);
        let darker = fg_luminance.min(bg_luminance);

        (lighter + 0.05) / (darker + 0.05)
    }

    /// 相対輝度を計算
    fn relative_luminance(color: Color) -> f32 {
        let rgba = color.to_linear();
        0.2126 * rgba.red + 0.7152 * rgba.green + 0.0722 * rgba.blue
    }

    /// コントラスト比がWCAG AA基準を満たすか
    pub fn meets_contrast_standard(foreground: Color, background: Color) -> bool {
        Self::calculate_contrast_ratio(foreground, background) >= 4.5
    }
}

/// アクセシビリティ設定変更イベント
#[derive(Event)]
pub struct AccessibilityChangedEvent {
    pub settings: AccessibilitySettings,
}

/// アクセシビリティ設定を適用
fn apply_accessibility_settings(
    mut events: EventReader<AccessibilityChangedEvent>,
    mut settings: ResMut<AccessibilitySettings>,
) {
    for event in events.read() {
        *settings = event.settings.clone();
        info!("Accessibility settings updated");
    }
}

/// 視覚的音響インジケーターコンポーネント
#[derive(Component)]
pub struct SoundIndicator {
    /// 音源の方向（ラジアン）
    pub direction: f32,
    /// 強度（0.0 - 1.0）
    pub intensity: f32,
    /// 表示時間
    pub lifetime: f32,
}

/// 字幕コンポーネント
#[derive(Component)]
pub struct Subtitle {
    /// テキスト
    pub text: String,
    /// 話者（オプション）
    pub speaker: Option<String>,
    /// 表示時間
    pub duration: f32,
    /// 経過時間
    pub elapsed: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_blind_modes() {
        let red = Color::srgb(1.0, 0.0, 0.0);

        // 各モードで変換が行われることを確認
        let normal = ColorBlindMode::Normal.transform_color(red);
        let proto = ColorBlindMode::Protanopia.transform_color(red);
        let deuter = ColorBlindMode::Deuteranopia.transform_color(red);

        // 通常モードは変換なし
        assert_eq!(normal.to_linear().red, red.to_linear().red);

        // P型・D型では赤が他の色に変換される
        assert!(proto.to_linear().green > 0.3);
        assert!(deuter.to_linear().green > 0.2);
    }

    #[test]
    fn test_contrast_ratio() {
        let white = Color::WHITE;
        let black = Color::BLACK;

        let ratio = AccessibilitySettings::calculate_contrast_ratio(white, black);
        // 白と黒のコントラスト比は21:1に近い
        assert!(ratio > 20.0);

        // WCAG AA基準を満たす
        assert!(AccessibilitySettings::meets_contrast_standard(white, black));
    }

    #[test]
    fn test_presets() {
        let visual = AccessibilitySettings::preset_visual_impaired();
        assert!(visual.high_contrast_ui);
        assert_eq!(visual.screen_shake_intensity, 0.0);

        let hearing = AccessibilitySettings::preset_hearing_impaired();
        assert!(hearing.subtitles_enabled);
        assert!(hearing.visual_sound_indicators);

        let motor = AccessibilitySettings::preset_motor_impaired();
        assert_eq!(motor.sprint_mode, InputMode::Toggle);
        assert!(motor.one_handed_mode);
    }

    #[test]
    fn test_color_mode_cycle() {
        let mut mode = ColorBlindMode::Normal;
        mode = mode.next();
        assert_eq!(mode, ColorBlindMode::Protanopia);
        mode = mode.next();
        assert_eq!(mode, ColorBlindMode::Deuteranopia);
        mode = mode.next();
        assert_eq!(mode, ColorBlindMode::Tritanopia);
        mode = mode.next();
        assert_eq!(mode, ColorBlindMode::HighContrast);
        mode = mode.next();
        assert_eq!(mode, ColorBlindMode::Normal);
    }
}
