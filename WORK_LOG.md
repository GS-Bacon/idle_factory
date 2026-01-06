# 作業ログ

## 2026-01-06: ディスク容量管理・タスク完了

### ディスク容量問題の解決

- **問題**: 放置されたworktree（4個×6.5GB = 26GB）でディスク容量不足
- **対応**:
  - 未マージブランチ2件をmasterにマージ後、全worktree削除
  - ディスク使用率: 54% → 26%（+25GB回復）

### 容量チェック機能追加

`scripts/parallel-run.sh` に安全機能を追加:

| コマンド | 説明 |
|----------|------|
| `check <数>` | N個のworktree用容量があるか事前確認 |
| `cleanup` | 放置worktreeを自動削除 |
| `start` | 開始前に自動容量チェック（不足なら中止）|

設定値:
- worktree 1つ: 7GB（安全マージン込み）
- 最低確保: 10GB

### 完了したタスク（並列実行）

1. **fix-furnace-input**: 製錬炉の搬入位置判定を`Furnace.position`を使用するよう修正
2. **tutorial-progress-bar**: チュートリアルの個数目標にプログレスバー追加
3. **hide-main-quest-tutorial**: チュートリアル中メインクエストUI非表示（前回からの継続）
4. **simplify-biome-hud**: バイオームHUD簡略化（前回からの継続）

### CLAUDE.md更新

- 容量チェック必須ルールを追記
- worktree放置禁止を明文化

---

## 2026-01-04: UI/UX改善・バグ修正

### 修正内容

#### 1. コンベア左曲がり時のアイテム移動方向バグ修正
- **問題**: コンベアのモデル位置は正しいが、左曲がり時にアイテムが右に曲がってしまう
- **原因**: コーナーコンベアで出力方向が常に`direction`（前方）になっていた
- **修正**:
  - `Conveyor`コンポーネントに`output_direction`フィールドを追加
  - `update_conveyor_shapes`でコーナーコンベアの正しい出力方向を計算
  - `conveyor_transfer`で`output_direction`を使用してアイテムを正しい方向に移動
- **関連ファイル**:
  - `src/components/machines.rs`
  - `src/systems/targeting/conveyor.rs`
  - `src/systems/conveyor.rs`
  - `src/systems/block_operations/placement.rs`
  - `src/systems/save_systems.rs`
  - `src/systems/command/handlers.rs`

#### 2. コンベア設置ゴーストの浮き問題修正
- **問題**: コンベアの設置プレビュー（ゴースト）が地面から浮いている
- **原因**: プレビュー位置が`pos.y + 0.5`（ブロック中心）になっていた
- **修正**: コンベア用のY座標を`pos.y`（地面レベル）に変更
- **関連ファイル**: `src/systems/targeting/highlight.rs`

#### 3. 機械にも設置ゴーストを表示
- **変更**: 機械（Miner, Furnace, Crusher）選択時に半透明青のプレビューキューブを表示
- **関連ファイル**: `src/systems/targeting/highlight.rs`

#### 4. 機械にも方向矢印を表示
- **変更**: 機械プレビューにも黄色い3D矢印を表示（Rキーで回転可能）
- **関連ファイル**: `src/systems/targeting/highlight.rs`

#### 5. 矢印の視認性改善
- **問題**: 矢印が細くて見にくい
- **修正**:
  - LineListから3D TriangleList（立体的なシャフト+ピラミッド型）に変更
  - 明るい黄色（`Color::srgb(1.0, 0.9, 0.0)`）で視認性向上
  - `create_3d_arrow_mesh()`関数を新規作成
- **関連ファイル**: `src/systems/targeting/highlight.rs`

#### 6. クエストUIをビジュアル改善
- **問題**: テキストベースでダサい
- **修正**:
  - プログレスバー付きの新UIに変更
  - 各アイテムの進捗を個別表示（バー+テキスト）
  - 完了状態に応じた色分け（青=進行中、黄緑=納品可能、緑=完了）
  - 日本語UI（「クエスト」「納品する」「報酬を受け取る」等）
  - ボーダーと背景のスタイリング改善
- **新規コンポーネント**:
  - `QuestProgressContainer`
  - `QuestProgressItem(usize)`
  - `QuestProgressBarBg(usize)`
  - `QuestProgressBarFill(usize)`
  - `QuestProgressText(usize)`
- **関連ファイル**:
  - `src/components/mod.rs`
  - `src/setup/ui/mod.rs`
  - `src/systems/quest.rs`

### テスト結果
- 全280件のテストがパス
- Clippy警告: 0件（許容範囲の警告のみ）
