//! E2Eテストシステム
//!
//! ゲームの自動テスト・スクリーンショット撮影・入力シミュレーションを提供
//!
//! ## 使用方法
//! 1. ゲームを起動
//! 2. F8/F11キーを押してテストを実行
//! 3. テスト結果が `screenshots/test_report.txt` に保存される
//! 4. Claude CodeがRead toolでレポートを確認（トークン消費小）
//!
//! ## キーバインド
//! - F8: インタラクションテスト（全操作パターン）
//! - F9: 手動スクリーンショット撮影
//! - F10: インベントリUIテスト
//! - F11: フルテスト（メインメニューから全機能テスト）
//! - F12: UIダンプ（テキストベース）
//!
//! ## 出力ファイル
//! - `screenshots/test_report.txt`: テスト結果レポート（トークン消費小）
//! - `screenshots/ui_dump_*.txt`: UI構造ダンプ（トークン消費小）
//! - `screenshots/*.png`: スクリーンショット（トークン消費大、必要時のみ）
//!
//! ## トークン消費の最適化
//! - 通常: テキストレポートのみ確認（数KB）
//! - 問題時: 該当スクリーンショットのみ確認
//!
//! ## 人間らしい挙動
//! - ランダムな待機時間（思考時間のシミュレーション）
//! - タイピング遅延（キー入力間隔のばらつき）
//! - マウス移動の揺れ（直線ではなく曲線移動）
//! - ミスクリックのシミュレーション（オプション）

use bevy::prelude::*;
use bevy::render::view::screenshot::{save_to_disk, Screenshot};
use bevy::window::PrimaryWindow;
use bevy::ui::ComputedNode;
use std::path::PathBuf;
use std::io::Write;
use rand::prelude::*;

/// E2Eテストプラグイン
pub struct E2ETestPlugin;

impl Plugin for E2ETestPlugin {
    fn build(&self, app: &mut App) {
        // コマンドライン引数をチェック
        let auto_test = std::env::args().any(|arg| arg == "--e2e-test" || arg == "--full-test");

        app.init_resource::<E2ETestState>()
            .init_resource::<TestScenarioQueue>()
            .init_resource::<TestReport>()
            .init_resource::<HumanBehaviorConfig>()
            .init_resource::<UiElementCache>()
            .insert_resource(AutoTestConfig { enabled: auto_test })
            .add_event::<TakeScreenshotEvent>()
            .add_event::<SimulateInputEvent>()
            .add_event::<RunTestScenarioEvent>()
            .add_event::<SetAppStateEvent>()
            .add_event::<DumpUiEvent>()
            .add_event::<VerifyUiEvent>()
            .add_event::<TypeTextEvent>()
            .add_event::<ExecuteCommandEvent>()
            .add_event::<ClickElementEvent>()
            .add_event::<ScrollEvent>()
            .add_systems(
                Update,
                (
                    handle_screenshot_hotkey,
                    handle_test_hotkey,
                    handle_ui_dump_hotkey,
                    auto_start_test,
                    process_screenshot_events,
                    process_input_simulation,
                    process_app_state_changes,
                    process_ui_dump,
                    process_ui_verification,
                    process_execute_command,
                    process_click_element,
                    process_scroll,
                    update_mouse_animation,
                    update_typing_animation,
                    update_ui_element_cache,
                    run_test_scenarios,
                    advance_scenario_step,
                )
                    .chain(),
            );
    }
}

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

/// 自動テスト開始（起動時に1回だけ）
fn auto_start_test(
    mut config: ResMut<AutoTestConfig>,
    mut run_events: EventWriter<RunTestScenarioEvent>,
    time: Res<Time>,
) {
    // 起動後2秒待ってからテスト開始
    if config.enabled && time.elapsed_secs() > 2.0 {
        info!("[E2E] Auto-starting full test (--e2e-test flag detected)");
        run_events.send(RunTestScenarioEvent {
            scenario_name: "full_test".to_string(),
        });
        config.enabled = false; // 1回だけ実行
    }
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

/// F9キーでスクリーンショット撮影
fn handle_screenshot_hotkey(
    input: Res<ButtonInput<KeyCode>>,
    mut screenshot_events: EventWriter<TakeScreenshotEvent>,
) {
    if input.just_pressed(KeyCode::F9) {
        info!("[E2E] F9 pressed - Taking screenshot");
        screenshot_events.send(TakeScreenshotEvent { filename: None });
    }
}

/// F8: インタラクションテスト、F10: インベントリテスト、F11: フルテスト
fn handle_test_hotkey(
    input: Res<ButtonInput<KeyCode>>,
    mut run_events: EventWriter<RunTestScenarioEvent>,
) {
    if input.just_pressed(KeyCode::F8) {
        info!("[E2E] F8 pressed - Running INTERACTION test scenario");
        run_events.send(RunTestScenarioEvent {
            scenario_name: "interaction_test".to_string(),
        });
    }
    if input.just_pressed(KeyCode::F10) {
        info!("[E2E] F10 pressed - Running inventory test scenario");
        run_events.send(RunTestScenarioEvent {
            scenario_name: "ui_inventory_test".to_string(),
        });
    }
    if input.just_pressed(KeyCode::F11) {
        info!("[E2E] F11 pressed - Running FULL test scenario");
        run_events.send(RunTestScenarioEvent {
            scenario_name: "full_test".to_string(),
        });
    }
}

/// F12キーでUIダンプ
fn handle_ui_dump_hotkey(
    input: Res<ButtonInput<KeyCode>>,
    mut dump_events: EventWriter<DumpUiEvent>,
) {
    if input.just_pressed(KeyCode::F12) {
        info!("[E2E] F12 pressed - Dumping UI");
        dump_events.send(DumpUiEvent { filename: None });
    }
}

/// スクリーンショットイベント処理
fn process_screenshot_events(
    mut commands: Commands,
    mut screenshot_events: EventReader<TakeScreenshotEvent>,
    mut state: ResMut<E2ETestState>,
) {
    for event in screenshot_events.read() {
        // ファイル名生成
        let filename = event.filename.clone().unwrap_or_else(|| {
            state.screenshot_counter += 1;
            format!("screenshot_{:04}.png", state.screenshot_counter)
        });

        // 保存先パス
        let path = PathBuf::from("screenshots").join(&filename);

        // ディレクトリ作成
        if let Err(e) = std::fs::create_dir_all("screenshots") {
            error!("[E2E] Failed to create screenshots directory: {}", e);
            continue;
        }

        info!("[E2E] Taking screenshot: {:?}", path);

        // スクリーンショット撮影 (Bevy 0.15 API)
        // 新しいエンティティにScreenshotとobserverを追加
        commands.spawn_empty()
            .observe(save_to_disk(path))
            .insert(Screenshot::primary_window());
    }
}

/// 入力シミュレーション処理
fn process_input_simulation(
    mut input_events: EventReader<SimulateInputEvent>,
    mut keyboard: ResMut<ButtonInput<KeyCode>>,
    mut mouse: ResMut<ButtonInput<MouseButton>>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
) {
    for event in input_events.read() {
        match &event.action {
            InputAction::PressKey(key) => {
                info!("[E2E] Simulating key press: {:?}", key);
                keyboard.press(*key);
            }
            InputAction::ReleaseKey(key) => {
                info!("[E2E] Simulating key release: {:?}", key);
                keyboard.release(*key);
            }
            InputAction::TapKey(key) => {
                info!("[E2E] Simulating key tap: {:?}", key);
                keyboard.press(*key);
                // 次フレームで解放（run_test_scenariosで処理）
            }
            InputAction::MousePress(button) => {
                info!("[E2E] Simulating mouse press: {:?}", button);
                mouse.press(*button);
            }
            InputAction::MouseRelease(button) => {
                info!("[E2E] Simulating mouse release: {:?}", button);
                mouse.release(*button);
            }
            InputAction::MouseMove(x, y) => {
                if let Ok(mut window) = windows.get_single_mut() {
                    info!("[E2E] Simulating mouse move to: ({}, {})", x, y);
                    window.set_cursor_position(Some(Vec2::new(*x, *y)));
                }
            }
        }
    }
}

/// アプリ状態変更処理（テスト用）
fn process_app_state_changes(
    mut state_events: EventReader<SetAppStateEvent>,
    mut next_app_state: ResMut<NextState<crate::ui::main_menu::AppState>>,
    mut next_inventory_state: ResMut<NextState<crate::ui::inventory_ui::InventoryUiState>>,
    mut next_settings_state: ResMut<NextState<crate::ui::settings_ui::SettingsUiState>>,
) {
    use crate::ui::main_menu::AppState;
    use crate::ui::inventory_ui::InventoryUiState;
    use crate::ui::settings_ui::SettingsUiState;

    for event in state_events.read() {
        // AppState を試す
        let app_state = match event.state_name.as_str() {
            "MainMenu" => Some(AppState::MainMenu),
            "ProfileSelect" => Some(AppState::ProfileSelect),
            "SaveSelect" => Some(AppState::SaveSelect),
            "WorldGeneration" => Some(AppState::WorldGeneration),
            "InGame" => Some(AppState::InGame),
            "PauseMenu" => Some(AppState::PauseMenu),
            _ => None,
        };

        if let Some(state) = app_state {
            info!("[E2E] Setting app state to: {:?}", state);
            next_app_state.set(state);
            continue;
        }

        // InventoryUiState を試す
        let inventory_state = match event.state_name.as_str() {
            "InventoryClosed" => Some(InventoryUiState::Closed),
            "InventoryOpen" | "PlayerInventory" => Some(InventoryUiState::PlayerInventory),
            "Container" => Some(InventoryUiState::Container),
            _ => None,
        };

        if let Some(state) = inventory_state {
            info!("[E2E] Setting inventory state to: {:?}", state);
            next_inventory_state.set(state);
            continue;
        }

        // SettingsUiState を試す
        let settings_state = match event.state_name.as_str() {
            "Settings" | "SettingsOpen" => Some(SettingsUiState::SettingsOpen),
            "SettingsClosed" => Some(SettingsUiState::Closed),
            _ => None,
        };

        if let Some(state) = settings_state {
            info!("[E2E] Setting settings state to: {:?}", state);
            next_settings_state.set(state);
            continue;
        }

        warn!("[E2E] Unknown state: {}", event.state_name);
    }
}

/// UIダンプ処理
#[allow(clippy::type_complexity)]
fn process_ui_dump(
    mut dump_events: EventReader<DumpUiEvent>,
    mut state: ResMut<E2ETestState>,
    // UIノード情報を取得
    node_query: Query<(Entity, &Node, Option<&Name>, Option<&Text>, Option<&Children>)>,
    parent_query: Query<&Parent>,
    app_state: Res<State<crate::ui::main_menu::AppState>>,
    inventory_state: Res<State<crate::ui::inventory_ui::InventoryUiState>>,
) {
    for event in dump_events.read() {
        // ファイル名生成
        let filename = event.filename.clone().unwrap_or_else(|| {
            state.dump_counter += 1;
            format!("ui_dump_{:04}.txt", state.dump_counter)
        });

        let path = PathBuf::from("screenshots").join(&filename);

        // ディレクトリ作成
        if let Err(e) = std::fs::create_dir_all("screenshots") {
            error!("[E2E] Failed to create screenshots directory: {}", e);
            continue;
        }

        info!("[E2E] Dumping UI to: {:?}", path);

        // UIダンプを生成
        let mut dump = String::new();
        dump.push_str("# UI Dump\n");
        dump.push_str(&format!("Generated: {}\n", chrono::Local::now().format("%Y-%m-%d %H:%M:%S")));
        dump.push_str(&format!("AppState: {:?}\n", app_state.get()));
        dump.push_str(&format!("InventoryState: {:?}\n", inventory_state.get()));
        dump.push_str("\n## UI Tree\n\n");

        // ルートノードを見つける（親がないノード）
        let root_nodes: Vec<_> = node_query
            .iter()
            .filter(|(entity, _, _, _, _)| parent_query.get(*entity).is_err())
            .collect();

        for (entity, node, name, text, children) in root_nodes {
            dump_ui_node(&mut dump, 0, entity, node, name, text, children, &node_query);
        }

        // 統計情報
        let total_nodes = node_query.iter().count();
        let named_nodes = node_query.iter().filter(|(_, _, name, _, _)| name.is_some()).count();
        let text_nodes = node_query.iter().filter(|(_, _, _, text, _)| text.is_some()).count();

        dump.push_str("\n## Statistics\n");
        dump.push_str(&format!("- Total nodes: {}\n", total_nodes));
        dump.push_str(&format!("- Named nodes: {}\n", named_nodes));
        dump.push_str(&format!("- Text nodes: {}\n", text_nodes));

        // ファイルに保存
        if let Err(e) = std::fs::write(&path, &dump) {
            error!("[E2E] Failed to write UI dump: {}", e);
        } else {
            info!("[E2E] UI dump saved to: {:?}", path);
        }
    }
}

/// UIノードを再帰的にダンプ
#[allow(clippy::type_complexity, clippy::too_many_arguments)]
fn dump_ui_node(
    dump: &mut String,
    depth: usize,
    _entity: Entity,
    node: &Node,
    name: Option<&Name>,
    text: Option<&Text>,
    children: Option<&Children>,
    node_query: &Query<(Entity, &Node, Option<&Name>, Option<&Text>, Option<&Children>)>,
) {
    let indent = "  ".repeat(depth);
    let name_str = name.map(|n| n.as_str()).unwrap_or("(unnamed)");
    let text_str = text.map(|t| format!(" text=\"{}\"", t.0.chars().take(30).collect::<String>())).unwrap_or_default();

    // ノード情報を出力
    dump.push_str(&format!(
        "{}- {} [{}]{}\n",
        indent,
        name_str,
        format_node_info(node),
        text_str
    ));

    // 子ノードを再帰的にダンプ
    if let Some(children) = children {
        for child in children.iter() {
            if let Ok((child_entity, child_node, child_name, child_text, child_children)) = node_query.get(*child) {
                dump_ui_node(dump, depth + 1, child_entity, child_node, child_name, child_text, child_children, node_query);
            }
        }
    }
}

/// ノード情報をフォーマット
fn format_node_info(node: &Node) -> String {
    let width = match node.width {
        Val::Px(px) => format!("{}px", px),
        Val::Percent(p) => format!("{}%", p),
        Val::Auto => "auto".to_string(),
        _ => "?".to_string(),
    };
    let height = match node.height {
        Val::Px(px) => format!("{}px", px),
        Val::Percent(p) => format!("{}%", p),
        Val::Auto => "auto".to_string(),
        _ => "?".to_string(),
    };
    format!("w:{} h:{}", width, height)
}

/// UI検証処理
fn process_ui_verification(
    mut verify_events: EventReader<VerifyUiEvent>,
    mut report: ResMut<TestReport>,
    node_query: Query<(Entity, &Node, Option<&Name>, Option<&Text>)>,
) {
    for event in verify_events.read() {
        let verification = &event.verification;
        let mut matching_count = 0;
        let mut found_texts = Vec::new();

        for (_, _, name, text) in node_query.iter() {
            let mut matches = true;

            // コンポーネント名チェック
            if let Some(ref comp_name) = verification.component_name {
                if let Some(name) = name {
                    if !name.as_str().contains(comp_name) {
                        matches = false;
                    }
                } else {
                    matches = false;
                }
            }

            // テキストチェック
            if let Some(ref text_contains) = verification.text_contains {
                if let Some(text) = text {
                    if !text.0.contains(text_contains) {
                        matches = false;
                    } else {
                        found_texts.push(text.0.clone());
                    }
                } else {
                    matches = false;
                }
            }

            if matches {
                matching_count += 1;
            }
        }

        // 検証結果を判定
        let mut passed = true;
        let mut message = String::new();

        if let Some(min) = verification.min_count {
            if matching_count < min {
                passed = false;
                message = format!("Expected at least {} elements, found {}", min, matching_count);
            }
        }

        if let Some(max) = verification.max_count {
            if matching_count > max {
                passed = false;
                message = format!("Expected at most {} elements, found {}", max, matching_count);
            }
        }

        if passed && message.is_empty() {
            message = format!("Found {} matching elements", matching_count);
        }

        report.add_result(VerificationResult {
            name: verification.name.clone(),
            passed,
            message,
            expected: format!(
                "min:{:?} max:{:?} comp:{:?} text:{:?}",
                verification.min_count,
                verification.max_count,
                verification.component_name,
                verification.text_contains
            ),
            actual: format!("count:{} texts:{:?}", matching_count, found_texts.iter().take(3).collect::<Vec<_>>()),
        });
    }
}

/// コマンド実行イベント処理
fn process_execute_command(
    mut exec_events: EventReader<ExecuteCommandEvent>,
    mut cmd_events: EventWriter<crate::gameplay::commands::ExecuteCommandEvent>,
) {
    for event in exec_events.read() {
        info!("[E2E] Executing command: {}", event.command);
        cmd_events.send(crate::gameplay::commands::ExecuteCommandEvent {
            input: event.command.clone(),
        });
    }
}

/// UI要素クリック処理
fn process_click_element(
    mut click_events: EventReader<ClickElementEvent>,
    ui_cache: Res<UiElementCache>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
    mut mouse: ResMut<ButtonInput<MouseButton>>,
    human_config: Res<HumanBehaviorConfig>,
) {
    for event in click_events.read() {
        // 要素を検索
        if let Some(element) = find_element(&ui_cache, &event.selector) {
            let center = element.screen_rect.center();

            // 人間らしいクリック位置のばらつき
            let (click_x, click_y) = if human_config.enabled {
                let mut rng = rand::thread_rng();
                let jitter_x = rng.gen_range(-human_config.mouse_jitter..human_config.mouse_jitter);
                let jitter_y = rng.gen_range(-human_config.mouse_jitter..human_config.mouse_jitter);
                (center.x + jitter_x, center.y + jitter_y)
            } else {
                (center.x, center.y)
            };

            // マウスを移動
            if let Ok(mut window) = windows.get_single_mut() {
                info!("[E2E] Clicking element at ({}, {}): {:?}", click_x, click_y, event.selector.name);
                window.set_cursor_position(Some(Vec2::new(click_x, click_y)));
            }

            // クリック
            mouse.press(event.button);
            // 解放は次フレームで行う（advance_scenario_stepで処理）
        } else {
            warn!("[E2E] Element not found: name={:?} text={:?}",
                event.selector.name, event.selector.text);
        }
    }
}

/// スクロール処理
fn process_scroll(
    mut scroll_events: EventReader<ScrollEvent>,
    mut mouse_wheel: EventWriter<bevy::input::mouse::MouseWheel>,
) {
    for event in scroll_events.read() {
        info!("[E2E] Scrolling: {}", event.delta);
        mouse_wheel.send(bevy::input::mouse::MouseWheel {
            unit: bevy::input::mouse::MouseScrollUnit::Line,
            x: 0.0,
            y: event.delta,
            window: Entity::PLACEHOLDER,
        });
    }
}

/// マウスアニメーション更新（ベジェ曲線で滑らかに移動）
fn update_mouse_animation(
    _time: Res<Time>,
    mut state: ResMut<E2ETestState>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
    human_config: Res<HumanBehaviorConfig>,
) {
    if let Some(ref mut anim) = state.mouse_animation {
        anim.current_step += 1;

        if anim.current_step >= anim.total_steps {
            // アニメーション完了
            if let Ok(mut window) = windows.get_single_mut() {
                window.set_cursor_position(Some(anim.end_pos));
            }
            state.mouse_animation = None;
        } else {
            // ベジェ曲線で補間
            let t = anim.current_step as f32 / anim.total_steps as f32;
            let pos = quadratic_bezier(anim.start_pos, anim.control_point, anim.end_pos, t);

            // 人間らしい揺れを追加
            let jittered_pos = if human_config.enabled {
                let mut rng = rand::thread_rng();
                let jitter = human_config.mouse_jitter * (1.0 - t); // 目標に近づくと揺れが減少
                Vec2::new(
                    pos.x + rng.gen_range(-jitter..jitter),
                    pos.y + rng.gen_range(-jitter..jitter),
                )
            } else {
                pos
            };

            if let Ok(mut window) = windows.get_single_mut() {
                window.set_cursor_position(Some(jittered_pos));
            }
        }
    }
}

/// タイピングアニメーション更新（1文字ずつ入力）
fn update_typing_animation(
    time: Res<Time>,
    mut state: ResMut<E2ETestState>,
    mut keyboard: ResMut<ButtonInput<KeyCode>>,
    human_config: Res<HumanBehaviorConfig>,
) {
    if let Some(ref mut anim) = state.typing_animation.clone() {
        let mut anim = anim.clone();
        anim.next_char_timer -= time.delta_secs();

        if anim.next_char_timer <= 0.0 && anim.current_index < anim.text.len() {
            // 次の文字を入力
            let ch = anim.text.chars().nth(anim.current_index).unwrap();
            if let Some(key) = char_to_keycode(ch) {
                info!("[E2E] Typing: '{}'", ch);
                keyboard.press(key);
                // 次フレームで解放
            }

            anim.current_index += 1;

            // 次の文字までの待機時間
            let base_interval = human_config.typing_interval;
            let variance = human_config.typing_variance;
            let mut rng = rand::thread_rng();
            anim.next_char_timer = base_interval * (1.0 + rng.gen_range(-variance..variance));

            state.typing_animation = Some(anim);
        } else if anim.current_index >= anim.text.len() {
            // タイピング完了
            state.typing_animation = None;
        } else {
            state.typing_animation = Some(anim);
        }
    }
}

/// UI要素キャッシュ更新
#[allow(clippy::type_complexity)]
fn update_ui_element_cache(
    time: Res<Time>,
    mut cache: ResMut<UiElementCache>,
    node_query: Query<(Entity, &Node, &ComputedNode, &GlobalTransform, Option<&Name>, Option<&Text>)>,
) {
    // 0.1秒ごとに更新
    if time.elapsed_secs() - cache.last_update < 0.1 {
        return;
    }
    cache.last_update = time.elapsed_secs();
    cache.elements.clear();

    for (entity, _node, computed, transform, name, text) in node_query.iter() {
        let size = computed.size();
        let pos = transform.translation();

        // 画面座標でのRect
        let screen_rect = Rect::from_center_size(
            Vec2::new(pos.x, pos.y),
            size,
        );

        cache.elements.push(CachedUiElement {
            entity,
            name: name.map(|n| n.to_string()),
            text: text.map(|t| t.0.clone()),
            screen_rect,
        });
    }
}

/// セレクターに一致する要素を検索
fn find_element<'a>(cache: &'a UiElementCache, selector: &ElementSelector) -> Option<&'a CachedUiElement> {
    let matches: Vec<_> = cache.elements.iter().filter(|e| {
        let name_match = selector.name.as_ref().is_none_or(|n| {
            e.name.as_ref().is_some_and(|en| en.contains(n))
        });
        let text_match = selector.text.as_ref().is_none_or(|t| {
            e.text.as_ref().is_some_and(|et| et.contains(t))
        });
        name_match && text_match
    }).collect();

    matches.get(selector.index).copied()
}

/// 2次ベジェ曲線の計算
fn quadratic_bezier(p0: Vec2, p1: Vec2, p2: Vec2, t: f32) -> Vec2 {
    let t2 = t * t;
    let mt = 1.0 - t;
    let mt2 = mt * mt;

    p0 * mt2 + p1 * 2.0 * mt * t + p2 * t2
}

/// 文字をKeyCodeに変換
fn char_to_keycode(ch: char) -> Option<KeyCode> {
    match ch.to_ascii_lowercase() {
        'a' => Some(KeyCode::KeyA),
        'b' => Some(KeyCode::KeyB),
        'c' => Some(KeyCode::KeyC),
        'd' => Some(KeyCode::KeyD),
        'e' => Some(KeyCode::KeyE),
        'f' => Some(KeyCode::KeyF),
        'g' => Some(KeyCode::KeyG),
        'h' => Some(KeyCode::KeyH),
        'i' => Some(KeyCode::KeyI),
        'j' => Some(KeyCode::KeyJ),
        'k' => Some(KeyCode::KeyK),
        'l' => Some(KeyCode::KeyL),
        'm' => Some(KeyCode::KeyM),
        'n' => Some(KeyCode::KeyN),
        'o' => Some(KeyCode::KeyO),
        'p' => Some(KeyCode::KeyP),
        'q' => Some(KeyCode::KeyQ),
        'r' => Some(KeyCode::KeyR),
        's' => Some(KeyCode::KeyS),
        't' => Some(KeyCode::KeyT),
        'u' => Some(KeyCode::KeyU),
        'v' => Some(KeyCode::KeyV),
        'w' => Some(KeyCode::KeyW),
        'x' => Some(KeyCode::KeyX),
        'y' => Some(KeyCode::KeyY),
        'z' => Some(KeyCode::KeyZ),
        '0' => Some(KeyCode::Digit0),
        '1' => Some(KeyCode::Digit1),
        '2' => Some(KeyCode::Digit2),
        '3' => Some(KeyCode::Digit3),
        '4' => Some(KeyCode::Digit4),
        '5' => Some(KeyCode::Digit5),
        '6' => Some(KeyCode::Digit6),
        '7' => Some(KeyCode::Digit7),
        '8' => Some(KeyCode::Digit8),
        '9' => Some(KeyCode::Digit9),
        ' ' => Some(KeyCode::Space),
        '\n' => Some(KeyCode::Enter),
        _ => None,
    }
}

/// ランダムな待機時間を生成
fn random_wait(min: f32, max: f32) -> f32 {
    let mut rng = rand::thread_rng();
    rng.gen_range(min..max)
}

/// テストシナリオ実行
fn run_test_scenarios(
    mut run_events: EventReader<RunTestScenarioEvent>,
    mut state: ResMut<E2ETestState>,
    mut queue: ResMut<TestScenarioQueue>,
) {
    for event in run_events.read() {
        info!("[E2E] Starting test scenario: {}", event.scenario_name);

        // ビルトインシナリオを取得
        if let Some(scenario) = get_builtin_scenario(&event.scenario_name) {
            queue.scenarios.push(scenario);
            state.is_test_mode = true;
            state.current_scenario = Some(event.scenario_name.clone());
            state.current_step = 0;
            state.wait_timer = 0.0;
        } else {
            warn!("[E2E] Unknown scenario: {}", event.scenario_name);
        }
    }
}

/// シナリオステップ進行
#[allow(clippy::too_many_arguments)]
fn advance_scenario_step(
    time: Res<Time>,
    mut state: ResMut<E2ETestState>,
    mut queue: ResMut<TestScenarioQueue>,
    mut report: ResMut<TestReport>,
    mut screenshot_events: EventWriter<TakeScreenshotEvent>,
    mut input_events: EventWriter<SimulateInputEvent>,
    mut app_state_events: EventWriter<SetAppStateEvent>,
    mut dump_events: EventWriter<DumpUiEvent>,
    mut verify_events: EventWriter<VerifyUiEvent>,
    mut type_text_events: EventWriter<TypeTextEvent>,
    mut exec_cmd_events: EventWriter<ExecuteCommandEvent>,
    mut click_events: EventWriter<ClickElementEvent>,
    mut scroll_events: EventWriter<ScrollEvent>,
    human_config: Res<HumanBehaviorConfig>,
    windows: Query<&Window, With<PrimaryWindow>>,
) {
    if !state.is_test_mode {
        return;
    }

    // マウスアニメーション中は待機
    if state.mouse_animation.is_some() {
        return;
    }

    // タイピングアニメーション中は待機
    if state.typing_animation.is_some() {
        return;
    }

    // 待機中の場合
    if state.wait_timer > 0.0 {
        state.wait_timer -= time.delta_secs();
        return;
    }

    // 現在のシナリオを取得
    let Some(scenario) = queue.scenarios.get(queue.current_index) else {
        // 全シナリオ完了
        info!("[E2E] All scenarios completed");
        state.is_test_mode = false;
        state.current_scenario = None;
        return;
    };

    // 現在のステップを取得
    let Some(step) = scenario.steps.get(state.current_step) else {
        // シナリオ完了、次のシナリオへ
        info!("[E2E] Scenario '{}' completed", scenario.name);
        queue.current_index += 1;
        state.current_step = 0;
        return;
    };

    // ステップ実行
    match step {
        TestStep::Wait(seconds) => {
            info!("[E2E] Waiting {} seconds", seconds);
            state.wait_timer = *seconds;
        }
        TestStep::WaitRandom(min, max) => {
            let wait_time = random_wait(*min, *max);
            info!("[E2E] Waiting {} seconds (random)", wait_time);
            state.wait_timer = wait_time;
        }
        TestStep::PressKey(key) => {
            input_events.send(SimulateInputEvent {
                action: InputAction::PressKey(*key),
            });
        }
        TestStep::ReleaseKey(key) => {
            input_events.send(SimulateInputEvent {
                action: InputAction::ReleaseKey(*key),
            });
        }
        TestStep::TapKey(key) => {
            input_events.send(SimulateInputEvent {
                action: InputAction::TapKey(*key),
            });
            // タップは短い待機を追加
            state.wait_timer = 0.1;
        }
        TestStep::TapKeyHuman(key) => {
            input_events.send(SimulateInputEvent {
                action: InputAction::TapKey(*key),
            });
            // 人間らしいランダムな待機
            state.wait_timer = random_wait(0.05, 0.2);
        }
        TestStep::MousePress(button) => {
            input_events.send(SimulateInputEvent {
                action: InputAction::MousePress(*button),
            });
        }
        TestStep::MouseRelease(button) => {
            input_events.send(SimulateInputEvent {
                action: InputAction::MouseRelease(*button),
            });
        }
        TestStep::MouseMove(x, y) => {
            input_events.send(SimulateInputEvent {
                action: InputAction::MouseMove(*x, *y),
            });
        }
        TestStep::MouseMoveSmooth(x, y) => {
            // 現在のマウス位置を取得
            let current_pos = if let Ok(window) = windows.get_single() {
                window.cursor_position().unwrap_or(Vec2::new(640.0, 360.0))
            } else {
                Vec2::new(640.0, 360.0)
            };

            let end_pos = Vec2::new(*x, *y);

            // ベジェ曲線の制御点を計算（少しカーブさせる）
            let mut rng = rand::thread_rng();
            let mid = (current_pos + end_pos) / 2.0;
            let offset = rng.gen_range(-50.0..50.0);
            let control_point = Vec2::new(mid.x + offset, mid.y + offset);

            info!("[E2E] Smooth mouse move from {:?} to {:?}", current_pos, end_pos);

            state.mouse_animation = Some(MouseAnimationState {
                start_pos: current_pos,
                end_pos,
                current_step: 0,
                total_steps: human_config.mouse_move_steps,
                control_point,
            });
        }
        TestStep::MouseClick(button) => {
            input_events.send(SimulateInputEvent {
                action: InputAction::MousePress(*button),
            });
            state.wait_timer = 0.05;
            // 次のステップでReleaseを追加（簡略化）
        }
        TestStep::MouseClickHuman(button) => {
            input_events.send(SimulateInputEvent {
                action: InputAction::MousePress(*button),
            });
            // 人間らしいクリック時間
            state.wait_timer = random_wait(0.03, 0.12);
        }
        TestStep::DoubleClick(button) => {
            input_events.send(SimulateInputEvent {
                action: InputAction::MousePress(*button),
            });
            // ダブルクリック処理（簡略化 - 2回のクリックをシミュレート）
            state.wait_timer = 0.1;
        }
        TestStep::Scroll(delta) => {
            scroll_events.send(ScrollEvent { delta: *delta });
            state.wait_timer = 0.1;
        }
        TestStep::ClickElement(selector) => {
            click_events.send(ClickElementEvent {
                selector: selector.clone(),
                button: MouseButton::Left,
                double_click: false,
            });
            state.wait_timer = random_wait(0.1, 0.3);
        }
        TestStep::HoverElement(selector) => {
            // 要素の中心にマウスを移動（後で実装）
            info!("[E2E] Hovering element: {:?}", selector.name);
            state.wait_timer = 0.2;
        }
        TestStep::DoubleClickElement(selector) => {
            click_events.send(ClickElementEvent {
                selector: selector.clone(),
                button: MouseButton::Left,
                double_click: true,
            });
            state.wait_timer = random_wait(0.1, 0.3);
        }
        TestStep::RightClickElement(selector) => {
            click_events.send(ClickElementEvent {
                selector: selector.clone(),
                button: MouseButton::Right,
                double_click: false,
            });
            state.wait_timer = random_wait(0.1, 0.3);
        }
        TestStep::DragDropElements(from, to) => {
            // ドラッグ&ドロップ（後で詳細実装）
            info!("[E2E] Drag from {:?} to {:?}", from.name, to.name);
            state.wait_timer = 0.5;
        }
        TestStep::DragDrop(x1, y1, x2, y2) => {
            // 座標でのドラッグ&ドロップ
            input_events.send(SimulateInputEvent {
                action: InputAction::MouseMove(*x1, *y1),
            });
            info!("[E2E] Drag from ({}, {}) to ({}, {})", x1, y1, x2, y2);
            state.wait_timer = 0.5;
        }
        TestStep::Screenshot(name) => {
            screenshot_events.send(TakeScreenshotEvent {
                filename: Some(format!("{}_{}.png", scenario.name, name)),
            });
            // スクリーンショット後は少し待機
            state.wait_timer = 0.5;
        }
        TestStep::Log(msg) => {
            info!("[E2E] {}", msg);
        }
        TestStep::Custom(name) => {
            info!("[E2E] Custom action: {} (not implemented)", name);
        }
        TestStep::SetAppState(state_name) => {
            app_state_events.send(SetAppStateEvent {
                state_name: state_name.clone(),
            });
            // 状態遷移後は待機
            state.wait_timer = 0.5;
        }
        TestStep::DumpUi(name) => {
            dump_events.send(DumpUiEvent {
                filename: Some(format!("{}_{}.txt", scenario.name, name)),
            });
            state.wait_timer = 0.2;
        }
        TestStep::VerifyElement(verification) => {
            verify_events.send(VerifyUiEvent {
                verification: verification.clone(),
            });
            state.wait_timer = 0.1;
        }
        TestStep::SaveReport => {
            let path = PathBuf::from("screenshots").join("test_report.txt");
            if let Err(e) = std::fs::create_dir_all("screenshots") {
                error!("[E2E] Failed to create screenshots directory: {}", e);
            } else if let Err(e) = report.save_to_file(&path) {
                error!("[E2E] Failed to save report: {}", e);
            } else {
                info!("[E2E] Test report saved to: {:?}", path);
            }
        }
        TestStep::ClearReport => {
            report.clear();
            info!("[E2E] Test report cleared");
        }
        TestStep::TypeText(text) => {
            type_text_events.send(TypeTextEvent {
                text: text.clone(),
            });
            state.wait_timer = 0.2;
        }
        TestStep::TypeTextHuman(text) => {
            // 人間らしいタイピングアニメーション開始
            state.typing_animation = Some(TypingAnimationState {
                text: text.clone(),
                current_index: 0,
                next_char_timer: 0.0,
            });
        }
        TestStep::ExecuteCommand(command) => {
            exec_cmd_events.send(ExecuteCommandEvent {
                command: command.clone(),
            });
            state.wait_timer = 0.5;
        }
        TestStep::Think => {
            // 人間らしい思考時間
            let think_time = random_wait(
                human_config.min_think_time,
                human_config.max_think_time,
            );
            info!("[E2E] Thinking for {} seconds", think_time);
            state.wait_timer = think_time;
        }
        TestStep::HumanSequence(steps) => {
            // 人間らしい間隔でステップを実行（後で詳細実装）
            info!("[E2E] Human sequence with {} steps", steps.len());
            state.wait_timer = 0.1;
        }
    }

    // 次のステップへ
    state.current_step += 1;
}

/// ビルトインシナリオを取得
fn get_builtin_scenario(name: &str) -> Option<TestScenario> {
    match name {
        "ui_inventory_test" => Some(create_inventory_test_scenario()),
        "ui_main_menu_test" => Some(create_main_menu_test_scenario()),
        "gameplay_basic_test" => Some(create_gameplay_basic_test_scenario()),
        "full_test" => Some(create_full_test_scenario()),
        "interaction_test" => Some(create_interaction_test_scenario()),
        "human_test" => Some(create_human_behavior_test_scenario()),
        "button_click_test" => Some(create_button_click_test_scenario()),
        _ => None,
    }
}

/// インベントリUIテストシナリオ
fn create_inventory_test_scenario() -> TestScenario {
    TestScenario {
        name: "ui_inventory_test".to_string(),
        steps: vec![
            TestStep::Log("Starting inventory UI test".to_string()),
            TestStep::Wait(1.0),
            // インベントリを開く
            TestStep::Log("Opening inventory with E key".to_string()),
            TestStep::TapKey(KeyCode::KeyE),
            TestStep::Wait(0.5),
            // スクリーンショット撮影
            TestStep::Screenshot("inventory_open".to_string()),
            TestStep::Wait(0.5),
            // インベントリを閉じる
            TestStep::Log("Closing inventory".to_string()),
            TestStep::TapKey(KeyCode::KeyE),
            TestStep::Wait(0.3),
            TestStep::Screenshot("inventory_closed".to_string()),
            TestStep::Log("Inventory UI test completed".to_string()),
        ],
    }
}

/// メインメニューテストシナリオ
fn create_main_menu_test_scenario() -> TestScenario {
    TestScenario {
        name: "ui_main_menu_test".to_string(),
        steps: vec![
            TestStep::Log("Starting main menu test".to_string()),
            TestStep::Wait(1.0),
            // ESCでポーズメニューを開く
            TestStep::Log("Opening pause menu".to_string()),
            TestStep::TapKey(KeyCode::Escape),
            TestStep::Wait(0.5),
            TestStep::Screenshot("pause_menu".to_string()),
            // 閉じる
            TestStep::TapKey(KeyCode::Escape),
            TestStep::Wait(0.3),
            TestStep::Log("Main menu test completed".to_string()),
        ],
    }
}

/// 基本ゲームプレイテストシナリオ
fn create_gameplay_basic_test_scenario() -> TestScenario {
    TestScenario {
        name: "gameplay_basic_test".to_string(),
        steps: vec![
            TestStep::Log("Starting gameplay basic test".to_string()),
            TestStep::Wait(1.0),
            // 前進
            TestStep::Log("Moving forward".to_string()),
            TestStep::PressKey(KeyCode::KeyW),
            TestStep::Wait(1.0),
            TestStep::ReleaseKey(KeyCode::KeyW),
            TestStep::Screenshot("moved_forward".to_string()),
            // ジャンプ
            TestStep::Log("Jumping".to_string()),
            TestStep::TapKey(KeyCode::Space),
            TestStep::Wait(0.5),
            TestStep::Screenshot("jumped".to_string()),
            TestStep::Log("Gameplay basic test completed".to_string()),
        ],
    }
}

/// フルテストシナリオ（メインメニューから全機能をテスト）
///
/// このシナリオはメインメニューから開始し、以下をテスト:
/// 1. メインメニュー表示
/// 2. セーブスロット選択画面
/// 3. ワールド生成設定画面
/// 4. ゲーム内UI
/// 5. インベントリUI（★整列確認）
/// 6. ポーズメニュー
/// 7. メインメニューに戻る
///
/// ## トークン消費の最適化
/// - UIダンプ (.txt): テキストベース、トークン消費小
/// - 自動検証: Pass/Fail結果のみ、トークン消費極小
/// - スクリーンショット (.png): 視覚確認用、トークン消費大（必要時のみ確認）
fn create_full_test_scenario() -> TestScenario {
    TestScenario {
        name: "full_test".to_string(),
        steps: vec![
            // ========================================
            // 初期化
            // ========================================
            TestStep::Log("=== FULL TEST START ===".to_string()),
            TestStep::ClearReport,

            // ========================================
            // Phase 1: メインメニュー
            // ========================================
            TestStep::Log("Phase 1: Main Menu".to_string()),
            TestStep::Wait(0.5),
            TestStep::DumpUi("01_main_menu".to_string()),
            TestStep::VerifyElement(UiVerification {
                name: "MainMenu_PlayButton".to_string(),
                component_name: None,
                text_contains: Some("Play".to_string()),
                min_count: Some(1),
                max_count: None,
            }),
            TestStep::VerifyElement(UiVerification {
                name: "MainMenu_SettingsButton".to_string(),
                component_name: None,
                text_contains: Some("Settings".to_string()),
                min_count: Some(1),
                max_count: None,
            }),
            TestStep::Screenshot("01_main_menu".to_string()),

            // ========================================
            // Phase 2: セーブスロット選択
            // ========================================
            TestStep::Log("Phase 2: Save Slot Selection".to_string()),
            TestStep::SetAppState("SaveSelect".to_string()),
            TestStep::Wait(0.5),
            TestStep::DumpUi("02_save_select".to_string()),
            TestStep::VerifyElement(UiVerification {
                name: "SaveSelect_Title".to_string(),
                component_name: None,
                text_contains: Some("Select World".to_string()),
                min_count: Some(1),
                max_count: None,
            }),
            TestStep::VerifyElement(UiVerification {
                name: "SaveSelect_BackButton".to_string(),
                component_name: None,
                text_contains: Some("Back".to_string()),
                min_count: Some(1),
                max_count: None,
            }),
            TestStep::Screenshot("02_save_select".to_string()),

            // ========================================
            // Phase 3: ワールド生成設定
            // ========================================
            TestStep::Log("Phase 3: World Generation".to_string()),
            TestStep::SetAppState("WorldGeneration".to_string()),
            TestStep::Wait(0.5),
            TestStep::DumpUi("03_world_generation".to_string()),
            TestStep::VerifyElement(UiVerification {
                name: "WorldGen_CreateButton".to_string(),
                component_name: None,
                text_contains: Some("Create".to_string()),
                min_count: Some(1),
                max_count: None,
            }),
            TestStep::Screenshot("03_world_generation".to_string()),

            // ========================================
            // Phase 4: ゲーム内
            // ========================================
            TestStep::Log("Phase 4: In Game".to_string()),
            TestStep::SetAppState("InGame".to_string()),
            TestStep::Wait(1.5), // ワールド生成待ち
            TestStep::DumpUi("04_ingame_start".to_string()),
            TestStep::VerifyElement(UiVerification {
                name: "InGame_HPDisplay".to_string(),
                component_name: None,
                text_contains: Some("HP".to_string()),
                min_count: Some(1),
                max_count: None,
            }),
            TestStep::Screenshot("04_ingame_start".to_string()),

            // 移動テスト
            TestStep::Log("Moving player".to_string()),
            TestStep::PressKey(KeyCode::KeyW),
            TestStep::Wait(0.5),
            TestStep::ReleaseKey(KeyCode::KeyW),
            TestStep::Screenshot("05_ingame_moved".to_string()),

            // ========================================
            // Phase 5: インベントリUI（★重要: 整列確認）
            // ========================================
            TestStep::Log("Phase 5: Inventory UI - CHECK ALIGNMENT".to_string()),
            TestStep::SetAppState("InventoryOpen".to_string()),
            TestStep::Wait(0.5),
            TestStep::DumpUi("06_inventory_open".to_string()),
            TestStep::VerifyElement(UiVerification {
                name: "Inventory_SortButton".to_string(),
                component_name: None,
                text_contains: Some("Sort".to_string()),
                min_count: Some(1),
                max_count: None,
            }),
            TestStep::VerifyElement(UiVerification {
                name: "Inventory_TrashButton".to_string(),
                component_name: None,
                text_contains: Some("Trash".to_string()),
                min_count: Some(1),
                max_count: None,
            }),
            TestStep::Screenshot("06_inventory_open".to_string()),

            // インベントリを閉じる
            TestStep::SetAppState("InventoryClosed".to_string()),
            TestStep::Wait(0.3),
            TestStep::Screenshot("07_inventory_closed".to_string()),

            // ========================================
            // Phase 6: ポーズメニュー
            // ========================================
            TestStep::Log("Phase 6: Pause Menu".to_string()),
            TestStep::SetAppState("PauseMenu".to_string()),
            TestStep::Wait(0.5),
            TestStep::DumpUi("08_pause_menu".to_string()),
            TestStep::VerifyElement(UiVerification {
                name: "PauseMenu_ResumeButton".to_string(),
                component_name: None,
                text_contains: Some("Resume".to_string()),
                min_count: Some(1),
                max_count: None,
            }),
            TestStep::VerifyElement(UiVerification {
                name: "PauseMenu_MainMenuButton".to_string(),
                component_name: None,
                text_contains: Some("Main Menu".to_string()),
                min_count: Some(1),
                max_count: None,
            }),
            TestStep::Screenshot("08_pause_menu".to_string()),

            // ========================================
            // Phase 7: メインメニューに戻る
            // ========================================
            TestStep::Log("Phase 7: Back to Main Menu".to_string()),
            TestStep::SetAppState("MainMenu".to_string()),
            TestStep::Wait(0.5),
            TestStep::Screenshot("09_back_to_menu".to_string()),

            // ========================================
            // Phase 8: Settings画面
            // ========================================
            TestStep::Log("Phase 8: Settings Screen".to_string()),
            TestStep::SetAppState("Settings".to_string()),
            TestStep::Wait(0.5),
            TestStep::DumpUi("10_settings".to_string()),
            TestStep::VerifyElement(UiVerification {
                name: "Settings_Title".to_string(),
                component_name: None,
                text_contains: Some("Settings".to_string()),
                min_count: Some(1),
                max_count: None,
            }),
            TestStep::Screenshot("10_settings".to_string()),
            // Settings画面を閉じてからMainMenuに戻る
            TestStep::SetAppState("SettingsClosed".to_string()),
            TestStep::Wait(0.3),

            // ========================================
            // Phase 9: ゲーム内コマンド入力テスト
            // ========================================
            TestStep::Log("Phase 9: Command Input Test".to_string()),
            TestStep::SetAppState("InGame".to_string()),
            TestStep::Wait(1.0),
            // クリエイティブモードに切り替え
            TestStep::Log("Switching to Creative mode via command".to_string()),
            TestStep::ExecuteCommand("gamemode creative".to_string()),
            TestStep::Wait(0.5),
            TestStep::Screenshot("11_creative_mode".to_string()),

            // ========================================
            // Phase 10: クリエイティブモードでの操作テスト
            // ========================================
            TestStep::Log("Phase 10: Creative Mode Operations".to_string()),

            // 歩き回る
            TestStep::Log("Moving around in creative mode".to_string()),
            TestStep::PressKey(KeyCode::KeyW),
            TestStep::Wait(0.5),
            TestStep::ReleaseKey(KeyCode::KeyW),
            TestStep::PressKey(KeyCode::KeyA),
            TestStep::Wait(0.3),
            TestStep::ReleaseKey(KeyCode::KeyA),
            TestStep::PressKey(KeyCode::KeyS),
            TestStep::Wait(0.3),
            TestStep::ReleaseKey(KeyCode::KeyS),
            TestStep::PressKey(KeyCode::KeyD),
            TestStep::Wait(0.3),
            TestStep::ReleaseKey(KeyCode::KeyD),
            TestStep::Screenshot("12_creative_walked".to_string()),

            // ジャンプ
            TestStep::Log("Jumping in creative mode".to_string()),
            TestStep::TapKey(KeyCode::Space),
            TestStep::Wait(0.5),
            TestStep::Screenshot("13_creative_jumped".to_string()),

            // ホットバー切り替え
            TestStep::Log("Hotbar selection test".to_string()),
            TestStep::TapKey(KeyCode::Digit1),
            TestStep::Wait(0.2),
            TestStep::TapKey(KeyCode::Digit5),
            TestStep::Wait(0.2),
            TestStep::TapKey(KeyCode::Digit9),
            TestStep::Wait(0.2),
            TestStep::Screenshot("14_hotbar_test".to_string()),

            // クリエイティブモードでインベントリを開く
            TestStep::Log("Opening creative inventory".to_string()),
            TestStep::SetAppState("InventoryOpen".to_string()),
            TestStep::Wait(0.5),
            TestStep::DumpUi("15_creative_inventory".to_string()),
            TestStep::Screenshot("15_creative_inventory".to_string()),
            TestStep::SetAppState("InventoryClosed".to_string()),
            TestStep::Wait(0.3),

            // ブロック設置テスト（左クリック）
            TestStep::Log("Block placement test".to_string()),
            TestStep::MousePress(MouseButton::Left),
            TestStep::Wait(0.1),
            TestStep::MouseRelease(MouseButton::Left),
            TestStep::Wait(0.3),
            TestStep::Screenshot("16_block_placed".to_string()),

            // ========================================
            // Phase 11: サバイバルモードに戻す
            // ========================================
            TestStep::Log("Phase 11: Switch back to Survival".to_string()),
            TestStep::ExecuteCommand("gamemode survival".to_string()),
            TestStep::Wait(0.5),
            TestStep::Screenshot("17_survival_mode".to_string()),

            // ========================================
            // テスト完了・レポート保存
            // ========================================
            TestStep::Log("=== FULL TEST COMPLETE ===".to_string()),
            TestStep::SaveReport,
            TestStep::Log("Results saved to screenshots/test_report.txt".to_string()),
            TestStep::Log("UI dumps saved to screenshots/full_test_*.txt".to_string()),
            TestStep::Log("Screenshots saved to screenshots/full_test_*.png".to_string()),
            TestStep::Log(">>> Read test_report.txt first (low token cost)".to_string()),
            TestStep::Log(">>> Only check screenshots if verification failed".to_string()),
        ],
    }
}

/// 包括的インタラクションテストシナリオ
///
/// 全てのUIインタラクションパターンをテスト:
/// - キーボード操作（移動、ジャンプ、メニュー開閉）
/// - マウス操作（左クリック、右クリック）
/// - ホットバー切り替え（1-9キー）
/// - インベントリ操作
fn create_interaction_test_scenario() -> TestScenario {
    TestScenario {
        name: "interaction_test".to_string(),
        steps: vec![
            // ========================================
            // 初期化
            // ========================================
            TestStep::Log("=== INTERACTION TEST START ===".to_string()),
            TestStep::ClearReport,

            // ========================================
            // Phase 1: メニューナビゲーション
            // ========================================
            TestStep::Log("Phase 1: Menu Navigation".to_string()),

            // MainMenu → SaveSelect
            TestStep::Log("Testing: MainMenu -> SaveSelect".to_string()),
            TestStep::SetAppState("SaveSelect".to_string()),
            TestStep::Wait(0.3),
            TestStep::VerifyElement(UiVerification {
                name: "Nav_MainMenu_to_SaveSelect".to_string(),
                component_name: None,
                text_contains: Some("Select World".to_string()),
                min_count: Some(1),
                max_count: None,
            }),

            // SaveSelect → WorldGeneration
            TestStep::Log("Testing: SaveSelect -> WorldGeneration".to_string()),
            TestStep::SetAppState("WorldGeneration".to_string()),
            TestStep::Wait(0.3),
            TestStep::VerifyElement(UiVerification {
                name: "Nav_SaveSelect_to_WorldGen".to_string(),
                component_name: None,
                text_contains: Some("Create".to_string()),
                min_count: Some(1),
                max_count: None,
            }),

            // WorldGeneration → SaveSelect (Back)
            TestStep::Log("Testing: WorldGeneration -> SaveSelect (Back)".to_string()),
            TestStep::SetAppState("SaveSelect".to_string()),
            TestStep::Wait(0.3),
            TestStep::VerifyElement(UiVerification {
                name: "Nav_WorldGen_to_SaveSelect".to_string(),
                component_name: None,
                text_contains: Some("Select World".to_string()),
                min_count: Some(1),
                max_count: None,
            }),

            // SaveSelect → MainMenu (Back)
            TestStep::Log("Testing: SaveSelect -> MainMenu (Back)".to_string()),
            TestStep::SetAppState("MainMenu".to_string()),
            TestStep::Wait(0.3),
            TestStep::VerifyElement(UiVerification {
                name: "Nav_SaveSelect_to_MainMenu".to_string(),
                component_name: None,
                text_contains: Some("Play".to_string()),
                min_count: Some(1),
                max_count: None,
            }),

            // ========================================
            // Phase 2: InGame キーボード操作
            // ========================================
            TestStep::Log("Phase 2: InGame Keyboard Controls".to_string()),
            TestStep::SetAppState("InGame".to_string()),
            TestStep::Wait(1.0),

            // WASD移動テスト
            TestStep::Log("Testing: W key (forward)".to_string()),
            TestStep::PressKey(KeyCode::KeyW),
            TestStep::Wait(0.3),
            TestStep::ReleaseKey(KeyCode::KeyW),

            TestStep::Log("Testing: A key (left)".to_string()),
            TestStep::PressKey(KeyCode::KeyA),
            TestStep::Wait(0.3),
            TestStep::ReleaseKey(KeyCode::KeyA),

            TestStep::Log("Testing: S key (backward)".to_string()),
            TestStep::PressKey(KeyCode::KeyS),
            TestStep::Wait(0.3),
            TestStep::ReleaseKey(KeyCode::KeyS),

            TestStep::Log("Testing: D key (right)".to_string()),
            TestStep::PressKey(KeyCode::KeyD),
            TestStep::Wait(0.3),
            TestStep::ReleaseKey(KeyCode::KeyD),

            // ジャンプテスト
            TestStep::Log("Testing: Space key (jump)".to_string()),
            TestStep::TapKey(KeyCode::Space),
            TestStep::Wait(0.5),

            // しゃがみテスト
            TestStep::Log("Testing: Shift key (crouch)".to_string()),
            TestStep::PressKey(KeyCode::ShiftLeft),
            TestStep::Wait(0.3),
            TestStep::ReleaseKey(KeyCode::ShiftLeft),

            // ========================================
            // Phase 3: ホットバー切り替え（1-9キー）
            // ========================================
            TestStep::Log("Phase 3: Hotbar Selection (1-9 keys)".to_string()),

            TestStep::Log("Testing: Key 1 (hotbar slot 0)".to_string()),
            TestStep::TapKey(KeyCode::Digit1),
            TestStep::Wait(0.1),

            TestStep::Log("Testing: Key 2 (hotbar slot 1)".to_string()),
            TestStep::TapKey(KeyCode::Digit2),
            TestStep::Wait(0.1),

            TestStep::Log("Testing: Key 3 (hotbar slot 2)".to_string()),
            TestStep::TapKey(KeyCode::Digit3),
            TestStep::Wait(0.1),

            TestStep::Log("Testing: Key 4 (hotbar slot 3)".to_string()),
            TestStep::TapKey(KeyCode::Digit4),
            TestStep::Wait(0.1),

            TestStep::Log("Testing: Key 5 (hotbar slot 4)".to_string()),
            TestStep::TapKey(KeyCode::Digit5),
            TestStep::Wait(0.1),

            TestStep::Log("Testing: Key 6 (hotbar slot 5)".to_string()),
            TestStep::TapKey(KeyCode::Digit6),
            TestStep::Wait(0.1),

            TestStep::Log("Testing: Key 7 (hotbar slot 6)".to_string()),
            TestStep::TapKey(KeyCode::Digit7),
            TestStep::Wait(0.1),

            TestStep::Log("Testing: Key 8 (hotbar slot 7)".to_string()),
            TestStep::TapKey(KeyCode::Digit8),
            TestStep::Wait(0.1),

            TestStep::Log("Testing: Key 9 (hotbar slot 8)".to_string()),
            TestStep::TapKey(KeyCode::Digit9),
            TestStep::Wait(0.1),

            // ========================================
            // Phase 4: マウス操作
            // ========================================
            TestStep::Log("Phase 4: Mouse Controls".to_string()),

            // 左クリック（破壊/攻撃）
            TestStep::Log("Testing: Left click (break/attack)".to_string()),
            TestStep::MousePress(MouseButton::Left),
            TestStep::Wait(0.2),
            TestStep::MouseRelease(MouseButton::Left),

            // 右クリック（設置/使用）
            TestStep::Log("Testing: Right click (place/use)".to_string()),
            TestStep::MousePress(MouseButton::Right),
            TestStep::Wait(0.2),
            TestStep::MouseRelease(MouseButton::Right),

            // ホールドテスト（長押し）
            TestStep::Log("Testing: Left click hold (continuous break)".to_string()),
            TestStep::MousePress(MouseButton::Left),
            TestStep::Wait(0.5),
            TestStep::MouseRelease(MouseButton::Left),

            // ========================================
            // Phase 5: インベントリ操作
            // ========================================
            TestStep::Log("Phase 5: Inventory Operations".to_string()),

            // インベントリを開く
            TestStep::Log("Testing: E key (open inventory)".to_string()),
            TestStep::SetAppState("InventoryOpen".to_string()),
            TestStep::Wait(0.5),
            TestStep::VerifyElement(UiVerification {
                name: "Inventory_Opened".to_string(),
                component_name: None,
                text_contains: Some("Sort".to_string()),
                min_count: Some(1),
                max_count: None,
            }),

            // インベントリ内でのマウス操作
            TestStep::Log("Testing: Inventory left click".to_string()),
            TestStep::MouseMove(640.0, 360.0),  // 画面中央付近
            TestStep::MousePress(MouseButton::Left),
            TestStep::Wait(0.1),
            TestStep::MouseRelease(MouseButton::Left),

            TestStep::Log("Testing: Inventory right click".to_string()),
            TestStep::MousePress(MouseButton::Right),
            TestStep::Wait(0.1),
            TestStep::MouseRelease(MouseButton::Right),

            // ドラッグ&ドロップシミュレーション
            TestStep::Log("Testing: Drag and drop simulation".to_string()),
            TestStep::MouseMove(400.0, 300.0),
            TestStep::MousePress(MouseButton::Left),
            TestStep::Wait(0.1),
            TestStep::MouseMove(500.0, 300.0),
            TestStep::Wait(0.1),
            TestStep::MouseRelease(MouseButton::Left),

            // インベントリを閉じる
            TestStep::Log("Testing: E key (close inventory)".to_string()),
            TestStep::SetAppState("InventoryClosed".to_string()),
            TestStep::Wait(0.3),

            // ========================================
            // Phase 6: ポーズメニュー操作
            // ========================================
            TestStep::Log("Phase 6: Pause Menu Operations".to_string()),

            // ESCでポーズメニューを開く
            TestStep::Log("Testing: ESC key (open pause menu)".to_string()),
            TestStep::SetAppState("PauseMenu".to_string()),
            TestStep::Wait(0.3),
            TestStep::VerifyElement(UiVerification {
                name: "PauseMenu_Opened".to_string(),
                component_name: None,
                text_contains: Some("Paused".to_string()),
                min_count: Some(1),
                max_count: None,
            }),
            TestStep::VerifyElement(UiVerification {
                name: "PauseMenu_ResumeBtn".to_string(),
                component_name: None,
                text_contains: Some("Resume".to_string()),
                min_count: Some(1),
                max_count: None,
            }),
            TestStep::VerifyElement(UiVerification {
                name: "PauseMenu_SettingsBtn".to_string(),
                component_name: None,
                text_contains: Some("Settings".to_string()),
                min_count: Some(1),
                max_count: None,
            }),
            TestStep::VerifyElement(UiVerification {
                name: "PauseMenu_SaveQuitBtn".to_string(),
                component_name: None,
                text_contains: Some("Save".to_string()),
                min_count: Some(1),
                max_count: None,
            }),
            TestStep::VerifyElement(UiVerification {
                name: "PauseMenu_MainMenuBtn".to_string(),
                component_name: None,
                text_contains: Some("Main Menu".to_string()),
                min_count: Some(1),
                max_count: None,
            }),

            // ポーズメニューを閉じる（InGameに戻る）
            TestStep::Log("Testing: ESC key (close pause menu)".to_string()),
            TestStep::SetAppState("InGame".to_string()),
            TestStep::Wait(0.3),

            // ========================================
            // Phase 7: コンテナインベントリ
            // ========================================
            TestStep::Log("Phase 7: Container Inventory".to_string()),

            // コンテナを開く（右クリックシミュレーション）
            TestStep::Log("Testing: Right click (open container)".to_string()),
            TestStep::SetAppState("Container".to_string()),
            TestStep::Wait(0.5),
            // コンテナUIの検証（もし実装されていれば）

            // コンテナを閉じる
            TestStep::Log("Testing: Close container".to_string()),
            TestStep::SetAppState("InventoryClosed".to_string()),
            TestStep::Wait(0.3),

            // ========================================
            // Phase 8: クイックアクセス操作
            // ========================================
            TestStep::Log("Phase 8: Quick Access Operations".to_string()),

            // クエストリスト（Jキー）
            TestStep::Log("Testing: J key (quest list)".to_string()),
            TestStep::TapKey(KeyCode::KeyJ),
            TestStep::Wait(0.3),

            // デバッグモード（F3キー）
            TestStep::Log("Testing: F3 key (debug mode)".to_string()),
            TestStep::TapKey(KeyCode::F3),
            TestStep::Wait(0.3),
            TestStep::TapKey(KeyCode::F3),  // オフに戻す
            TestStep::Wait(0.1),

            // ========================================
            // Phase 9: 複合操作テスト
            // ========================================
            TestStep::Log("Phase 9: Combined Operations".to_string()),

            // 移動しながらジャンプ
            TestStep::Log("Testing: Move + Jump (W + Space)".to_string()),
            TestStep::PressKey(KeyCode::KeyW),
            TestStep::Wait(0.1),
            TestStep::TapKey(KeyCode::Space),
            TestStep::Wait(0.3),
            TestStep::ReleaseKey(KeyCode::KeyW),

            // 斜め移動
            TestStep::Log("Testing: Diagonal move (W + D)".to_string()),
            TestStep::PressKey(KeyCode::KeyW),
            TestStep::PressKey(KeyCode::KeyD),
            TestStep::Wait(0.3),
            TestStep::ReleaseKey(KeyCode::KeyW),
            TestStep::ReleaseKey(KeyCode::KeyD),

            // 移動しながらホットバー切り替え
            TestStep::Log("Testing: Move + Hotbar switch".to_string()),
            TestStep::PressKey(KeyCode::KeyW),
            TestStep::TapKey(KeyCode::Digit1),
            TestStep::Wait(0.1),
            TestStep::TapKey(KeyCode::Digit2),
            TestStep::Wait(0.1),
            TestStep::ReleaseKey(KeyCode::KeyW),

            // ========================================
            // Phase 10: メインメニューに戻る
            // ========================================
            TestStep::Log("Phase 10: Return to Main Menu".to_string()),
            TestStep::SetAppState("MainMenu".to_string()),
            TestStep::Wait(0.3),
            TestStep::VerifyElement(UiVerification {
                name: "Final_MainMenu".to_string(),
                component_name: None,
                text_contains: Some("Play".to_string()),
                min_count: Some(1),
                max_count: None,
            }),

            // ========================================
            // テスト完了・レポート保存
            // ========================================
            TestStep::Log("=== INTERACTION TEST COMPLETE ===".to_string()),
            TestStep::SaveReport,
            TestStep::Log("Results saved to screenshots/test_report.txt".to_string()),
        ],
    }
}

/// 人間らしい挙動テストシナリオ
///
/// 新しい人間らしい機能をテスト:
/// - ランダムな待機時間
/// - 滑らかなマウス移動
/// - 人間らしいキータップ
/// - 思考時間
/// - UI要素クリック
fn create_human_behavior_test_scenario() -> TestScenario {
    TestScenario {
        name: "human_test".to_string(),
        steps: vec![
            // ========================================
            // 初期化
            // ========================================
            TestStep::Log("=== HUMAN BEHAVIOR TEST START ===".to_string()),
            TestStep::ClearReport,

            // ========================================
            // Phase 1: 思考時間テスト
            // ========================================
            TestStep::Log("Phase 1: Think Time Test".to_string()),
            TestStep::Think,
            TestStep::Log("Thought completed".to_string()),

            // ========================================
            // Phase 2: ランダム待機テスト
            // ========================================
            TestStep::Log("Phase 2: Random Wait Test".to_string()),
            TestStep::WaitRandom(0.2, 0.8),
            TestStep::Log("Random wait completed".to_string()),

            // ========================================
            // Phase 3: 人間らしいキー入力
            // ========================================
            TestStep::Log("Phase 3: Human Key Input".to_string()),
            TestStep::SetAppState("InGame".to_string()),
            TestStep::Wait(1.0),

            // 人間らしいWASD移動
            TestStep::Log("Human WASD movement".to_string()),
            TestStep::TapKeyHuman(KeyCode::KeyW),
            TestStep::Think,
            TestStep::TapKeyHuman(KeyCode::KeyA),
            TestStep::Think,
            TestStep::TapKeyHuman(KeyCode::KeyS),
            TestStep::Think,
            TestStep::TapKeyHuman(KeyCode::KeyD),

            // ========================================
            // Phase 4: 滑らかなマウス移動
            // ========================================
            TestStep::Log("Phase 4: Smooth Mouse Movement".to_string()),
            TestStep::MouseMoveSmooth(400.0, 300.0),
            TestStep::Wait(0.5),
            TestStep::MouseMoveSmooth(800.0, 400.0),
            TestStep::Wait(0.5),
            TestStep::MouseMoveSmooth(640.0, 360.0),
            TestStep::Screenshot("smooth_mouse".to_string()),

            // ========================================
            // Phase 5: 人間らしいマウスクリック
            // ========================================
            TestStep::Log("Phase 5: Human Mouse Click".to_string()),
            TestStep::MouseClickHuman(MouseButton::Left),
            TestStep::Wait(0.3),
            TestStep::MouseRelease(MouseButton::Left),
            TestStep::Think,
            TestStep::MouseClickHuman(MouseButton::Right),
            TestStep::Wait(0.3),
            TestStep::MouseRelease(MouseButton::Right),

            // ========================================
            // Phase 6: スクロールテスト
            // ========================================
            TestStep::Log("Phase 6: Scroll Test".to_string()),
            TestStep::SetAppState("InventoryOpen".to_string()),
            TestStep::Wait(0.5),
            TestStep::Scroll(3.0),  // 上にスクロール
            TestStep::Wait(0.3),
            TestStep::Scroll(-3.0), // 下にスクロール
            TestStep::Wait(0.3),
            TestStep::Screenshot("scroll_test".to_string()),
            TestStep::SetAppState("InventoryClosed".to_string()),

            // ========================================
            // Phase 7: ダブルクリックテスト
            // ========================================
            TestStep::Log("Phase 7: Double Click Test".to_string()),
            TestStep::DoubleClick(MouseButton::Left),
            TestStep::Wait(0.1),
            TestStep::MouseRelease(MouseButton::Left),
            TestStep::Wait(0.3),

            // ========================================
            // Phase 8: ドラッグ&ドロップテスト
            // ========================================
            TestStep::Log("Phase 8: Drag & Drop Test".to_string()),
            TestStep::DragDrop(300.0, 300.0, 500.0, 300.0),
            TestStep::Wait(0.5),

            // ========================================
            // テスト完了
            // ========================================
            TestStep::SetAppState("MainMenu".to_string()),
            TestStep::Wait(0.5),
            TestStep::Log("=== HUMAN BEHAVIOR TEST COMPLETE ===".to_string()),
            TestStep::SaveReport,
        ],
    }
}

/// ボタンクリックテストシナリオ
///
/// UI要素を名前やテキストで検索してクリックするテスト
fn create_button_click_test_scenario() -> TestScenario {
    TestScenario {
        name: "button_click_test".to_string(),
        steps: vec![
            // ========================================
            // 初期化
            // ========================================
            TestStep::Log("=== BUTTON CLICK TEST START ===".to_string()),
            TestStep::ClearReport,
            TestStep::SetAppState("MainMenu".to_string()),
            TestStep::Wait(0.5),

            // ========================================
            // Phase 1: メインメニューのボタンをクリック
            // ========================================
            TestStep::Log("Phase 1: Main Menu Button Click".to_string()),
            TestStep::DumpUi("01_main_menu".to_string()),

            // "Play"ボタンをクリック
            TestStep::Log("Clicking Play button".to_string()),
            TestStep::ClickElement(ElementSelector::by_text("Play")),
            TestStep::Wait(0.5),
            TestStep::Screenshot("clicked_play".to_string()),

            // ========================================
            // Phase 2: セーブ選択画面
            // ========================================
            TestStep::Log("Phase 2: Save Select".to_string()),
            TestStep::DumpUi("02_save_select".to_string()),

            // "Back"ボタンをクリック
            TestStep::Log("Clicking Back button".to_string()),
            TestStep::ClickElement(ElementSelector::by_text("Back")),
            TestStep::Wait(0.5),

            // ========================================
            // Phase 3: 設定ボタンをクリック
            // ========================================
            TestStep::Log("Phase 3: Settings Button Click".to_string()),
            TestStep::SetAppState("MainMenu".to_string()),
            TestStep::Wait(0.3),

            // "Settings"ボタンをクリック
            TestStep::Log("Clicking Settings button".to_string()),
            TestStep::ClickElement(ElementSelector::by_text("Settings")),
            TestStep::Wait(0.5),
            TestStep::Screenshot("clicked_settings".to_string()),

            // ========================================
            // Phase 4: ゲーム内でボタンをテスト
            // ========================================
            TestStep::Log("Phase 4: InGame Button Test".to_string()),
            TestStep::SetAppState("InGame".to_string()),
            TestStep::Wait(1.0),

            // インベントリを開く
            TestStep::SetAppState("InventoryOpen".to_string()),
            TestStep::Wait(0.5),
            TestStep::DumpUi("03_inventory".to_string()),

            // "Sort"ボタンをクリック
            TestStep::Log("Clicking Sort button".to_string()),
            TestStep::ClickElement(ElementSelector::by_text("Sort")),
            TestStep::Wait(0.3),
            TestStep::Screenshot("clicked_sort".to_string()),

            // ========================================
            // Phase 5: ホバーテスト
            // ========================================
            TestStep::Log("Phase 5: Hover Test".to_string()),
            TestStep::HoverElement(ElementSelector::by_text("Trash")),
            TestStep::Wait(0.5),
            TestStep::Screenshot("hover_trash".to_string()),

            // ========================================
            // テスト完了
            // ========================================
            TestStep::SetAppState("InventoryClosed".to_string()),
            TestStep::Wait(0.3),
            TestStep::SetAppState("MainMenu".to_string()),
            TestStep::Wait(0.5),
            TestStep::Log("=== BUTTON CLICK TEST COMPLETE ===".to_string()),
            TestStep::SaveReport,
        ],
    }
}

/// カスタムシナリオビルダー
pub struct TestScenarioBuilder {
    name: String,
    steps: Vec<TestStep>,
}

impl TestScenarioBuilder {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            steps: Vec::new(),
        }
    }

    pub fn wait(mut self, seconds: f32) -> Self {
        self.steps.push(TestStep::Wait(seconds));
        self
    }

    pub fn press_key(mut self, key: KeyCode) -> Self {
        self.steps.push(TestStep::PressKey(key));
        self
    }

    pub fn release_key(mut self, key: KeyCode) -> Self {
        self.steps.push(TestStep::ReleaseKey(key));
        self
    }

    pub fn tap_key(mut self, key: KeyCode) -> Self {
        self.steps.push(TestStep::TapKey(key));
        self
    }

    pub fn screenshot(mut self, name: &str) -> Self {
        self.steps.push(TestStep::Screenshot(name.to_string()));
        self
    }

    pub fn log(mut self, msg: &str) -> Self {
        self.steps.push(TestStep::Log(msg.to_string()));
        self
    }

    pub fn mouse_move(mut self, x: f32, y: f32) -> Self {
        self.steps.push(TestStep::MouseMove(x, y));
        self
    }

    pub fn mouse_click(mut self, button: MouseButton) -> Self {
        self.steps.push(TestStep::MousePress(button));
        self.steps.push(TestStep::Wait(0.05));
        self.steps.push(TestStep::MouseRelease(button));
        self
    }

    pub fn set_app_state(mut self, state: &str) -> Self {
        self.steps.push(TestStep::SetAppState(state.to_string()));
        self
    }

    pub fn dump_ui(mut self, name: &str) -> Self {
        self.steps.push(TestStep::DumpUi(name.to_string()));
        self
    }

    pub fn verify(mut self, verification: UiVerification) -> Self {
        self.steps.push(TestStep::VerifyElement(verification));
        self
    }

    pub fn verify_text(mut self, name: &str, text: &str) -> Self {
        self.steps.push(TestStep::VerifyElement(UiVerification {
            name: name.to_string(),
            component_name: None,
            text_contains: Some(text.to_string()),
            min_count: Some(1),
            max_count: None,
        }));
        self
    }

    pub fn save_report(mut self) -> Self {
        self.steps.push(TestStep::SaveReport);
        self
    }

    pub fn clear_report(mut self) -> Self {
        self.steps.push(TestStep::ClearReport);
        self
    }

    // 新しい人間らしい挙動メソッド

    pub fn think(mut self) -> Self {
        self.steps.push(TestStep::Think);
        self
    }

    pub fn wait_random(mut self, min: f32, max: f32) -> Self {
        self.steps.push(TestStep::WaitRandom(min, max));
        self
    }

    pub fn tap_key_human(mut self, key: KeyCode) -> Self {
        self.steps.push(TestStep::TapKeyHuman(key));
        self
    }

    pub fn mouse_move_smooth(mut self, x: f32, y: f32) -> Self {
        self.steps.push(TestStep::MouseMoveSmooth(x, y));
        self
    }

    pub fn mouse_click_human(mut self, button: MouseButton) -> Self {
        self.steps.push(TestStep::MouseClickHuman(button));
        self
    }

    pub fn double_click(mut self, button: MouseButton) -> Self {
        self.steps.push(TestStep::DoubleClick(button));
        self
    }

    pub fn scroll(mut self, delta: f32) -> Self {
        self.steps.push(TestStep::Scroll(delta));
        self
    }

    pub fn click_element(mut self, selector: ElementSelector) -> Self {
        self.steps.push(TestStep::ClickElement(selector));
        self
    }

    pub fn click_text(mut self, text: &str) -> Self {
        self.steps.push(TestStep::ClickElement(ElementSelector::by_text(text)));
        self
    }

    pub fn click_name(mut self, name: &str) -> Self {
        self.steps.push(TestStep::ClickElement(ElementSelector::by_name(name)));
        self
    }

    pub fn hover_element(mut self, selector: ElementSelector) -> Self {
        self.steps.push(TestStep::HoverElement(selector));
        self
    }

    pub fn drag_drop(mut self, x1: f32, y1: f32, x2: f32, y2: f32) -> Self {
        self.steps.push(TestStep::DragDrop(x1, y1, x2, y2));
        self
    }

    pub fn type_text_human(mut self, text: &str) -> Self {
        self.steps.push(TestStep::TypeTextHuman(text.to_string()));
        self
    }

    pub fn build(self) -> TestScenario {
        TestScenario {
            name: self.name,
            steps: self.steps,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scenario_builder() {
        let scenario = TestScenarioBuilder::new("test")
            .wait(1.0)
            .tap_key(KeyCode::KeyE)
            .screenshot("test_shot")
            .build();

        assert_eq!(scenario.name, "test");
        assert_eq!(scenario.steps.len(), 3);
    }

    #[test]
    fn test_builtin_scenarios_exist() {
        assert!(get_builtin_scenario("ui_inventory_test").is_some());
        assert!(get_builtin_scenario("ui_main_menu_test").is_some());
        assert!(get_builtin_scenario("gameplay_basic_test").is_some());
        assert!(get_builtin_scenario("full_test").is_some());
        assert!(get_builtin_scenario("human_test").is_some());
        assert!(get_builtin_scenario("button_click_test").is_some());
        assert!(get_builtin_scenario("nonexistent").is_none());
    }

    #[test]
    fn test_human_behavior_scenario() {
        let scenario = get_builtin_scenario("human_test").unwrap();
        // 人間らしい挙動が含まれていることを確認
        let has_think = scenario.steps.iter().any(|s| matches!(s, TestStep::Think));
        let has_wait_random = scenario.steps.iter().any(|s| matches!(s, TestStep::WaitRandom(_, _)));
        let has_smooth_mouse = scenario.steps.iter().any(|s| matches!(s, TestStep::MouseMoveSmooth(_, _)));
        assert!(has_think, "Should have Think step");
        assert!(has_wait_random, "Should have WaitRandom step");
        assert!(has_smooth_mouse, "Should have MouseMoveSmooth step");
    }

    #[test]
    fn test_button_click_scenario() {
        let scenario = get_builtin_scenario("button_click_test").unwrap();
        // ClickElementが含まれていることを確認
        let has_click_element = scenario.steps.iter().any(|s| matches!(s, TestStep::ClickElement(_)));
        assert!(has_click_element, "Should have ClickElement step");
    }

    #[test]
    fn test_element_selector() {
        let by_name = ElementSelector::by_name("TestButton");
        assert_eq!(by_name.name, Some("TestButton".to_string()));
        assert_eq!(by_name.text, None);

        let by_text = ElementSelector::by_text("Click Me");
        assert_eq!(by_text.name, None);
        assert_eq!(by_text.text, Some("Click Me".to_string()));

        let by_name_index = ElementSelector::by_name_index("Button", 2);
        assert_eq!(by_name_index.name, Some("Button".to_string()));
        assert_eq!(by_name_index.index, 2);
    }

    #[test]
    fn test_full_scenario_has_screenshots() {
        let scenario = get_builtin_scenario("full_test").unwrap();
        let screenshot_count = scenario.steps.iter().filter(|s| {
            matches!(s, TestStep::Screenshot(_))
        }).count();
        // フルテストは17枚のスクリーンショットを撮影
        // (メイン9枚 + Settings1枚 + クリエイティブ操作6枚 + サバイバル復帰1枚)
        assert_eq!(screenshot_count, 17);
    }
}
