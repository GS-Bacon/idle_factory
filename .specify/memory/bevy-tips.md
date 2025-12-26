# Bevy技術知見

問題が発生した際に参照するBevyの技術的ノウハウ。

---

## カメラ入力遅延の解消（2024-12-26確認済み）

### 症状
マウスを動かすとカメラが遅れて追従する感じがある。

### 原因
1. **VSync (AutoVsync)**: フレームバッファが2フレーム分の遅延を引き起こす
2. **パイプラインレンダリング**: 入力が1フレーム遅れて反映される
3. **フレームレイテンシ**: デフォルト値が2で追加遅延

### 解決策

```rust
use bevy::render::pipelined_rendering::PipelinedRenderingPlugin;
use bevy::window::PresentMode;

App::new()
    .add_plugins(
        DefaultPlugins
            .build()
            // パイプラインレンダリング無効化（1フレーム遅延削減）
            .disable::<PipelinedRenderingPlugin>()
            .set(WindowPlugin {
                primary_window: Some(Window {
                    // VSync無効化（フレームバッファ待ち削減）
                    present_mode: PresentMode::AutoNoVsync,
                    // フレームレイテンシを2→1に
                    desired_maximum_frame_latency: std::num::NonZeroU32::new(1),
                    ..default()
                }),
                ..default()
            }),
    )
```

### トレードオフ
- フレームレート: 10-30%低下の可能性
- 入力遅延: 大幅改善（2-3フレーム → 1フレーム以下）

FPSゲームでは入力レスポンスが優先。

### マウス入力の推奨設定

```rust
use bevy::input::mouse::AccumulatedMouseMotion;

// MouseMotionイベントではなくAccumulatedMouseMotionを使用
fn player_look(accumulated_mouse_motion: Res<AccumulatedMouseMotion>) {
    let delta = accumulated_mouse_motion.delta;
    // スムージングなし、delta_time掛けない（1:1入力）
    camera.yaw -= delta.x * MOUSE_SENSITIVITY;
    camera.pitch -= delta.y * MOUSE_SENSITIVITY;
}
```

### 参考
- https://github.com/bevyengine/bevy/discussions/16335
- https://github.com/bevyengine/bevy/issues/3317
- https://bevy-cheatbook.github.io/setup/perf.html

---

## FPS表示

### 推奨: ウィンドウタイトルに表示

Bevy 0.15ではUIテキスト表示に問題があるため、ウィンドウタイトルが最も確実。

```rust
use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};

// プラグイン追加
.add_plugins(FrameTimeDiagnosticsPlugin)

// システム追加
fn update_window_title_fps(diagnostics: Res<DiagnosticsStore>, mut windows: Query<&mut Window>) {
    if let Some(fps) = diagnostics.get(&FrameTimeDiagnosticsPlugin::FPS) {
        if let Some(value) = fps.smoothed() {
            if let Ok(mut window) = windows.get_single_mut() {
                window.title = format!("Idle Factory - FPS: {:.0}", value);
            }
        }
    }
}
```

### 失敗パターン（2024-12-26）

以下は動作しなかった:
- `Text` + `Node` + `BackgroundColor` で独立UIノード → テキストが表示されない
- `FpsOverlayPlugin` → 同様に表示されない

原因は不明だが、UIテキスト全般に問題がある可能性。インベントリUIも黒い四角のみ表示。
