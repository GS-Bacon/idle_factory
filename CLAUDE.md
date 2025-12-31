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
| コミット | 日本語、短く、技術詳細より「何をしたか」重視。**コミットは自己判断でOK**、pushは指示待ち |
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

**例外: プレイテスト起因バグが1セッションで2件以上出た場合**
→ メタワーク要件を緩めて仕組み改善を実施（ビジュアルリグレッションテスト、操作シミュレーション等）

## マイルストーン駆動

```
仕様（1ページ）→ 実装 → 動かして確認 → 次へ
```

「done」= テストが通る、ではなく「遊べる」

## 新機能実装ルール（重要）

**原則**: 新機能は必ずユーザーと確認してから実装

### 確認が必要なケース
- 新しいゲーム要素（クエスト、機械、アイテム）
- UI/UXの変更
- 初期装備や報酬の変更
- ゲームバランスに影響する変更

### 確認不要なケース
- バグ修正
- リファクタリング（動作変更なし）
- テスト追加
- ドキュメント更新

### 確認フロー
1. **提案**: 機能の目的と具体的な内容を説明
2. **確認**: ユーザーからOKをもらう
3. **実装**: game_spec.rsに仕様を追加してから実装
4. **テスト**: 仕様通りかテストで検証

### 仕様の管理（Single Source of Truth）

| ファイル | 役割 |
|----------|------|
| `src/game_spec.rs` | **正** - 初期装備、クエスト定義 |
| `.specify/specs/first-30-minutes.md` | 参考 - 設計意図の説明 |

**変更手順**:
1. game_spec.rsを更新
2. テストで検証（`cargo test`）
3. 必要に応じてfirst-30-minutes.mdも更新

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

## v0.1 MVP定義

→ 詳細: `.specify/specs/mvp-v0.1.md`

現在は「クイックスタートモード」（機械を最初から所持、すぐに自動化を試せる）。

## 次の作業（優先度順）

**指示**: 「タスクリストを消化して」と言われたらP0から順に実行

### P0: バグ防止・安定性 ✅完了

- 入力ハンドラーの状態チェック統一（player_look, furnace_interact, crusher_interact, inventory_toggle）
- 未使用コード削除（allows_camera）
- pre-commit強化（cargo build + test + clippy）

### P1: v0.1 MVP ✅完了

- 初期装備: 現在のクイックスタートモード（Miner×3等）を維持、仕様コメント更新
- クエスト: game_spec.rsで仕様通り（Quest2=鉄インゴット100個）

### P2: v0.1 推奨 ✅完了

- チュートリアルポップアップ: 実装済み（起動時に操作説明表示）

### P3: メンテナンス ✅完了

- CLAUDE.md分割: 全ファイル作成済み
  - MVP定義 → .specify/specs/mvp-v0.1.md
  - 入力ルール → .specify/memory/input-rules.md
  - 作業ログ → .specify/memory/work-log.md
  - フェーズ計画 → .specify/roadmap.md
- InputStateテスト: test_input_state_priority, test_input_state_allows_methods追加済み
- main.rs: 7362行（継続的に分割中）

### P4: 後回し ✅完了

- ガイドマーカー: update_guide_markers実装済み
- 資源バイオーム: get_biome + test_biome_generation実装済み

### 修正済みバグ

| BUG | 内容 | 修正 |
|-----|------|------|
| BUG-1 | 納品プラットフォーム上にブロック置けない | レイキャスト判定追加 |
| BUG-2 | マウスが飛ぶ | skip_frames=1 |
| BUG-3 | コンベアアイテムちらつき | visual_entity引き継ぎ |
| BUG-4 | コンベアアイテム重なる | サイズ/間隔調整 |
| BUG-5 | 横合流アニメ不自然 | lateral_offset導入 |
| BUG-6 | 機械UIをESCで閉じるとポインターロック解除 | JS側でdata-ui-open監視、UI閉じ後に自動再ロック |
| BUG-7 | プレイ中に長時間フリーズ | チャンク処理を1フレーム2個に制限、隣接チャンク重複排除 |

### 修正待ちバグ

なし

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

| アイデア | メモ | 推奨タイミング |
|----------|------|----------------|
| Discord連携 | ゲーム専用チャンネルのログをIssue化、フィードバック自動収集 | 外部テスター5人以上 |
| CI/CD | mainプッシュ時にWASM自動ビルド・デプロイ | 手動公開が週3回以上 |
| コード生成 | YAML→BlockType/Quest自動生成。Mod対応にも有用 | アイテム50種 or Mod対応時 |

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

## 入力マトリクス

→ 詳細: `.specify/memory/input-rules.md`

入力系を変更するときは上記ファイルで全状態での動作を確認する。

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

### 10. インベントリ表示中にShiftで降下する
**原因**: player_move関数がInventoryOpenをチェックしていなかった
**症状**: インベントリを開いてShiftキーを押すとプレイヤーが降下する
**対策**: player_moveにInventoryOpen、InteractingCrusher、CommandInputStateチェックを追加
**ファイル**: `src/main.rs` の `player_move` 関数

### 11. クリックでアイテムが意図せず置き換わる
**原因**: HeldItemをカーソル位置に表示していなかった
**症状**: スロットクリックでアイテムが消えた/置き換わったように見える
**対策**: update_held_item_display関数でHeldItemをカーソル追従表示、インベントリ閉じる時にHeldItemをインベントリに返却
**ファイル**: `src/main.rs` の `update_held_item_display`, `inventory_toggle` 関数

### 12. インベントリ表示中にマウスホイールでホットバーが変わる
**原因**: select_block_type関数がInventoryOpenをチェックしていなかった
**症状**: インベントリUI内でスクロールするとホットバー選択が変わる
**対策**: select_block_typeにInventoryOpenチェックを追加
**ファイル**: `src/main.rs` の `select_block_type` 関数

### 13. インベントリUIの上にオーバーレイが表示される
**原因**: WASM側で`data-ui-open`属性の変更を監視していなかった
**症状**: Eキーでインベントリを開いても「Click to Resume」オーバーレイがUI上に被る
**対策**: MutationObserverで`data-ui-open`属性を監視し、trueになったら即座にオーバーレイを非表示
**ファイル**: `web/index.html` の`uiOpenObserver`

### 14. プレビューと実際の動作が異なる
**原因**: プレビュー表示と実際の処理で異なるロジックを使用
**症状**: コンベア設置ガイド（緑ワイヤーフレーム）と実際に設置される向きが違う
**対策**: プレビューと実際の処理で同じ関数を呼び出す（例: `auto_conveyor_direction`を両方で使用）
**ファイル**: `src/main.rs` の `update_target_highlight`, `block_place` 関数
**教訓**: 同じ計算を複数箇所で行う場合は関数に切り出し、全箇所で同じ関数を使う

### 15. 自動向き決定が分岐設置を妨げる
**原因**: auto_conveyor_directionで隣接コンベアの向きに合わせていた（Priority 3）
**症状**: コンベアの隣に分岐用コンベアを設置しても同じ向きになる
**対策**: Priority 3（隣接コンベア方向への自動合わせ）を削除し、プレイヤーの向きを使用
**ファイル**: `src/main.rs` の `auto_conveyor_direction` 関数
**教訓**: 自動化は便利だが、ユーザーの意図を上書きしないこと

### 16. 機械UI ESC後にゲームプレイに戻れない
**原因**: furnace_interact/crusher_interactでESC時にcursor_state.pausedを設定していなかった
**症状**: 精錬炉/粉砕機UIをESCで閉じた後、クリックしないとゲームプレイに戻れない
**対策**: ESCでUI閉じる時にcursor_state.paused = trueを設定し、「Click to Resume」オーバーレイを表示
**ファイル**: `src/main.rs` の `furnace_interact`, `crusher_interact` 関数
**教訓**: ESC処理は統一的に。handle_cursor_lockと各UI関数の連携を確認

### 17. UI要素のツールチップが表示されない
**原因**: ツールチップシステムが特定のコンポーネント（InventorySlotUI）のみを対象にしていた
**症状**: クリエイティブカタログのアイテムにホバーしてもツールチップが表示されない
**対策**: update_inventory_tooltipにCreativeItemButtonのクエリを追加
**ファイル**: `src/main.rs` の `update_inventory_tooltip` 関数
**教訓**: ツールチップ等の共通機能は、新しいUI要素を追加する度に対応が必要

### 18. ESCでUIを閉じた後にポインターロック解除（BUG-6）
**原因**: ブラウザはESCでポインターロックを自動解除するが、Rust側でpaused=trueにしてオーバーレイを表示していた
**症状**: 精錬炉/粉砕機UIをESCで閉じると「Click to Resume」が表示され、再クリック必要
**対策**: JS側で`data-ui-open`属性の`false`変更を監視し、50ms後に自動でポインターロック再取得
**ファイル**: `web/index.html` の `uiOpenObserver`、`src/main.rs` の各UI閉じ処理
**教訓**: WASMでのESC処理はブラウザの制約を考慮。paused状態を設定せず、JS側で自動復帰

### 19. チャンク生成/受信で長時間フリーズ（BUG-7）
**原因**: receive_chunk_meshesで複数チャンクが同時に完了すると、各チャンク+隣接4チャンクのメッシュ再生成がまとめて実行される
**症状**: FREEZE DETECTED: duration=16秒以上、FPS=0、delta clamping大量発生
**対策**:
1. 1フレームで処理するチャンク数を2個に制限（MAX_CHUNKS_PER_FRAME=2）
2. 同時にロードされたチャンクは隣接再生成をスキップ（自分で再生成するため）
**ファイル**: `src/main.rs` の `receive_chunk_meshes` 関数
**教訓**: O(n)の処理が毎フレーム実行される場合、nが増加するとフリーズする。バッチ処理に制限を設ける

### 実装時のチェックリスト

新機能追加時に確認:
- [ ] メッシュ生成を変更した → ワインディング順序テスト確認
- [ ] 機械を追加した → レイキャスト判定追加、破壊時クリーンアップ追加、**スロットアイテム返却**
- [ ] 子エンティティを持つ機械 → 破壊時に子もdespawn
- [ ] チャンク操作を変更した → 境界テスト確認
- [ ] ブロック操作を変更した → フリーズテスト確認
- [ ] モード専用UIを追加した → モードチェック、マーカーコンポーネント追加
- [ ] UIを追加した → `set_ui_open_state(true/false)` 呼び出し確認
- [ ] UI表示中に入力が効かないべき → player_move, select_block_typeでInventoryOpenチェック追加
- [ ] プレビュー/ガイドを表示する機能 → 実際の処理と同じロジックを使う
- [ ] ESCで閉じるUIを追加した → JS側で自動再ロックを確認（paused=true不要）
- [ ] ホバー可能なUI要素を追加した → update_inventory_tooltipに対応クエリ追加
- [ ] 毎フレーム実行される処理を追加した → バッチ処理に制限を設ける（フリーズ防止）

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

## 参照ファイル

| ファイル | 用途 |
|----------|------|
| `API_REFERENCE.md` | 関数・構造体 |
| `.specify/specs/` | 仕様（実装対象） |
| `.specify/roadmap.md` | 将来機能（実装しない） |
| `.specify/memory/modeling-rules.md` | 3Dモデル |
| `.specify/memory/ui-design-rules.md` | UI実装時 |
| `.specify/memory/input-rules.md` | 入力マトリクス |
| `.specify/memory/work-log.md` | 作業履歴 |
