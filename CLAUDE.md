# Claude Code メモリ

## 用語定義（混同注意）

| 用語 | 意味 | コード上の対応 |
|------|------|---------------|
| **収納** | コンベア経由でプラットフォーム（倉庫）にアイテムが入ること | `TransferTarget::Delivery` |
| **納品** | クエストのためにアイテムを消費すること | `quest_deliver_button` |

※ユーザーが混同していそうな場合は確認すること

## 互換性ポリシー

- **既存プレイヤーはいない** → セーブデータ移行は不要
- **しばらくは後方互換性を気にしなくてよい**
- 破壊的変更OK、古いセーブ切り捨てOK

## メタワーク判定基準（重要）

### メタワーク（やらない）

| 例 | 理由 |
|----|------|
| 仕様書を詳細化する | ゲームが1行も進まない |
| ドキュメント整理・圧縮 | 誰も読まない |
| ドキュメントコメント追加 | 誰も読まない |
| レビュースキル・パターン集作成 | 使わない道具 |
| コード移動だけのリファクタ | バグ確率変わらない |

### メタワークではない（やる）

| 例 | 理由 |
|----|------|
| 自動テスト追加 | バグを早期発見、開発速度UP |
| E2E強化 | 遊べないバグを防ぐ |
| CI/起動確認スクリプト | クラッシュを即検出 |
| バグ修正 | 遊べるようにする |
| リファクタ（複雑さ解消） | 将来のバグ確率を減らす |
| アーキテクチャ改善 | 機能追加時のバグ確率を減らす |
| 重複コード統合 | 片方だけ直して忘れるバグを防ぐ |
| 機能追加 | ゲームが進む |
| UI/UX改善 | 体験が良くなる |
| パフォーマンス改善 | 遊べる体験に直結 |

### 判定フロー

```
これをやると...
  → ゲームが遊べるようになる？ → やる
  → 今のバグが減る？ → やる
  → 将来のバグ確率が減る？ → やる
  → 機能追加が速くなる？ → やる
  → ドキュメントが増えるだけ？ → やらない
```

**過去の失敗**: 仕様39ファイル・パターン52個・レビュースキル群を作り、誰もゲームを遊んでいなかった

## 基本ルール

| ルール | 詳細 |
|--------|------|
| ビルド | `cargo build` (2秒)、テスト後 `cargo test && cargo clippy` |
| コミット | 日本語、短く。**コミットは自己判断でOK**、pushは指示待ち |
| 中断禁止 | 確認を求めず最後まで完了 |
| ゲーム起動 | `./run.sh` |
| バグ修正 | **再現テストなしの修正禁止**。下記参照 |
| **タスク実行** | **必ず `./scripts/parallel-run.sh` を使う**。下記参照 |

## タスク実行ルール（必須）

**全てのタスクは git worktree + Task tool で並列実行する**

### ⚠️ 容量チェック（必須）

並列実行前に**必ず**容量チェックを行う:

```bash
# 実行予定のworktree数で容量チェック
./scripts/parallel-run.sh check <数>

# 例: 2タスク並列なら
./scripts/parallel-run.sh check 2
```

- **容量不足**の場合: `./scripts/parallel-run.sh cleanup` で放置worktreeを削除
- **start時も自動チェック**: 容量不足なら開始を拒否

### ワークフロー

```bash
# 0. 容量チェック（必須）
./scripts/parallel-run.sh check <並列数>

# 1. タスクを parallel-tasks.json に登録
./scripts/parallel-run.sh add

# 2. 並列実行可能なタスクを確認
./scripts/parallel-run.sh list

# 3. 複数タスクを同時に start（worktree作成）
./scripts/parallel-run.sh start task-a
./scripts/parallel-run.sh start task-b

# 4. Task tool で並列に作業（サブエージェント）
# ※ 必ず複数の Task tool を同時に呼び出す

# 5. 各worktreeで作業完了後、finish（masterにマージ）
./scripts/parallel-run.sh finish task-a
./scripts/parallel-run.sh finish task-b
```

### 並列実行の原則

| 条件 | 判断 |
|------|------|
| 異なるファイル | **並列可** |
| 同じファイル | **並列可**（worktreeは独立、マージ時に解決） |
| 依存関係あり | 順次実行 |

### 禁止事項

- タスクを1つずつ順番に実行（並列可能なのに直列でやる）
- worktreeを使わずにmasterで直接作業
- Task toolを1つずつ呼び出す（並列呼び出しすること）
- **worktreeの放置**（finish後は即削除される。セッション終了前に必ずfinish or abort）
- 容量チェックなしでの並列実行開始

### マージコンフリクト時

```bash
git checkout --theirs Cargo.lock Cargo.toml
git add Cargo.lock Cargo.toml
git commit -m "Merge branch 'xxx'"
```

## バグ修正ルール

1. **再現テストを先に書く**（またはE2Eで再現確認）
2. テストが失敗することを確認
3. 修正する
4. テストがパスすることを確認
5. 関連テストも全部パスすることを確認

**禁止**: 「多分これが原因」で修正開始。必ずログかテストで原因特定してから。

## 禁止事項

- 仕様だけのセッション2回連続
- 憶測での修正（ログ/スクリーンショットで確認してから）
- ドキュメント整理・圧縮作業

## 新機能実装

**確認が必要**: 新ゲーム要素、UI/UX変更、ゲームバランス変更
**確認不要**: バグ修正、リファクタリング、テスト追加

仕様は `src/game_spec.rs` に定義（Single Source of Truth）

## 検証の分担

| 担当 | 内容 |
|------|------|
| AI | ビルド、テスト、E2E、スクリーンショット確認、UI動作 |
| 人間 | ゲームプレイ体験（面白さ、直感性） |

## 開発フロー

```
ユーザー: 修正 → プッシュ → AI: プル → 作業 → プッシュ → ユーザー: 確認
```

## Gemini連携

**使用条件**: Gemini 3/2.5モデルの時のみ使用（トークン制限でモデル切り替わったら使わない）

### Claude vs Gemini 使い分け

| 観点 | Claude | Gemini |
|------|--------|--------|
| **強み** | コード詳細分析、安全性、リファクタリング | 俯瞰的アーキテクチャ、ツール評価 |
| **視点** | 「進捗」を評価 | 「現状の課題」を厳しく指摘 |
| **具体性** | 行番号付きの具体的指摘 | 理想形のコード例を提示 |
| **速度** | 即座に回答 | ファイル読み込みに時間がかかる |

### タスク別の使い分け

| タスク | 使うAI | 理由 |
|--------|--------|------|
| 新機能のアーキテクチャ設計 | **Gemini** | 俯瞰的な設計が得意 |
| 実装コード | **Claude** | 詳細なコーディング |
| コードレビュー | **両方** | 比較して精度向上 |
| バグ修正 | **Claude** | ログ・詳細分析 |
| リファクタリング計画 | **Gemini** | 全体像の把握 |
| リファクタリング実行 | **Claude** | コード変更 |

### Geminiが得意なこと

| 用途 | 例 |
|------|-----|
| アーキテクチャ設計 | Plugin構成、モジュール分割の提案 |
| ベストプラクティス | Bevy公式パターンとの比較 |
| 俯瞰レビュー | 12,000行全体の構造評価 |
| CI/CDツール評価 | スクリプト、自動化の評価 |
| 空間・ベクトル計算の検証 | 回転行列、座標変換、方向計算 |
| 数学的な正しさの確認 | 三角関数、線形代数 |

### Claudeが得意なこと

| 用途 | 例 |
|------|-----|
| unwrap()安全化 | パターンマッチングの提案 |
| リファクタリング差分 | 変更前後の比較 |
| バグ特定 | ログやスタックトレースからの推論 |
| コードの細部 | 命名、型、エラーハンドリング |

### 使い方

```bash
# コードレビュー
gemini review src/conveyor.rs

# アーキテクチャ評価（デフォルト: src/*.rs）
gemini arch

# 数学・座標計算の検証
gemini math "Vec3::NEG_Z を Y軸で90度回転させると？"

# バグ分析
gemini bug "コンベアが逆方向に動く" src/conveyor.rs

# git diffをレビュー
gemini diff

# 自由質問（従来通り）
gemini ask "質問" src/main.rs
```

### コマンド一覧

| コマンド | 用途 | 例 |
|----------|------|-----|
| `review` | コードレビュー | `gemini review src/*.rs` |
| `arch` | アーキテクチャ評価 | `gemini arch` |
| `math` | 数学・座標計算 | `gemini math "回転行列"` |
| `bug` | バグ分析 | `gemini bug "症状" file.rs` |
| `diff` | git diffレビュー | `gemini diff` |
| `ask` | 自由質問 | `gemini ask "質問" files...` |

### セットアップ

```bash
# PATHに追加（オプション）
ln -sf /home/bacon/idle_factory/scripts/gemini ~/.local/bin/gemini
```

### 連携パターン

| パターン | 手順 |
|----------|------|
| **設計レビュー** | Gemini→設計案→Claude→実装 |
| **バグ修正** | Claude→修正→Gemini→妥当性確認 |
| **リファクタリング** | Gemini→計画→Claude→実行→Gemini→レビュー |
| **数学計算** | Claude→コード→Gemini→計算検証 |

### ベストプラクティス

1. **大きな設計変更前**: Geminiに俯瞰レビューを依頼
2. **バグが取れない時**: Geminiにセカンドオピニオン
3. **座標・回転計算**: Geminiで数学的正しさを確認

### 注意点

- ファイル読み込みには30-60秒かかることがある
- 大きなファイル（1000行超）は自動でタイムアウト延長（+60秒）
- タイムアウト時は自動リトライ（最大2回、リトライ毎に+60秒）
- デフォルトタイムアウト: 180秒（`GEMINI_TIMEOUT`で変更可）
- リトライ回数: 2回（`GEMINI_RETRIES`で変更可）

## VLMビジュアルチェック

Claude Vision APIでスクリーンショットの視覚バグを検出。

### 使い方

```bash
# クイックチェック（起動→スクショ→解析）
./scripts/vlm_check.sh --quick

# 詳細チェック
./scripts/vlm_check.sh --thorough screenshot.png

# コンベア専用チェック
./scripts/vlm_check.sh --conveyor

# リリース前フルスイート
./scripts/vlm_check.sh --full-suite
```

### 推奨タイミング

| タイミング | コマンド |
|-----------|---------|
| テクスチャ/モデル変更後 | `--thorough` |
| コンベアロジック変更後 | `--conveyor` |
| UI変更後 | `--ui` |
| チャンク/地形変更後 | `--chunk` |
| **リリース前（必須）** | `--full-suite` |

### E2Eテストとの違い

| 観点 | E2Eテスト | VLMチェック |
|------|----------|-------------|
| 検証対象 | データ・ロジック | 視覚的表示 |
| テクスチャ抜け | ❌ | ✅ |
| モデル不良 | ❌ | ✅ |
| アイテム移動 | ✅ | ❌ |
| 速度 | 0.02秒 | 5-30秒 |
| コスト | 無料 | API課金 |

詳細: `scripts/vlm_check/README.md`

## 並列タスク実行

git worktreeを使って複数タスクを並列実行。

### 使い方

```bash
# タスク一覧
./scripts/parallel-run.sh list

# タスク開始（worktree作成）
./scripts/parallel-run.sh start <task-id>

# 実行中タスクの状態
./scripts/parallel-run.sh status

# タスク完了（masterにマージ）
./scripts/parallel-run.sh finish <task-id>

# タスク中止
./scripts/parallel-run.sh abort <task-id>

# 新規タスク追加（対話式）
./scripts/parallel-run.sh add
```

### 並列グループ

| グループ | 説明 | 最大同時 |
|----------|------|----------|
| ui | UIコンポーネント | 2 |
| logistics | 物流システム（conveyor等） | 1 |
| machines | 機械システム | 2 |
| core | コアシステム（単独推奨） | 1 |

### 衝突検知

- 同じファイルパターンを触るタスクは警告
- 依存タスクが未完了なら開始不可

### タスク定義

`.claude/parallel-tasks.json` に定義：

```json
{
  "id": "fix-ui-bug",
  "name": "UIバグ修正",
  "parallel_group": "ui",
  "depends_on": [],
  "branch": "fix/ui-bug",
  "files": ["src/ui/*.rs"],
  "status": "pending"
}
```

## 参照ドキュメント

| ファイル | 内容 |
|----------|------|
| `.claude/implementation-plan.md` | **統合実装計画（タスク一覧）** |
| `.claude/architecture.md` | モジュール構成、依存関係、分割ルール |
| `.claude/coding-rules.md` | コーディング規約、命名規則 |
| `.claude/bugs.md` | よくあるバグと対策、チェックリスト |
| `.claude/build.md` | ビルド最適化、プロファイル設定 |
| `.specify/memory/input-rules.md` | 入力マトリクス |
| `.specify/memory/modeling-rules.md` | 3Dモデル作成ルール |
| `scripts/gemini` | Gemini連携（メイン） |
| `scripts/ask_gemini.sh` | Gemini自由質問（低レベル） |
| `scripts/vlm_check.sh` | VLMビジュアルチェック |
| `scripts/vlm_check/README.md` | VLMチェック詳細ドキュメント |
| `scripts/parallel-run.sh` | 並列タスク実行管理 |
| `.claude/parallel-tasks.json` | 並列タスク定義 |

## 修正済みバグ

BUG-1〜9: 全て修正済み。詳細は `.claude/bugs.md` 参照。

## 現在の状態

| 項目 | 値 |
|------|-----|
| コード行数 | **11,500行** (目標12,500行以下 ✅) |
| テスト | **280件** 通過 (lib:74, bin:38, e2e:146, fuzz:11, proptest:8, ssim:3) |
| unwrap() | **17箇所** (全てテストコード内) |
| Clippy警告 | **0件** |
| カバレッジ | **8.54%** 全体、ロジック部分70%+ |

## タスク

**詳細は `.claude/implementation-plan.md` 参照**

### 優先順位

| 順位 | カテゴリ | 状態 |
|------|----------|------|
| 1 | パフォーマンス改善 | Phase 1 |
| 2 | セキュリティ・エラー処理 | Phase 2 |
| 3 | v0.2機能実装 | Phase 3 |
| 4 | テスト強化 | Phase 4 |
| 5 | コードダイエット | Phase 5 |

### 次のアクション

1. **Phase 1**: highlight.rs + conveyor.rs パフォーマンス改善
2. **Phase 2**: unwrap()削除
3. **Phase 3**: v0.2機能（UI改修、クエスト拡張、機械入出力）

### 将来機能 (v0.3以降)

- リプレイシステム
- ブループリント
- 電力システム
- 流体パイプ
- マルチプレイ基盤
- Modding API
- ビジュアルプログラミング
