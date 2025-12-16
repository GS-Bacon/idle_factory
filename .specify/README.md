# SpecKit: Spec-Driven Development for Infinite Voxel Factory

このディレクトリには、GitHub SpecKitを使用したSpec-Driven Development（仕様駆動開発）のための設定とテンプレートが含まれています。

## 📁 ディレクトリ構造

```
.specify/
├── memory/
│   └── constitution.md    # プロジェクトの憲章（原則、標準、ガイドライン）
├── templates/
│   ├── spec-template.md   # 機能仕様書のテンプレート
│   ├── plan-template.md   # 実装計画のテンプレート
│   └── tasks-template.md  # タスクリストのテンプレート
└── specs/                 # 生成された仕様書（機能ごと）
```

## 🚀 使い方

### 前提条件

- AIエージェント（Claude、GitHub Copilot、Cursorなど）
- このプロジェクトで利用可能なスラッシュコマンド

### ワークフロー

#### 1. 憲章の確認（初回のみ）

```
/speckit.constitution
```

プロジェクトの原則、コーディング標準、アーキテクチャガイドラインを確認します。
すべての仕様、計画、実装はこの憲章に従います。

#### 2. 機能の仕様化

新しい機能を追加する場合：

```
/speckit.specify
```

AIエージェントと対話しながら、以下を定義します：
- **何を作るか** (What): 機能の目的と範囲
- **なぜ作るか** (Why): ビジネス価値とユーザーメリット
- **成功基準** (Success Criteria): 測定可能な目標

結果: `.specify/specs/feature-name.md` に仕様書が生成されます。

#### 3. 実装計画の作成

```
/speckit.plan
```

仕様書を元に、技術的な実装戦略を作成します：
- アーキテクチャの選択
- データモデルの設計
- 依存関係の特定
- 複雑性の管理

結果: `.specify/specs/feature-name-plan.md` に計画が生成されます。

#### 4. タスクの分割

```
/speckit.tasks
```

計画を実行可能なタスクに分割します：
- セットアップタスク
- 基盤タスク（他のタスクのブロッカー）
- ユーザーストーリーごとのタスク（優先度付き）
- 仕上げタスク

結果: `.specify/specs/feature-name-tasks.md` にタスクリストが生成されます。

#### 5. 実装の実行

```
/speckit.implement
```

タスクリストに従って、AIエージェントが実装を支援します。
各タスクは独立してテスト・デプロイ可能です。

### オプションのコマンド

- `/speckit.clarify` - 仕様の曖昧な部分を明確化
- `/speckit.analyze` - 既存コードの分析と改善提案

## 📝 例: 新機能の追加

例えば、「ジェットパック機能」を追加する場合：

1. `/speckit.specify` でジェットパックの仕様を定義
   - 燃料消費の仕組み
   - 上昇速度と制御
   - UIの表示

2. `/speckit.plan` で実装計画を作成
   - `JetpackComponent` の設計
   - 燃料管理システムの統合
   - 入力処理の拡張

3. `/speckit.tasks` でタスクに分割
   - [T1] JetpackComponentの実装
   - [T2] 燃料消費ロジックの追加
   - [T3] UIの更新
   - [T4] テストの作成

4. `/speckit.implement` で実装開始

## 🎯 ベストプラクティス

### 仕様書作成時
- **測定可能な成功基準** を設定する
- **エッジケース** を考慮する
- **[NEEDS CLARIFICATION]** タグで不明点を明示する

### 計画作成時
- **憲章との整合性** をチェックする（Constitution Check）
- **複雑性の正当化** を記録する
- **段階的な実装** を心がける（Phase 0, 1, 2）

### タスク実行時
- **独立したテスト** を書く
- **並列実行可能** なタスクは `[P]` マークを付ける
- **優先度** に従って実装する（P1 → P2 → P3）

## 🔧 このプロジェクト固有のガイドライン

### アーキテクチャ
- ECS（Entity Component System）アーキテクチャを使用
- データ駆動設計（YAML定義）
- 決定性シミュレーション

### テスト
- 最小カバレッジ: 70%（コアシステムは90%+）
- TDD（テストファーストデベロップメント）推奨
- 統合テストでシステム間の連携を検証

### パフォーマンス
- 目標FPS: 60
- インスタンシング描画（1000+アイテム）
- 非同期チャンク読み込み

## 📚 参考リンク

- [GitHub SpecKit 公式リポジトリ](https://github.com/github/spec-kit)
- [Spec-Driven Development ガイド](https://github.com/github/spec-kit/blob/main/spec-driven.md)
- [プロジェクト憲章](./memory/constitution.md)

---

*このドキュメントはプロジェクトの進化とともに更新されます。*
