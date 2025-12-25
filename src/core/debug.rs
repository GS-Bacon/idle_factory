use bevy::prelude::*;
use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};

#[derive(Resource, Default)]
pub struct DebugSettings {
    pub is_enabled: bool,
}

// FPSテキストを識別するためのマーカー
#[derive(Component)]
struct FpsText;

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DebugSettings>()
            .add_systems(Startup, setup_debug_ui)
            .add_systems(Update, (
                toggle_debug, 
                draw_debug_compass,
                update_fps_text
            ));
    }
}

// UIセットアップ
fn setup_debug_ui(mut commands: Commands) {
    commands.spawn((
        Text::new("FPS: N/A"),
        TextFont {
            font_size: 20.0,
            ..default()
        },
        // ★修正: Color::GREEN -> Color::srgb(0.0, 1.0, 0.0)
        TextColor(Color::srgb(0.0, 1.0, 0.0)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            display: Display::None, // デフォルト非表示
            ..default()
        },
        FpsText,
    ));
}

fn toggle_debug(
    input: Res<ButtonInput<KeyCode>>,
    mut settings: ResMut<DebugSettings>,
    mut fps_query: Query<&mut Node, With<FpsText>>,
) {
    if input.just_pressed(KeyCode::F3) {
        settings.is_enabled = !settings.is_enabled;
        debug!("Debug Mode: {}", settings.is_enabled);

        // UIの表示切り替え
        if let Ok(mut node) = fps_query.get_single_mut() {
            node.display = if settings.is_enabled {
                Display::Flex
            } else {
                Display::None
            };
        }
    }
}

// FPS更新システム
fn update_fps_text(
    diagnostics: Res<DiagnosticsStore>,
    mut query: Query<&mut Text, With<FpsText>>,
    settings: Res<DebugSettings>,
) {
    if !settings.is_enabled { return; }

    if let Some(fps) = diagnostics.get(&FrameTimeDiagnosticsPlugin::FPS) {
        if let Some(value) = fps.smoothed() {
            for mut text in &mut query {
                text.0 = format!("FPS: {:.1}", value);
            }
        }
    }
}

// コンパス描画 (ビルボード対応)
fn draw_debug_compass(
    settings: Res<DebugSettings>,
    mut gizmos: Gizmos,
    player_query: Query<&Transform, With<crate::gameplay::player::Player>>,
    camera_query: Query<&GlobalTransform, With<Camera3d>>,
) {
    if !settings.is_enabled {
        return;
    }

    // カメラの向き情報を取得 (文字を正面に向けるため)
    let cam_transform = if let Ok(t) = camera_query.get_single() {
        t
    } else {
        return;
    };
    
    // ★修正: Dir3 -> Vec3 への変換 (*をつけるだけ)
    let cam_right = *cam_transform.right();
    let cam_up = *cam_transform.up();

    if let Ok(player_tf) = player_query.get_single() {
        let center = player_tf.translation; 
        let distance = 50.0; 
        let height = 2.0;

        // 北 (N)
        let n_pos = center + Vec3::new(0.0, height, -distance);
        draw_billboard_letter(&mut gizmos, n_pos, 'N', Color::srgb(1.0, 0.0, 0.0), cam_right, cam_up);

        // 南 (S)
        let s_pos = center + Vec3::new(0.0, height, distance);
        draw_billboard_letter(&mut gizmos, s_pos, 'S', Color::WHITE, cam_right, cam_up);

        // 東 (E)
        let e_pos = center + Vec3::new(distance, height, 0.0);
        draw_billboard_letter(&mut gizmos, e_pos, 'E', Color::WHITE, cam_right, cam_up);

        // 西 (W)
        let w_pos = center + Vec3::new(-distance, height, 0.0);
        draw_billboard_letter(&mut gizmos, w_pos, 'W', Color::WHITE, cam_right, cam_up);
    }
}

// ビルボード文字描画ヘルパー
fn draw_billboard_letter(
    gizmos: &mut Gizmos, 
    pos: Vec3, 
    char: char, 
    color: Color, 
    right: Vec3, 
    up: Vec3
) {
    let h = 1.0; // 高さ
    let w = 0.5; // 幅の半分

    // 各頂点をカメラ基準で計算
    let tl = pos - right * w + up * h; // Top Left
    let tr = pos + right * w + up * h; // Top Right
    let bl = pos - right * w - up * h; // Bottom Left
    let br = pos + right * w - up * h; // Bottom Right
    // let tm = pos + up * h;             // Top Middle (未使用変数警告回避のためコメントアウト)
    // let bm = pos - up * h;             // Bottom Middle (未使用変数警告回避のためコメントアウト)
    let ml = pos - right * w;          // Middle Left
    let mr = pos + right * w;          // Middle Right
    let mm = pos;                      // Middle Middle

    match char {
        'N' => {
            gizmos.line(bl, tl, color); // 左縦
            gizmos.line(tl, br, color); // 斜め
            gizmos.line(br, tr, color); // 右縦
        },
        'S' => {
            gizmos.line(tr, tl, color); // 上横
            gizmos.line(tl, mm, color); // 左上縦
            gizmos.line(mm, mr, color); // 中横
            gizmos.line(mr, br, color); // 右下縦
            gizmos.line(br, bl, color); // 下横
        },
        'E' => {
            gizmos.line(tr, tl, color); // 上横
            gizmos.line(tl, bl, color); // 左縦
            gizmos.line(bl, br, color); // 下横
            gizmos.line(ml, mm, color); // 中横 (短め)
        },
        'W' => {
            gizmos.line(tl, bl, color); // 左縦
            gizmos.line(bl, mm, color); // 左下斜め
            gizmos.line(mm, br, color); // 右下斜め
            gizmos.line(br, tr, color); // 右縦
        },
        _ => {}
    }
}