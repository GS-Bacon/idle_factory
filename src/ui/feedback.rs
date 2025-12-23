// src/ui/feedback.rs
//! UIフィードバックシステム (U2)
//!
//! - 全操作に0.1秒以内のフィードバック
//! - 視覚+音声の両方でフィードバック
//! - 操作成功/失敗/警告の明確な区別

use bevy::prelude::*;

/// UIフィードバックプラグイン
pub struct UiFeedbackPlugin;

impl Plugin for UiFeedbackPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FeedbackSettings>()
            .add_event::<FeedbackEvent>()
            .add_systems(
                Update,
                (
                    process_feedback_events,
                    update_visual_feedback,
                    cleanup_feedback_entities,
                ),
            );
    }
}

/// フィードバックタイプ
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FeedbackType {
    /// 成功（緑）
    Success,
    /// 失敗/エラー（赤）
    Error,
    /// 警告（黄）
    Warning,
    /// 情報（青）
    Info,
    /// ニュートラル（白/グレー）
    Neutral,
}

impl FeedbackType {
    /// フィードバックタイプに対応する色
    pub fn color(&self) -> Color {
        match self {
            FeedbackType::Success => Color::srgb(0.2, 0.8, 0.2),
            FeedbackType::Error => Color::srgb(0.9, 0.2, 0.2),
            FeedbackType::Warning => Color::srgb(0.9, 0.7, 0.1),
            FeedbackType::Info => Color::srgb(0.2, 0.6, 0.9),
            FeedbackType::Neutral => Color::srgb(0.8, 0.8, 0.8),
        }
    }

    /// フィードバックタイプに対応するサウンドID
    pub fn sound_id(&self) -> &'static str {
        match self {
            FeedbackType::Success => "ui_success",
            FeedbackType::Error => "ui_error",
            FeedbackType::Warning => "ui_warning",
            FeedbackType::Info => "ui_info",
            FeedbackType::Neutral => "ui_click",
        }
    }
}

/// フィードバック設定
#[derive(Resource)]
pub struct FeedbackSettings {
    /// 視覚フィードバック有効
    pub visual_enabled: bool,
    /// 音声フィードバック有効
    pub audio_enabled: bool,
    /// フラッシュ効果有効
    pub flash_enabled: bool,
    /// 画面揺れ有効
    pub shake_enabled: bool,
    /// フィードバック表示時間（秒）
    pub display_duration: f32,
    /// フラッシュ透明度
    pub flash_opacity: f32,
}

impl Default for FeedbackSettings {
    fn default() -> Self {
        Self {
            visual_enabled: true,
            audio_enabled: true,
            flash_enabled: true,
            shake_enabled: true,
            display_duration: 0.5,
            flash_opacity: 0.3,
        }
    }
}

/// フィードバックイベント
#[derive(Event)]
pub struct FeedbackEvent {
    /// フィードバックタイプ
    pub feedback_type: FeedbackType,
    /// メッセージ（オプション）
    pub message: Option<String>,
    /// 位置（ワールド座標、Noneなら画面中央）
    pub position: Option<Vec3>,
    /// 画面フラッシュを表示するか
    pub show_flash: bool,
    /// 画面揺れを発生させるか
    pub shake_screen: bool,
}

impl FeedbackEvent {
    /// 成功フィードバック
    pub fn success() -> Self {
        Self {
            feedback_type: FeedbackType::Success,
            message: None,
            position: None,
            show_flash: false,
            shake_screen: false,
        }
    }

    /// エラーフィードバック
    pub fn error() -> Self {
        Self {
            feedback_type: FeedbackType::Error,
            message: None,
            position: None,
            show_flash: true,
            shake_screen: true,
        }
    }

    /// 警告フィードバック
    pub fn warning() -> Self {
        Self {
            feedback_type: FeedbackType::Warning,
            message: None,
            position: None,
            show_flash: true,
            shake_screen: false,
        }
    }

    /// メッセージ付き
    pub fn with_message(mut self, message: impl Into<String>) -> Self {
        self.message = Some(message.into());
        self
    }

    /// 位置指定
    pub fn at_position(mut self, position: Vec3) -> Self {
        self.position = Some(position);
        self
    }
}

/// 視覚フィードバックコンポーネント
#[derive(Component)]
pub struct VisualFeedback {
    /// フィードバックタイプ
    pub feedback_type: FeedbackType,
    /// 残り時間
    pub remaining: f32,
    /// 初期時間
    pub initial_duration: f32,
}

/// 画面フラッシュコンポーネント
#[derive(Component)]
pub struct ScreenFlash {
    /// 色
    pub color: Color,
    /// 残り時間
    pub remaining: f32,
}

/// 画面揺れコンポーネント
#[derive(Component)]
pub struct ScreenShake {
    /// 強度
    pub intensity: f32,
    /// 残り時間
    pub remaining: f32,
    /// 元のカメラ位置
    pub original_position: Vec3,
}

/// フィードバックテキスト
#[derive(Component)]
pub struct FeedbackText {
    /// 残り時間
    pub remaining: f32,
    /// 上昇速度
    pub rise_speed: f32,
}

/// フィードバックイベントを処理
fn process_feedback_events(
    mut commands: Commands,
    mut events: EventReader<FeedbackEvent>,
    settings: Res<FeedbackSettings>,
) {
    for event in events.read() {
        // 視覚フィードバック
        if settings.visual_enabled {
            // フラッシュ効果
            if event.show_flash && settings.flash_enabled {
                commands.spawn((
                    ScreenFlash {
                        color: event
                            .feedback_type
                            .color()
                            .with_alpha(settings.flash_opacity),
                        remaining: 0.15,
                    },
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Px(0.0),
                        top: Val::Px(0.0),
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        ..default()
                    },
                    BackgroundColor(
                        event
                            .feedback_type
                            .color()
                            .with_alpha(settings.flash_opacity),
                    ),
                ));
            }

            // メッセージ表示
            if let Some(ref message) = event.message {
                commands.spawn((
                    FeedbackText {
                        remaining: settings.display_duration,
                        rise_speed: 50.0,
                    },
                    Text::new(message.clone()),
                    TextColor(event.feedback_type.color()),
                    TextFont {
                        font_size: 24.0,
                        ..default()
                    },
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Percent(50.0),
                        top: Val::Percent(40.0),
                        ..default()
                    },
                ));
            }
        }

        // 音声フィードバック（サウンドシステムにイベントを送信）
        if settings.audio_enabled {
            info!("Play sound: {}", event.feedback_type.sound_id());
            // 実際にはPlaySoundEventを送信
        }
    }
}

/// 視覚フィードバックを更新
fn update_visual_feedback(
    mut flashes: Query<(&mut ScreenFlash, &mut BackgroundColor)>,
    mut texts: Query<(&mut FeedbackText, &mut Node)>,
    time: Res<Time>,
) {
    let dt = time.delta_secs();

    // フラッシュ更新
    for (mut flash, mut bg_color) in flashes.iter_mut() {
        flash.remaining -= dt;
        if flash.remaining > 0.0 {
            // フェードアウト
            let alpha = flash.remaining / 0.15;
            bg_color.0 = flash.color.with_alpha(alpha * 0.3);
        }
    }

    // テキスト更新（上昇アニメーション）
    for (mut text, mut node) in texts.iter_mut() {
        text.remaining -= dt;
        if let Val::Percent(top) = node.top {
            node.top = Val::Percent(top - text.rise_speed * dt / 10.0);
        }
    }
}

/// フィードバックエンティティをクリーンアップ
fn cleanup_feedback_entities(
    mut commands: Commands,
    flashes: Query<(Entity, &ScreenFlash)>,
    texts: Query<(Entity, &FeedbackText)>,
) {
    for (entity, flash) in flashes.iter() {
        if flash.remaining <= 0.0 {
            commands.entity(entity).despawn();
        }
    }

    for (entity, text) in texts.iter() {
        if text.remaining <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}

/// 操作結果をフィードバック
pub trait OperationFeedback {
    /// 操作成功時のフィードバック
    fn feedback_success(&self, writer: &mut EventWriter<FeedbackEvent>);
    /// 操作失敗時のフィードバック
    fn feedback_error(&self, writer: &mut EventWriter<FeedbackEvent>, reason: &str);
}

/// ボタンホバーフィードバック
#[derive(Component)]
pub struct ButtonFeedback {
    /// 通常色
    pub normal_color: Color,
    /// ホバー色
    pub hover_color: Color,
    /// 押下色
    pub pressed_color: Color,
    /// 無効色
    pub disabled_color: Color,
}

impl Default for ButtonFeedback {
    fn default() -> Self {
        Self {
            normal_color: Color::srgb(0.15, 0.15, 0.15),
            hover_color: Color::srgb(0.25, 0.25, 0.25),
            pressed_color: Color::srgb(0.35, 0.35, 0.35),
            disabled_color: Color::srgb(0.1, 0.1, 0.1),
        }
    }
}

/// アクション確認ダイアログ
pub struct ConfirmationDialog {
    /// タイトル
    pub title: String,
    /// メッセージ
    pub message: String,
    /// 確認ボタンテキスト
    pub confirm_text: String,
    /// キャンセルボタンテキスト
    pub cancel_text: String,
    /// 確認タイプ（危険な操作は赤）
    pub confirm_type: FeedbackType,
}

impl Default for ConfirmationDialog {
    fn default() -> Self {
        Self {
            title: "Confirm".to_string(),
            message: "Are you sure?".to_string(),
            confirm_text: "OK".to_string(),
            cancel_text: "Cancel".to_string(),
            confirm_type: FeedbackType::Info,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feedback_type_colors() {
        let success_color = FeedbackType::Success.color();
        let error_color = FeedbackType::Error.color();

        // 成功は緑系
        assert!(success_color.to_linear().green > success_color.to_linear().red);
        // エラーは赤系
        assert!(error_color.to_linear().red > error_color.to_linear().green);
    }

    #[test]
    fn test_feedback_event_builder() {
        let event = FeedbackEvent::success()
            .with_message("Item placed")
            .at_position(Vec3::new(1.0, 2.0, 3.0));

        assert_eq!(event.feedback_type, FeedbackType::Success);
        assert_eq!(event.message, Some("Item placed".to_string()));
        assert!(event.position.is_some());
    }

    #[test]
    fn test_button_feedback_default() {
        let feedback = ButtonFeedback::default();

        // ホバー色は通常色より明るい
        assert!(feedback.hover_color.to_linear().red > feedback.normal_color.to_linear().red);
    }
}
