# 次セッション: リモート環境テスト

## 前回の作業（2024-12-26 ローカル環境）

### 実施内容

1. **カメラ入力遅延を修正** - 操作性が大幅に改善
   - `PipelinedRenderingPlugin` 無効化
   - `PresentMode::AutoNoVsync` に変更
   - `desired_maximum_frame_latency: 1` に設定

2. **FPS表示をウィンドウタイトルに変更**
   - UIテキスト表示に問題があったため
   - `FrameTimeDiagnosticsPlugin` を使用

3. **知見をメモ**
   - `.specify/memory/bevy-tips.md` に記録

### 現在の設定

```rust
// 入力遅延対策
PipelinedRenderingPlugin 無効
PresentMode::AutoNoVsync
desired_maximum_frame_latency: 1

// マウス設定
MOUSE_SENSITIVITY: 0.002
AccumulatedMouseMotion 使用
```

### 確認済み

- カメラの遅延感が解消された（ローカル環境）
- テスト全件パス
- clippy警告なし

### 未解決

- UIテキスト（インベントリ等）が表示されない問題
  - 黒い四角のみ表示される
  - Bevy 0.15特有の問題の可能性

## 次にやること

1. リモート環境（Linux）でカメラ操作を確認
2. UIテキスト表示問題を調査（必要なら）

## ファイル

- `src/main.rs` - メイン
- `.specify/memory/bevy-tips.md` - 技術知見
- `run.sh` - Linux用起動スクリプト

## 操作方法

- クリック: カーソルロック
- Esc: カーソル解除
- WASD/Space/Shift: 移動
- マウス: 視点操作
- 左クリック（ロック中）: ブロック破壊
