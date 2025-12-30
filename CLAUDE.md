# Claude Code メモリ

## 批判的姿勢（重要）

ユーザーの提案に対しても以下を確認し、問題があれば指摘する:
- 仕様地獄に戻っていないか？（詳細化・レビュー・パターン確認の連鎖）
- メタワークになっていないか？（ゲームを作らず道具を作っている）
- 実装より仕様が先行していないか？
- 「完璧」を目指して「完成」を遅らせていないか？

**過去の失敗**: 仕様39ファイル・パターン52個・レビュースキル群を作り、誰もゲームを遊んでいなかった

## 基本ルール

| ルール | 詳細 |
|--------|------|
| ビルド | 開発: `cargo build` (2秒)、テスト: `--profile release-fast` (10秒) |
| テスト | 実装後は `cargo test` と `cargo clippy` |
| コミット | 日本語、短く、技術詳細より「何をしたか」重視、pushは指示待ち |
| 中断禁止 | 確認を求めず最後まで完了 |
| 動作確認 | E2Eテストで基本動作を検証 |
| ゲーム起動 | `./run.sh` を使用（RDPディスプレイ自動検出） |
| Gemini CLI | `gemini -m gemini-3-pro-preview -p "プロンプト"` を使用 |
| Gemini活用 | 3Dモデリング・座標計算・幾何学処理で精度検証中。効果確認後に本格採用 |
| Discord通知 | コミット時に自動でwebhook送信（post-commit hook設定済み） |

## 検証の分担

| 担当 | 内容 |
|------|------|
| AI | ビルド、テスト、E2E、クリック反応、UI動作、**画面スクリーンショット確認** |
| 人間 | ゲームプレイ体験（面白さ、直感性） |

**AIが確認すべき**:
- 「クリックしても反応しない」等の技術的バグ
- スクリーンショットで表示崩れ、透け、黒い穴がないか
- ログでエラーが出ていないか
- **UI操作**（Playwrightでキー入力シミュレート）:
  - Eキーでインベントリ開閉
  - T or /キーでコマンド入力（/creative, /survival）
  - 1-9キーでホットバー選択
  - 左クリックでブロック破壊
  - 右クリックでブロック設置/機械UI開く
  - ESCでUI閉じる/ポーズ
  - F3でデバッグHUD

**人間が確認すべき**: 「遊んでいて楽しいか」等の体験

## 禁止事項

- 仕様だけのセッション2回連続
- パターン集の事前確認（問題が起きたら見る）
- レビュー・評価スキルの使用
- ドキュメント整理・圧縮作業
- **憶測での修正（必ずログまたはスクリーンショットで確認してから修正）**

## 却下済みの提案（再提案時は指摘する）

| 提案 | 却下理由 |
|------|----------|
| 定期レビュー導入 | メタワーク復活の入口。問題が起きたらで十分 |
| TDD導入 | 「テストが通る」が目的化する。実装後テストで十分 |
| プロセス追加 | Milestone 1-2は規模が小さい。E2Eで十分 |

**導入検討タイミング**: Milestone 3以降、または問題が頻発したとき

## マイルストーン駆動

```
仕様（1ページ）→ 実装 → 動かして確認 → 次へ
```

「done」= テストが通る、ではなく「遊べる」

## 3Dモデル生成

「〇〇のモデルを作成」指示で:
1. `.specify/memory/modeling-rules.md` 参照
2. サブエージェント起動
3. 完了報告

## 実プレイ検証

実プレイ検証は後回しにし、ユーザーが確認することリストにまとめる。

## バグ取りノウハウの蓄積（必須）

**ルール**: バグを発見・修正したら、必ずノウハウを蓄積する。

1. **CLAUDE.md**: 「よくあるバグと対策」セクションに追記
2. **スキル**: `.claude/commands/e2e-test.md` の「直近で発見したバグパターン」に追記
3. **テスト**: `tests/e2e_test.rs` に対応するテストを追加

**目的**: 同じバグを二度と踏まない仕組みを作る

## バグ修正の手順

**重要**: バグ修正タスクを受けたら、まずログを解析してから修正に着手する。
**絶対禁止**: 憶測での修正。「多分これが原因」で修正してはいけない。必ず証拠を見つけてから。

### 1. ログ取得（AI実行）

**ネイティブ版**:
```bash
# ゲーム起動時にログ自動保存
./run.sh

# ログ確認
./show-logs.sh              # 最新ログ（末尾50行）
./show-logs.sh 100          # 末尾100行
./show-logs.sh errors       # エラーのみ
./show-logs.sh events       # ゲームイベントのみ（BLOCK, MACHINE, QUEST）
./show-logs.sh list         # ログファイル一覧
```

**WASM版**:
```bash
# ブラウザコンソールをキャプチャ（30秒間）
node capture-wasm-logs.js

# 60秒間キャプチャ
node capture-wasm-logs.js 60

# ログ確認
./show-logs.sh wasm
```

### ログカテゴリ
- **BLOCK**: ブロック設置・破壊
- **MACHINE**: 機械（採掘機、コンベア、粉砕機、精錬炉）の設置・破壊
- **QUEST**: 納品イベント

### 2. ログ解析
ログファイル（`logs/game_latest.log` or `logs/wasm_latest.log`）を確認:
- `ERROR`, `WARN` を検索
- `BLOCK`, `MACHINE`, `QUEST` でイベント追跡
- タイムスタンプでフリーズ直前の操作を特定

### 3. 修正タスクに追記
ログから判明した情報を修正タスクに追加:
- フリーズ発生タイミング（開始から何秒後）
- 直前の操作（何をしたらフリーズしたか）
- エラーメッセージの内容

## AI画面検証

AIがバグ修正や機能実装後に実際の画面を確認する方法。

### ネイティブ版検証
```bash
./verify-native.sh
```
- ゲームを起動して自動でスクリーンショット撮影
- `screenshots/verify/native_*.png` に保存
- Readツールで画像を確認可能

### WASM版検証
```bash
./verify-wasm.sh
```
- サーバー状態確認
- WASMファイルの存在確認
- 可能であればブラウザスクリーンショット撮影
- `screenshots/verify/wasm_*.png` に保存

### 検証フロー（修正時の必須手順）

1. **修正前**: スクリーンショットでバグを確認
2. **修正後**: 再度スクリーンショットで修正を確認
3. **ログ確認**: 修正後にテスト実行してログを確認

```bash
# 例: バグ修正フロー
./verify-native.sh                    # 修正前の状態を確認
# ... コード修正 ...
cargo test && cargo clippy            # テスト実行
./verify-native.sh                    # 修正後の状態を確認
```

### スクリーンショット読み取り
```bash
# AIがスクリーンショットを確認する場合
Read /home/bacon/idle_factory/screenshots/verify/native_1_*.png
```

### WASM操作テスト（Playwright）
```bash
# 操作テストスクリプト実行
node /home/bacon/idle_factory/test-wasm-interactions.js
```

テストする操作:
1. 画面クリック（ポインターロック取得）
2. Eキー → インベントリUI開閉
3. T or /キー → コマンド入力（/creative, /survival）
4. 1-9キー → ホットバー選択
5. 左クリック → ブロック破壊
6. 右クリック → ブロック設置/機械UI
7. F3キー → デバッグHUD
8. ESCキー → ポーズ

各操作後にスクリーンショットを撮影し、UI変化を確認。

## WASMデプロイ

### 自動リビルド（推奨）
```bash
./watch-wasm.sh
```
ファイル変更を監視して自動的にWASMをビルド・デプロイする。
- `src/`, `Cargo.toml`, `assets/` を監視
- 変更検知後500ms待機して実行（連続保存対策）
- Ctrl+C で終了

### 手動デプロイ
```bash
./deploy-wasm.sh
```

### アクセスURL
- ローカル: http://10.13.1.1:8080
- Tailscale: http://100.84.170.32:8080

## 次の作業

### プレイ確認不要（AIだけで完結）

#### 1. ワインディング順序の修正（パフォーマンス向上）✅完了
**結果**: 全6面の頂点順序をCCW（反時計回り）に修正、両面レンダリングを無効化
**効果**: GPU負荷半減（バックフェースカリング有効化）

#### 2. E2Eテスト拡充（基本操作の検証）✅完了
**追加したテスト**:
- ホットバー選択（1-9キー）
- ブロック設置によるインベントリ消費
- UI開閉（Eキー/ESCキー）
- フレーム安定性テスト（100フレームシミュレーション）
- ブロック破壊の連続実行

#### 3. cargo clippy警告を全て解消 ✅完了
- 未使用メソッドに`#[allow(dead_code)]`追加
- Range::containsへのリファクタリング
- too_many_arguments警告を抑制

#### 4. main.rsのモジュール分割 ✅完了
- `constants.rs`: 定数を外部ファイルに分離
- `block_type.rs`: BlockType enumを外部ファイルに分離
- main.rsから参照、3306行→3250行

#### 5. ユニットテストのカバレッジ確認・拡充 ✅完了
- 23テスト（ユニット11 + E2E12）すべて成功

#### 6. WASMビルドの自動テスト ✅完了
- GitHub Actions CI設定（.github/workflows/ci.yml）
- test / clippy / wasm の3ジョブ

#### 7. README整備 ✅完了
- ビルド方法、起動方法、操作説明、ゲーム目標を記載

#### 8. デバッグ用コマンド追加 ✅完了
- F3キーでデバッグHUDトグル
- FPS、プレイヤー座標、チャンク数を表示

#### 9. ログ出力の整理 ✅完了
- println!は既に存在しない（整理済み）

#### 10. エラーハンドリング改善 ✅完了
- unwrap()をis_none_or/let elseパターンに置換
- レイキャスト判定、精錬炉/粉砕機処理を改善

#### 11. 設定ファイル外出し ⏸️見送り
- WASMではファイルシステムアクセスが制限される
- 現状のconstants.rsで十分機能している

### プレイ確認必要

#### 12. 実プレイで他のバグがないか確認

### 修正待ちバグ

なし

### 修正済みバグ

#### BUG-1: 納品プラットフォームの上にブロックが置けない ✅修正済み
**修正内容**: `block_place`関数にDeliveryPlatformへのレイキャスト判定を追加

#### BUG-2: 一定方向に向き続けるとマウスが飛ぶ ✅修正済み
**修正内容**: カーソルリセット時に`skip_frames = 1`を設定して次フレームをスキップ

#### BUG-3: コンベア間でアイテムがちらつく ✅修正済み
**修正内容**: 転送時にvisual_entityを引き継ぎ、despawn→新規作成による1フレーム空白を解消

#### BUG-4: コンベア上のアイテムが重なる ✅修正済み
**修正内容**: アイテムサイズ0.4→0.25、間隔0.33→0.4に調整

#### BUG-5: 横合流時のアニメーションが不自然 ✅修正済み
**修正内容**: lateral_offset導入でサイドから中央へのスムーズアニメーション実装

## 開発フロー

```
ユーザー: ローカルで修正 → プッシュ
    ↓
AI: プルしてから作業開始
    ↓
AI: 作業完了 → プッシュ
    ↓
ユーザー: プルして確認
```

**ポイント**:
- ユーザーの修正は必ずプッシュしてからAIに依頼
- AIは作業前に必ず最新をプル
- コンフリクト防止のため同時編集を避ける

## 並列作業（Git Worktree）

サブエージェントとworktreeを組み合わせて並列作業を行う。

### ユースケース1: 複数アプローチの比較
パフォーマンス最適化などで異なる手法を試す場合:

```bash
git worktree add ../idle_factory_opt1 -b opt/approach-a
git worktree add ../idle_factory_opt2 -b opt/approach-b
# → 各worktreeでサブエージェントが実装・ベンチマーク
# → 結果比較、最良をmainにマージ
```

### ユースケース2: 複数機能の並列実装
独立した機能を同時に実装する場合:

```bash
git worktree add ../idle_factory_feat1 -b feat/power-system
git worktree add ../idle_factory_feat2 -b feat/liquid-pipes
# → 各worktreeでサブエージェントが実装
# → 完了順にmainへマージ
```

### 共通フロー
```bash
# 作業完了後は即座にクリーンアップ
git worktree remove ../idle_factory_xxx
git branch -d feat/xxx  # または -D で強制削除
```

**使用条件**:
- 互いに独立した作業（コンフリクトしない）
- 並列化の恩恵がある（ビルド時間、待ち時間の削減）

**注意**: target/が各worktreeで生成されるためディスク消費大

## サブエージェント活用

コンテキスト共有が必要な並列作業にはサブエージェントを使用。

### 使い分け

| 手法 | 適したケース |
|------|-------------|
| Git Worktree | ファイルが独立、後でマージ可能 |
| サブエージェント | 同じルール参照、出力の整合性が必要 |

### サブエージェントが適切な例
- **3Dモデル生成**: 複数モデルを同じスタイルで作成
- **UI実装**: 同じデザインルールで複数画面
- **テスト作成**: 同じテストパターンで複数モジュール
- **リファクタリング**: 同じパターンを複数ファイルに適用

### 使用方法
```
# 並列でサブエージェントを起動（Taskツール）
Task 1: 「ハンマーの3Dモデルを作成。modeling-rules.md参照」
Task 2: 「コンベアの3Dモデルを作成。modeling-rules.md参照」
Task 3: 「精錬炉の3Dモデルを作成。modeling-rules.md参照」
→ 同時に起動、各自がルールを読んで作業
```

### 判断基準
- 別ファイルを編集 & 後でマージ可能 → **Worktree**
- 同じ設定参照 & 出力の整合性必要 → **サブエージェント**

## 将来のアイデア（実装しない）

| アイデア | メモ |
|----------|------|
| Discord連携 | ゲーム専用チャンネルのログをIssue化、フィードバック自動収集。レビューが増えたら検討 |
| CI/CD | mainプッシュ時にWASM自動ビルド。手動公開が面倒になったら検討 |

## 実プレイ確認後の計画

### 作業順序（ユーザー確認を最小化）

| 順番 | フェーズ | 内容 | 状態 |
|------|----------|------|------|
| 1 | フェーズ1 | バグ修正（BUG-3〜5） | ✅完了 |
| 2 | フェーズ6 | テスト改善 | ✅完了 |
| 3 | フェーズ3 | コンベア改善 | 次 |
| 4 | フェーズ4 | インベントリ改善 | 次 |
| 5 | フェーズ5 | モード・UI改善 | ✅完了 |
| → | **ユーザー確認** | フェーズ3-5の動作確認 | |
| 6 | フェーズ7 | リファクタ＋マルチ準備 | ✅完了 |
| 7 | フェーズ8 | セーブ/ロード | 未着手 |

### フェーズ1: バグ修正 ✅完了
- ~~BUG-3: コンベア間でアイテムがちらつく~~
- ~~BUG-4: コンベア上のアイテムが重なる~~
- ~~BUG-5: 横合流時のアニメーションが不自然~~

### フェーズ2: ゲームループ拡充 ✅完了
- ~~銅鉱石・銅インゴット追加~~
- ~~新機械1つ（粉砕機 or プレス機）~~
- ~~クエスト3-4追加~~

### フェーズ3: コンベア改善 ✅完了
| 機能 | 仕様 | 状態 |
|------|------|------|
| 細いベルト形状 | 0.6ブロック幅、0.2高さ | ✅ |
| ジッパー合流 | 複数入力を交互に受け入れ（last_input_source使用） | ✅ |
| 設置向き自動判断 | 隣接機械/コンベアを見て向きを決定（auto_conveyor_direction） | ✅ |
| 方向テクスチャ | 黄色の矢印メッシュで方向表示 | ✅ |
| 自動スプリッター | 将来機能へ移動（別ブロックとして実装予定） | ⏸️ |
| レンチ | 将来機能へ移動 | ⏸️ |

### フェーズ4: インベントリ改善 ✅完了
| 機能 | 仕様 | 状態 |
|------|------|------|
| メインインベントリ | 27スロット（3×9）追加 | ✅ |
| スタック上限 | 999 | ✅ |
| 操作 | クリックで移動 | ✅ |
| Shift+クリック | 一括移動（将来機能） | ⏸️ |
| ゴミ箱 | インベントリUI内に配置 | ✅ |
| Eキーで開閉 | 全モードで使用可能 | ✅ |

### フェーズ5: モード・UI改善 ✅完了
| 機能 | 仕様 | 状態 |
|------|------|------|
| コマンド入力 | T or / キーでチャット欄、`/creative` `/survival` | ✅ |
| Cキー | 廃止（コマンドに移行） | ✅ |
| 機械UI | 右クリックで開く（Eキーから変更） | ✅ |
| UI統一デザイン | `.specify/memory/ui-design-rules.md` 参照 | - |

**実装詳細**:
- コマンド入力: T or / キーで開く、Enterで実行、ESCでキャンセル
- 対応コマンド: `/creative`, `/survival`, `/give <item> [count]`, `/clear`
- 機械UI: 精錬炉・粉砕機を見て右クリックでUI開く、ESC or Eキーで閉じる

### フェーズ6: テスト改善 ✅完了
- 実プレイは「遊ぶだけ」で良い状態にする
- 技術的なバグはE2Eテスト・スクリーンショット確認でAIが検出
- テストカバレッジを継続的に拡充
- 新機能追加時は対応するテストも追加

**追加したテスト（7件）:**
- test_conveyor_item_no_overlap: アイテム間隔検証
- test_conveyor_side_merge_offset: 横合流アニメーション検証
- test_inventory_stack_limit_999: スタック上限検証
- test_multiple_conveyor_merge: 同時合流検証
- test_conveyor_loop_handling: ループ処理検証
- test_entity_count_stability: エンティティ増加防止
- test_visual_entity_handoff: ビジュアル引き継ぎ検証

**テスト総数**: 12ユニット + 47 E2E = 59件

### フェーズ7: マルチプレイ準備（アーキテクチャ対応）+ モジュール分割 ✅完了

#### 調査結果: bevy_replicon
- **クレート**: [bevy_replicon](https://github.com/projectharmonia/bevy_replicon)
- **バージョン**: 0.37.1（Bevy 0.17対応）、現在Bevy 0.15なので0.29-0.32を使用
- **方式**: サーバー権威型、自動レプリケーション
- **特徴**: I/O抽象化、シングル/クライアント/専用サーバー/リッスンサーバー全対応

#### 完了した対応
| 対応 | 内容 | 状態 |
|------|------|------|
| イベント駆動化 | events/mod.rsにイベント構造体を定義 | ✅完了 |
| プレイヤーID導入 | イベント構造体にplayer_idフィールド追加 | ✅完了 |
| 決定論的処理 | 乱数をシード付きに、サーバー時間同期準備 | 未着手 |
| Replicatedマーカー | 同期対象コンポーネントにマーカー追加準備 | 未着手 |

#### モジュール分割（完了分）
- player/inventory.rs: Inventory構造体を分離（117行）
- events/mod.rs: イベント構造体とGameEventsPluginを定義（67行）
- main.rs: 5246行→5136行（110行削減）

#### 残りのモジュール分割（将来）

```
src/
├── main.rs              # エントリポイント、プラグイン登録
├── block_type.rs        # (既存)
├── constants.rs         # (既存)
├── player/
│   ├── mod.rs
│   ├── movement.rs      # プレイヤー移動
│   ├── camera.rs        # カメラ操作
│   └── inventory.rs     # インベントリ
├── world/
│   ├── mod.rs
│   ├── chunk.rs         # チャンク生成・管理
│   ├── mesh.rs          # メッシュ生成
│   └── raycast.rs       # レイキャスト
├── machines/
│   ├── mod.rs
│   ├── miner.rs         # 採掘機
│   ├── conveyor.rs      # コンベア
│   ├── furnace.rs       # 精錬炉
│   └── crusher.rs       # 粉砕機
├── ui/
│   ├── mod.rs
│   ├── hotbar.rs        # ホットバー
│   ├── inventory_ui.rs  # インベントリUI
│   ├── machine_ui.rs    # 機械UI
│   └── quest_ui.rs      # クエストUI
├── events/              # ★マルチプレイ用
│   ├── mod.rs
│   ├── block_events.rs  # BlockPlace, BlockBreak
│   └── machine_events.rs
└── quest/
    ├── mod.rs
    └── delivery.rs      # 納品プラットフォーム
```

**リファクタリングのタイミング**: フェーズ1-6完了後、イベント駆動化と同時に実施

#### イベント設計（例）
```rust
// 状態変更イベント（将来サーバーに送信）
enum GameEvent {
    BlockPlace { pos: IVec3, block_type: BlockType, player_id: u64 },
    BlockBreak { pos: IVec3, player_id: u64 },
    MachineInteract { machine_pos: IVec3, action: MachineAction, player_id: u64 },
    ItemTransfer { from: TransferTarget, to: TransferTarget, item: BlockType, count: u32 },
}
```

#### マルチプレイ仕様（将来実装）
| 項目 | 仕様 |
|------|------|
| 方式 | 専用サーバー or ホスト兼用 |
| 人数 | サーバースペック依存（無制限） |
| プレイスタイル | 協力 |
| 権限 | 設定可能（オーナー、管理者、一般） |
| 参加方式 | IP + パスワード（任意）、Minecraft方式 |
| バックエンド | bevy_replicon_renet2（WebTransport対応） |

### フェーズ8: セーブ/ロード

| 項目 | 仕様 |
|------|------|
| 形式 | JSON（開発中）→ バイナリ（将来） |
| 自動セーブ | 1分ごと |
| スロット数 | 無制限 |
| WASM版 | LocalStorage（テスト用、後回し） |
| バージョン互換 | 可能な限り後方互換、開発初期は気にしない |

### 将来機能（優先度低）
- 優先度設定（入力/出力の優先順位）
- チェスト（アイテム保管用ブロック）
- 電力システム
- 液体・パイプ

### 後回し
- チュートリアルポップアップ
- ガイドマーカー
- 資源バイオーム

**原則**: 遊べる状態を維持しながら機能追加

## ビルド最適化（80コア環境）

### プロファイル一覧

| プロファイル | 用途 | インクリメンタル |
|-------------|------|----------------|
| dev | 開発（デバッグ情報あり） | **2秒** |
| dev-fast | 開発（最適化あり） | **2.5秒** |
| release-fast | リリーステスト | **10秒** |
| release | 最終リリース | 29秒 |

開発中は `cargo build`（devプロファイル）を使用。

### 設定ファイル

**.cargo/config.toml**:
```toml
[build]
jobs = 80
rustc-wrapper = "/home/bacon/.cargo/bin/sccache"

[target.x86_64-unknown-linux-gnu]
linker = "clang"
rustflags = ["-C", "link-arg=-fuse-ld=mold"]
```

**Cargo.toml**:
```toml
[profile.dev]
split-debuginfo = "unpacked"

[profile.dev.package."*"]
opt-level = 3

[profile.dev-fast]
inherits = "dev"
opt-level = 1
split-debuginfo = "unpacked"

[profile.release]
opt-level = 3
lto = "thin"
codegen-units = 16
strip = true

[profile.release-fast]
inherits = "release"
lto = false
opt-level = 2
```

### 必要なツール
```bash
sudo apt-get install -y mold
cargo install sccache --locked
```

### 最適化のポイント
- **split-debuginfo = "unpacked"**: デバッグ情報分離でリンク高速化（4.3秒→2秒）
- **lto = false**: LTO無効で高速ビルド（29秒→10秒）
- **sccache**: コンパイル結果をキャッシュ
- **mold**: 高速リンカー
- **jobs = 80**: 80コア並列コンパイル

### ビルド最適化を試す時の注意
- 複数プロファイルを試すとtargetが肥大化（21GB超）
- 通常開発では1-2GB程度
- 最適化作業前: `du -sh target/` で確認
- 容量不足時: `cargo clean` で全削除、または `rm -rf target/プロファイル名`

## よくあるバグと対策

**重要**: 新機能実装時、このセクションのパターンに該当する場合は対応するテストも追加すること。

### 1. 地面が透ける（黒い穴）
**原因**: メッシュのワインディング順序が間違っている
**症状**: 地面や壁に黒い穴が見える（バックフェースカリングで消える）
**自動検出**: `cargo test test_mesh_winding_order`
**対策**: 面定義の頂点順序を修正（`cross(v1-v0, v2-v0)` が法線方向を向くように）
**ファイル**: `src/main.rs` の `generate_mesh_with_neighbors` 内の `faces` 配列

### 2. 機械設置時に地面が透ける
**原因**: 機械設置時に`set_block()`で偽のブロックを登録していた
**症状**: 採掘機・コンベア設置後、その場所の地面が消える
**自動検出**: `cargo test test_machine_placement_no_block_registration`
**対策**: 機械はエンティティなので`set_block()`を呼ばない
**ファイル**: `src/main.rs` の `block_place` 関数

### 3. エンティティ破壊時に子エンティティが残る
**原因**: 親エンティティ削除時に子（アイテムビジュアル等）を削除していない
**症状**: コンベア破壊後もアイテムが浮いている
**自動検出**: `cargo test test_conveyor_destroy_cleans_item_visual`
**対策**: 破壊時に関連エンティティもdespawn
**ファイル**: `src/main.rs` の `block_break` 関数

### 4. チャンク境界でブロックが消える
**原因**: 隣接チャンクの情報なしでメッシュ生成
**症状**: チャンク境界で片面だけ描画されない
**自動検出**: `cargo test test_chunk_boundary_mesh`
**対策**: `generate_mesh_with_neighbors` で隣接チャンク情報を渡す
**ファイル**: `src/main.rs` の `chunk_mesh_update` システム

### 5. ブロック設置/破壊時のフリーズ
**原因**: チャンク再生成パターンが不統一
**症状**: 特定操作で1-2秒フリーズ
**自動検出**: `cargo test test_block_operations_no_freeze`
**対策**: `block_place`と`block_break`で同じ再生成パターンを使用
**ファイル**: `src/main.rs` の `block_place`, `block_break` 関数

### 6. レイキャスト判定漏れ
**原因**: 新しい機械/ブロックタイプを追加時にレイキャスト判定を追加し忘れ
**症状**: 特定の機械に対してクリックが効かない
**自動検出**: `cargo test test_raycast_hits_all_machine_types`
**対策**: 新機械追加時は必ず`block_break`と`block_place`のレイキャスト判定を更新
**ファイル**: `src/main.rs` の `block_break`, `block_place`, `furnace_interaction` 関数

### 7. 機械破壊時にアイテム消失
**原因**: Crusher/Furnace破壊時に入出力スロットの内容をインベントリに返却していない
**症状**: 精錬途中の精錬炉や処理中の粉砕機を壊すと、中のアイテムが消える
**自動検出**: `cargo test test_crusher_break_returns_items test_furnace_break_returns_items`
**対策**: `block_break`のHitType::Crusher/Furnace処理で、despawn前に全スロットをインベントリに返却
**ファイル**: `src/main.rs` の `block_break` 関数

### 8. モード別UIの表示制御漏れ
**原因**: クリエイティブ専用UIがCreativeModeのチェックなしで常に表示
**症状**: サバイバルモードでもクリエイティブカタログが表示される
**対策**:
- モード専用UIにはマーカーコンポーネント追加（例: `CreativePanel`）
- UI表示/非表示時にモード状態をチェック
- 対応するクリックハンドラーにもモードチェックを追加
**ファイル**: `src/main.rs` の `inventory_toggle`, `creative_inventory_click` 関数

### 9. UI表示中のポインターロック
**原因**: WASM側でcanvasクリック時にUI状態をチェックせずポインターロック要求
**症状**: インベントリUI内でクリックするとカーソルが消える
**対策**: `canvas.addEventListener('click', ...)` 内で `isGameUIOpen()` をチェック
**ファイル**: `web/index.html` のcanvasクリックハンドラー

### 実装時のチェックリスト

新機能追加時に確認:
- [ ] メッシュ生成を変更した → ワインディング順序テスト確認
- [ ] 機械を追加した → レイキャスト判定追加、破壊時クリーンアップ追加、**スロットアイテム返却**
- [ ] 子エンティティを持つ機械 → 破壊時に子もdespawn
- [ ] チャンク操作を変更した → 境界テスト確認
- [ ] ブロック操作を変更した → フリーズテスト確認
- [ ] モード専用UIを追加した → モードチェック、マーカーコンポーネント追加
- [ ] UIを追加した → `set_ui_open_state(true/false)` 呼び出し確認

### 自動整合性チェック

仕様と実装の矛盾を自動検出するテスト群。`cargo test`で毎回実行される。

| テスト名 | チェック内容 |
|----------|-------------|
| `test_hotbar_scroll_stays_in_bounds` | マウスホイールがホットバー範囲(0-8)内で循環 |
| `test_inventory_consumption_mechanism` | サバイバルモードでブロック消費、クリエイティブで無消費 |
| `test_mode_constants_consistency` | HOTBAR_SLOTS=9, NUM_SLOTS=36の整合性 |
| `test_inventory_slot_indices_consistency` | スロットインデックス範囲の正当性 |
| `test_block_type_consistency` | BlockTypeの設置可能/採掘可能フラグ整合性 |
| `test_quest_rewards_consistency` | クエスト報酬がインベントリに収まる |
| `test_initial_game_state_consistency` | 初期ゲーム状態の整合性（サバイバルモード開始等） |

**追加すべき場面**:
- モード（Creative/Survival）の挙動が異なる機能を追加したとき
- 定数（スロット数、スタック上限等）に依存する機能を追加したとき
- 初期状態に影響する変更を加えたとき

## 作業ログ

### 2025-12-30
- **整合性チェックテスト追加**
  - 仕様と実装の矛盾を自動検出するテスト7件追加
  - test_hotbar_scroll_stays_in_bounds: ホイール範囲チェック
  - test_inventory_consumption_mechanism: モード別消費チェック
  - test_mode_constants_consistency: 定数整合性
  - test_inventory_slot_indices_consistency: スロット範囲
  - test_block_type_consistency: ブロックタイプ整合性
  - test_quest_rewards_consistency: クエスト報酬
  - test_initial_game_state_consistency: 初期状態
- **バグ修正（BUG-8, BUG-9）**
  - サバイバルモードでクリエイティブUIが表示される問題を修正
  - インベントリUI内クリックでカーソルが消える問題を修正
  - CreativePanelマーカーコンポーネント追加
  - web/index.htmlのcanvasクリックハンドラーにUI状態チェック追加
- **入力ハンドラー修正**
  - select_block_type: ホイールをHOTBAR_SLOTS範囲に制限、command_stateチェック追加
  - quest_claim_rewards: command_stateチェック追加
- **WASM修正**
  - RuntimeError: unreachableを修正（LogPlugin無効化）
  - キャッシュバスティング追加（.js/.wasmファイル）
- **テスト**: 69件成功、clippy警告なし

### 2025-12-29
- **ロギングシステム簡素化**
  - GameLogBuffer, LogEntry, 未使用マクロを削除
  - logging.rsを43行に簡素化（157行→43行）
  - run.shにログファイル出力を追加（logs/game_YYYYMMDD_HHMMSS.log）
  - capture-wasm-logs.js: WASM版のブラウザコンソールキャプチャ
  - show-logs.sh: ログ確認ヘルパースクリプト
  - CLAUDE.mdの「バグ修正の手順」をAI主導のログ収集に更新
  - テスト81件成功、clippy警告なし
- **コンベア複数アイテム対応**
  - ConveyorItem構造体を追加（block_type, progress, visual_entity）
  - 1コンベアあたり最大3アイテム（CONVEYOR_MAX_ITEMS定数）
  - アイテム間の最小間隔0.33（CONVEYOR_ITEM_SPACING定数）
  - conveyor_transferを大幅リファクタリング（2フェーズ処理でborrow競合回避）
- **コンベア途中合流対応**
  - Conveyor::get_join_progress()メソッド追加
  - 側面（垂直）からの合流はprogress=0.5から開始
  - 後方からの合流はprogress=0.0から開始
  - miner_output, crusher_output, furnace_output, conveyor_transferを更新
- **納品アニメーション改善**
  - アイテムがprogress=1.0（コンベア端）に到達してから納品
  - 途中で消えず、最後まで流れてから納品
- **コンベア用ワイヤーフレーム（向き矢印付き）**
  - create_conveyor_wireframe_mesh(direction)関数追加
  - 矢印で方向を表示
  - コンベア設置時は設置予定の向きを表示
  - 既存コンベア選択時は実際の向きを表示
- **その他の改善**
  - Inventory::get_selected_type()メソッド追加
  - update_target_highlightにコンベア検出を追加
  - constants.rsにCONVEYOR_MAX_ITEMS, CONVEYOR_ITEM_SPACING追加
- **テスト**: 40テスト成功、clippy警告なし
- **フェーズ1: バグ修正（BUG-3,4,5）**
  - BUG-3: 転送時にvisual_entityを引き継ぎ、ちらつき解消
  - BUG-4: アイテムサイズ0.4→0.25、間隔0.33→0.4に調整
  - BUG-5: lateral_offset導入でスムーズな横合流アニメーション
  - ConveyorItem構造体にlateral_offsetフィールド追加
  - get_join_info()メソッド追加（progress, lateral_offset両方返す）
  - update_conveyor_item_visualsでlateral_vec計算を追加
- **フェーズ6: テスト改善**
  - test_conveyor_item_no_overlap追加
  - test_conveyor_side_merge_offset追加
  - test_inventory_stack_limit_999追加
  - test_multiple_conveyor_merge追加
  - test_conveyor_loop_handling追加
  - test_entity_count_stability追加
  - test_visual_entity_handoff追加
  - テスト総数: 59件（12ユニット + 47 E2E）
- **フェーズ7: モジュール分割・イベント駆動化**
  - player/inventory.rs: Inventory構造体を分離（117行）
  - events/mod.rs: イベント構造体とGameEventsPluginを定義（67行）
  - BlockPlaceEvent, BlockBreakEvent, MachineInteractEvent等
  - main.rs: 5246行→5136行（110行削減）
- **ログシステム改善**
  - logging.rs: GameLoggingPlugin, GameLogBuffer追加
  - tracing crateでRust側の構造化ログ
  - BLOCK/MACHINE/QUESTカテゴリでログ分類
  - 主要操作にinfo!ログ追加（block_break, block_place, 機械設置/破壊, 納品）
  - WASM: ブラウザコンソールでログ確認可能
    - showGameLogs() - ログ一覧
    - filterLogs('MACHINE') - カテゴリフィルタ
    - recentLogs(20) - 直近N件
- **フェーズ3: コンベア改善**
  - 細いベルト形状（0.6幅 x 0.2高）
  - CONVEYOR_BELT_WIDTH, CONVEYOR_BELT_HEIGHT定数追加
  - アイテム位置をベルト上面に調整
  - 設置向き自動判断（auto_conveyor_direction関数）
    - 隣接コンベアの向きを継続
    - 隣接機械から離れる方向
    - フォールバック: プレイヤーの向き
  - スプリッター/ジッパー用ステート（last_output_index, last_input_source）追加
- **フェーズ4: インベントリ改善**
  - 36スロット（ホットバー9 + メイン27）
  - HOTBAR_SLOTS, MAIN_INVENTORY_SLOTS, MAX_STACK_SIZE定数追加
  - 999スタック上限対応
  - move_itemsメソッド追加（スワップ/スタック対応）
  - is_hotbar_slot, is_main_slotヘルパー追加
- **テスト**: 47件成功、clippy警告なし
- **ツール追加**
  - discord-notify.sh: Discord Webhook通知スクリプト

### 2025-12-28
- **ESC後のカメラ操作無効化を修正**
  - CursorLockStateに`paused`フラグを追加
  - ESC押下時にpaused=true、クリック復帰時にpaused=false
  - player_look, player_move, block_break, block_place, update_target_blockでpausedチェック
  - "Click to Resume"表示中は全ての操作が無効になるように
- **レイキャスト精度をDDAアルゴリズムに改善**
  - 0.5ブロックステップの粗いレイキャストからDDA（Digital Differential Analyzer）に変更
  - ボクセル単位で正確に通過ブロックをチェック
  - 下のブロックを貫通して破壊できる問題を修正
  - update_target_block, block_break, block_placeの3箇所を更新
- **デバッグHUDにモード表示を追加**
  - F3でCreative/Survivalモードを表示
  - [PAUSED]表示を追加
- **クリエイティブモードUI改善**
  - Cキーでクリエイティブモード有効時、全9スロットに各アイテム64個を自動付与
  - クリエイティブモードではブロック設置時にインベントリを消費しない
  - F1-F9は現在のスロットのアイテムを変更するように変更
- **ターゲットブロックハイライト追加**
  - プレイヤーが見ているブロックに赤い半透明キューブを表示
  - 破壊対象が視覚的にわかるように
- **クリエイティブモード追加**
  - Cキーでトグル
  - F1-F10で各種アイテムを64個取得
  - Stone, Grass, IronOre, Coal, IronIngot, CopperOre, CopperIngot, Miner, Conveyor, Crusher
- **ESC処理の再修正**
  - ESCでUIを閉じた時はカーソルをアンロック（ブラウザの状態と一致）
  - Eキーでの閉じはロック維持
  - カメラ操作がロック状態に正しく連動
- **CHUNK_HEIGHTを8→32に拡大**
  - 地面の上にブロックを積めるように修正
  - GROUND_LEVEL定数を追加（Y=7）
  - 地下はY=0-6、地面はY=7、上空はY=8-31
- **Pointer Lockとカメラ操作を改善**
  - ESCでUIを閉じた後にポインターロックを再取得
  - JavaScript側で100ms遅延を入れてオーバーレイ表示を制御
  - RDPフォールバックを削除し、シンプルなdeltaクランプ方式に変更
  - MAX_DELTA=100.0で高速マウス移動時のジャンプを防止
- **E2Eテスト大幅拡充（30→52テスト）**
  - 機械コンポーネントテスト: Miner, Conveyor, Furnace, Crusher
  - エンティティクリーンアップテスト: コンベア破壊時のアイテム残留検出
  - クエスト・納品プラットフォームテスト
  - チャンク境界メッシュテスト
  - 自動化ライン統合テスト（Miner→Conv→Crusher→Conv→Furnace→Conv→Delivery）
  - レイキャスト全機械タイプテスト
- **CLAUDE.md「よくあるバグと対策」拡張**
  - 6つのバグパターンをドキュメント化
  - 実装時のチェックリスト追加
  - 過去の失敗から学習し、同じバグを再発させない仕組み
- **インベントリをスロットベースに完全リファクタリング**
  - HashMap方式から固定9スロット配列に変更
  - `add_item`, `consume_selected`, `consume_item`, `get_slot`, `get_slot_count`メソッド実装
  - スロットが空になっても自動補充されなくなった（バグ修正）
  - 空スロット選択時も位置を維持
- **E2Eテスト拡充（18テスト）**
  - スロットベースインベントリのテスト6件追加
  - `test_slot_inventory_add_stacks`
  - `test_slot_inventory_consume_selected`
  - `test_slot_inventory_empty_slot_stays_selected`
  - `test_slot_inventory_consume_specific_item`
  - `test_slot_inventory_full`
  - `test_slot_inventory_selection_with_empty_slots`
- **OGP設定追加**
  - Open Graph / Twitterカードメタタグ追加
  - ゲームスクリーンショットをOGP画像として使用（ogp.png）
  - ファビコン追加（⚙️絵文字SVG）
- **Pointer Lock API対応改善**
  - ESC後のマウス挙動問題を修正
  - オーバーレイ追加（「Click to Resume」表示）
  - pointerlockchange/pointerlockerrorイベント対応
  - 冗長なexitPointerLock()呼び出しを削除
- **エラーハンドリング改善**
  - unwrap()をis_none_or/let elseパターンに置換
  - レイキャスト判定（block_break, block_place, furnace_interaction）
  - 精錬炉/粉砕機処理（furnace_smelting, crusher_processing, crusher_output）
  - clippy警告ゼロ、テスト30件成功（ユニット12 + E2E18）
- **設定ファイル外出し見送り**
  - WASMではファイルシステムアクセスが制限される
  - 現状のconstants.rsで十分機能
- **クリエイティブインベントリUIを実装（マイクラ風）**
  - Cキーでクリエイティブモード → Eキーでインベントリ画面を開く
  - 全11種類のアイテムがグリッド表示（70x70pxボタン）
  - クリックで選択スロットに64個追加
  - ホバー/クリック時の視覚フィードバック
  - ESCで閉じる
- **精錬炉UIをスロットベース＆クリック操作に改善**
  - テキストベース → 3スロット（燃料/入力/出力）のクリックUI
  - プログレスバーで精錬進捗を視覚的に表示
  - キー操作（1,2,3,4キー）不要に
  - UIを開くとカーソルが解放される
- **粉砕機UIを新規追加（精錬炉と同様のスロットUI）**
  - InteractingCrusherリソース、CrusherUIコンポーネント追加
  - 入力/出力の2スロット（燃料なし）
  - Eキーで粉砕機を見てUIを開く
  - クリックで鉱石を追加/取り出し
  - crusher_interact, crusher_ui_input, update_crusher_uiシステム追加
- **40テスト成功、clippy警告ゼロ**

### 2025-12-27
- **ワインディング順序の修正（パフォーマンス向上）**
  - 全6面の頂点順序をCCW（反時計回り）に修正
  - `double_sided: true, cull_mode: None`を削除（4箇所）
  - バックフェースカリング有効化でGPU負荷半減
- **E2Eテスト拡充**
  - ホットバー選択テスト追加
  - ブロック設置によるインベントリ消費テスト追加
  - UI開閉テスト追加（Eキー/ESCキー）
  - フレーム安定性テスト追加（100フレームシミュレーション）
  - ブロック破壊の連続実行テスト追加
  - テスト総数: 11 → 23（ユニット11 + E2E12）
- **cargo clippy警告を全て解消**
  - 未使用メソッド（`index_to_pos`, `has_block_at`, `is_machine`, `can_add_input`）に`#[allow(dead_code)]`追加
  - Range::containsへのリファクタリング（6箇所）
  - `#[allow(clippy::too_many_arguments)]`追加（`player_look`, `block_break`）
- **3つのバグを修正**
  - ブロック設置時のフリーズ: block_placeのchunk再生成パターンをblock_breakと統一
  - 採掘機の視覚的フィードバック: miner_visual_feedbackシステム追加（パルスアニメーション）
  - デフォルト機械の破壊: block_breakにCrusher/Furnaceのレイキャスト追加
- **機械設置時の透ける問題を修正（再修正）** ✅確認済み
  - 原因: 機械設置時に`set_block(place_pos, BlockType::Stone)`で偽の石ブロックを登録していた
  - 修正: 機械（Miner, Conveyor, Crusher）設置時は`set_block`を呼ばない
  - 機械は独自エンティティなのでワールドデータ登録不要
- **block_placeのマテリアル設定を統一**
  - `StandardMaterial::default()`を使用していた箇所を修正
  - `double_sided: true, cull_mode: None`に統一
- **カメラ回転の滑らかさ改善** ✅確認済み
  - 原因: `MAX_REASONABLE_DELTA = 200.0`が低すぎて高速マウス移動がフィルタリングされていた
  - 修正: `MAX_REASONABLE_DELTA = 500.0`に変更
- **バグ修正3件**
  - 空スロット(8,9)を数字キーで選択可能に（選択解除として動作）
  - 機械設置時の地面透け問題を修正（WorldDataにブロック登録してチャンク再生成）
  - 採掘機を無限資源生成に変更（地面を掘らず、下のブロックタイプに応じて資源生成）
- **ビルド最適化（80コア環境）**
  - sccache導入でコンパイルキャッシュ
  - moldリンカーで高速リンク
  - codegen-units=16で並列コード生成
  - ネイティブ: 2分5秒→55秒、WASM: 2分13秒→7秒
- **ブロック掘削後の透ける問題を修正** ✅確認済み
  - ~~両面レンダリング（double_sided: true, cull_mode: None）で対処~~
  - ワインディング順序を修正し、バックフェースカリング再有効化済み（上記参照）
- **チャンク生成パフォーマンス改善**
  - ChunkData: HashMapから配列ベースの実装に変更
  - メッシュ生成: Y-Z-Xループ順でキャッシュ効率向上
  - 事前メモリ割り当て: Vec::with_capacity使用
  - VIEW_DISTANCE: ネイティブ2→3、WASM1→2に増加
  - フレームあたりのチャンク生成数: 2→4に増加
  - CHUNK_HEIGHT定数を追加（8固定）
- **WASM対応を準備**
  - Cargo.tomlにWASM用条件付き依存関係を追加
  - main.rsでPipelinedRenderingPluginを条件付きコンパイル
  - .cargo/config.tomlにgetrandom設定
  - web/index.html、build-wasm.shを作成
  - WASMビルド成功（wasm-bindgenはローカルで実行必要）

### 2025-12-26
- **クエストシステムを実装**
  - QuestDef: 目標アイテム、必要数、報酬を定義
  - CurrentQuestリソース: 進捗状態を管理
  - Quest 1: 鉄インゴット3個納品 → 採掘機×2, コンベア×20
  - Quest 2: 鉄インゴット100個納品 → 採掘機×2, コンベア×40
  - 画面上部中央にクエストUI表示
  - Q キーで報酬受け取り
  - 全15テスト成功
- **納品プラットフォームを実装**
  - DeliveryPlatformコンポーネント（delivered: HashMap<BlockType, u32>）
  - 12×12の緑色プラットフォーム（位置: 20,8,10）
  - 16個の黄色ポートマーカー（辺に4個ずつ）
  - コンベアからアイテム受け取りロジック
  - 右上に納品数UIを表示
  - 全15テスト成功
- **採掘機・コンベアの設置機能を実装**
  - BlockType: MinerBlock, ConveyorBlock追加
  - block_place: 機械設置に対応（Miner, Conveyorコンポーネント自動付与）
  - yaw_to_direction: プレイヤー視線→コンベア向き変換
  - 初期インベントリ: 採掘機×3, コンベア×10追加
  - 右クリックで向いている方向にコンベアを設置
- **採掘機・コンベアを実装**
  - Minerコンポーネント（position, progress, buffer）
  - Conveyorコンポーネント（position, direction, item, progress）
  - Direction列挙型（North, South, East, West）
  - miner_mining: 5秒ごとに下のブロックを採掘、バッファに保持
  - miner_output: 隣接コンベアにアイテム出力
  - conveyor_transfer: コンベアチェーンでアイテム移動、精錬炉に自動投入
  - デモ配置: Miner(5,8,15) → Conv(6,8,15) → Conv(7,8,15) → Furnace(8,8,15)
  - 全15テスト成功
- **精錬炉を実装**
  - BlockType: IronOre, Coal, IronIngot追加
  - Furnaceコンポーネント（fuel, input, output, progress）
  - 初期アイテム（鉄鉱石×5, 石炭×5）
  - Eキーで精錬炉UI表示（見ている精錬炉をターゲット）
  - 精錬ロジック（3秒で1個精錬、燃料+原料→鉄インゴット）
  - 進捗バー付きUI
- **複数チャンク対応を実装**
  - `WorldData`リソースで複数チャンクを管理
  - プレイヤー周囲5x5チャンク（VIEW_DISTANCE=2）を動的ロード/アンロード
  - 座標変換（ワールド↔チャンク↔ローカル）
  - テスト10件追加、全15件パス
- **run.sh修正**
  - パスを`/home/bacon/idle_factory`に修正
  - Xvfb自動検出を追加
  - `source ~/.cargo/env`を追加
- **libxkbcommon-x11-0インストール**（Bevy起動に必要）

## ユーザー確認待ちリスト

### 複数チャンク対応（2025-12-26）✅確認済み

### ブロック設置機能（2025-12-27）✅確認済み
- [x] 右クリックでブロック設置ができる
- [x] 設置時にインベントリから消費される
- [x] 既存の機械と同じ位置には設置できない

### ホットバーUI（2025-12-27）✅確認済み
- [x] 画面下部中央に9スロットのホットバーが表示される
- [x] 1-9キーでスロットを選択できる（空スロット含む）
- [x] 選択中のスロットがハイライトされる
- [x] 数量が表示される

### 精錬炉（2025-12-26）✅確認済み
- [x] 精錬炉を見てEキーでUIが開く
- [x] 精錬は正常に動作
- [x] ESCでUIを閉じた時にカーソルが正常に戻る

### 採掘機・コンベア設置（2025-12-27）✅確認済み
- [x] インベントリに採掘機×3、コンベア×10、粉砕機×2がある
- [x] 右クリックで採掘機を設置できる
- [x] 右クリックでコンベアを設置できる（向いている方向に向く）
- [x] 設置した採掘機が無限に資源を生成する（地面を掘らない）
- [x] 設置したコンベアがアイテムを運ぶ
- [x] 機械設置時に地面が透けない

### 納品プラットフォーム（2025-12-26）✅確認済み
- [x] 緑色の12×12プラットフォームが見える
- [x] 黄色のポートマーカーが見える
- [x] コンベアをプラットフォームに接続するとアイテムが納品される
- [x] 納品数がUIに反映される

### クエストシステム（2025-12-26）✅確認済み
- [x] 画面上部中央に「Quest」UIが表示されている
- [x] 鉄インゴットを納品すると進捗が更新される
- [x] 3個納品で報酬を受け取れる

### パフォーマンス改善（2025-12-27）✅確認済み
- [x] VIEW_DISTANCE=3（7x7=49チャンク）で遠くまで見える
- [x] チャンク読み込み時のカクつきが軽減された

### WASMプレイ確認 ✅確認済み
- **URL**: http://100.84.170.32:8080 (Tailscale)
- [x] ブラウザでゲームが起動する
- [x] 操作（WASD移動、マウス視点）が効く
- [x] FPSが30以上出る
- [x] ブロック採掘・設置ができる

**デプロイ手順**:
```bash
./deploy-wasm.sh  # ビルド＆サーバー再起動
```

## 参照ファイル

| ファイル | 用途 |
|----------|------|
| `API_REFERENCE.md` | 関数・構造体 |
| `.specify/specs/` | 仕様（実装対象） |
| `.specify/roadmap.md` | 将来機能（実装しない） |
| `.specify/memory/modeling-rules.md` | 3Dモデル |
| `.specify/memory/ui-design-rules.md` | UI実装時 |
