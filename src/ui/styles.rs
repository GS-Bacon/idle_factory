// src/ui/styles.rs
//! モダンUIスタイルシステム
//!
//! Glassmorphism + Soft UI を組み合わせたモダンなデザインシステム
//! - 半透明のフロストガラス効果
//! - ソフトシャドウ
//! - グラデーションアクセント
//! - 滑らかなアニメーション

use bevy::prelude::*;

// ========================================
// カラーパレット
// ========================================

pub mod colors {
    use bevy::prelude::*;

    // === ベースカラー ===
    pub const BG_DARK: Color = Color::srgba(0.08, 0.09, 0.12, 1.0);
    pub const BG_PANEL: Color = Color::srgba(0.12, 0.13, 0.17, 0.95);
    pub const BG_CARD: Color = Color::srgba(0.15, 0.16, 0.20, 0.90);
    pub const BG_INPUT: Color = Color::srgba(0.10, 0.11, 0.14, 0.95);

    // === オーバーレイ ===
    pub const OVERLAY: Color = Color::srgba(0.0, 0.0, 0.0, 0.75);

    // === アクセントカラー ===
    pub const ACCENT_PRIMARY: Color = Color::srgb(0.25, 0.55, 0.95);
    pub const ACCENT_PRIMARY_HOVER: Color = Color::srgb(0.35, 0.65, 1.0);
    pub const SUCCESS: Color = Color::srgb(0.25, 0.75, 0.45);
    pub const WARNING: Color = Color::srgb(0.95, 0.65, 0.25);
    pub const DANGER: Color = Color::srgb(0.85, 0.30, 0.35);

    // === テキストカラー ===
    pub const TEXT_PRIMARY: Color = Color::srgba(0.95, 0.95, 0.97, 1.0);
    pub const TEXT_SECONDARY: Color = Color::srgba(0.70, 0.72, 0.78, 1.0);
    pub const TEXT_DISABLED: Color = Color::srgba(0.45, 0.47, 0.52, 1.0);

    // === ボーダーカラー ===
    pub const BORDER: Color = Color::srgba(0.30, 0.32, 0.38, 0.5);
    pub const BORDER_ACTIVE: Color = Color::srgba(0.45, 0.65, 0.95, 0.8);

    // === ボタンカラー ===
    pub const BUTTON_DEFAULT: Color = Color::srgba(0.22, 0.24, 0.30, 0.95);
    pub const BUTTON_HOVER: Color = Color::srgba(0.28, 0.30, 0.38, 0.95);
    pub const BUTTON_PRESSED: Color = Color::srgba(0.18, 0.20, 0.26, 0.95);
    pub const BUTTON_PRIMARY: Color = Color::srgba(0.25, 0.55, 0.95, 0.95);

    // === スロットカラー ===
    pub const SLOT_DEFAULT: Color = Color::srgba(0.18, 0.19, 0.24, 0.90);
    pub const SLOT_HOVER: Color = Color::srgba(0.25, 0.27, 0.34, 0.95);
    pub const SLOT_SELECTED: Color = Color::srgba(0.30, 0.45, 0.65, 0.95);
    pub const SLOT_FILLED: Color = Color::srgba(0.22, 0.25, 0.32, 0.95);
}

// ========================================
// サイズ定数
// ========================================

pub mod sizes {
    pub const SLOT: f32 = 56.0;
    pub const SLOT_GAP: f32 = 4.0;
    pub const PANEL_PADDING: f32 = 24.0;
    pub const PANEL_GAP: f32 = 16.0;
    pub const BUTTON_HEIGHT: f32 = 48.0;
    pub const BUTTON_HEIGHT_SM: f32 = 36.0;
    pub const RADIUS_SM: f32 = 6.0;
    pub const RADIUS_MD: f32 = 10.0;
    pub const RADIUS_LG: f32 = 16.0;
    pub const BORDER_THIN: f32 = 1.0;
    pub const BORDER_NORMAL: f32 = 2.0;
    pub const BORDER_THICK: f32 = 3.0;
}

// ========================================
// フォントサイズ
// ========================================

pub mod fonts {
    pub const TITLE_LG: f32 = 42.0;
    pub const TITLE_MD: f32 = 32.0;
    pub const TITLE_SM: f32 = 24.0;
    pub const BODY_LG: f32 = 18.0;
    pub const BODY_MD: f32 = 16.0;
    pub const BODY_SM: f32 = 14.0;
    pub const CAPTION: f32 = 12.0;
    pub const LABEL: f32 = 11.0;
}

// ========================================
// ボタンインタラクションシステム
// ========================================

/// モダンなボタンインタラクション
#[allow(clippy::type_complexity)]
pub fn modern_button_interaction(
    mut query: Query<
        (&Interaction, &mut BackgroundColor, &mut BorderColor),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, mut bg_color, mut border_color) in &mut query {
        match *interaction {
            Interaction::Pressed => {
                *bg_color = BackgroundColor(colors::BUTTON_PRESSED);
                *border_color = BorderColor(colors::BORDER_ACTIVE);
            }
            Interaction::Hovered => {
                *bg_color = BackgroundColor(colors::BUTTON_HOVER);
                *border_color = BorderColor(colors::BORDER_ACTIVE);
            }
            Interaction::None => {
                *bg_color = BackgroundColor(colors::BUTTON_DEFAULT);
                *border_color = BorderColor(colors::BORDER);
            }
        }
    }
}

/// モダンUIスタイルプラグイン
pub struct ModernUiStylePlugin;

impl Plugin for ModernUiStylePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, modern_button_interaction);
    }
}
