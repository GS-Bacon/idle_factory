# AI自動評価・改善フィードバックループシステム

## 概要

AIが自動でゲームをプレイし、複数のペルソナ視点で評価・改善提案を行う回帰的改善システム。
コアコンセプト（creative-sandbox, no-combat, stress-free）を守りながら仕様変更も提案可能。

## コアコンセプト（変更不可）

constitution.mdより:
- **creative-sandbox**: 戦闘なし、敵なし、空腹なし、落下ダメージなし
- **stress-free**: プレイヤーを急かさない、ペナルティを与えない
- **data-driven**: YAML+Luaによる拡張性
- **player-empower**: プレイヤーに力を与える（制限しない）

これらに反する改善提案は自動却下される。

## アーキテクチャ

```
┌─────────────────────────────────────────────────────────────────┐
│                    AI Feedback Loop                             │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐         │
│  │  AI Player  │───▶│  Collector  │───▶│  Analyzer   │         │
│  │ (ペルソナ)   │    │ (ログ+目標) │    │ (LLM評価)   │         │
│  └─────────────┘    └─────────────┘    └─────────────┘         │
│         │                                      │                │
│         │                                      ▼                │
│         │         ┌─────────────┐      ┌─────────────┐         │
│         │         │  Executor   │◀─────│  Reporter   │         │
│         │         │ (自動実装)   │      │ (改善提案)  │         │
│         │         └─────────────┘      └─────────────┘         │
│         │                │                     │                │
│         ▼                ▼                     ▼                │
│  ┌─────────────┐  ┌─────────────┐      ┌─────────────┐         │
│  │ screenshots/│  │ 自動コミット │      │ feedback/   │         │
│  │ *.png, *.txt│  │ (軽微な修正) │      │ *.md        │         │
│  └─────────────┘  └─────────────┘      └─────────────┘         │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

## コンポーネント

### 1. AI Player (ペルソナベースプレイヤー)

**目標駆動型プレイ**: 各ペルソナは明確な目標を持ち、達成までの時間を測定。

#### ペルソナ定義

| ペルソナ | 特徴 | 目標例 | 測定項目 |
|---------|------|--------|----------|
| **Casual** | のんびり、探索好き | 最初の機械を設置 | 達成時間、迷い回数 |
| **Optimizer** | 効率厨、最適化マニア | 毎分100個生産ライン | スループット、試行回数 |
| **Explorer** | 全要素を触りたい | 全UIタブを開く | カバレッジ、発見順序 |
| **Critic** | 批判的、粗探し | バグ・矛盾を見つける | 問題発見数、再現手順 |
| **Speedrunner** | 最速クリア志向 | Tier2到達 | 所要時間、無駄操作 |
| **Builder** | 建築・見た目重視 | 綺麗な工場レイアウト | 対称性、整列度 |
| **Newbie** | 完全初心者 | チュートリアル完了 | 詰まり箇所、ヘルプ参照 |

#### 目標システム

```rust
pub struct PlayGoal {
    pub name: String,
    pub description: String,
    pub success_condition: GoalCondition,
    pub time_limit_secs: Option<f32>,  // 期待達成時間
    pub started_at: f32,
    pub completed_at: Option<f32>,
}

pub enum GoalCondition {
    /// アイテムを所持
    HasItem { item_id: String, count: u32 },
    /// 機械を設置
    PlacedMachine { machine_id: String, count: u32 },
    /// UIを開いた
    OpenedUI { ui_name: String },
    /// 生産レート達成
    ProductionRate { item_id: String, per_minute: f32 },
    /// カスタム条件（Lua）
    Custom { lua_expr: String },
}

pub struct GoalResult {
    pub goal: PlayGoal,
    pub achieved: bool,
    pub time_taken_secs: f32,
    pub expected_time_secs: f32,
    pub efficiency: f32,  // expected / actual (>1.0 = 早い)
    pub obstacles: Vec<String>,  // 達成を妨げた要因
}
```

#### ペルソナ別シナリオ

```rust
// 批判的プレイヤー: 粗を探す
"critic_play" => Persona {
    name: "Critic",
    goals: vec![
        Goal::find_ui_inconsistency(),
        Goal::find_missing_feedback(),
        Goal::find_confusing_flow(),
    ],
    behavior: Behavior {
        click_everything: true,
        read_all_text: true,
        try_edge_cases: true,
        report_nitpicks: true,
    },
}

// 効率厨: 最適化を試みる
"optimizer_play" => Persona {
    name: "Optimizer",
    goals: vec![
        Goal::production_rate("iron_ingot", 100.0),
        Goal::minimize_footprint(),
    ],
    behavior: Behavior {
        plan_before_action: true,
        measure_throughput: true,
        iterate_layout: true,
    },
}
```

### 2. Gameplay Collector (データ収集)

目標達成プロセスを詳細に記録。

**収集データ:**
```rust
pub struct PlaySession {
    // メタデータ
    pub session_id: String,
    pub timestamp: DateTime<Utc>,
    pub persona: String,
    pub duration_secs: f32,

    // 目標と結果
    pub goals: Vec<GoalResult>,
    pub overall_success_rate: f32,

    // プレイログ
    pub events: Vec<GameEvent>,
    pub screenshots: Vec<ScreenshotRef>,
    pub ui_dumps: Vec<UiDump>,

    // 統計
    pub stats: PlayStats,
}

pub struct PlayStats {
    pub total_actions: u32,
    pub failed_actions: u32,      // 失敗した操作
    pub confusion_moments: u32,   // 迷った瞬間（3秒以上停止）
    pub backtrack_count: u32,     // 戻った回数（試行錯誤）
    pub menu_open_count: u32,
    pub help_access_count: u32,   // ヘルプを見た回数
    pub rage_quit_score: f32,     // イライラ度（キー連打等）

    // 目標関連
    pub goal_efficiency: f32,     // 平均目標達成効率
    pub stuck_points: Vec<StuckPoint>,  // 詰まった箇所
}

pub struct StuckPoint {
    pub location: String,         // どこで
    pub duration_secs: f32,       // どれくらい
    pub attempted_actions: Vec<String>,  // 何を試したか
    pub resolution: Option<String>,      // どう解決したか
}

pub struct GameEvent {
    pub time: f32,
    pub event_type: EventType,
    pub context: String,
    pub success: bool,
    pub related_goal: Option<String>,  // どの目標に関連するか
}
```

### 3. AI Analyzer (LLM評価)

Claude Codeがプレイログを分析し、ペルソナ視点で評価。

**評価プロンプト:**
```markdown
# ゲームプレイ評価

あなたは{persona}タイプのゲーマーです。以下のプレイログを分析し、
そのペルソナ視点で評価してください。

## コアコンセプト（これに反する提案は却下）
- creative-sandbox: 戦闘なし、敵なし
- stress-free: プレイヤーを急かさない
- player-empower: 制限より自由

## プレイセッション
- ペルソナ: {persona}
- 時間: {duration}秒
- 目標達成率: {goal_success_rate}%

## 目標と結果
{goal_results}
- 目標A: {goal_a} → {time_a}秒で達成（期待: {expected_a}秒）
- 目標B: {goal_b} → 未達成（詰まり箇所: {stuck_point}）

## 詰まった箇所
{stuck_points}

## 統計
- 目標達成効率: {efficiency}%
- 失敗操作: {failed_actions}回
- 混乱: {confusion_moments}回

## 評価項目
1. **目標達成容易性**: 目標に到達しやすいか？
2. **直感性**: 操作は直感的か？
3. **フィードバック**: 操作結果は明確か？
4. **学習曲線**: 自然に学べるか？
5. **満足感**: 達成感はあるか？

## 出力形式
- 総合評価: S/A/B/C/D
- 目標達成分析: 各目標の達成しやすさ
- 良い点: 箇条書き
- 改善点: 箇条書き（優先度付き）
- 仕様変更提案: コアコンセプトに反しない範囲で
- 具体的な修正提案: 実装可能な形式で
```

**仕様変更提案の例:**
```markdown
## 仕様変更提案

### [提案] クイックスロット追加
- 現状: ホットバー9スロットのみ
- 問題: Optimizerペルソナが頻繁にインベントリを開く
- 提案: Shift+数字で第2ホットバーにアクセス
- コアコンセプト適合: ✅ player-empower（利便性向上）
- 実装コスト: 低（UI追加のみ）

### [却下例] 時間制限クエスト
- 提案: 制限時間内にアイテム収集
- コアコンセプト違反: ❌ stress-free に反する
- 判定: 自動却下
```

### 4. Feedback Reporter (改善提案)

分析結果をfeedback/ディレクトリに保存し、自動実装可能なものはExecutorに渡す。

**出力形式:**
```markdown
# フィードバック: {session_id}
生成日時: {timestamp}
ペルソナ: {persona}

## 総合評価: B

## 目標達成分析
| 目標 | 期待時間 | 実際 | 効率 | 判定 |
|------|----------|------|------|------|
| 最初の機械設置 | 60秒 | 45秒 | 133% | ✅ |
| インベントリを開く | 10秒 | 35秒 | 29% | ⚠️ 改善必要 |

## 良い点
- ホットバーの操作は直感的
- ブロック配置のフィードバックが明確

## 改善点
1. **[高]** インベントリの開き方がわかりにくい
   - 観察: Eキーに到達するまで35秒迷った
   - 詰まり箇所: 画面右下を何度もクリック
   - 提案: キーヒント「E: インベントリ」を表示
   - 自動実装: ✅ 可能（UIヒント追加）

2. **[中]** クラフトレシピの発見が難しい
   - 観察: クラフトタブを3回見逃した
   - 提案: 初回はパルスハイライト
   - 自動実装: ✅ 可能

## 仕様変更提案
### クイックアクセスホットキー
- 問題: Optimizerが頻繁にメニュー切替
- 提案: Tab=次のタブ、Shift+Tab=前のタブ
- コアコンセプト: ✅ player-empower
- 自動実装: ❌ 要検討

## 実装タスク（優先度順）
- [ ] [auto] キーヒントUI追加
- [ ] [auto] クラフトタブハイライト
- [ ] [manual] タブ切替ホットキー検討
```

### 5. Auto Executor (自動実装)

軽微な改善を自動実装してコミット。

**自動実装の条件:**
- UIテキスト・ヒントの追加/変更
- 既存パターンに沿った修正
- テストが通る
- コアコンセプトに違反しない

**自動実装フロー:**
```
1. 改善提案を受け取る
2. 実装可能か判定
3. コード変更を生成
4. cargo test 実行
5. 成功 → 自動コミット
6. 失敗 → 手動タスクとして記録
```

## ファイル構成

```
feedback/
├── sessions/
│   ├── 2025-12-25_casual_001.json    # セッションデータ
│   └── 2025-12-25_critic_002.json
├── reports/
│   ├── 2025-12-25_casual_001.md      # 評価レポート
│   └── 2025-12-25_critic_002.md
├── auto-implemented/
│   ├── 2025-12-25_hint_ui.md         # 自動実装履歴
│   └── 2025-12-25_highlight.md
├── pending/
│   └── spec-change-quickslot.md      # 要検討の仕様変更
├── trends.md                          # 傾向分析
└── summary.md                         # 累積サマリ
```

## スキル定義

### /evaluate スキル

```toml
[evaluate]
description = "AIがゲームをプレイして評価・改善提案を生成"
```

**使用方法:**
- `/evaluate` - Casualペルソナで評価
- `/evaluate critic` - 批判的視点で評価
- `/evaluate optimizer` - 効率厨視点で評価
- `/evaluate all` - 全ペルソナで順次評価
- `/evaluate fix` - pending/の改善提案を実装
- `/evaluate report` - 最新の評価レポートを表示
- `/evaluate trends` - 傾向分析を表示

### 評価サイクル

```
┌──────────────────────────────────────────────────────────────┐
│                      評価サイクル                            │
├──────────────────────────────────────────────────────────────┤
│                                                              │
│  1. /evaluate casual     → 初回評価（基本的なUX確認）        │
│                 ↓                                            │
│  2. 問題を自動修正       → Auto Executorが軽微な問題を修正  │
│                 ↓                                            │
│  3. /evaluate critic     → 批判的視点で残りの問題を発見     │
│                 ↓                                            │
│  4. /evaluate optimizer  → 効率性の問題を発見               │
│                 ↓                                            │
│  5. /evaluate fix        → pending/の提案を順次実装         │
│                 ↓                                            │
│  6. /evaluate all        → 全ペルソナで回帰テスト           │
│                                                              │
└──────────────────────────────────────────────────────────────┘
```

## 実装フェーズ

### Phase 1: 目標システム基盤
- [ ] PlayGoal / GoalCondition 構造体
- [ ] GoalResult 達成判定システム
- [ ] StuckPoint 検出ロジック

### Phase 2: ペルソナシステム
- [ ] Persona 定義（7種類）
- [ ] ペルソナ別行動パターン
- [ ] 目標達成時間の期待値設定

### Phase 3: データ収集拡張
- [ ] PlaySession拡張（目標結果含む）
- [ ] stuck_points 自動検出
- [ ] 効率メトリクス計算

### Phase 4: AI分析・評価
- [ ] ペルソナ別評価プロンプト
- [ ] コアコンセプト適合チェック
- [ ] 仕様変更提案生成

### Phase 5: 自動実装
- [ ] Auto Executor実装
- [ ] 安全な自動コミット
- [ ] 失敗時のロールバック

### Phase 6: 傾向分析
- [ ] trends.md 自動更新
- [ ] 回帰検出アラート
- [ ] 改善の可視化

## 成功指標

| 指標 | 目標値 | 測定方法 |
|------|--------|----------|
| 評価サイクル時間 | 5分以内 | ゲーム起動→レポート生成 |
| 目標達成率 | 80%以上 | 全ペルソナ平均 |
| 自動実装成功率 | 70%以上 | 自動修正/提案数 |
| 回帰検出率 | 100% | 悪化した項目の検出 |
| コアコンセプト違反 | 0件 | 自動却下システム |

## ペルソナの経験レベル定義

各ペルソナが「何を知っている前提」でプレイするかを3軸で定義。

### 経験軸

| 軸 | 説明 |
|----|------|
| **ゲーム経験** | ゲーム自体が初めてか |
| **ジャンル経験** | マイクラ・工場ゲー経験 |
| **本ゲーム経験** | このゲームのプレイ回数 |

### ペルソナ別経験マトリクス

| ペルソナ | ゲーム経験 | ジャンル経験 | 本ゲーム経験 | 想定行動 |
|---------|-----------|-------------|-------------|----------|
| **Newbie** | なし | なし | 初回 | WASDも知らない、全てが新しい |
| **Casual** | あり | なし | 初回 | 基本操作OK、ジャンル知識なし |
| **Gamer** | あり | あり | 初回 | マイクラ経験者、E=インベントリと推測 |
| **Optimizer** | あり | あり | 複数回 | 仕様を理解、効率を追求 |
| **Critic** | あり | あり | 複数回 | 他ゲームと比較、粗を探す |
| **Speedrunner** | あり | あり | 多数回 | 最適ルート熟知、限界を攻める |
| **Builder** | あり | あり | 複数回 | 見た目重視、レイアウト凝る |

### 経験による行動の違い

```rust
// Newbie: ゲーム経験なし
behavior.try_random_keys = true;      // 適当にキーを押す
behavior.read_all_tooltips = true;    // 全てのヒントを読む
behavior.slow_mouse_movement = true;  // マウス操作がぎこちない

// Gamer: ジャンル経験あり
behavior.try_common_keys = true;      // E, I, Tab, Escを最初に試す
behavior.expect_inventory = true;     // インベントリがあると想定
behavior.expect_crafting = true;      // クラフトがあると想定

// Optimizer: 本ゲーム経験あり
behavior.knows_all_recipes = true;    // レシピを暗記している
behavior.skip_tutorial = true;        // チュートリアルをスキップ
behavior.use_hotkeys = true;          // ショートカット多用
```

## 基準時間の定義

「基準時間」は**現実的に達成可能な時間**。
理想ではなく、実際のプレイヤーが達成できる現実的な目安。

### 基準時間の設定方法

| 目標タイプ | 基準の考え方 | 例 |
|-----------|-------------|-----|
| **UI発見** | 3回以内のトライで見つかる | インベントリを開く: 15秒 |
| **基本操作** | 説明なしで直感的にできる | ブロック配置: 5秒 |
| **最初の成果** | 達成感を感じるまで | 最初の機械設置: 60秒 |
| **ゲームループ理解** | 一連の流れを体験 | 採掘→加工→納品: 5分 |

### 効率の解釈

```
効率 = 基準時間 / 実際の時間 × 100%

効率 > 100%  → 想定より早い（良いUX、または簡単すぎ？）
効率 80-100% → 想定通り（理想）
効率 50-80%  → やや遅い（改善余地あり）
効率 < 50%   → 問題あり（詰まっている、UIがわかりにくい）
```

### ペルソナ別の基準時間調整

同じ目標でもペルソナによって基準時間を変える:

| 目標 | Newbie | Casual | Gamer |
|------|--------|--------|-------|
| インベントリを開く | 30秒 | 15秒 | 5秒 |
| 最初の機械設置 | 120秒 | 60秒 | 30秒 |
| Tier1クリア | 30分 | 15分 | 10分 |

## 累積学習

**採用**: 累積学習あり（毎回リセットしない）

### 理由

1. **現実のプレイヤー体験に近い**: 人間も学習する
2. **回帰テストに有効**: 「前は30秒だったのに60秒かかる」を検出
3. **チュートリアル評価**: 初見と2回目の差で学習効果を測定

### 学習の実装

```rust
pub struct PersonaMemory {
    /// 過去のセッション履歴
    pub sessions: Vec<SessionSummary>,
    /// 学習済みの操作（次回からスキップ可能）
    pub learned_actions: HashSet<String>,
    /// 発見済みのUI要素
    pub discovered_ui: HashSet<String>,
    /// 知っているレシピ
    pub known_recipes: HashSet<String>,
}

// 2回目以降は学習済みとして扱う
if persona.memory.learned_actions.contains("open_inventory") {
    // Eキーを直接押す（迷わない）
    behavior.try_key(KeyCode::KeyE);
} else {
    // 初見: 色々試す
    behavior.explore_ui();
}
```

### 初見評価モード

累積学習をリセットして初見体験を再評価:

```
/evaluate casual --fresh   # メモリをリセットして初見評価
/evaluate casual           # 累積学習ありで評価
```

## 人間フィードバック統合（将来）

### フェーズ1: ログ収集のみ
- 人間プレイ時も同じPlaySession形式で記録
- オプトイン（同意した場合のみ）

### フェーズ2: 比較分析
```markdown
## 人間 vs AIペルソナ比較

| 目標 | 人間平均 | AI(Gamer) | 差異 |
|------|---------|-----------|------|
| インベントリ開く | 8秒 | 5秒 | AI早すぎ→基準緩和 |
| 初クラフト | 45秒 | 30秒 | 妥当 |
| レシピ探索 | 120秒 | 60秒 | AI楽観的→問題見逃し |
```

### フェーズ3: 基準時間の自動調整
- 人間の実績から基準時間を更新
- AIペルソナの行動パターンを人間に近づける

## メタ評価（評価システム自体の改善）

評価システムが正しく機能しているかを検証し、継続的に改善する。

### メタ評価の観点

| 観点 | 質問 | 検証方法 |
|------|------|----------|
| **検出精度** | 問題を見逃していないか？ | 人間レビューとの差異 |
| **誤検出率** | 問題でないものを問題と判定していないか？ | 実装後の効果測定 |
| **提案品質** | 改善提案は実装可能か？ | 自動実装成功率 |
| **基準時間妥当性** | 基準時間は現実的か？ | 人間データとの乖離 |
| **ペルソナ精度** | ペルソナは実際のプレイヤーを反映しているか？ | 人間行動との類似度 |
| **トークン効率** | 無駄なトークンを消費していないか？ | 1サイクルあたりのトークン数 |
| **サイクル速度** | フィードバックループは速いか？ | 検出→実装→検証の所要時間 |

### 効率指標

評価システム自体のコストパフォーマンスを測定:

| 指標 | 目標 | 測定方法 |
|------|------|----------|
| **トークン/セッション** | 10K以下 | ゲーム起動→レポート生成 |
| **トークン/改善** | 5K以下 | 1件の改善実装に必要なトークン |
| **サイクル時間** | 10分以下 | 検出→実装→検証の完了まで |
| **自動化率** | 70%以上 | 人間介入なしで完了した割合 |
| **ROI** | 正 | (改善効果) / (消費トークン) |

### トークン効率化戦略

```rust
pub struct TokenOptimization {
    // レポート圧縮
    pub max_report_tokens: u32,        // レポート上限: 2000トークン
    pub compress_screenshots: bool,    // スクショはテキスト要約に変換
    pub incremental_analysis: bool,    // 差分のみ分析

    // キャッシュ活用
    pub cache_persona_memory: bool,    // ペルソナ記憶をキャッシュ
    pub cache_goal_definitions: bool,  // 目標定義をキャッシュ

    // 優先度フィルタ
    pub min_issue_severity: Severity,  // 軽微な問題はスキップ
    pub batch_similar_issues: bool,    // 類似問題をまとめて報告
}
```

### サイクル速度の最適化

```
高速サイクル（目標: 10分以内）

1. ゲーム起動 + 自動プレイ    [2分]
   └─ 並列: セッション記録

2. ログ分析 + 問題検出        [1分]
   └─ 圧縮レポート生成

3. 改善提案生成               [2分]
   └─ 自動実装可能か判定

4. 自動実装 + テスト          [3分]
   └─ cargo test並列実行

5. 効果検証                   [2分]
   └─ 再プレイで確認
```

### 効率メトリクス追跡

```markdown
# 効率レポート: 2025-12-25

## トークン消費
| 工程 | トークン数 | 目標 | 状態 |
|------|-----------|------|------|
| セッション記録 | 1,200 | - | - |
| ログ分析 | 3,500 | 5,000 | ✅ |
| 改善提案 | 2,800 | 3,000 | ✅ |
| 自動実装 | 4,200 | 5,000 | ✅ |
| **合計** | **11,700** | **15,000** | ✅ |

## サイクル時間
| 工程 | 時間 | 目標 | 状態 |
|------|------|------|------|
| 自動プレイ | 2:30 | 3:00 | ✅ |
| 分析 | 1:15 | 2:00 | ✅ |
| 実装 | 4:20 | 5:00 | ✅ |
| 検証 | 1:50 | 2:00 | ✅ |
| **合計** | **9:55** | **12:00** | ✅ |

## 改善履歴（効率追跡）
| 日付 | トークン/改善 | サイクル時間 | 傾向 |
|------|--------------|-------------|------|
| 12/20 | 8,500 | 18分 | - |
| 12/22 | 6,200 | 14分 | ↑ |
| 12/25 | 4,100 | 10分 | ↑ |
```

### メタ評価サイクル

```
┌────────────────────────────────────────────────────────────┐
│                   メタ評価サイクル                          │
├────────────────────────────────────────────────────────────┤
│                                                            │
│  1. 評価実行      → AI評価で問題Xを検出                    │
│         ↓                                                  │
│  2. 改善実装      → 問題Xを修正                            │
│         ↓                                                  │
│  3. 効果測定      → 再評価で改善を確認                     │
│         ↓                                                  │
│  4. メタ分析      → 「この検出は正しかったか？」           │
│         ↓                                                  │
│  4a. 正しかった   → 評価システムの信頼度UP                 │
│  4b. 誤検出      → 評価基準を修正                          │
│  4c. 見逃し      → 検出ロジックを強化                      │
│                                                            │
└────────────────────────────────────────────────────────────┘
```

### 自動調整項目

```rust
pub struct EvaluationConfig {
    // 基準時間（自動調整対象）
    pub base_times: HashMap<String, f32>,

    // 検出閾値（自動調整対象）
    pub stuck_threshold_secs: f32,      // 詰まり判定: 3秒 → 調整可能
    pub confusion_threshold_secs: f32,  // 混乱判定: 5秒 → 調整可能
    pub rage_key_threshold: u32,        // イライラ判定: 10連打 → 調整可能

    // ペルソナ行動パラメータ（自動調整対象）
    pub persona_params: HashMap<String, PersonaParams>,
}

pub struct MetaEvaluationResult {
    /// 検出が正しかった割合
    pub precision: f32,
    /// 問題を見逃さなかった割合
    pub recall: f32,
    /// 提案が実装成功した割合
    pub implementation_success_rate: f32,
    /// 基準時間の妥当性（人間との乖離）
    pub base_time_accuracy: f32,
    /// 調整提案
    pub adjustments: Vec<ConfigAdjustment>,
}
```

### フィードバック分類

実装後に各フィードバックを分類:

| 分類 | 説明 | 評価システムへの影響 |
|------|------|---------------------|
| **True Positive** | 検出→修正→効果あり | 信頼度UP |
| **False Positive** | 検出→修正→効果なし/悪化 | 検出基準を緩和 |
| **False Negative** | 未検出→人間が発見 | 検出ロジック強化 |
| **True Negative** | 問題なし→正しく無視 | 変更なし |

### メタ評価レポート

```markdown
# メタ評価レポート: 2025-12

## 評価システム性能
| 指標 | 今月 | 先月 | 傾向 |
|------|------|------|------|
| 検出精度 (Precision) | 85% | 80% | ↑ 改善 |
| 検出網羅率 (Recall) | 70% | 65% | ↑ 改善 |
| 自動実装成功率 | 72% | 68% | ↑ 改善 |
| 基準時間乖離 | 15% | 25% | ↑ 改善 |

## 誤検出の傾向
- Criticペルソナが細かすぎる問題を報告（5件）
  → 重要度フィルタを追加

## 見逃しの傾向
- 初心者向けツールチップ不足（3件）
  → Newbieペルソナの検出強化

## 調整実施
- stuck_threshold: 3秒 → 4秒（誤検出削減）
- Gamerペルソナの探索範囲を拡大
- 基準時間「インベントリを開く」: 15秒 → 12秒

## 次月の改善計画
- [ ] Criticの重要度フィルタ実装
- [ ] Newbieのツールチップ検出強化
```

### スキル拡張

```
/evaluate meta          # メタ評価レポートを表示
/evaluate calibrate     # 基準時間を人間データで再調整
/evaluate validate      # 過去の改善の効果を検証
```

### 継続的改善ループ

```
評価システムv1
    ↓ 問題検出
ゲーム改善
    ↓ 効果測定
メタ評価
    ↓ 評価システム調整
評価システムv2
    ↓ より精度の高い検出
...（繰り返し）
```

### 評価システム自体のバージョン管理

```
feedback/
├── config/
│   ├── evaluation_config_v1.yaml   # 評価設定履歴
│   ├── evaluation_config_v2.yaml
│   └── current.yaml → v2.yaml
├── meta/
│   ├── 2025-12_meta_report.md      # 月次メタ評価
│   └── adjustments_log.md          # 調整履歴
```
