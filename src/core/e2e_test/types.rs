//! E2Eテストシステムの型定義

use bevy::prelude::*;
use std::path::PathBuf;
use std::io::Write;

/// UI要素キャッシュ（座標検索用）
#[derive(Resource, Default)]
pub struct UiElementCache {
    /// 要素情報（名前、テキスト、画面座標）
    pub elements: Vec<CachedUiElement>,
    /// 最終更新時刻
    pub last_update: f32,
}

/// キャッシュされたUI要素
#[derive(Clone)]
pub struct CachedUiElement {
    pub entity: Entity,
    pub name: Option<String>,
    pub text: Option<String>,
    pub screen_rect: Rect,
}

/// UI要素クリックイベント
#[derive(Event)]
pub struct ClickElementEvent {
    pub selector: ElementSelector,
    pub button: MouseButton,
    pub double_click: bool,
}

/// スクロールイベント
#[derive(Event)]
pub struct ScrollEvent {
    pub delta: f32,
}

/// 自動テスト設定
#[derive(Resource)]
pub struct AutoTestConfig {
    pub enabled: bool,
}

/// 人間らしい挙動の設定
#[derive(Resource)]
pub struct HumanBehaviorConfig {
    /// 人間らしい挙動を有効にするか
    pub enabled: bool,
    /// 最小思考時間（秒）
    pub min_think_time: f32,
    /// 最大思考時間（秒）
    pub max_think_time: f32,
    /// タイピング間隔（秒）
    pub typing_interval: f32,
    /// タイピング間隔のばらつき（0.0-1.0）
    pub typing_variance: f32,
    /// マウス移動の揺れ強度
    pub mouse_jitter: f32,
    /// マウス移動のステップ数
    pub mouse_move_steps: u32,
    /// ミスクリック確率（0.0-1.0）
    pub misclick_chance: f32,
}

impl Default for HumanBehaviorConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            min_think_time: 0.1,
            max_think_time: 0.5,
            typing_interval: 0.08,
            typing_variance: 0.3,
            mouse_jitter: 3.0,
            mouse_move_steps: 10,
            misclick_chance: 0.0, // デフォルトではミスクリックなし
        }
    }
}

/// E2Eテスト状態
#[derive(Resource, Default)]
pub struct E2ETestState {
    /// テストモードが有効か
    pub is_test_mode: bool,
    /// 現在実行中のシナリオ
    pub current_scenario: Option<String>,
    /// 現在のステップ
    pub current_step: usize,
    /// 次のステップまでの待機時間（秒）
    pub wait_timer: f32,
    /// スクリーンショットカウンター
    pub screenshot_counter: u32,
    /// UIダンプカウンター
    pub dump_counter: u32,
    /// マウスアニメーション状態
    pub mouse_animation: Option<MouseAnimationState>,
    /// タイピングアニメーション状態
    pub typing_animation: Option<TypingAnimationState>,
}

/// マウスアニメーション状態
#[derive(Clone)]
pub struct MouseAnimationState {
    pub start_pos: Vec2,
    pub end_pos: Vec2,
    pub current_step: u32,
    pub total_steps: u32,
    pub control_point: Vec2, // ベジェ曲線の制御点
}

/// タイピングアニメーション状態
#[derive(Clone)]
pub struct TypingAnimationState {
    pub text: String,
    pub current_index: usize,
    pub next_char_timer: f32,
}

/// テストレポート（検証結果を蓄積）
#[derive(Resource, Default)]
pub struct TestReport {
    /// 検証結果のリスト
    pub results: Vec<VerificationResult>,
    /// テスト開始時刻
    pub start_time: Option<std::time::Instant>,
}

/// 検証結果
#[derive(Clone)]
pub struct VerificationResult {
    /// 検証名
    pub name: String,
    /// 成功したか
    pub passed: bool,
    /// 詳細メッセージ
    pub message: String,
    /// 期待値
    pub expected: String,
    /// 実際の値
    pub actual: String,
}

impl TestReport {
    /// レポートをファイルに保存
    pub fn save_to_file(&self, path: &PathBuf) -> std::io::Result<()> {
        let mut file = std::fs::File::create(path)?;

        writeln!(file, "# E2E Test Report")?;
        writeln!(file, "Generated: {}", chrono::Local::now().format("%Y-%m-%d %H:%M:%S"))?;
        writeln!(file)?;

        let passed = self.results.iter().filter(|r| r.passed).count();
        let failed = self.results.iter().filter(|r| !r.passed).count();
        let total = self.results.len();

        writeln!(file, "## Summary")?;
        writeln!(file, "- Total: {}", total)?;
        writeln!(file, "- Passed: {} ✅", passed)?;
        writeln!(file, "- Failed: {} ❌", failed)?;
        writeln!(file)?;

        if failed > 0 {
            writeln!(file, "## Failed Tests")?;
            for result in self.results.iter().filter(|r| !r.passed) {
                writeln!(file, "### ❌ {}", result.name)?;
                writeln!(file, "- Message: {}", result.message)?;
                writeln!(file, "- Expected: {}", result.expected)?;
                writeln!(file, "- Actual: {}", result.actual)?;
                writeln!(file)?;
            }
        }

        writeln!(file, "## All Results")?;
        for result in &self.results {
            let icon = if result.passed { "✅" } else { "❌" };
            writeln!(file, "- {} {}: {}", icon, result.name, result.message)?;
        }

        Ok(())
    }

    /// レポートをクリア
    pub fn clear(&mut self) {
        self.results.clear();
        self.start_time = Some(std::time::Instant::now());
    }

    /// 検証結果を追加
    pub fn add_result(&mut self, result: VerificationResult) {
        let icon = if result.passed { "✅" } else { "❌" };
        info!("[E2E] {} {}: {}", icon, result.name, result.message);
        self.results.push(result);
    }
}

/// テストシナリオキュー
#[derive(Resource, Default)]
pub struct TestScenarioQueue {
    pub scenarios: Vec<TestScenario>,
    pub current_index: usize,
}

/// テストシナリオ
#[derive(Clone)]
pub struct TestScenario {
    pub name: String,
    pub steps: Vec<TestStep>,
}

/// テストステップ
#[derive(Clone)]
pub enum TestStep {
    /// 指定時間待機（秒）
    Wait(f32),
    /// ランダムな待機時間（最小秒, 最大秒）- 人間らしい思考時間
    WaitRandom(f32, f32),
    /// キー入力シミュレーション
    PressKey(KeyCode),
    /// キー解放シミュレーション
    ReleaseKey(KeyCode),
    /// キーを押して離す
    TapKey(KeyCode),
    /// 人間らしいキータップ（ランダムな押下時間）
    TapKeyHuman(KeyCode),
    /// マウスボタン押下
    MousePress(MouseButton),
    /// マウスボタン解放
    MouseRelease(MouseButton),
    /// マウス移動（画面座標）- 瞬間移動
    MouseMove(f32, f32),
    /// 人間らしいマウス移動（ベジェ曲線で滑らかに移動）
    MouseMoveSmooth(f32, f32),
    /// マウスクリック（Press + Wait + Release）
    MouseClick(MouseButton),
    /// 人間らしいマウスクリック（揺れ + クリック）
    MouseClickHuman(MouseButton),
    /// ダブルクリック
    DoubleClick(MouseButton),
    /// マウスホイールスクロール（正: 上, 負: 下）
    Scroll(f32),
    /// UI要素をクリック（名前またはテキストで検索）
    ClickElement(ElementSelector),
    /// UI要素にマウスを移動
    HoverElement(ElementSelector),
    /// UI要素をダブルクリック
    DoubleClickElement(ElementSelector),
    /// UI要素を右クリック
    RightClickElement(ElementSelector),
    /// ドラッグ&ドロップ（開始要素 → 終了要素）
    DragDropElements(ElementSelector, ElementSelector),
    /// ドラッグ&ドロップ（座標）
    DragDrop(f32, f32, f32, f32),
    /// スクリーンショット撮影（トークン消費大）
    Screenshot(String),
    /// ログ出力
    Log(String),
    /// カスタムアクション（名前）
    Custom(String),
    /// アプリ状態を直接変更（テスト用）
    SetAppState(String),
    /// UIダンプ（テキスト、トークン消費小）
    DumpUi(String),
    /// UI検証（要素存在チェック）
    VerifyElement(UiVerification),
    /// テストレポート保存
    SaveReport,
    /// テストレポートクリア
    ClearReport,
    /// テキスト入力（コマンド入力用）- 瞬間入力
    TypeText(String),
    /// 人間らしいテキスト入力（1文字ずつ）
    TypeTextHuman(String),
    /// コマンド実行（/を開いてテキスト入力してEnter）
    ExecuteCommand(String),
    /// 人間らしい思考時間を追加
    Think,
    /// 複数のステップをグループ化（人間らしい間隔で実行）
    HumanSequence(Vec<TestStep>),
}

/// UI要素セレクター
#[derive(Clone)]
pub struct ElementSelector {
    /// 要素名（部分一致）
    pub name: Option<String>,
    /// テキスト内容（部分一致）
    pub text: Option<String>,
    /// インデックス（同名の要素が複数ある場合）
    pub index: usize,
}

impl ElementSelector {
    /// 名前で要素を選択
    pub fn by_name(name: &str) -> Self {
        Self {
            name: Some(name.to_string()),
            text: None,
            index: 0,
        }
    }

    /// テキストで要素を選択
    pub fn by_text(text: &str) -> Self {
        Self {
            name: None,
            text: Some(text.to_string()),
            index: 0,
        }
    }

    /// 名前とインデックスで要素を選択
    pub fn by_name_index(name: &str, index: usize) -> Self {
        Self {
            name: Some(name.to_string()),
            text: None,
            index,
        }
    }
}

/// UI検証条件
#[derive(Clone)]
pub struct UiVerification {
    /// 検証名
    pub name: String,
    /// 検索するコンポーネント名（部分一致）
    pub component_name: Option<String>,
    /// 検索するテキスト（部分一致）
    pub text_contains: Option<String>,
    /// 最小要素数
    pub min_count: Option<usize>,
    /// 最大要素数
    pub max_count: Option<usize>,
}

/// スクリーンショット撮影イベント
#[derive(Event)]
pub struct TakeScreenshotEvent {
    pub filename: Option<String>,
}

/// 入力シミュレーションイベント
#[derive(Event)]
pub struct SimulateInputEvent {
    pub action: InputAction,
}

/// シミュレート可能な入力アクション
#[derive(Clone)]
pub enum InputAction {
    PressKey(KeyCode),
    ReleaseKey(KeyCode),
    TapKey(KeyCode),
    MousePress(MouseButton),
    MouseRelease(MouseButton),
    MouseMove(f32, f32),
}

/// テストシナリオ実行イベント
#[derive(Event)]
pub struct RunTestScenarioEvent {
    pub scenario_name: String,
}

/// アプリ状態変更イベント（テスト用）
#[derive(Event)]
pub struct SetAppStateEvent {
    pub state_name: String,
}

/// UIダンプイベント
#[derive(Event)]
pub struct DumpUiEvent {
    pub filename: Option<String>,
}

/// UI検証イベント
#[derive(Event)]
pub struct VerifyUiEvent {
    pub verification: UiVerification,
}

/// テキスト入力イベント
#[derive(Event)]
pub struct TypeTextEvent {
    pub text: String,
}

/// コマンド実行イベント
#[derive(Event)]
pub struct ExecuteCommandEvent {
    pub command: String,
}
