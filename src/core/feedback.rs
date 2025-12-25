//! AI自動評価・改善フィードバックループシステム
//!
//! AIが自動でゲームをプレイし、複数のペルソナ視点で評価・改善提案を行う。
//! コアコンセプト（creative-sandbox, no-combat, stress-free）を守りながら改善を推進。

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use chrono::{DateTime, Utc};

/// フィードバックループプラグイン
pub struct FeedbackLoopPlugin;

impl Plugin for FeedbackLoopPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PersonaRegistry>()
            .init_resource::<GoalRegistry>()
            .init_resource::<PlaySessionRecorder>()
            .init_resource::<EvaluationConfig>()
            .init_resource::<MetaEvaluationState>()
            .add_event::<StartEvaluationEvent>()
            .add_event::<GoalCompletedEvent>()
            .add_event::<StuckDetectedEvent>()
            .add_systems(Update, (
                detect_stuck_points,
                check_goal_conditions,
                record_game_events,
                update_efficiency_metrics,
            ).chain());
    }
}

// ============================================================================
// Phase 1: 目標システム基盤
// ============================================================================

/// プレイ目標
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlayGoal {
    /// 目標ID
    pub id: String,
    /// 目標名
    pub name: String,
    /// 説明
    pub description: String,
    /// 成功条件
    pub success_condition: GoalCondition,
    /// 期待達成時間（秒）
    pub expected_time_secs: f32,
    /// 時間制限（秒）- オプション
    pub time_limit_secs: Option<f32>,
    /// 開始時刻
    #[serde(skip)]
    pub started_at: Option<f32>,
    /// 完了時刻
    #[serde(skip)]
    pub completed_at: Option<f32>,
}

/// 目標達成条件
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum GoalCondition {
    /// アイテムを所持
    HasItem { item_id: String, count: u32 },
    /// 機械を設置
    PlacedMachine { machine_id: String, count: u32 },
    /// UIを開いた
    OpenedUI { ui_name: String },
    /// 生産レート達成（アイテム/分）
    ProductionRate { item_id: String, per_minute: f32 },
    /// キーを押した
    PressedKey { key: String },
    /// 複合条件（AND）
    All(Vec<GoalCondition>),
    /// 複合条件（OR）
    Any(Vec<GoalCondition>),
    /// カスタム条件（Lua式）
    Custom { lua_expr: String },
}

/// 目標達成結果
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GoalResult {
    /// 目標
    pub goal: PlayGoal,
    /// 達成したか
    pub achieved: bool,
    /// 実際にかかった時間（秒）
    pub time_taken_secs: f32,
    /// 期待時間（秒）
    pub expected_time_secs: f32,
    /// 効率（expected / actual）- 1.0以上なら想定より早い
    pub efficiency: f32,
    /// 達成を妨げた要因
    pub obstacles: Vec<String>,
    /// 詰まった箇所
    pub stuck_points: Vec<StuckPoint>,
}

impl GoalResult {
    /// 効率を計算
    pub fn calculate_efficiency(&mut self) {
        if self.time_taken_secs > 0.0 {
            self.efficiency = self.expected_time_secs / self.time_taken_secs;
        } else {
            self.efficiency = 1.0;
        }
    }

    /// 評価を取得（S/A/B/C/D）
    pub fn get_rating(&self) -> char {
        if !self.achieved {
            return 'D';
        }
        match self.efficiency {
            e if e >= 1.2 => 'S',
            e if e >= 1.0 => 'A',
            e if e >= 0.8 => 'B',
            e if e >= 0.5 => 'C',
            _ => 'D',
        }
    }
}

/// 詰まった箇所
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StuckPoint {
    /// 場所（状態名やUI名）
    pub location: String,
    /// 詰まっていた時間（秒）
    pub duration_secs: f32,
    /// 試行したアクション
    pub attempted_actions: Vec<String>,
    /// 解決方法（None = 未解決）
    pub resolution: Option<String>,
    /// タイムスタンプ
    pub timestamp: f32,
}

// ============================================================================
// Phase 2: ペルソナシステム
// ============================================================================

/// ペルソナ定義
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Persona {
    /// ペルソナ名
    pub name: String,
    /// 説明
    pub description: String,
    /// 経験レベル
    pub experience: ExperienceLevel,
    /// 目標リスト
    pub goals: Vec<String>,
    /// 行動パラメータ
    pub behavior: BehaviorParams,
    /// 累積学習メモリ
    #[serde(default)]
    pub memory: PersonaMemory,
}

/// 経験レベル
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExperienceLevel {
    /// ゲーム経験全般
    pub gaming: ExperienceTier,
    /// ジャンル経験（マイクラ・工場ゲー）
    pub genre: ExperienceTier,
    /// 本ゲーム経験
    pub this_game: ExperienceTier,
}

/// 経験の度合い
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExperienceTier {
    None,
    Some,
    Expert,
}

/// 行動パラメータ
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct BehaviorParams {
    /// 全てをクリックして試す
    pub click_everything: bool,
    /// 全てのテキストを読む
    pub read_all_text: bool,
    /// エッジケースを試す
    pub try_edge_cases: bool,
    /// 細かい問題も報告
    pub report_nitpicks: bool,
    /// 行動前に計画
    pub plan_before_action: bool,
    /// スループットを測定
    pub measure_throughput: bool,
    /// レイアウトを反復改善
    pub iterate_layout: bool,
    /// ランダムキーを試す（完全初心者）
    pub try_random_keys: bool,
    /// 全ツールチップを読む
    pub read_all_tooltips: bool,
    /// マウス操作がぎこちない
    pub slow_mouse_movement: bool,
    /// 一般的なキーを試す（E, I, Tab, Esc）
    pub try_common_keys: bool,
    /// インベントリがあると想定
    pub expect_inventory: bool,
    /// クラフトがあると想定
    pub expect_crafting: bool,
    /// レシピを暗記している
    pub knows_all_recipes: bool,
    /// チュートリアルをスキップ
    pub skip_tutorial: bool,
    /// ショートカット多用
    pub use_hotkeys: bool,
    /// 基準時間調整係数（1.0 = 標準）
    pub time_multiplier: f32,
}

impl Default for Persona {
    fn default() -> Self {
        Self {
            name: "Default".to_string(),
            description: "Default persona".to_string(),
            experience: ExperienceLevel {
                gaming: ExperienceTier::Some,
                genre: ExperienceTier::Some,
                this_game: ExperienceTier::None,
            },
            goals: vec![],
            behavior: BehaviorParams {
                time_multiplier: 1.0,
                ..Default::default()
            },
            memory: PersonaMemory::default(),
        }
    }
}

/// ペルソナ累積学習メモリ
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct PersonaMemory {
    /// 過去のセッション履歴サマリ
    pub session_summaries: Vec<SessionSummary>,
    /// 学習済みの操作
    pub learned_actions: HashSet<String>,
    /// 発見済みのUI要素
    pub discovered_ui: HashSet<String>,
    /// 知っているレシピ
    pub known_recipes: HashSet<String>,
}

/// セッションサマリ
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SessionSummary {
    pub timestamp: DateTime<Utc>,
    pub duration_secs: f32,
    pub goals_achieved: u32,
    pub goals_failed: u32,
    pub average_efficiency: f32,
}

/// ペルソナレジストリ
#[derive(Resource)]
pub struct PersonaRegistry {
    pub personas: HashMap<String, Persona>,
}

impl Default for PersonaRegistry {
    fn default() -> Self {
        let mut personas = HashMap::new();

        // Newbie: 完全初心者
        personas.insert("newbie".to_string(), Persona {
            name: "Newbie".to_string(),
            description: "完全初心者。WASDも知らない、全てが新しい".to_string(),
            experience: ExperienceLevel {
                gaming: ExperienceTier::None,
                genre: ExperienceTier::None,
                this_game: ExperienceTier::None,
            },
            goals: vec![
                "open_inventory".to_string(),
                "place_first_block".to_string(),
                "complete_tutorial".to_string(),
            ],
            behavior: BehaviorParams {
                try_random_keys: true,
                read_all_tooltips: true,
                slow_mouse_movement: true,
                time_multiplier: 2.0,
                ..Default::default()
            },
            memory: PersonaMemory::default(),
        });

        // Casual: のんびりプレイヤー
        personas.insert("casual".to_string(), Persona {
            name: "Casual".to_string(),
            description: "のんびり、探索好き。基本操作OK、ジャンル知識なし".to_string(),
            experience: ExperienceLevel {
                gaming: ExperienceTier::Some,
                genre: ExperienceTier::None,
                this_game: ExperienceTier::None,
            },
            goals: vec![
                "open_inventory".to_string(),
                "place_first_machine".to_string(),
                "explore_all_tabs".to_string(),
            ],
            behavior: BehaviorParams {
                read_all_tooltips: true,
                time_multiplier: 1.5,
                ..Default::default()
            },
            memory: PersonaMemory::default(),
        });

        // Gamer: ジャンル経験者
        personas.insert("gamer".to_string(), Persona {
            name: "Gamer".to_string(),
            description: "マイクラ経験者。E=インベントリと推測".to_string(),
            experience: ExperienceLevel {
                gaming: ExperienceTier::Some,
                genre: ExperienceTier::Some,
                this_game: ExperienceTier::None,
            },
            goals: vec![
                "open_inventory".to_string(),
                "craft_first_item".to_string(),
                "setup_production_line".to_string(),
            ],
            behavior: BehaviorParams {
                try_common_keys: true,
                expect_inventory: true,
                expect_crafting: true,
                time_multiplier: 1.0,
                ..Default::default()
            },
            memory: PersonaMemory::default(),
        });

        // Optimizer: 効率厨
        personas.insert("optimizer".to_string(), Persona {
            name: "Optimizer".to_string(),
            description: "効率厨、最適化マニア。仕様を理解、効率を追求".to_string(),
            experience: ExperienceLevel {
                gaming: ExperienceTier::Some,
                genre: ExperienceTier::Expert,
                this_game: ExperienceTier::Some,
            },
            goals: vec![
                "production_rate_100".to_string(),
                "minimize_footprint".to_string(),
                "maximize_throughput".to_string(),
            ],
            behavior: BehaviorParams {
                plan_before_action: true,
                measure_throughput: true,
                iterate_layout: true,
                knows_all_recipes: true,
                use_hotkeys: true,
                time_multiplier: 0.8,
                ..Default::default()
            },
            memory: PersonaMemory::default(),
        });

        // Critic: 批判的プレイヤー
        personas.insert("critic".to_string(), Persona {
            name: "Critic".to_string(),
            description: "批判的、粗探し。他ゲームと比較".to_string(),
            experience: ExperienceLevel {
                gaming: ExperienceTier::Expert,
                genre: ExperienceTier::Expert,
                this_game: ExperienceTier::Some,
            },
            goals: vec![
                "find_ui_inconsistency".to_string(),
                "find_missing_feedback".to_string(),
                "find_confusing_flow".to_string(),
            ],
            behavior: BehaviorParams {
                click_everything: true,
                read_all_text: true,
                try_edge_cases: true,
                report_nitpicks: true,
                time_multiplier: 1.2,
                ..Default::default()
            },
            memory: PersonaMemory::default(),
        });

        // Speedrunner: 最速クリア志向
        personas.insert("speedrunner".to_string(), Persona {
            name: "Speedrunner".to_string(),
            description: "最速クリア志向。最適ルート熟知、限界を攻める".to_string(),
            experience: ExperienceLevel {
                gaming: ExperienceTier::Expert,
                genre: ExperienceTier::Expert,
                this_game: ExperienceTier::Expert,
            },
            goals: vec![
                "reach_tier2_fast".to_string(),
                "minimize_waste".to_string(),
                "optimize_route".to_string(),
            ],
            behavior: BehaviorParams {
                knows_all_recipes: true,
                skip_tutorial: true,
                use_hotkeys: true,
                time_multiplier: 0.5,
                ..Default::default()
            },
            memory: PersonaMemory::default(),
        });

        // Builder: 建築・見た目重視
        personas.insert("builder".to_string(), Persona {
            name: "Builder".to_string(),
            description: "建築・見た目重視。綺麗なレイアウトを目指す".to_string(),
            experience: ExperienceLevel {
                gaming: ExperienceTier::Some,
                genre: ExperienceTier::Some,
                this_game: ExperienceTier::Some,
            },
            goals: vec![
                "symmetric_layout".to_string(),
                "organized_factory".to_string(),
                "aesthetic_design".to_string(),
            ],
            behavior: BehaviorParams {
                iterate_layout: true,
                plan_before_action: true,
                time_multiplier: 1.5, // 見た目にこだわるので時間かかる
                ..Default::default()
            },
            memory: PersonaMemory::default(),
        });

        // Explorer: 全要素探索
        personas.insert("explorer".to_string(), Persona {
            name: "Explorer".to_string(),
            description: "全要素を触りたい。発見と探索を楽しむ".to_string(),
            experience: ExperienceLevel {
                gaming: ExperienceTier::Some,
                genre: ExperienceTier::Some,
                this_game: ExperienceTier::None,
            },
            goals: vec![
                "open_all_tabs".to_string(),
                "discover_all_machines".to_string(),
                "explore_map".to_string(),
            ],
            behavior: BehaviorParams {
                click_everything: true,
                read_all_text: true,
                time_multiplier: 1.3,
                ..Default::default()
            },
            memory: PersonaMemory::default(),
        });

        Self { personas }
    }
}

// ============================================================================
// Phase 3: データ収集システム
// ============================================================================

/// ゲームイベント
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GameEvent {
    /// 時刻（ゲーム開始からの秒数）
    pub time: f32,
    /// イベント種別
    pub event_type: GameEventType,
    /// コンテキスト
    pub context: String,
    /// 成功したか
    pub success: bool,
    /// 関連する目標ID
    pub related_goal: Option<String>,
}

/// ゲームイベント種別
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum GameEventType {
    KeyPress(String),
    KeyRelease(String),
    MouseClick { button: String, x: f32, y: f32 },
    UiOpen(String),
    UiClose(String),
    ItemPickup { item_id: String, count: u32 },
    MachinePlaced { machine_id: String },
    MachineRemoved { machine_id: String },
    CraftStarted { recipe_id: String },
    CraftCompleted { recipe_id: String },
    GoalStarted { goal_id: String },
    GoalCompleted { goal_id: String },
    GoalFailed { goal_id: String },
    StuckDetected { location: String },
    Custom(String),
}

/// プレイセッション
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlaySession {
    /// セッションID
    pub session_id: String,
    /// タイムスタンプ
    pub timestamp: DateTime<Utc>,
    /// ペルソナ名
    pub persona: String,
    /// プレイ時間（秒）
    pub duration_secs: f32,
    /// 目標結果
    pub goal_results: Vec<GoalResult>,
    /// 全体成功率
    pub overall_success_rate: f32,
    /// ゲームイベント
    pub events: Vec<GameEvent>,
    /// スクリーンショット参照
    pub screenshots: Vec<String>,
    /// UIダンプ参照
    pub ui_dumps: Vec<String>,
    /// 統計
    pub stats: PlayStats,
}

/// プレイ統計
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct PlayStats {
    /// 総アクション数
    pub total_actions: u32,
    /// 失敗したアクション数
    pub failed_actions: u32,
    /// 混乱した瞬間（3秒以上停止）
    pub confusion_moments: u32,
    /// 戻った回数（試行錯誤）
    pub backtrack_count: u32,
    /// メニューを開いた回数
    pub menu_open_count: u32,
    /// ヘルプを見た回数
    pub help_access_count: u32,
    /// イライラ度（キー連打等）
    pub rage_quit_score: f32,
    /// 平均目標達成効率
    pub goal_efficiency: f32,
    /// 詰まった箇所
    pub stuck_points: Vec<StuckPoint>,
}

/// プレイセッション記録器
#[derive(Resource, Default)]
pub struct PlaySessionRecorder {
    /// 現在のセッション
    pub current_session: Option<PlaySession>,
    /// 現在のペルソナ
    pub current_persona: Option<String>,
    /// アクティブな目標
    pub active_goals: Vec<PlayGoal>,
    /// 最後のアクション時刻
    pub last_action_time: f32,
    /// 現在の場所/状態
    pub current_location: String,
    /// 詰まり検出用: 同じ場所にいる時間
    pub stuck_timer: f32,
    /// 詰まり検出用: 試行したアクション
    pub stuck_actions: Vec<String>,
}

impl PlaySessionRecorder {
    /// 新しいセッションを開始
    pub fn start_session(&mut self, persona_name: &str) {
        let session_id = format!("{}_{}",
            chrono::Local::now().format("%Y%m%d_%H%M%S"),
            persona_name
        );

        self.current_session = Some(PlaySession {
            session_id,
            timestamp: Utc::now(),
            persona: persona_name.to_string(),
            duration_secs: 0.0,
            goal_results: vec![],
            overall_success_rate: 0.0,
            events: vec![],
            screenshots: vec![],
            ui_dumps: vec![],
            stats: PlayStats::default(),
        });

        self.current_persona = Some(persona_name.to_string());
        self.active_goals.clear();
        self.last_action_time = 0.0;
        self.current_location = "unknown".to_string();
        self.stuck_timer = 0.0;
        self.stuck_actions.clear();
    }

    /// イベントを記録
    pub fn record_event(&mut self, event: GameEvent) {
        if let Some(session) = &mut self.current_session {
            session.events.push(event.clone());
            session.stats.total_actions += 1;
            if !event.success {
                session.stats.failed_actions += 1;
            }
        }
        self.last_action_time = event.time;
        self.stuck_timer = 0.0; // アクションがあったのでリセット
    }

    /// セッションを終了して保存
    pub fn end_session(&mut self) -> Option<PlaySession> {
        if let Some(mut session) = self.current_session.take() {
            // 成功率を計算
            let total = session.goal_results.len();
            if total > 0 {
                let achieved = session.goal_results.iter().filter(|r| r.achieved).count();
                session.overall_success_rate = achieved as f32 / total as f32;

                // 効率を計算
                let efficiencies: Vec<f32> = session.goal_results.iter()
                    .filter(|r| r.achieved)
                    .map(|r| r.efficiency)
                    .collect();
                if !efficiencies.is_empty() {
                    session.stats.goal_efficiency =
                        efficiencies.iter().sum::<f32>() / efficiencies.len() as f32;
                }
            }

            // ファイルに保存
            let path = PathBuf::from("feedback/sessions")
                .join(format!("{}.json", session.session_id));
            if let Ok(json) = serde_json::to_string_pretty(&session) {
                let _ = std::fs::write(&path, json);
            }

            return Some(session);
        }
        None
    }
}

// ============================================================================
// Phase 4: AI分析・評価システム
// ============================================================================

/// 評価設定
#[derive(Resource, Clone, Debug, Serialize, Deserialize)]
pub struct EvaluationConfig {
    /// 基準時間（目標ID → 秒）
    pub base_times: HashMap<String, f32>,
    /// 詰まり判定閾値（秒）
    pub stuck_threshold_secs: f32,
    /// 混乱判定閾値（秒）
    pub confusion_threshold_secs: f32,
    /// イライラ判定閾値（連打回数）
    pub rage_key_threshold: u32,
    /// ペルソナ別パラメータ
    pub persona_params: HashMap<String, PersonaEvalParams>,
    /// コアコンセプトキーワード（違反検出用）
    pub core_concept_violations: Vec<String>,
    /// トークン最適化設定
    pub token_optimization: TokenOptimization,
}

/// ペルソナ別評価パラメータ
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct PersonaEvalParams {
    /// 基準時間調整係数
    pub time_multiplier: f32,
    /// 重要度フィルタ閾値
    pub min_severity: f32,
}

/// トークン最適化設定
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TokenOptimization {
    /// レポート上限トークン数
    pub max_report_tokens: u32,
    /// スクリーンショットをテキスト要約に変換
    pub compress_screenshots: bool,
    /// 差分のみ分析
    pub incremental_analysis: bool,
    /// ペルソナメモリをキャッシュ
    pub cache_persona_memory: bool,
    /// 目標定義をキャッシュ
    pub cache_goal_definitions: bool,
    /// 最小問題重要度（軽微はスキップ）
    pub min_issue_severity: f32,
    /// 類似問題をまとめて報告
    pub batch_similar_issues: bool,
}

impl Default for TokenOptimization {
    fn default() -> Self {
        Self {
            max_report_tokens: 2000,
            compress_screenshots: true,
            incremental_analysis: true,
            cache_persona_memory: true,
            cache_goal_definitions: true,
            min_issue_severity: 0.3,
            batch_similar_issues: true,
        }
    }
}

impl Default for EvaluationConfig {
    fn default() -> Self {
        let mut base_times = HashMap::new();
        // 基本的な目標の基準時間
        base_times.insert("open_inventory".to_string(), 15.0);
        base_times.insert("place_first_block".to_string(), 10.0);
        base_times.insert("place_first_machine".to_string(), 60.0);
        base_times.insert("craft_first_item".to_string(), 45.0);
        base_times.insert("complete_tutorial".to_string(), 300.0);
        base_times.insert("reach_tier2".to_string(), 900.0);

        Self {
            base_times,
            stuck_threshold_secs: 4.0,
            confusion_threshold_secs: 5.0,
            rage_key_threshold: 10,
            persona_params: HashMap::new(),
            core_concept_violations: vec![
                "time_limit".to_string(),
                "death".to_string(),
                "enemy".to_string(),
                "combat".to_string(),
                "hunger".to_string(),
                "penalty".to_string(),
                "rush".to_string(),
                "stress".to_string(),
            ],
            token_optimization: TokenOptimization::default(),
        }
    }
}

/// 評価レポート
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EvaluationReport {
    /// セッションID
    pub session_id: String,
    /// タイムスタンプ
    pub timestamp: DateTime<Utc>,
    /// ペルソナ
    pub persona: String,
    /// 総合評価（S/A/B/C/D）
    pub overall_rating: char,
    /// 目標達成分析
    pub goal_analysis: Vec<GoalAnalysis>,
    /// 良い点
    pub positives: Vec<String>,
    /// 改善点（優先度付き）
    pub improvements: Vec<ImprovementItem>,
    /// 仕様変更提案
    pub spec_change_proposals: Vec<SpecChangeProposal>,
    /// 実装タスク
    pub implementation_tasks: Vec<ImplementationTask>,
    /// 効率メトリクス
    pub efficiency_metrics: EfficiencyMetrics,
}

/// 目標分析
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GoalAnalysis {
    pub goal_name: String,
    pub expected_time_secs: f32,
    pub actual_time_secs: f32,
    pub efficiency_percent: f32,
    pub status: GoalStatus,
    pub obstacles: Vec<String>,
}

/// 目標ステータス
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum GoalStatus {
    Achieved,
    Failed,
    NeedsImprovement,
}

/// 改善項目
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ImprovementItem {
    /// 優先度（High/Medium/Low）
    pub priority: Priority,
    /// タイトル
    pub title: String,
    /// 観察内容
    pub observation: String,
    /// 提案
    pub suggestion: String,
    /// 自動実装可能か
    pub auto_implementable: bool,
}

/// 優先度
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Priority {
    High,
    Medium,
    Low,
}

/// 仕様変更提案
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SpecChangeProposal {
    /// タイトル
    pub title: String,
    /// 現状の問題
    pub current_problem: String,
    /// 提案内容
    pub proposal: String,
    /// コアコンセプト適合性
    pub core_concept_compliant: bool,
    /// 違反理由（不適合の場合）
    pub violation_reason: Option<String>,
    /// 実装コスト（Low/Medium/High）
    pub implementation_cost: String,
}

/// 実装タスク
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ImplementationTask {
    /// 自動実装可能か
    pub auto: bool,
    /// タスク説明
    pub description: String,
    /// 優先度
    pub priority: Priority,
    /// ステータス
    pub status: TaskStatus,
}

/// タスクステータス
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
}

/// 効率メトリクス
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct EfficiencyMetrics {
    /// トークン消費（工程別）
    pub token_usage: HashMap<String, u32>,
    /// 合計トークン
    pub total_tokens: u32,
    /// サイクル時間（工程別、秒）
    pub cycle_times: HashMap<String, f32>,
    /// 合計サイクル時間
    pub total_cycle_time_secs: f32,
    /// 自動化率
    pub automation_rate: f32,
}

// ============================================================================
// Phase 5: 自動実装システム
// ============================================================================

/// 自動実装可能な改善タイプ
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum AutoImplementableChange {
    /// UIテキスト追加
    AddUiText { element: String, text: String },
    /// キーヒント追加
    AddKeyHint { key: String, action: String },
    /// ツールチップ追加
    AddTooltip { element: String, tooltip: String },
    /// ハイライト追加
    AddHighlight { element: String, trigger: String },
    /// 設定値変更
    ChangeConfig { key: String, value: String },
}

/// 自動実装結果
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AutoImplementResult {
    /// 成功したか
    pub success: bool,
    /// 変更内容
    pub change: AutoImplementableChange,
    /// コミットハッシュ（成功時）
    pub commit_hash: Option<String>,
    /// エラーメッセージ（失敗時）
    pub error: Option<String>,
    /// テスト結果
    pub test_passed: bool,
}

// ============================================================================
// Phase 6: メタ評価・傾向分析
// ============================================================================

/// メタ評価状態
#[derive(Resource, Default)]
pub struct MetaEvaluationState {
    /// 精度（正しく検出した割合）
    pub precision: f32,
    /// 再現率（問題を見逃さなかった割合）
    pub recall: f32,
    /// 自動実装成功率
    pub implementation_success_rate: f32,
    /// 基準時間妥当性
    pub base_time_accuracy: f32,
    /// 調整履歴
    pub adjustments: Vec<ConfigAdjustment>,
    /// フィードバック分類
    pub feedback_classifications: Vec<FeedbackClassification>,
}

/// 設定調整
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConfigAdjustment {
    pub timestamp: DateTime<Utc>,
    pub config_key: String,
    pub old_value: String,
    pub new_value: String,
    pub reason: String,
}

/// フィードバック分類
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FeedbackClassification {
    pub improvement_id: String,
    pub classification: FeedbackType,
    pub verified_at: DateTime<Utc>,
}

/// フィードバックタイプ
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum FeedbackType {
    /// 正しく検出して効果あり
    TruePositive,
    /// 検出したが効果なし/悪化
    FalsePositive,
    /// 未検出で人間が発見
    FalseNegative,
    /// 問題なしを正しく無視
    TrueNegative,
}

/// 傾向レポート
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TrendReport {
    /// 期間
    pub period: String,
    /// 評価システム性能
    pub system_performance: SystemPerformance,
    /// 誤検出傾向
    pub false_positive_trends: Vec<String>,
    /// 見逃し傾向
    pub false_negative_trends: Vec<String>,
    /// 実施した調整
    pub adjustments_made: Vec<ConfigAdjustment>,
    /// 改善履歴
    pub improvement_history: Vec<ImprovementHistoryEntry>,
}

/// システム性能
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct SystemPerformance {
    pub precision: f32,
    pub recall: f32,
    pub implementation_success_rate: f32,
    pub base_time_deviation: f32,
    pub precision_trend: f32,  // 前期との差
    pub recall_trend: f32,
}

/// 改善履歴エントリ
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ImprovementHistoryEntry {
    pub date: String,
    pub tokens_per_improvement: u32,
    pub cycle_time_minutes: f32,
    pub trend: String, // "↑", "↓", "-"
}

// ============================================================================
// 目標レジストリ
// ============================================================================

/// 目標レジストリ
#[derive(Resource, Default)]
pub struct GoalRegistry {
    pub goals: HashMap<String, PlayGoal>,
}

impl GoalRegistry {
    /// デフォルト目標を登録
    pub fn register_defaults(&mut self) {
        // インベントリを開く
        self.goals.insert("open_inventory".to_string(), PlayGoal {
            id: "open_inventory".to_string(),
            name: "インベントリを開く".to_string(),
            description: "Eキーでインベントリを開く".to_string(),
            success_condition: GoalCondition::OpenedUI {
                ui_name: "inventory".to_string()
            },
            expected_time_secs: 15.0,
            time_limit_secs: None,
            started_at: None,
            completed_at: None,
        });

        // 最初のブロック設置
        self.goals.insert("place_first_block".to_string(), PlayGoal {
            id: "place_first_block".to_string(),
            name: "最初のブロック設置".to_string(),
            description: "ブロックを1つ設置する".to_string(),
            success_condition: GoalCondition::Custom {
                lua_expr: "placed_blocks >= 1".to_string()
            },
            expected_time_secs: 10.0,
            time_limit_secs: None,
            started_at: None,
            completed_at: None,
        });

        // 最初の機械設置
        self.goals.insert("place_first_machine".to_string(), PlayGoal {
            id: "place_first_machine".to_string(),
            name: "最初の機械設置".to_string(),
            description: "機械を1つ設置する".to_string(),
            success_condition: GoalCondition::PlacedMachine {
                machine_id: "*".to_string(),
                count: 1
            },
            expected_time_secs: 60.0,
            time_limit_secs: None,
            started_at: None,
            completed_at: None,
        });
    }
}

// ============================================================================
// イベント
// ============================================================================

/// 評価開始イベント
#[derive(Event)]
pub struct StartEvaluationEvent {
    pub persona: String,
    pub fresh: bool, // メモリをリセットするか
}

/// 目標完了イベント
#[derive(Event)]
pub struct GoalCompletedEvent {
    pub goal_id: String,
    pub time_taken_secs: f32,
}

/// 詰まり検出イベント
#[derive(Event)]
pub struct StuckDetectedEvent {
    pub location: String,
    pub duration_secs: f32,
    pub attempted_actions: Vec<String>,
}

// ============================================================================
// システム
// ============================================================================

/// 詰まり検出システム
fn detect_stuck_points(
    time: Res<Time>,
    mut recorder: ResMut<PlaySessionRecorder>,
    config: Res<EvaluationConfig>,
    mut stuck_events: EventWriter<StuckDetectedEvent>,
) {
    if recorder.current_session.is_none() {
        return;
    }

    recorder.stuck_timer += time.delta_secs();

    // 閾値を超えたら詰まりとして検出
    if recorder.stuck_timer >= config.stuck_threshold_secs {
        let stuck_point = StuckPoint {
            location: recorder.current_location.clone(),
            duration_secs: recorder.stuck_timer,
            attempted_actions: recorder.stuck_actions.clone(),
            resolution: None,
            timestamp: time.elapsed_secs(),
        };

        if let Some(session) = &mut recorder.current_session {
            session.stats.stuck_points.push(stuck_point.clone());
            session.stats.confusion_moments += 1;
        }

        stuck_events.send(StuckDetectedEvent {
            location: stuck_point.location,
            duration_secs: stuck_point.duration_secs,
            attempted_actions: stuck_point.attempted_actions,
        });

        // 検出後はタイマーをリセット（連続検出を防ぐ）
        recorder.stuck_timer = 0.0;
        recorder.stuck_actions.clear();
    }
}

/// 目標条件チェックシステム
fn check_goal_conditions(
    mut recorder: ResMut<PlaySessionRecorder>,
    time: Res<Time>,
    mut goal_events: EventWriter<GoalCompletedEvent>,
    // 必要に応じて他のクエリを追加
) {
    if recorder.current_session.is_none() {
        return;
    }

    let elapsed = time.elapsed_secs();
    let mut completed_goals = vec![];

    for goal in &mut recorder.active_goals {
        if goal.completed_at.is_some() {
            continue;
        }

        // 目標条件をチェック（簡略化版 - 実際にはより詳細なチェックが必要）
        let achieved = match &goal.success_condition {
            GoalCondition::OpenedUI { ui_name: _ } => {
                // UI状態をチェック
                false // 実際の実装では状態を確認
            }
            GoalCondition::PressedKey { key: _ } => {
                false // 実際の実装ではキー入力を確認
            }
            _ => false,
        };

        if achieved {
            let started = goal.started_at.unwrap_or(0.0);
            let time_taken = elapsed - started;
            goal.completed_at = Some(elapsed);

            completed_goals.push((goal.id.clone(), time_taken));
        }
    }

    for (goal_id, time_taken) in completed_goals {
        goal_events.send(GoalCompletedEvent {
            goal_id,
            time_taken_secs: time_taken,
        });
    }
}

/// ゲームイベント記録システム
fn record_game_events(
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    time: Res<Time>,
    mut recorder: ResMut<PlaySessionRecorder>,
) {
    if recorder.current_session.is_none() {
        return;
    }

    let elapsed = time.elapsed_secs();

    // キー入力を記録
    for key in keyboard.get_just_pressed() {
        let key_name = format!("{:?}", key);
        let context = recorder.current_location.clone();
        recorder.record_event(GameEvent {
            time: elapsed,
            event_type: GameEventType::KeyPress(key_name.clone()),
            context,
            success: true,
            related_goal: None,
        });
        recorder.stuck_actions.push(format!("press_{}", key_name));
    }

    // マウスクリックを記録
    for button in mouse.get_just_pressed() {
        let button_name = format!("{:?}", button);
        let context = recorder.current_location.clone();
        recorder.record_event(GameEvent {
            time: elapsed,
            event_type: GameEventType::MouseClick {
                button: button_name.clone(),
                x: 0.0, // 実際にはカーソル位置を取得
                y: 0.0,
            },
            context,
            success: true,
            related_goal: None,
        });
        recorder.stuck_actions.push(format!("click_{}", button_name));
    }
}

/// 効率メトリクス更新システム
fn update_efficiency_metrics(
    time: Res<Time>,
    mut recorder: ResMut<PlaySessionRecorder>,
) {
    if let Some(session) = &mut recorder.current_session {
        session.duration_secs = time.elapsed_secs();
    }
}

// ============================================================================
// ユーティリティ
// ============================================================================

/// コアコンセプト違反チェック
pub fn check_core_concept_violation(proposal: &str, config: &EvaluationConfig) -> Option<String> {
    let proposal_lower = proposal.to_lowercase();
    for keyword in &config.core_concept_violations {
        if proposal_lower.contains(keyword) {
            return Some(format!(
                "提案が「{}」を含んでおり、コアコンセプトに違反する可能性があります",
                keyword
            ));
        }
    }
    None
}

/// レポートをMarkdown形式で出力
impl EvaluationReport {
    pub fn to_markdown(&self) -> String {
        let mut md = String::new();

        md.push_str(&format!("# フィードバック: {}\n", self.session_id));
        md.push_str(&format!("生成日時: {}\n", self.timestamp.format("%Y-%m-%d %H:%M:%S")));
        md.push_str(&format!("ペルソナ: {}\n\n", self.persona));

        md.push_str(&format!("## 総合評価: {}\n\n", self.overall_rating));

        // 目標達成分析
        md.push_str("## 目標達成分析\n");
        md.push_str("| 目標 | 期待時間 | 実際 | 効率 | 判定 |\n");
        md.push_str("|------|----------|------|------|------|\n");
        for analysis in &self.goal_analysis {
            let status_icon = match analysis.status {
                GoalStatus::Achieved => "✅",
                GoalStatus::Failed => "❌",
                GoalStatus::NeedsImprovement => "⚠️",
            };
            md.push_str(&format!(
                "| {} | {:.0}秒 | {:.0}秒 | {:.0}% | {} |\n",
                analysis.goal_name,
                analysis.expected_time_secs,
                analysis.actual_time_secs,
                analysis.efficiency_percent,
                status_icon
            ));
        }
        md.push('\n');

        // 良い点
        if !self.positives.is_empty() {
            md.push_str("## 良い点\n");
            for positive in &self.positives {
                md.push_str(&format!("- {}\n", positive));
            }
            md.push('\n');
        }

        // 改善点
        if !self.improvements.is_empty() {
            md.push_str("## 改善点\n");
            for (i, item) in self.improvements.iter().enumerate() {
                let priority = match item.priority {
                    Priority::High => "[高]",
                    Priority::Medium => "[中]",
                    Priority::Low => "[低]",
                };
                let auto = if item.auto_implementable { "✅ 可能" } else { "❌ 手動" };
                md.push_str(&format!(
                    "{}. **{}** {}\n   - 観察: {}\n   - 提案: {}\n   - 自動実装: {}\n\n",
                    i + 1, priority, item.title, item.observation, item.suggestion, auto
                ));
            }
        }

        // 仕様変更提案
        if !self.spec_change_proposals.is_empty() {
            md.push_str("## 仕様変更提案\n");
            for proposal in &self.spec_change_proposals {
                let compliant = if proposal.core_concept_compliant { "✅" } else { "❌" };
                md.push_str(&format!("### {}\n", proposal.title));
                md.push_str(&format!("- 問題: {}\n", proposal.current_problem));
                md.push_str(&format!("- 提案: {}\n", proposal.proposal));
                md.push_str(&format!("- コアコンセプト: {}\n", compliant));
                if let Some(reason) = &proposal.violation_reason {
                    md.push_str(&format!("- 違反理由: {}\n", reason));
                }
                md.push_str(&format!("- 実装コスト: {}\n\n", proposal.implementation_cost));
            }
        }

        // 実装タスク
        if !self.implementation_tasks.is_empty() {
            md.push_str("## 実装タスク（優先度順）\n");
            for task in &self.implementation_tasks {
                let auto = if task.auto { "[auto]" } else { "[manual]" };
                let status = match task.status {
                    TaskStatus::Pending => "[ ]",
                    TaskStatus::InProgress => "[~]",
                    TaskStatus::Completed => "[x]",
                    TaskStatus::Failed => "[!]",
                };
                md.push_str(&format!("- {} {} {}\n", status, auto, task.description));
            }
        }

        md
    }

    /// レポートをファイルに保存
    pub fn save(&self) -> std::io::Result<PathBuf> {
        let path = PathBuf::from("feedback/reports")
            .join(format!("{}.md", self.session_id));
        std::fs::write(&path, self.to_markdown())?;
        Ok(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_goal_result_efficiency() {
        let mut result = GoalResult {
            goal: PlayGoal {
                id: "test".to_string(),
                name: "Test".to_string(),
                description: "".to_string(),
                success_condition: GoalCondition::PressedKey { key: "E".to_string() },
                expected_time_secs: 10.0,
                time_limit_secs: None,
                started_at: None,
                completed_at: None,
            },
            achieved: true,
            time_taken_secs: 8.0,
            expected_time_secs: 10.0,
            efficiency: 0.0,
            obstacles: vec![],
            stuck_points: vec![],
        };

        result.calculate_efficiency();
        assert!((result.efficiency - 1.25).abs() < 0.01);
        assert_eq!(result.get_rating(), 'S');
    }

    #[test]
    fn test_goal_result_rating() {
        let make_result = |achieved: bool, efficiency: f32| -> GoalResult {
            GoalResult {
                goal: PlayGoal {
                    id: "test".to_string(),
                    name: "Test".to_string(),
                    description: "".to_string(),
                    success_condition: GoalCondition::PressedKey { key: "E".to_string() },
                    expected_time_secs: 10.0,
                    time_limit_secs: None,
                    started_at: None,
                    completed_at: None,
                },
                achieved,
                time_taken_secs: 10.0 / efficiency,
                expected_time_secs: 10.0,
                efficiency,
                obstacles: vec![],
                stuck_points: vec![],
            }
        };

        assert_eq!(make_result(true, 1.5).get_rating(), 'S');
        assert_eq!(make_result(true, 1.0).get_rating(), 'A');
        assert_eq!(make_result(true, 0.9).get_rating(), 'B');
        assert_eq!(make_result(true, 0.6).get_rating(), 'C');
        assert_eq!(make_result(true, 0.3).get_rating(), 'D');
        assert_eq!(make_result(false, 1.0).get_rating(), 'D');
    }

    #[test]
    fn test_core_concept_violation() {
        let config = EvaluationConfig::default();

        // 違反あり（英語キーワード）
        let result = check_core_concept_violation("Add time_limit quest", &config);
        assert!(result.is_some());

        let result = check_core_concept_violation("Add enemy spawner", &config);
        assert!(result.is_some());

        let result = check_core_concept_violation("Combat system", &config);
        assert!(result.is_some());

        // 違反なし
        let result = check_core_concept_violation("UIヒント追加", &config);
        assert!(result.is_none());

        let result = check_core_concept_violation("Add tooltip for inventory", &config);
        assert!(result.is_none());
    }

    #[test]
    fn test_persona_registry_defaults() {
        let registry = PersonaRegistry::default();

        assert!(registry.personas.contains_key("newbie"));
        assert!(registry.personas.contains_key("casual"));
        assert!(registry.personas.contains_key("gamer"));
        assert!(registry.personas.contains_key("optimizer"));
        assert!(registry.personas.contains_key("critic"));
        assert!(registry.personas.contains_key("speedrunner"));
        assert!(registry.personas.contains_key("builder"));
        assert!(registry.personas.contains_key("explorer"));

        // Newbieの時間調整係数が2.0であることを確認
        let newbie = registry.personas.get("newbie").unwrap();
        assert!((newbie.behavior.time_multiplier - 2.0).abs() < 0.01);
    }
}
